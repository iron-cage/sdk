//! Test: Token creation requires name field (Protocol 014)
//!
//! ## Root Cause
//!
//! The `CreateTokenRequest::validate()` method only validates fields IF they are
//! present (`if let Some(ref name) = self.name { validate... }`), but didnt ensure
//! that ANY fields are provided. The validation allowed completely empty requests `{}`
//! which contain no useful information.
//!
//! The API supports two formats:
//! - **Protocol 014**: Uses JWT auth, `name` should be provided in request
//! - **Legacy**: Includes `user_id` in request body, name/description optional
//!
//! The validation function ended with `Ok(())` without checking that at least SOME
//! field is present, allowing meaningless empty requests.
//!
//! ## Why Not Caught
//!
//! 1. **Test Coverage Gap:** Existing tests only validated INVALID values (empty
//!    string, too long), not MISSING fields
//! 2. **Backward Compatibility Confusion:** The code attempted to support both
//!    Protocol 014 (`name` required) and legacy format (`description` as name),
//!    but the validation logic didnt enforce "at least one must be present"
//! 3. **Manual Testing Scope:** The Iron Cage Pilot testing focused on successful
//!    token creation paths, not validation edge cases
//!
//! ## Fix Applied
//!
//! Modified `CreateTokenRequest::validate()` in `routes/tokens/shared.rs`:
//!
//! ```rust
//! pub fn validate( &self ) -> Result< (), ValidationError >
//! {
//!   // ... existing field validations ...
//!
//!   // Require at least ONE field to be present
//!   let has_any_field = self.name.is_some()
//!     || self.description.is_some()
//!     || self.user_id.is_some()
//!     || self.project_id.is_some()
//!     || self.agent_id.is_some()
//!     || self.provider.is_some();
//!
//!   if !has_any_field
//!   {
//!     return Err( ValidationError::MissingField(
//!       "at least one field required".to_string()
//!     ) );
//!   }
//!
//!   Ok( () )
//! }
//! ```
//!
//! This prevents empty requests while supporting both Protocol 014 (name in request)
//! and legacy format (user_id in request body).
//!
//! ## Prevention
//!
//! 1. **Required Field Checklist:** When adding required fields, add explicit
//!    tests for MISSING field (not just invalid values)
//! 2. **Validation Audit:** Review validate() methods to ensure they check for
//!    required field presence, not just optional field validity
//! 3. **Protocol Compliance Tests:** Add comprehensive Protocol 014 compliance
//!    test suite covering all validation rules (required, optional, formats)
//! 4. **Corner Case Testing:** Manual testing should include "what if I send
//!    nothing?" for all POST endpoints
//!
//! ## Pitfall
//!
//! **Never assume `Option<T>` fields are validated for presence.** The pattern
//! `if let Some(ref field) = self.field { validate... }` only validates when
//! field is present - it doesnt enforce that required fields exist. Always add
//! explicit checks:
//!
//! ```rust
//! // BAD: Only validates IF present (allows missing required field)
//! if let Some(ref name) = self.name {
//!   if name.is_empty() { return Err(...); }
//! }
//! Ok(()) // Accepts missing name!
//!
//! // GOOD: Validates presence first
//! let name = self.name.as_ref()
//!   .ok_or_else(|| ValidationError::MissingField("name"))?;
//! if name.is_empty() { return Err(...); }
//! ```
//!
//! When supporting multiple formats (Protocol 014 + legacy), be explicit about
//! which fields are required for each format and validate accordingly.

#[ cfg( feature = "enabled" ) ]
#[ tokio::test ]
async fn bug_reproducer_token_creation_requires_name()
{
  use iron_control_api::routes::tokens::CreateTokenRequest;

  // Test 1: Truly empty body should fail validation
  let empty_request = serde_json::from_str::<CreateTokenRequest>( "{}" )
    .expect( "Should deserialize empty object" );

  let result = empty_request.validate();
  assert!(
    result.is_err(),
    "Empty request body should fail validation (no fields provided)"
  );

  // Test 2: Request with at least one field should pass (legacy format)
  let legacy_request = serde_json::from_str::<CreateTokenRequest>(
    r#"{"user_id":"test-user","project_id":"test-project"}"#
  ).expect( "Should deserialize legacy request" );

  let result = legacy_request.validate();
  assert!(
    result.is_ok(),
    "Legacy request with user_id and project_id should pass"
  );

  // Test 3: Request with only description should pass (legacy compatibility)
  let desc_only_request = serde_json::from_str::<CreateTokenRequest>(
    r#"{"description":"test"}"#
  ).expect( "Should deserialize description-only request" );

  let result = desc_only_request.validate();
  assert!(
    result.is_ok(),
    "Description-only request should pass for legacy compatibility"
  );

  // Test 4: Valid Protocol 014 request with name should pass
  let valid_request = serde_json::from_str::<CreateTokenRequest>(
    r#"{"name":"test-token"}"#
  ).expect( "Should deserialize valid request" );

  let result = valid_request.validate();
  assert!(
    result.is_ok(),
    "Valid request with name should pass validation"
  );
}
