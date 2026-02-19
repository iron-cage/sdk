//! JWT authentication middleware
//!
//! Phase 4 Day 26: JWT Authentication implementation
//!
//! Per plan:
//! - JWT signing/verification logic
//! - Access token (1hr) + refresh token (7 days)
//! - Token blacklisting for logout

use axum::extract::FromRef;
use jsonwebtoken::{ encode, decode, Header, Validation, EncodingKey, DecodingKey };
use serde::{ Serialize, Deserialize };
use std::time::{ SystemTime, UNIX_EPOCH };

/// JWT claims for access tokens (30 days expiry)
#[ derive( Debug, Serialize, Deserialize, Clone ) ]
pub struct AccessTokenClaims
{
  /// User ID
  pub sub: String,
  /// User Role
  pub role: String,
  /// Issued at (Unix timestamp)
  /// User email
  pub email: String,
  pub iat: i64,
  /// Expiration time (Unix timestamp)
  pub exp: i64,
  /// Token ID for blacklist tracking
  pub jti: String,
}

/// JWT claims for refresh tokens (7 days expiry)
#[ derive( Debug, Serialize, Deserialize, Clone ) ]
pub struct RefreshTokenClaims
{
  /// User ID
  pub sub: String,
  /// User email
  pub email: String,
  /// User role
  pub role: String,
  /// Issued at (Unix timestamp)
  pub iat: u64,
  /// Expiration time (Unix timestamp)
  pub exp: u64,
  /// Token ID for blacklist tracking
  pub jti: String,
}

/// JWT secret manager
pub struct JwtSecret
{
  secret: String,
}

impl JwtSecret
{
  /// Create new JWT secret manager
  ///
  /// # Arguments
  ///
  /// * `secret` - Secret key for signing JWTs (should be from environment)
  #[ must_use ]
  pub fn new( secret: String ) -> Self
  {
    Self { secret }
  }

  /// Generate access token (30 days expiry)
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID to encode in token
  /// * `role` - User role to encode in token
  /// * `token_id` - Unique token ID for blacklist tracking
  ///
  /// # Errors
  ///
  /// Returns error if JWT encoding fails
  pub fn generate_access_token( &self, user_id: &str, email: &str,role: &str, token_id: &str ) -> Result< String, jsonwebtoken::errors::Error >
  {
    let now = chrono::Utc::now().timestamp();

    let claims = AccessTokenClaims
    {
      sub: user_id.to_string(),
      role: role.to_string(),
      iat: now,
      exp: now + 60 * 60 * 24 * 30, // 30 days
      email: email.to_string(),
      jti: token_id.to_string(),
    };

    encode(
      &Header::default(),
      &claims,
      &EncodingKey::from_secret( self.secret.as_bytes() ),
    )
  }

  /// Generate refresh token (7 days expiry)
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID to encode in token
  /// * `token_id` - Unique token ID for blacklist tracking
  ///
  /// # Errors
  ///
  /// Returns error if JWT encoding fails
  pub fn generate_refresh_token(
    &self,
    user_id: &str,
    email: &str,
    role: &str,
    token_id: &str,
  ) -> Result< String, jsonwebtoken::errors::Error >
  {
    let now = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .expect( "LOUD FAILURE: Time went backwards" )
      .as_secs();

    let claims = RefreshTokenClaims
    {
      sub: user_id.to_string(),
      email: email.to_string(),
      role: role.to_string(),
      iat: now,
      exp: now + ( 7 * 24 * 3600 ), // 7 days
      jti: token_id.to_string(),
    };

    encode(
      &Header::default(),
      &claims,
      &EncodingKey::from_secret( self.secret.as_bytes() ),
    )
  }

  /// Verify access token
  ///
  /// # Arguments
  ///
  /// * `token` - JWT token to verify
  ///
  /// # Errors
  ///
  /// Returns error if token is invalid or expired
  pub fn verify_access_token(
    &self,
    token: &str,
  ) -> Result< AccessTokenClaims, jsonwebtoken::errors::Error >
  {
    let token_data = decode::<AccessTokenClaims>(
      token,
      &DecodingKey::from_secret( self.secret.as_bytes() ),
      &Validation::default(),
    )?;

    Ok( token_data.claims )
  }

  /// Verify refresh token
  ///
  /// # Arguments
  ///
  /// * `token` - JWT token to verify
  ///
  /// # Errors
  ///
  /// Returns error if token is invalid or expired
  pub fn verify_refresh_token(
    &self,
    token: &str,
  ) -> Result< RefreshTokenClaims, jsonwebtoken::errors::Error >
  {
    let token_data = decode::<RefreshTokenClaims>(
      token,
      &DecodingKey::from_secret( self.secret.as_bytes() ),
      &Validation::default(),
    )?;

    Ok( token_data.claims )
  }
}

/// Axum extractor for authenticated user claims
///
/// Automatically extracts and verifies JWT access tokens from the `Authorization` header.
/// Returns 401 errors for missing, invalid, or expired tokens.
///
/// # Architecture
///
/// This extractor requires the application state to implement `FromRef<S>` for `AuthState`.
/// This allows the extractor to access the JWT secret for token verification without
/// coupling route handlers to a specific state structure.
///
/// # State Management Pattern
///
/// For combined application states (e.g., auth + database), implement `FromRef`:
///
/// ```rust
/// use iron_control_api::routes::auth::AuthState;
/// use axum::extract::FromRef;
///
/// #[derive(Clone)]
/// struct AppState {
///   auth: AuthState,
/// }
///
/// impl FromRef<AppState> for AuthState {
///   fn from_ref(state: &AppState) -> Self {
///     state.auth.clone()
///   }
/// }
/// ```
///
/// This pattern enables extractors to access only the state they need while
/// maintaining modularity and avoiding tight coupling.
///
/// # Usage in Route Handlers
///
/// ```rust
/// use iron_control_api::jwt_auth::AuthenticatedUser;
///
/// async fn my_handler( AuthenticatedUser( claims ): AuthenticatedUser ) {
///   let user_id = claims.sub;  // Extracted from verified JWT
///   // Route logic here
/// }
/// ```
///
/// # Pitfalls
///
/// **Pitfall:** If `FromRef<S>` is not implemented for your state type, compilation
/// will fail with "the trait bound `AuthState: FromRef<YourState>` is not satisfied".
///
/// **Solution:** Implement `FromRef<YourState> for AuthState` to expose the auth
/// sub-state to extractors. See example above.
///
/// # Security
///
/// - Tokens are verified using the JWT secret from `AuthState`
/// - Expired tokens are rejected (checked against `exp` claim)
/// - Invalid signatures are rejected (HMAC-SHA256 verification)
/// - Missing `Authorization` header returns 401 Unauthorized
/// - Malformed headers (not "Bearer <token>") return 401 Unauthorized
pub struct AuthenticatedUser( pub AccessTokenClaims );

#[ axum::async_trait ]
impl< S > axum::extract::FromRequestParts< S > for AuthenticatedUser
where
  S: Send + Sync,
  crate::routes::auth::AuthState: axum::extract::FromRef< S >,
{
  type Rejection = ( axum::http::StatusCode, axum::Json< serde_json::Value > );

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &S,
  ) -> Result< Self, Self::Rejection >
  {
    // Extract auth state
    let auth_state = crate::routes::auth::AuthState::from_ref( state );

    // Extract Authorization header
    let auth_header = parts
      .headers
      .get( axum::http::header::AUTHORIZATION )
      .and_then( |h| h.to_str().ok() )
      .ok_or_else( || (
        axum::http::StatusCode::UNAUTHORIZED,
        axum::Json( serde_json::json!({ "error": {
          "code": "AUTH_MISSING_TOKEN",
          "message": "Missing authentication token"
        } }) ),
      ) )?;

    // Parse "Bearer <token>" format
    let token = auth_header
      .strip_prefix( "Bearer ")
      .ok_or_else( || (
        axum::http::StatusCode::UNAUTHORIZED,
        axum::Json( serde_json::json!({ "error": {
          "code": "AUTH_MISSING_TOKEN",
          "message": "Missing authentication token"
        } }) ),
      ) )?;

    // Verify token and extract claims
    let claims = auth_state
      .jwt_secret
      .verify_access_token( token )
      .map_err( |_| (
        axum::http::StatusCode::UNAUTHORIZED,
        axum::Json( serde_json::json!({ "error": {
          "code": "AUTH_INVALID_TOKEN",
          "message": "Invalid or expired authentication token"
        } }) ),
      ) )?;

    Ok( AuthenticatedUser( claims ) )
  }
}

