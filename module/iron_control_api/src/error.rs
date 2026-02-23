//! Custom error types and JSON error responses for API
//!
//! Provides consistent JSON error responses per FR-5 specification.
//! All error responses follow the format:
//! ```json
//! {"error": "description", "code": "ERROR_CODE", "details": "optional details"}
//! ```

use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::Serialize;

/// Standard JSON error response format (FR-5)
///
/// All API errors return this structure to ensure consistent error handling
/// in frontend applications.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
  /// Human-readable error description
  pub error: String,
  /// Machine-readable error code
  #[serde(skip_serializing_if = "Option::is_none")]
  pub code: Option<String>,
  /// Additional error details
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<String>,
}

impl ErrorResponse {
  /// Create error response with just a message
  pub fn new(error: impl Into<String>) -> Self {
    Self {
      error: error.into(),
      code: None,
      details: None,
    }
  }

  /// Create error response with code
  pub fn with_code(error: impl Into<String>, code: impl Into<String>) -> Self {
    Self {
      error: error.into(),
      code: Some(code.into()),
      details: None,
    }
  }

  /// Create full error response with all fields
  pub fn with_details(
    error: impl Into<String>,
    code: impl Into<String>,
    details: impl Into<String>,
  ) -> Self {
    Self {
      error: error.into(),
      code: Some(code.into()),
      details: Some(details.into()),
    }
  }
}

impl IntoResponse for ErrorResponse {
  fn into_response(self) -> Response {
    (StatusCode::BAD_REQUEST, Json(self)).into_response()
  }
}

/// Custom extractor wrapper that provides JSON error responses for Path parameter failures
///
/// **Fix for Issue #2:** Axum's default `Path<T>` extractor returns plain text errors when
/// path parameter parsing fails (e.g., non-numeric ID). This wrapper intercepts those
/// failures and converts them to JSON error responses per FR-5.
///
/// **Usage:**
/// Replace `Path<T>` with `JsonPath<T>` in route handlers:
/// ```rust,ignore
/// // Before:
/// async fn get_limit( Path(id): Path<i64> ) { ... }
///
/// // After:
/// async fn get_limit( JsonPath(id): JsonPath<i64> ) { ... }
/// ```
///
/// **Pitfall:**
/// Axum's built-in extractors have default rejection responses. Always wrap extractors
/// or implement custom rejection handling for consistent API error responses.
#[derive(Debug)]
pub struct JsonPath<T>(pub T);

impl<T, S> axum::extract::FromRequestParts<S> for JsonPath<T>
where
  T: serde::de::DeserializeOwned + Send,
  S: Send + Sync,
{
  type Rejection = ErrorResponse;

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    match axum::extract::Path::<T>::from_request_parts(parts, state).await {
      Ok(value) => Ok(Self(value.0)),
      Err(rejection) => {
        // Convert Axum's path rejection to our JSON error format
        let error_msg = rejection.to_string();

        // Parse the error message to provide better context
        if error_msg.contains("Cannot parse") {
          Err(ErrorResponse::with_code(
            "Invalid path parameter",
            "INVALID_PARAMETER",
          ))
        } else {
          Err(ErrorResponse::new(error_msg))
        }
      }
    }
  }
}

/// Custom extractor wrapper that provides JSON error responses for request body parsing failures
///
/// **Fix for Issue #3:** Axum's default `Json<T>` extractor returns 422 Unprocessable Entity
/// for malformed JSON or type mismatches. This wrapper intercepts those failures and converts
/// them to 400 Bad Request with JSON error responses per FR-5 for consistency.
///
/// **Usage:**
/// Replace `Json<T>` with `JsonBody<T>` in route handlers:
/// ```rust,ignore
/// // Before:
/// async fn create_token( Json(request): Json<CreateTokenRequest> ) { ... }
///
/// // After:
/// async fn create_token( JsonBody(request): JsonBody<CreateTokenRequest> ) { ... }
/// ```
///
/// **Pitfall:**
/// Axum returns 422 for JSON parsing errors by default. For consistent client error handling,
/// wrap JSON extractors to return 400 instead.
#[derive(Debug)]
pub struct JsonBody<T>(pub T);

impl<T, S> axum::extract::FromRequest<S> for JsonBody<T>
where
  T: serde::de::DeserializeOwned,
  S: Send + Sync,
{
  type Rejection = (StatusCode, Json<ErrorResponse>);

  async fn from_request(
    req: axum::http::Request<axum::body::Body>,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    match axum::Json::<T>::from_request(req, state).await {
      Ok(value) => Ok(Self(value.0)),
      Err(rejection) => {
        // Convert Axum's JSON rejection (422) to 400 with JSON error format
        let error_msg = rejection.to_string();

        let error_response = if error_msg.contains("missing field") {
          ErrorResponse::with_code(
            format!("Missing required field: {error_msg}"),
            "MISSING_FIELD",
          )
        } else if error_msg.contains("invalid type") || error_msg.contains("expected") {
          ErrorResponse::with_code(
            "Invalid JSON: type mismatch or malformed structure",
            "INVALID_JSON",
          )
        } else {
          ErrorResponse::with_code("Malformed JSON request body", "MALFORMED_JSON")
        };

        Err((StatusCode::BAD_REQUEST, Json(error_response)))
      }
    }
  }
}

/// Validation error types for request validation
///
/// Replaces `Result<(), String>` with typed error handling per `code_design.rulebook.md`
#[derive(Debug)]
pub enum ValidationError {
  /// Missing required field
  MissingField(String),
  /// Invalid field value
  InvalidValue {
    /// Field name that failed validation
    field: String,
    /// Why the value is invalid
    reason: String,
  },
  /// Field value too long
  TooLong {
    /// Field name that exceeded length
    field: String,
    /// Maximum allowed length
    max_length: usize,
  },
  /// Field value too short
  TooShort {
    /// Field name that is too short
    field: String,
    /// Minimum required length
    min_length: usize,
  },
  /// Invalid format
  InvalidFormat {
    /// Field name with wrong format
    field: String,
    /// Expected format description
    expected: String,
  },
  /// Contains invalid character
  InvalidCharacter {
    /// Field name with invalid character
    field: String,
    /// The invalid character found
    character: String,
  },
  /// Custom validation error
  Custom(String),
}

impl core::fmt::Display for ValidationError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::MissingField(field) => write!(f, "{field} cannot be empty"),
      Self::InvalidValue { field, reason } => write!(f, "Invalid {field}: {reason}"),
      Self::TooLong { field, max_length } => {
        write!(f, "{field} too long (max {max_length} characters)")
      }
      Self::TooShort { field, min_length } => {
        write!(f, "{field} too short (min {min_length} characters)")
      }
      Self::InvalidFormat { field, expected } => write!(f, "Invalid {field}: must be {expected}"),
      Self::InvalidCharacter { field, character } => {
        write!(f, "{field} contains invalid {character} character)")
      }
      Self::Custom(msg) => write!(f, "{msg}"),
    }
  }
}

impl core::error::Error for ValidationError {}
