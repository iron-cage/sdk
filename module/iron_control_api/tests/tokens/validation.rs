//! CreateTokenRequest validation tests.
//!
//! Test Matrix: CreateTokenRequest validation
//!
//! | Field | Valid Values | Invalid Values | Edge Cases |
//! |-------|--------------|----------------|------------|
//! | user_id | Non-empty string | "" (empty) | 1 char OK |
//! | project_id | None, Some(non-empty) | Some("") | - |
//! | description | None, Some(≤500 chars) | Some(>500 chars) | Exactly 500 OK |
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_valid_minimal_request` | Valid request with only user_id | user_id=Some("user_test"), all else None | Validation passes | ✅ |
//! | `test_valid_complete_request` | Valid request with all fields | All fields populated with valid values | Validation passes | ✅ |
//! | `test_empty_user_id_rejected` | Empty user_id validation | user_id=Some("") | Validation error "user_id cannot be empty" | ✅ |
//! | `test_empty_project_id_rejected` | Empty project_id validation | project_id=Some("") | Validation error "project_id cannot be empty" | ✅ |
//! | `test_description_too_long_rejected` | Description exceeds max length | description=Some(501 chars) | Validation error "description too long (max 500)" | ✅ |
//! | `test_description_max_length_accepted` | Description at max length | description=Some(500 chars) | Validation passes | ✅ |
//! | `test_single_char_user_id_accepted` | Minimal user_id length | user_id=Some("a") | Validation passes | ✅ |
//! | `test_whitespace_user_id_rejected` | Whitespace-only user_id | user_id=Some("   ") | Validation error "user_id cannot be empty" | ✅ |
//! | `test_whitespace_project_id_rejected` | Whitespace-only project_id | project_id=Some("   ") | Validation error "project_id cannot be empty" | ✅ |

use iron_control_api::routes::tokens::CreateTokenRequest;

/// Test valid request with only user_id.
#[ tokio::test ]
async fn test_valid_minimal_request()
{
  let request = CreateTokenRequest
  {
    name: None,
    user_id: Some( "user_test".to_string() ),
    project_id: None,
    description: None,
    agent_id: None,
    provider: None,
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Valid minimal request must pass validation. Error: {:?}",
    result.err()
  );
}

/// Test valid request with all fields.
#[ tokio::test ]
async fn test_valid_complete_request()
{
  let request = CreateTokenRequest
  {
    name: None,
    user_id: Some( "user_test".to_string() ),
    project_id: Some( "project_abc".to_string() ),
    description: Some( "Production API key".to_string() ),
    agent_id: None,
    provider: None,
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Valid complete request must pass validation. Error: {:?}",
    result.err()
  );
}

/// Test empty user_id rejected.
#[ tokio::test ]
async fn test_empty_user_id_rejected()
{
  let request = CreateTokenRequest
  {
    name: None,
    user_id: Some( "".to_string() ),
    project_id: None,
    description: None,
    agent_id: None,
    provider: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Empty user_id must be rejected"
  );

  let err_msg = result.unwrap_err();
  assert!(
    err_msg.contains( "user_id" ) && ( err_msg.contains( "empty" ) || err_msg.contains( "required" ) ),
    "LOUD FAILURE: Error message must indicate user_id is empty/required. Got: {}",
    err_msg
  );
}

/// Test empty project_id rejected.
#[ tokio::test ]
async fn test_empty_project_id_rejected()
{
  let request = CreateTokenRequest
  {
    name: None,
    user_id: Some( "user_test".to_string() ),
    project_id: Some( "".to_string() ),
    description: None,
    agent_id: None,
    provider: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Empty project_id must be rejected"
  );

  let err_msg = result.unwrap_err();
  assert!(
    err_msg.contains( "project_id" ) && err_msg.contains( "empty" ),
    "LOUD FAILURE: Error message must indicate project_id cannot be empty. Got: {}",
    err_msg
  );
}

/// Test description too long rejected.
#[ tokio::test ]
async fn test_description_too_long_rejected()
{
  let long_description = "a".repeat( 501 );
  let request = CreateTokenRequest
  {
    name: None,
    user_id: Some( "user_test".to_string() ),
    project_id: None,
    description: Some( long_description ),
    agent_id: None,
    provider: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Description exceeding 500 chars must be rejected"
  );

  let err_msg = result.unwrap_err();
  assert!(
    err_msg.contains( "description" ) && ( err_msg.contains( "long" ) || err_msg.contains( "500" ) ),
    "LOUD FAILURE: Error message must indicate description too long. Got: {}",
    err_msg
  );
}

/// Test description exactly 500 chars accepted.
#[ tokio::test ]
async fn test_description_max_length_accepted()
{
  let max_description = "a".repeat( 500 );
  let request = CreateTokenRequest
  {
    name: None,
    user_id: Some( "user_test".to_string() ),
    project_id: None,
    description: Some( max_description ),
    agent_id: None,
    provider: None,
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Description with exactly 500 chars must be accepted. Error: {:?}",
    result.err()
  );
}

/// Test single char user_id accepted.
#[ tokio::test ]
async fn test_single_char_user_id_accepted()
{
  let request = CreateTokenRequest
  {
    name: None,
    user_id: Some( "a".to_string() ),
    project_id: None,
    description: None,
    agent_id: None,
    provider: None,
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Single char user_id must be accepted. Error: {:?}",
    result.err()
  );
}

/// Test whitespace-only user_id rejected.
#[ tokio::test ]
async fn test_whitespace_user_id_rejected()
{
  let request = CreateTokenRequest
  {
    name: None,
    user_id: Some( "   ".to_string() ),
    project_id: None,
    description: None,
    agent_id: None,
    provider: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Whitespace-only user_id must be rejected"
  );

  let err_msg = result.unwrap_err();
  assert!(
    err_msg.contains( "user_id" ) && ( err_msg.contains( "empty" ) || err_msg.contains( "whitespace" ) ),
    "LOUD FAILURE: Error message must indicate user_id is empty/whitespace. Got: {}",
    err_msg
  );
}

/// Test whitespace-only project_id rejected.
#[ tokio::test ]
async fn test_whitespace_project_id_rejected()
{
  let request = CreateTokenRequest
  {
    name: None,
    user_id: Some( "user_test".to_string() ),
    project_id: Some( "   ".to_string() ),
    description: None,
    agent_id: None,
    provider: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Whitespace-only project_id must be rejected"
  );

  let err_msg = result.unwrap_err();
  assert!(
    err_msg.contains( "project_id" ) && ( err_msg.contains( "empty" ) || err_msg.contains( "whitespace" ) ),
    "LOUD FAILURE: Error message must indicate project_id is empty/whitespace. Got: {}",
    err_msg
  );
}
