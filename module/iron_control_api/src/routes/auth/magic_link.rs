//! Magic-link authentication handlers - Task 029 Slide 1
//!
//! Magic-link sign-in is the only authentication surface for the `FreeForm`
//! onboarding flow: the admin (or member) enters an email and receives a
//! one-time link; clicking it returns them authenticated, with no password.
//!
//! Endpoints:
//! - POST `/api/v1/auth/magic-link/send`   - issue a one-time link for an email (Public)
//! - POST `/api/v1/auth/magic-link/verify` - redeem a link, return a session (Public)
//!
//! Email delivery is out of scope (Task 029 "Out of scope"), so `send` returns
//! the link for the caller to copy rather than mailing it - the same copy-link
//! pattern used by invite generation.

use core::net::SocketAddr;

use axum::{
  extract::{ConnectInfo, State},
  http::StatusCode,
  response::{IntoResponse, Json},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::shared::{AuthState, ErrorDetail, ErrorResponse, UserInfo};

/// Lifetime of a magic link before it expires (15 minutes, in milliseconds).
const MAGIC_LINK_TTL_MS: i64 = 15 * 60 * 1000;

/// `BCrypt` cost for the placeholder password hash (matches the rest of the app).
const BCRYPT_COST: u32 = 12;

/// Access-token lifetime in seconds (30 days), mirroring the login handler.
const ACCESS_TOKEN_EXPIRES_IN: u64 = 2_592_000;

/// Request body for `POST /api/v1/auth/magic-link/send`.
#[derive(Debug, Deserialize)]
pub struct MagicLinkSendRequest {
  /// Email address to issue the magic link for.
  pub email: String,
}

/// Response from `POST /api/v1/auth/magic-link/send`.
///
/// Email delivery is out of scope, so the link is returned for copy.
#[derive(Debug, Serialize)]
pub struct MagicLinkSendResponse {
  /// Relative verification URL carrying the one-time token.
  pub magic_link: String,
  /// Link expiration timestamp (Unix epoch milliseconds).
  pub expires_at: i64,
}

/// Request body for `POST /api/v1/auth/magic-link/verify`.
#[derive(Debug, Deserialize)]
pub struct MagicLinkVerifyRequest {
  /// Raw one-time token extracted from the magic link.
  pub token: String,
}

/// Response from `POST /api/v1/auth/magic-link/verify`.
///
/// Mirrors `LoginResponse` plus `needs_registration`, which tells the frontend
/// whether to route the user to Complete Your Registration (Slides 2-4/15-17).
#[derive(Debug, Serialize)]
pub struct MagicLinkVerifyResponse {
  /// JWT access token.
  pub user_token: String,
  /// Token type, always "Bearer".
  pub token_type: String,
  /// Token lifetime in seconds.
  pub expires_in: u64,
  /// Token expiration timestamp (ISO 8601).
  pub expires_at: String,
  /// Optional refresh token.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub refresh_token: Option<String>,
  /// Authenticated user information.
  pub user: UserInfo,
  /// True when the user has not yet completed registration (no first name).
  pub needs_registration: bool,
}

/// Generate 32 random bytes encoded as URL-safe base64 (no padding).
fn generate_magic_token() -> String {
  let mut rng = rand::rng();
  let mut bytes = [0u8; 32];
  rng.fill(&mut bytes[..]);
  URL_SAFE_NO_PAD.encode(bytes)
}

/// SHA-256 of a token string, hex-encoded.
fn hash_token(token: &str) -> String {
  hex::encode(Sha256::digest(token.as_bytes()))
}

/// Current time in milliseconds since Unix epoch.
fn now_ms() -> i64 {
  let ms = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_millis())
    .unwrap_or(0);
  i64::try_from(ms).unwrap_or(i64::MAX)
}

/// Build a Protocol-007 error response.
fn error(status: StatusCode, code: &str, message: &str) -> axum::response::Response {
  (
    status,
    Json(ErrorResponse {
      error: ErrorDetail {
        code: code.to_string(),
        message: message.to_string(),
        details: None,
      },
    }),
  )
    .into_response()
}

/// `POST /api/v1/auth/magic-link/send`
///
/// Issue a one-time magic link for the given email. The link is returned in the
/// response body for the caller to copy (email delivery is out of scope). The
/// endpoint always returns 200 for a well-formed email so it never reveals
/// whether an account already exists.
pub async fn magic_link_send(
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  State(state): State<AuthState>,
  Json(request): Json<MagicLinkSendRequest>,
) -> impl IntoResponse {
  let email = request.email.trim();
  if email.is_empty() || email.len() > 255 || !email.contains('@') {
    return error(
      StatusCode::BAD_REQUEST,
      "VALIDATION_ERROR",
      "A valid email is required",
    );
  }

  // Per-IP rate limiting (reuses the login limiter) to prevent link spam.
  if state.rate_limiting_enabled {
    if let Err(retry_after_secs) = state.rate_limiter.check_and_record(addr.ip()) {
      return error(
        StatusCode::TOO_MANY_REQUESTS,
        "RATE_LIMIT_EXCEEDED",
        &format!("Too many requests. Please try again in {retry_after_secs} seconds."),
      );
    }
  }

  let raw_token = generate_magic_token();
  let token_hash = hash_token(&raw_token);
  let created_at = now_ms();
  let expires_at = created_at + MAGIC_LINK_TTL_MS;

  let insert = sqlx::query(
    r"
      INSERT INTO magic_link_tokens (token_hash, email, created_at, expires_at)
      VALUES (?, ?, ?, ?)
    ",
  )
  .bind(&token_hash)
  .bind(email)
  .bind(created_at)
  .bind(expires_at)
  .execute(&state.db_pool)
  .await;

  if let Err(err) = insert {
    tracing::error!("Failed to store magic-link token: {err}");
    return error(
      StatusCode::INTERNAL_SERVER_ERROR,
      "INTERNAL_ERROR",
      "Failed to issue magic link",
    );
  }

  (
    StatusCode::OK,
    Json(MagicLinkSendResponse {
      magic_link: format!("/auth/verify?token={raw_token}"),
      expires_at,
    }),
  )
    .into_response()
}

/// DB row for an unredeemed magic-link lookup.
#[derive(sqlx::FromRow)]
struct MagicTokenRow {
  email: String,
  expires_at: i64,
}

/// `POST /api/v1/auth/magic-link/verify`
///
/// Redeem a one-time token: mark it used, resolve (or create) the user for the
/// email, and return a session. `needs_registration` is true when the user has
/// not yet supplied their first name, routing the frontend to the registration
/// form. The first user ever created becomes an admin ("no separate sign-up").
pub async fn magic_link_verify(
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  State(state): State<AuthState>,
  Json(request): Json<MagicLinkVerifyRequest>,
) -> impl IntoResponse {
  if request.token.trim().is_empty() {
    return error(
      StatusCode::BAD_REQUEST,
      "VALIDATION_ERROR",
      "A token is required",
    );
  }

  if state.rate_limiting_enabled {
    if let Err(retry_after_secs) = state.rate_limiter.check_and_record(addr.ip()) {
      return error(
        StatusCode::TOO_MANY_REQUESTS,
        "RATE_LIMIT_EXCEEDED",
        &format!("Too many requests. Please try again in {retry_after_secs} seconds."),
      );
    }
  }

  let token_hash = hash_token(&request.token);

  let row = sqlx::query_as::<_, MagicTokenRow>(
    r"
      SELECT email, expires_at
      FROM magic_link_tokens
      WHERE token_hash = ? AND used_at IS NULL
    ",
  )
  .bind(&token_hash)
  .fetch_optional(&state.db_pool)
  .await
  .unwrap_or(None);

  let Some(row) = row else {
    return error(
      StatusCode::UNAUTHORIZED,
      "INVALID_TOKEN",
      "Magic link is invalid or already used",
    );
  };

  let now = now_ms();
  if row.expires_at < now {
    return error(
      StatusCode::UNAUTHORIZED,
      "TOKEN_EXPIRED",
      "Magic link has expired",
    );
  }

  // Consume the token atomically: only the request that flips used_at from NULL
  // proceeds, so a concurrent double-submit cannot redeem the same link twice.
  let consumed = sqlx::query(
    r"
      UPDATE magic_link_tokens SET used_at = ?
      WHERE token_hash = ? AND used_at IS NULL
    ",
  )
  .bind(now)
  .bind(&token_hash)
  .execute(&state.db_pool)
  .await;

  match consumed {
    Ok(result) if result.rows_affected() == 1 => {}
    Ok(_) => {
      return error(
        StatusCode::UNAUTHORIZED,
        "INVALID_TOKEN",
        "Magic link is invalid or already used",
      );
    }
    Err(err) => {
      tracing::error!("Failed to consume magic-link token: {err}");
      return error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_ERROR",
        "Failed to verify magic link",
      );
    }
  }

  // Resolve or create the user for this email.
  let existing = sqlx::query_as::<_, (String, String, Option<String>)>(
    "SELECT id, role, first_name FROM users WHERE email = ?",
  )
  .bind(&row.email)
  .fetch_optional(&state.db_pool)
  .await
  .unwrap_or(None);

  let (user_id, role, needs_registration) = if let Some((id, role, first_name)) = existing {
    (id, role, first_name.is_none())
  } else {
    // First user ever becomes admin (Slide 1: magic-link is the only signup).
    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
      .fetch_one(&state.db_pool)
      .await
      .unwrap_or(0);
    let role = if user_count == 0 {
      "admin"
    } else {
      "developer"
    };

    let user_id = Uuid::new_v4().to_string();
    let username = format!("user_{}", &user_id[..8]);
    // Magic-link users have no password; store a bcrypt hash of random bytes so
    // the NOT NULL column is satisfied and password login can never succeed.
    let placeholder = generate_magic_token();
    let Ok(password_hash) = bcrypt::hash(&placeholder, BCRYPT_COST) else {
      return error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_ERROR",
        "Failed to create account",
      );
    };

    let insert = sqlx::query(
      r"
        INSERT INTO users (id, username, password_hash, email, role, is_active, created_at)
        VALUES (?, ?, ?, ?, ?, 1, ?)
      ",
    )
    .bind(&user_id)
    .bind(&username)
    .bind(&password_hash)
    .bind(&row.email)
    .bind(role)
    .bind(now)
    .execute(&state.db_pool)
    .await;

    if let Err(err) = insert {
      tracing::error!("Failed to create magic-link user: {err}");
      return error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_ERROR",
        "Failed to create account",
      );
    }

    (user_id, role.to_string(), true)
  };

  // Issue the session (access + refresh), mirroring the login handler.
  let access_token_id = format!("access_{user_id}_{}", Uuid::new_v4());
  let Ok(user_token) =
    state
      .jwt_secret
      .generate_access_token(&user_id, &row.email, &role, &access_token_id)
  else {
    return error(
      StatusCode::INTERNAL_SERVER_ERROR,
      "TOKEN_GENERATION_ERROR",
      "Failed to generate access token",
    );
  };

  let refresh_token_id = format!("refresh_{user_id}_{now}");
  let refresh_token = state
    .jwt_secret
    .generate_refresh_token(&user_id, &row.email, &role, &refresh_token_id)
    .ok();

  let expires_at = chrono::Utc::now()
    + chrono::Duration::seconds(i64::try_from(ACCESS_TOKEN_EXPIRES_IN).unwrap_or(i64::MAX));

  (
    StatusCode::OK,
    Json(MagicLinkVerifyResponse {
      user_token,
      token_type: "Bearer".to_string(),
      expires_in: ACCESS_TOKEN_EXPIRES_IN,
      expires_at: expires_at.to_rfc3339(),
      refresh_token,
      user: UserInfo {
        id: user_id,
        email: row.email,
        role,
        name: String::new(),
      },
      needs_registration,
    }),
  )
    .into_response()
}
