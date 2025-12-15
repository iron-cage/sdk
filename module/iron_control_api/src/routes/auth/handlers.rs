//! Authentication handler functions

use super::shared::*;
use crate::jwt_auth::AuthenticatedUser;
use crate::user_auth;
use axum::{
  extract::{ConnectInfo, State},
  http::StatusCode,
  response::{IntoResponse, Json},
};
use std::net::SocketAddr;
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader
};

// ============================================================================
// Login Endpoint - POST /api/v1/auth/login
// ============================================================================

/// POST /api/v1/auth/login
///
/// Authenticate user and return User Token (JWT, 30 days)
///
/// # Arguments
///
/// * `state` - Authentication state (JWT secret + database)
/// * `request` - Login credentials (email, password)
///
/// # Returns
///
/// - 200 OK with User Token if authentication successful
/// - 400 Bad Request if validation fails
/// - 401 Unauthorized if credentials invalid
/// - 403 Forbidden if account disabled
/// - 429 Too Many Requests if rate limit exceeded
/// - 500 Internal Server Error if token generation or database query fails
///
/// # Security
///
/// - Password never logged or exposed in responses
/// - Rate limiting: 5 attempts per 5 minutes per IP
/// - Failed attempts logged for security monitoring
/// - Account lockout after 10 failed attempts (manual unlock by admin)
// Fix(issue-GAP-006): Add per-IP rate limiting via ConnectInfo
// Root cause: Pilot used hardcoded 127.0.0.1, applying global rate limit instead of per-client
// Pitfall: Never use X-Forwarded-For (spoofable) or hardcoded IPs for rate limiting - use ConnectInfo
pub async fn login(
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  State(state): State<AuthState>,
  Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
  // Validate request
  if let Err(validation_error) = request.validate() {
    return (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: ErrorDetail {
          code: "VALIDATION_ERROR".to_string(),
          message: validation_error.to_string(),
          details: None,
        },
      }),
    )
      .into_response();
  }

  // GAP-006: Rate limiting check (5 attempts per 5 minutes per IP)
  // Extract real client IP from TCP connection (secure, cannot be spoofed)
  let client_ip = addr.ip();

  if let Err( retry_after_secs ) = state.rate_limiter.check_and_record( client_ip )
  {
    tracing::warn!(
      email = %request.email,
      client_ip = %client_ip,
      retry_after_secs = retry_after_secs,
      "Rate limit exceeded for login attempt"
    );
    return (
      StatusCode::TOO_MANY_REQUESTS,
      Json( ErrorResponse {
        error: ErrorDetail {
          code: "RATE_LIMIT_EXCEEDED".to_string(),
          message: format!( "Too many login attempts. Please try again in {} seconds.", retry_after_secs ),
          details: Some( serde_json::json!({
            "retry_after": retry_after_secs
          })),
        },
      }),
    )
      .into_response();
  }

  // Check account lockout before attempting authentication
  // Protocol 007: "Account lockout after 10 failed attempts"
  let lockout_check: Option<( i64, Option< i64 > )> = sqlx::query_as(
    "SELECT failed_login_count, locked_until FROM users WHERE email = ?"
  )
    .bind( &request.email )
    .fetch_optional( &state.db_pool )
    .await
    .unwrap_or( None );

  if let Some(( failed_count, Some( locked_until_ts ) )) = lockout_check
  {
    let now = chrono::Utc::now().timestamp_millis();
    if locked_until_ts > now
    {
      let retry_after_secs = ( locked_until_ts - now ) / 1000;
      tracing::warn!(
        email = %request.email,
        failed_login_count = failed_count,
        locked_until = locked_until_ts,
        "Login attempt blocked - account locked"
      );
      return (
        StatusCode::FORBIDDEN,
        Json( ErrorResponse {
          error: ErrorDetail {
            code: "AUTH_ACCOUNT_LOCKED".to_string(),
            message: format!( "Account locked due to too many failed login attempts. Try again in {} seconds.", retry_after_secs ),
            details: Some( serde_json::json!({
              "retry_after": retry_after_secs,
              "locked_until": locked_until_ts
            })),
          },
        }),
      )
        .into_response();
    }
  }

  // Authenticate user against database
  // Note: Using username field for email (database schema uses username)
  let user = match user_auth::authenticate_user(&state.db_pool, &request.email, &request.password)
    .await
  {
    Ok(Some(user)) => user,
    Ok(None) => {
      // Invalid credentials - increment failed login counter
      // Protocol 007: Account lockout after 10 failed attempts (15-30 min duration)
      let now = chrono::Utc::now().timestamp_millis();

      let failed_count: Option<i64> = sqlx::query_scalar(
        "UPDATE users SET
         failed_login_count = failed_login_count + 1,
         last_failed_login = ?
         WHERE email = ?
         RETURNING failed_login_count"
      )
        .bind( now )
        .bind( &request.email )
        .fetch_optional( &state.db_pool )
        .await
        .unwrap_or( None );

      // Lock account if threshold reached (10 failed attempts)
      if let Some( count ) = failed_count
      {
        if count >= 10
        {
          // Lock for 30 minutes (1800000 milliseconds)
          let locked_until = now + 1800000;
          sqlx::query(
            "UPDATE users SET locked_until = ? WHERE email = ?"
          )
            .bind( locked_until )
            .bind( &request.email )
            .execute( &state.db_pool )
            .await
            .ok();

          tracing::warn!(
            email = %request.email,
            failed_login_count = count,
            locked_until = locked_until,
            "Account locked after 10 failed login attempts"
          );
        }
      }

      // GAP-004: Log failed login attempt for security monitoring
      tracing::warn!(
        email = %request.email,
        failure_reason = "invalid_credentials",
        "Failed login attempt - invalid credentials"
      );
      return (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "AUTH_INVALID_CREDENTIALS".to_string(),
            message: "Invalid email or password".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
    Err(err) => {
      // Database error - return 500
      tracing::error!("Database error during authentication: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "INTERNAL_ERROR".to_string(),
            message: "Authentication service unavailable".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  };

  // Check if account is active
  if !user.is_active {
    // GAP-004: Log failed login attempt (account disabled)
    tracing::warn!(
      email = %request.email,
      user_id = %user.id,
      failure_reason = "account_disabled",
      "Failed login attempt - account disabled"
    );
    return (
      StatusCode::FORBIDDEN,
      Json(ErrorResponse {
        error: ErrorDetail {
          code: "AUTH_ACCOUNT_DISABLED".to_string(),
          message: "Account has been disabled".to_string(),
          details: Some(serde_json::json!({
            "user_id": format!("{}", user.id)
          })),
        },
      }),
    )
      .into_response();
  }

  let user_id = &user.id;
  let user_role = &user.role;

  // Reset failed login counter on successful authentication
  sqlx::query(
    "UPDATE users SET
     failed_login_count = 0,
     last_failed_login = NULL,
     locked_until = NULL
     WHERE id = ?"
  )
    .bind( user_id )
    .execute( &state.db_pool )
    .await
    .ok();

  // Generate User Token (30 days expiration)
  // Generate unique token ID for blacklist tracking (UUID for session fixation prevention)
  let access_token_id = format!("access_{}_{}", user_id, uuid::Uuid::new_v4());
  let user_token = match state.jwt_secret.generate_access_token(user_id, &user.email, user_role, &access_token_id) {
    Ok(token) => token,
    Err(err) => {
      tracing::error!("Failed to generate user token: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "TOKEN_GENERATION_ERROR".to_string(),
            message: "Failed to generate access token".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  };

  // Generate refresh token (optional, future feature)
  // Per Protocol 007: refresh_token is optional
  let refresh_token_id = format!("refresh_{}_{}", user_id, chrono::Utc::now().timestamp());
  let refresh_token = match state
    .jwt_secret
    .generate_refresh_token(user_id, &user.email, user_role, &refresh_token_id)
  {
    Ok(token) => Some(token),
    Err(err) => {
      tracing::warn!("Failed to generate refresh token: {}", err);
      None
    }
  };

  // Calculate expiration (30 days from now)
  let expires_in = 2592000u64; // 30 days in seconds
  let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);

  (
    StatusCode::OK,
    Json(LoginResponse {
      user_token,
      token_type: "Bearer".to_string(),
      expires_in,
      expires_at: expires_at.to_rfc3339(),
      refresh_token,
      user: UserInfo::from_user(&user),
    }),
  )
    .into_response()
}

// ============================================================================
// Logout Endpoint - POST /api/v1/auth/logout
// ============================================================================

/// Logout request (User Token in Authorization header)
///
/// Per Protocol 007:
/// ```http
/// POST /api/v1/auth/logout
/// Authorization: Bearer <USER_TOKEN>
/// ```
///
/// No request body required.
/// POST /api/v1/auth/logout
///
/// Logout user by blacklisting User Token
///
/// # Arguments
///
/// * `state` - Authentication state (JWT secret + database)
/// * `user_token` - User Token from Authorization header (extracted by middleware)
///
/// # Returns
///
/// - 204 No Content if logout successful
/// - 401 Unauthorized if token invalid or expired
///
/// # Implementation
///
/// - User Token added to blacklist (redis/database)
/// - Blacklist checked on every authenticated request
/// - Token remains blacklisted until original expiration time
/// - Multiple User Tokens per user supported (logout only invalidates current token)
///
/// # Side Effects
///
/// - Logged out User Token immediately invalid
/// - All subsequent requests with logged out token return 401 Unauthorized
/// - Other User Tokens for same user remain valid (if user has multiple sessions)
pub async fn logout(
  State(state): State<AuthState>,
  AuthenticatedUser( claims ): AuthenticatedUser
) -> impl IntoResponse {
  let jti = claims.jti;
  let user_id = claims.sub;

  // INSERT INTO token_blacklist (jti, blacklisted_at, expires_at) VALUES (?, ?, ?)
  // - jti: Token ID from JWT claims
  // - blacklisted_at: Current timestamp
  // - expires_at: Original token expiration (for cleanup)
  let expires_at = match chrono::DateTime::from_timestamp( claims.exp, 0 ) {
    Some( dt ) => dt,
    None => {
      tracing::error!( "Invalid expiration timestamp in JWT claims: {}", claims.exp );
      return ( StatusCode::BAD_REQUEST, Json( ErrorResponse {
        error: ErrorDetail {
          code: "INVALID_TOKEN".to_string(),
          message: "Token contains invalid expiration timestamp".to_string(),
          details: None,
        },
      } ) ).into_response();
    }
  };
  match user_auth::add_token_to_blacklist(&state.db_pool, &jti, &user_id, expires_at).await {
    Ok(()) => {},
    Err(err) => {
      tracing::error!("Failed to add token to blacklist: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "TOKEN_BLACKLIST_ERROR".to_string(),
            message: "Failed to add token to blacklist".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  }

  // GAP-005: Log logout event for security monitoring
  tracing::info!(
    user_id = %user_id,
    session_id = %jti,
    "User logout - session terminated"
  );

  StatusCode::NO_CONTENT.into_response()
}

// ============================================================================
// Refresh Endpoint - POST /api/v1/auth/refresh
// ============================================================================

/// POST /api/v1/auth/refresh
///
/// Refresh User Token (extend expiration)
///
/// # Arguments
///
/// * `state` - Authentication state (JWT secret + database)
/// * `user_token` - Current User Token from Authorization header
///
/// # Returns
///
/// - 200 OK with new User Token
/// - 401 Unauthorized if token expired or invalid
///
/// # Refresh Window
///
/// - Can refresh anytime before expiration
/// - Recommended: Refresh when < 7 days remaining
/// - Old token invalidated when new token issued
/// - Atomic operation (old invalidated, new generated)
///
/// # CLI Behavior
///
/// - `iron_cli` automatically refreshes token when < 7 days remaining
/// - Refresh happens transparently during any CLI command
/// - User prompted to re-login if token expired
pub async fn refresh(
  State(state): State<AuthState>,
  TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
  // AuthenticatedUser( claims ): AuthenticatedUser

) -> impl IntoResponse {
  let claims = match state.jwt_secret.verify_refresh_token(bearer.token()) {
    Ok(claims) => claims,
    Err(_) => {
      return (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "AUTH_INVALID_TOKEN".to_string(),
            message: "Invalid or expired authentication token".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  };

  let blacklisted = match user_auth::get_blacklisted_token(&state.db_pool, &claims.jti).await {
    Ok(blacklisted) => blacklisted,
    Err(err) => {
      tracing::error!("Failed to check token blacklist: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "TOKEN_BLACKLIST_ERROR".to_string(),
            message: "Failed to check token blacklist".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  };
  if blacklisted.is_some() {
    return (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "AUTH_INVALID_TOKEN".to_string(),
            message: "Invalid or expired authentication token".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
  }

  // Fetch user to get current role
  let user = match user_auth::get_user_by_id(&state.db_pool, &claims.sub).await {
    Ok(user) => user,
    Err(err) => {
      tracing::error!("Failed to get user by ID: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "USER_NOT_FOUND".to_string(),
            message: "User not found".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  };

  let user = match user {
    Some(user) => user,
    None => {
      return (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          error: ErrorDetail {
          code: "USER_NOT_FOUND".to_string(),
          message: "User not found".to_string(),
          details: None,
        },
      }),
    )
      .into_response();
    }
  };

  // Generate new User Token (30 days) with unique JTI (session fixation prevention)
  let new_token_id = format!("refresh_{}_{}", user.id, uuid::Uuid::new_v4());
  let new_user_token = match state.jwt_secret.generate_access_token(&user.id, &user.email, &user.role, &new_token_id) {
    Ok( token ) => token,
    Err( e ) => {
      tracing::error!( "Failed to generate new access token during refresh: {}", e );
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( ErrorResponse {
        error: ErrorDetail {
          code: "TOKEN_GENERATION_FAILED".to_string(),
          message: "Failed to generate new access token".to_string(),
          details: None,
        },
      } ) ).into_response();
    }
  };

  // Generate new refresh token (token rotation security feature)
  // Per Protocol 007 enhancement: rotate refresh tokens to limit exposure window
  // Use nanosecond timestamp to ensure uniqueness even within same second
  let new_refresh_token_id = format!("refresh_{}_{}", user.id, chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
  let new_refresh_token = match state
    .jwt_secret
    .generate_refresh_token(&user.id, &user.email, &user.role, &new_refresh_token_id)
  {
    Ok(token) => Some(token),
    Err(err) => {
      tracing::warn!("Failed to generate new refresh token during rotation: {}", err);
      None
    }
  };

  // Blacklist old User Token (atomic operation)
  let expires_at = chrono::Utc::now() + chrono::Duration::seconds(claims.exp as i64);
  match user_auth::add_token_to_blacklist(&state.db_pool, &claims.jti, &user.id, expires_at).await {
    Ok(()) => {},
    Err(err) => {
      tracing::error!("Failed to add token to blacklist: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "TOKEN_BLACKLIST_ERROR".to_string(),
            message: "Failed to add token to blacklist".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  }

  // Calculate expiration (30 days from now)
  let expires_in = 2592000u64; // 30 days in seconds
  let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);

  // Return response with new tokens (both access and refresh)
  (
    StatusCode::OK,
    Json(RefreshResponse {
      user_token: new_user_token,
      token_type: "Bearer".to_string(),
      expires_in,
      expires_at: expires_at.to_rfc3339(),
      refresh_token: new_refresh_token,
      user: UserInfo::from_user(&user),
    }),
  )
    .into_response()
}

// ============================================================================
// Validate Endpoint - POST /api/v1/auth/validate
// ============================================================================

/// POST /api/v1/auth/validate
///
/// Validate User Token (check if valid)
///
/// # Arguments
///
/// * `state` - Authentication state (JWT secret + database)
/// * `user_token` - User Token from Authorization header
///
/// # Returns
///
/// - 200 OK with validation result (always returns 200, even for invalid tokens)
///
/// # Note
///
/// Validate returns 200 OK even for invalid tokens (result in response body)
///
/// # Use Cases
///
/// - CLI checks token validity before operations
/// - Dashboard validates token on page load
/// - Pre-flight check before batch operations
pub async fn validate(
  State(state): State<AuthState>,
  TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
  // Verify User Token
  let claims = match state.jwt_secret.verify_access_token(bearer.token()) {
    Ok(claims) => claims,
    Err(_err) => {
      // Token expired or invalid
      return (StatusCode::OK, Json(ValidateResponse::Invalid {
        valid: false,
        reason: "TOKEN_EXPIRED".to_string(),
        expired_at: Some(chrono::Utc::now().to_rfc3339()),
        revoked_at: None,
      })).into_response();
    }
  };

  // Check if token is blacklisted
  let blacklisted = user_auth::get_blacklisted_token(&state.db_pool, &claims.jti).await;

  let blacklisted = match blacklisted {
    Ok(blacklisted) => blacklisted,
    Err(err) => {
      tracing::error!("Failed to check if token is blacklisted: {}", err);
      return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
        error: ErrorDetail {
          code: "TOKEN_BLACKLIST_ERROR".to_string(),
          message: "Failed to check if token is blacklisted".to_string(),
          details: None,
        },
      })).into_response();
    }
  };

  if let Some(blacklisted) = blacklisted {
    let blacklisted_at = chrono::DateTime::from_timestamp(blacklisted.blacklisted_at, 0);

    match blacklisted_at {
      Some(timestamp) => {
        return (StatusCode::OK, Json(ValidateResponse::Invalid {
          valid: false,
          reason: "TOKEN_REVOKED".to_string(),
          expired_at: None,
          revoked_at: Some(timestamp.to_rfc3339()),
        })).into_response();
      },
      None => {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
          error: ErrorDetail {
            code: "TOKEN_BLACKLIST_ERROR".to_string(),
            message: "Failed to check if token is blacklisted".to_string(),
            details: None,
          },
        })).into_response();
      },
    }
  }

  // Fetch user to get current info
  let user = user_auth::get_user_by_id(&state.db_pool, &claims.sub).await;

  let user_option = match user {
    Ok(user) => user,
    Err(err) => {
      tracing::error!("Failed to fetch user: {}", err);
      return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
        error: ErrorDetail {
          code: "USER_FETCH_ERROR".to_string(),
          message: "Failed to fetch user".to_string(),
          details: None,
        },
      })).into_response();
    }
  };

  let user = match user_option {
    Some(user) => user,
    None => {
      return (StatusCode::NOT_FOUND, Json(ErrorResponse {
        error: ErrorDetail {
          code: "USER_NOT_FOUND".to_string(),
          message: "User not found".to_string(),
          details: None,
        },
      })).into_response();
    }
  };

  let expires_at = match chrono::DateTime::from_timestamp(claims.exp, 0) {
    Some( dt ) => dt,
    None => {
      tracing::error!( "Invalid expiration timestamp in JWT claims: {}", claims.exp );
      return ( StatusCode::BAD_REQUEST, Json( ErrorResponse {
        error: ErrorDetail {
          code: "INVALID_TOKEN".to_string(),
          message: "Token contains invalid expiration timestamp".to_string(),
          details: None,
        },
      } ) ).into_response();
    }
  };
  let expires_in = (expires_at - chrono::Utc::now()).num_seconds() as u64;

  // Placeholder response
  (
    StatusCode::OK,
    Json(ValidateResponse::Valid {
      valid: true,
      user: UserInfo::from_user(&user),
      expires_at: expires_at.to_rfc3339(),
      expires_in,
    }),
  )
    .into_response()
}
