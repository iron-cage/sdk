// ! IC Token (Iron Cage Token) JWT handling
//!
//! Protocol 005: Budget Control Protocol
//!
//! IC Tokens are JWT tokens that authenticate agent requests to the Control Panel.
//! They contain agent identity, budget allocation, and permissions.
//!
//! Key properties:
//! - Long-lived (expires_at can be null for no auto-expiration)
//! - Signed with HMAC-SHA256
//! - Issued by "iron-control-panel"
//! - Contains agent_id, budget_id, permissions

use crate::error::ValidationError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use core::fmt::{Display, Formatter, Result as FmtResult};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;
use uuid::Uuid;

/// Compute SHA-256 hash of a token string (hex-encoded)
pub fn sha256_hash(token: &str) -> String {
  format!("{:x}", Sha256::digest(token.as_bytes()))
}

/// Error from IC Token runtime validation (JWT + hash-check)
#[derive(Debug)]
pub enum IcTokenRuntimeError {
  /// JWT signature invalid, claims invalid, or expired
  InvalidToken(String),
  /// agent_id format invalid (not agent_<positive_number>)
  InvalidAgentId(String),
  /// Agent not found, token revoked (hash NULL), or token rotated (hash mismatch)
  TokenInactive(String),
  /// Database query failed
  DatabaseError(String),
}

impl Display for IcTokenRuntimeError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    match self {
      Self::InvalidToken(msg) => write!(f, "Invalid IC Token: {msg}"),
      Self::InvalidAgentId(msg) => write!(f, "Invalid agent_id: {msg}"),
      Self::TokenInactive(msg) => write!(f, "IC Token inactive: {msg}"),
      Self::DatabaseError(msg) => write!(f, "Database error: {msg}"),
    }
  }
}

impl std::error::Error for IcTokenRuntimeError {}

/// IC Token JWT claims
///
/// Per Protocol 005 specification, IC Tokens contain:
/// - jti: Unique token identifier (UUID v4) — guarantees uniqueness even when
///   two tokens are issued in the same second with identical claims
/// - agent_id: Unique agent identifier (format: agent_<id>)
/// - budget_id: Links to budget allocation
/// - issued_at: Token creation time (Unix timestamp seconds)
/// - expires_at: Optional expiration (null for long-lived tokens)
/// - issuer: Must be "iron-control-panel"
/// - permissions: Array of allowed operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IcTokenClaims {
  /// Unique token identifier (UUID v4, RFC 7519 §4.1.7)
  /// Ensures each JWT has a unique signature even when issued in the same second
  #[serde(rename = "jti")]
  pub token_id: Uuid,

  /// Agent identifier (format: agent_<id>)
  pub agent_id: String,

  /// Budget allocation identifier
  pub budget_id: String,

  /// Token creation time (Unix timestamp, seconds)
  #[serde(rename = "iat")]
  pub issued_at: u64,

  /// Optional expiration time (Unix timestamp, seconds)
  /// null = long-lived, no auto-expiration
  #[serde(rename = "exp", skip_serializing_if = "Option::is_none")]
  pub expires_at: Option<u64>,

  /// Token issuer (must be "iron-control-panel")
  #[serde(rename = "iss")]
  pub issuer: String,

  /// Allowed operations (e.g., ["llm:call", "data:read"])
  pub permissions: Vec<String>,
}

impl IcTokenClaims {
  /// Create new IC Token claims
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent identifier (format: agent_<id>)
  /// * `budget_id` - Budget allocation identifier
  /// * `permissions` - List of allowed operations
  /// * `expires_at` - Optional expiration time (None for long-lived)
  ///
  /// # Returns
  ///
  /// New `IcTokenClaims` with current timestamp as issued_at
  #[must_use]
  pub fn new(
    agent_id: String,
    budget_id: String,
    permissions: Vec<String>,
    expires_at: Option<u64>,
  ) -> Self {
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("LOUD FAILURE: Time went backwards")
      .as_secs();

    Self {
      token_id: Uuid::new_v4(),
      agent_id,
      budget_id,
      issued_at: now,
      expires_at,
      issuer: "iron-control-panel".to_string(),
      permissions,
    }
  }

  /// Validate IC Token claims
  ///
  /// Checks:
  /// - Issuer is "iron-control-panel"
  /// - If expires_at is set, token hasn't expired
  /// - agent_id format is valid
  ///
  /// # Errors
  ///
  /// Returns error if validation fails
  pub fn validate(&self) -> Result<(), ValidationError> {
    // Check issuer
    if self.issuer != "iron-control-panel" {
      return Err(ValidationError::InvalidValue {
        field: "issuer".to_string(),
        reason: format!("expected 'iron-control-panel', got '{}'", self.issuer),
      });
    }

    // Check expiration
    if let Some(exp) = self.expires_at {
      let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("LOUD FAILURE: Time went backwards")
        .as_secs();

      if now > exp {
        return Err(ValidationError::Custom("Token expired".to_string()));
      }
    }

    // Check agent_id format (basic validation)
    if !self.agent_id.starts_with("agent_") {
      return Err(ValidationError::InvalidFormat {
        field: "agent_id".to_string(),
        expected: "must start with 'agent_'".to_string(),
      });
    }

    Ok(())
  }
}

/// IC Token manager for generating and validating IC Tokens
pub struct IcTokenManager {
  secret: String,
}

impl IcTokenManager {
  /// Create new IC Token manager
  ///
  /// # Arguments
  ///
  /// * `secret` - Secret key for signing JWTs (should be from environment)
  #[must_use]
  pub fn new(secret: String) -> Self {
    Self { secret }
  }

  /// Generate IC Token JWT
  ///
  /// # Arguments
  ///
  /// * `claims` - IC Token claims to encode
  ///
  /// # Errors
  ///
  /// Returns error if JWT encoding fails
  pub fn generate_token(
    &self,
    claims: &IcTokenClaims,
  ) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
      &Header::default(),
      claims,
      &EncodingKey::from_secret(self.secret.as_bytes()),
    )
  }

  /// Verify and decode IC Token JWT
  ///
  /// # Arguments
  ///
  /// * `token` - JWT token string to verify
  ///
  /// # Errors
  ///
  /// Returns error if token is invalid, expired, or signature verification fails
  pub fn verify_token(&self, token: &str) -> Result<IcTokenClaims, String> {
    // Create custom validation that doesn't require exp claim
    // IC Tokens can have expires_at=null for long-lived tokens
    let mut validation = Validation::default();
    validation.required_spec_claims.clear(); // Dont require standard claims
    validation.validate_exp = false; // Manual expiration check in validate()

    // Decode JWT
    let token_data = decode::<IcTokenClaims>(
      token,
      &DecodingKey::from_secret(self.secret.as_bytes()),
      &validation,
    )
    .map_err(|e| format!("JWT decode error: {e}"))?;

    // Validate claims
    token_data.claims.validate().map_err(|e| e.to_string())?;

    Ok(token_data.claims)
  }

  // Fix(ic-token-invalidation): Add runtime hash-check to prevent revoked/rotated tokens
  // Root cause: Runtime endpoints only verified JWT signature, never checked if token was still active in database
  // Pitfall: Never rely on JWT signature alone for stateful authentication — always validate against authoritative state
  /// Validate IC Token for runtime endpoints: JWT verification + hash-check against database
  ///
  /// Validation steps:
  /// 1. Verify JWT signature and claims (existing `verify_token`)
  /// 2. Parse `agent_id` from claims (format: `agent_<id>`)
  /// 3. Hash raw token with SHA-256 and compare against `agents.ic_token_hash`
  /// 4. Reject if agent missing, hash NULL (revoked), or hash mismatch (rotated)
  ///
  /// # Arguments
  ///
  /// * `token` - Raw JWT token string
  /// * `pool` - Database connection pool for hash lookup
  ///
  /// # Returns
  ///
  /// `(agent_id, claims)` on success, `IcTokenRuntimeError` on failure
  pub async fn validate_ic_token_runtime(
    &self,
    token: &str,
    pool: &SqlitePool,
  ) -> Result<(i64, IcTokenClaims), IcTokenRuntimeError> {
    // 1. Verify JWT signature and claims
    let claims = self
      .verify_token(token)
      .map_err(IcTokenRuntimeError::InvalidToken)?;

    // 2. Parse agent_id (format: agent_<id>)
    let agent_id: i64 = claims
      .agent_id
      .strip_prefix("agent_")
      .ok_or_else(|| IcTokenRuntimeError::InvalidAgentId("Invalid agent_id format".to_string()))?
      .parse()
      .map_err(|_| {
        IcTokenRuntimeError::InvalidAgentId("Invalid agent_id - must be numeric".to_string())
      })?;

    if agent_id <= 0 {
      return Err(IcTokenRuntimeError::InvalidAgentId(
        "Invalid agent_id - must be positive".to_string(),
      ));
    }

    // 3. Hash raw token and compare with stored hash
    let token_hash = sha256_hash(token);

    let stored_hash: Option<Option<String>> =
      sqlx::query_scalar("SELECT ic_token_hash FROM agents WHERE id = ?")
        .bind(agent_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
          IcTokenRuntimeError::DatabaseError(format!("Database error fetching agent: {e}"))
        })?;

    // 4. Match stored hash
    match stored_hash {
      // Agent not found — generic error to prevent enumeration
      None => Err(IcTokenRuntimeError::TokenInactive(
        "Agent not found".to_string(),
      )),
      // ic_token_hash is NULL — token was revoked
      Some(None) => Err(IcTokenRuntimeError::TokenInactive(
        "IC Token has been revoked".to_string(),
      )),
      // Hash mismatch — token was rotated via regenerate
      // Constant-time comparison prevents timing attacks on hash extraction
      Some(Some(ref hash)) if hash.as_bytes().ct_eq(token_hash.as_bytes()).unwrap_u8() == 0 => Err(
        IcTokenRuntimeError::TokenInactive("IC Token has been invalidated".to_string()),
      ),
      // Hash matches — token is active
      Some(Some(_)) => Ok((agent_id, claims)),
    }
  }
}

/// Rate limiter configuration for IC Token validation
const IC_TOKEN_MAX_FAILURES: usize = 20;
const IC_TOKEN_WINDOW: Duration = Duration::from_secs(60);

/// Per-token-hash rate limiter for IC Token validation endpoints.
///
/// Tracks failed validation attempts keyed by a prefix of the token's SHA-256 hash.
/// After `IC_TOKEN_MAX_FAILURES` failures within `IC_TOKEN_WINDOW`, further attempts
/// with the same token are rejected with 429 Too Many Requests.
///
/// Design: per-token-hash (not per-IP) because:
/// 1. Runtime endpoints lack IP extractors (ConnectInfo<SocketAddr>)
/// 2. Per-token targeting is more precise — blocks compromised tokens without
///    affecting legitimate users behind the same NAT
/// 3. Matches the threat model: timing attacks require many requests with the same token
#[derive(Clone)]
pub struct IcTokenRateLimiter {
  /// Map from token hash prefix (16 hex chars) to list of failure timestamps
  failures: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl IcTokenRateLimiter {
  /// Create new rate limiter
  #[must_use]
  pub fn new() -> Self {
    Self {
      failures: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  /// Extract rate-limit key from raw token (first 16 chars of SHA-256 hash).
  /// Using a prefix saves memory while maintaining sufficient uniqueness.
  fn token_key(token: &str) -> String {
    sha256_hash(token)[..16].to_string()
  }

  /// Check if token is rate-limited. Returns `Err(retry_after_secs)` if blocked.
  /// Acquire lock (with poison recovery) and sweep all stale entries.
  fn lock_and_sweep(&self) -> (MutexGuard<'_, HashMap<String, Vec<Instant>>>, Instant) {
    let now = Instant::now();
    let mut map = self.failures.lock().unwrap_or_else(|e| {
      tracing::error!("IcTokenRateLimiter mutex poisoned, recovering");
      e.into_inner()
    });

    map.retain(|_, ts| {
      ts.retain(|t| now.duration_since(*t) < IC_TOKEN_WINDOW);
      !ts.is_empty()
    });

    (map, now)
  }

  /// Check if token is rate-limited. Returns `Err(retry_after_secs)` if blocked.
  pub fn check(&self, token: &str) -> Result<(), u64> {
    let key = Self::token_key(token);
    let (map, now) = self.lock_and_sweep();

    let Some(entries) = map.get(&key) else {
      return Ok(());
    };

    if entries.len() >= IC_TOKEN_MAX_FAILURES {
      let oldest = entries.first().unwrap();
      let retry_after = IC_TOKEN_WINDOW
        .saturating_sub(now.duration_since(*oldest))
        .as_secs()
        .max(1);
      return Err(retry_after);
    }

    Ok(())
  }

  /// Record a failed validation attempt for the given token.
  pub fn record_failure(&self, token: &str) {
    let key = Self::token_key(token);
    let (mut map, now) = self.lock_and_sweep();
    map.entry(key).or_default().push(now);
  }
}

impl Default for IcTokenRateLimiter {
  fn default() -> Self {
    Self::new()
  }
}

/// Target duration for all validation responses (success and error).
/// All paths are padded to this floor so external observers cannot
/// distinguish error types by response latency.
const VALIDATION_TIMING_TARGET: Duration = Duration::from_millis(5);

/// Validate IC Token for a runtime endpoint, returning HTTP response on error.
///
/// Consolidates the duplicated error handling pattern from budget, analytics,
/// and agent_provider_key endpoints into a single function.
///
/// Checks:
/// 1. Rate limit — rejects if token exceeded failure threshold (429)
/// 2. JWT signature + hash-check against database
/// 3. Timing padding to `VALIDATION_TIMING_TARGET`
///
/// On failure:
/// - Records failure in rate limiter (except database errors)
/// - Logs the error (error! for database, warn! for auth failures)
/// - Returns appropriate HTTP status (429/500/401)
///
/// On success: returns `(agent_id, claims)` for the endpoint to use.
pub async fn validate_ic_token_for_endpoint(
  manager: &IcTokenManager,
  token: &str,
  pool: &SqlitePool,
  rate_limiter: &IcTokenRateLimiter,
) -> Result<(i64, IcTokenClaims), axum::response::Response> {
  // 1. Check rate limit before doing any work
  if let Err(retry_after) = rate_limiter.check(token) {
    tracing::warn!("IC Token rate-limited (retry_after={retry_after}s)");
    return Err(
      (
        StatusCode::TOO_MANY_REQUESTS,
        [("retry-after", retry_after.to_string())],
        Json(serde_json::json!({ "error": "Too many failed attempts" })),
      )
        .into_response(),
    );
  }

  // 2. Validate token with timing padding
  let start = Instant::now();
  let result = manager.validate_ic_token_runtime(token, pool).await;

  // Pad to target duration so all paths (Ok / DatabaseError / auth error)
  // take the same wall-clock time from the caller's perspective.
  let elapsed = start.elapsed();
  if elapsed < VALIDATION_TIMING_TARGET {
    tokio::time::sleep(VALIDATION_TIMING_TARGET - elapsed).await;
  }

  match result {
    Ok(result) => Ok(result),
    Err(IcTokenRuntimeError::DatabaseError(e)) => {
      // Don't count database errors as auth failures — not the client's fault
      tracing::error!("IC Token validation database error: {e}");
      Err(
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(serde_json::json!({ "error": "Database error" })),
        )
          .into_response(),
      )
    }
    Err(e) => {
      rate_limiter.record_failure(token);
      tracing::warn!("IC Token validation failed: {e}");
      Err(
        (
          StatusCode::UNAUTHORIZED,
          Json(serde_json::json!({ "error": "Invalid IC Token" })),
        )
          .into_response(),
      )
    }
  }
}
