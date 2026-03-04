//! Error types for server initialization and runtime.

use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};

use core::{
  error::Error,
  fmt::{Display, Formatter, Result as FmtResult},
};

/// Errors that occur during server initialization or runtime.
///
/// Distinct from `ProxyError` which handles per-request HTTP errors.
/// `ServerError` covers startup failures: database, crypto, networking.
#[derive(Debug)]
pub enum ServerError {
  /// Database connection or migration failure.
  Database(String),
  /// Crypto service initialization failure (invalid or missing master key).
  Crypto(String),
  /// HTTP client builder failure.
  HttpClient(String),
  /// Pricing data load failure.
  Pricing(String),
  /// TCP listener bind or axum serve failure.
  Server(String),
}

impl Display for ServerError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    match self {
      Self::Database(msg) => write!(f, "Database error: {msg}"),
      Self::Crypto(msg) => write!(f, "Crypto init error: {msg}"),
      Self::HttpClient(msg) => write!(f, "HTTP client error: {msg}"),
      Self::Pricing(msg) => write!(f, "Pricing load error: {msg}"),
      Self::Server(msg) => write!(f, "Server error: {msg}"),
    }
  }
}

impl Error for ServerError {}

/// Proxy error types with OpenAI-compatible error responses.
#[derive(Debug)]
pub enum ProxyError {
  /// 401 - missing, invalid, or revoked IC Token.
  Unauthorized(&'static str),
  /// 403 - no key assigned, key disabled.
  Forbidden(&'static str),
  /// 402 - spending cap exceeded for provider key.
  SpendingCapExceeded,
  /// 429 - too many failed auth attempts from this IP.
  RateLimited(u64),
  /// 400 - request body too large or malformed.
  BadRequest(&'static str),
  /// 502 - LLM provider unreachable or returned error.
  BadGateway(String),
  /// 500 - database query failed.
  Database(sqlx::Error),
  /// 500 - decryption error or other internal failure.
  Internal(String),
}

impl IntoResponse for ProxyError {
  fn into_response(self) -> Response {
    let (status, message, code) = match &self {
      Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, *msg, "invalid_token"),
      Self::Forbidden(msg) => (StatusCode::FORBIDDEN, *msg, "forbidden"),
      Self::SpendingCapExceeded => (
        StatusCode::PAYMENT_REQUIRED,
        "Spending cap exceeded for this provider key",
        "spending_cap_exceeded",
      ),
      Self::RateLimited(_) => (
        StatusCode::TOO_MANY_REQUESTS,
        "Too many failed authentication attempts",
        "rate_limited",
      ),
      Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, *msg, "bad_request"),
      Self::BadGateway(msg) => (StatusCode::BAD_GATEWAY, msg.as_str(), "upstream_error"),
      Self::Database(_) | Self::Internal(_) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error",
        "internal_error",
      ),
    };

    // Log server-side errors for debugging (no secrets in messages)
    match &self {
      Self::Database(e) => tracing::error!("Database error in proxy: {e}"),
      Self::Internal(e) => tracing::error!("Internal error in proxy: {e}"),
      _ => {}
    }

    let body = iron_llm_core::create_openai_error(message, "error", code);
    let mut response = (status, Json(body)).into_response();

    // Add Retry-After header for rate-limited responses
    if let Self::RateLimited(retry_after) = &self {
      response.headers_mut().insert(
        "retry-after",
        retry_after.to_string().parse().expect("valid header value"),
      );
    }

    response
  }
}
