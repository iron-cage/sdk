//! Pure business logic handlers for CLI commands
//!
//! ## Architecture
//!
//! Handlers are pure functions: `HashMap<String, String> → Result<String>`
//! - No I/O operations (no async, no .await)
//! - No external service calls
//! - Only validation and formatting
//!
//! ## Implementation (Phase 2 Complete, Phase 4 User Management Added)
//!
//! **Handlers:** 30 pure functions across 7 categories
//! **Tests:** 137 total (127 handlers + 10 validation helpers)
//! **Coverage:** 100% handler test coverage
//!
//! ## Testing Strategy
//!
//! All handlers tested with real implementations (no mocking).
//! Tests located in tests/handlers/ directory.
//! Test matrix documents all 100 test cases: `tests/handlers/-test_matrix.md`
//!
//! ## Handler Signature
//!
//! ```rust,ignore
//! pub fn handler_name(
//!     params: &HashMap<String, String>,
//! ) -> Result<String, CliError>
//! {
//!     // 1. Validate required parameters (MissingParameter)
//!     // 2. Validate formats/patterns (InvalidParameter)
//!     // 3. Format output string
//!     // 4. Return result
//! }
//! ```
//!
//! ## Validation Helpers
//!
//! Common validation patterns extracted to `validation` module:
//! - `validate_token_id()` - validates tok_* format
//! - `validate_non_empty()` - ensures non-empty strings
//! - `validate_non_negative_integer()` - parses positive integers
//! - `validate_date_format()` - validates YYYY-MM-DD dates
//!
//! ## Known Limitations (Phase 2)
//!
//! 1. **Format parameter not validated**: Accepts any string.
//!    Will be validated in Phase 3 when Formatter is implemented.
//!
//! 2. **Output is simple text**: Not using structured formatting yet.
//!    Will be improved in Phase 3 with Formatter.
//!
//! 3. **No actual data loading**: Handlers return placeholder strings.
//!    Data loading happens in adapter layer (Phase 4).

mod error;
pub use error::CliError;

pub mod validation;

pub mod auth_handlers;
pub mod token_handlers;
pub mod usage_handlers;
pub mod limits_handlers;
pub mod traces_handlers;
pub mod health_handlers;
pub mod user_handlers;

// Control API handlers (for iron binary)
pub mod control;
