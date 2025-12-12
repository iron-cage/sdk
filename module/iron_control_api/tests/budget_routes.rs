//! Budget Control Protocol endpoint tests
//!
//! Protocol 005: Budget Control Protocol
//!
//! Tests for budget handshake, usage reporting, and budget refresh endpoints.
//! These tests validate request validation, IC Token verification, and response formatting.
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_handshake_request_validation` | HandshakeRequest validation | Valid/empty/oversized ic_token and provider fields | Valid passes, empty/oversized fail | ✅ |
//! | `test_usage_report_request_validation` | UsageReportRequest validation | Valid/empty/negative/oversized fields | Valid passes, invalid fail | ✅ |
//! | `test_budget_refresh_request_validation` | BudgetRefreshRequest validation | Valid/empty/negative/oversized fields | Valid passes, invalid fail | ✅ |
//! | `test_ic_token_lifecycle` | IC Token generation and validation | Generate IC Token, validate claims | Token valid, claims match input | ✅ |
//! | `test_ip_token_encryption` | IP Token encryption/decryption | Encrypt lease data, decrypt IP Token | Data round-trips correctly | ✅ |
//! | `test_ip_token_format_validation` | IP Token format validation | Valid/invalid IP Token formats | Valid passes, malformed fails | ✅ |
//! | `test_ic_token_expiration` | IC Token expiration handling | Create expired IC Token, validate | Validation fails for expired token | ✅ |
//! | `test_ic_token_agent_id_format` | IC Token agent_id format validation | Valid/invalid agent_id in claims | Valid passes, invalid fails | ✅ |
//! | `test_budget_state_creation` | BudgetState initialization | Create BudgetState with all components | State created successfully | ✅ |
//! | `test_handshake_response_serialization` | HandshakeResponse JSON serialization | Serialize HandshakeResponse to JSON | Valid JSON with required fields | ✅ |
//! | `test_usage_report_response_serialization` | UsageReportResponse JSON serialization | Serialize UsageReportResponse to JSON | Valid JSON with required fields | ✅ |
//! | `test_budget_refresh_response_serialization` | BudgetRefreshResponse JSON serialization | Serialize BudgetRefreshResponse to JSON | Valid JSON with required fields | ✅ |

use iron_control_api::
{
  ic_token::{ IcTokenClaims, IcTokenManager },
  ip_token::IpTokenCrypto,
  routes::budget::*,
};

/// Test handshake request validation
#[ test ]
fn test_handshake_request_validation()
{
  // Valid request
  let valid_req = HandshakeRequest
  {
    ic_token: "valid_token_string".to_string(),
    provider: "openai".to_string(),
    provider_key_id: Some( 1 ),
  };
  assert!( valid_req.validate().is_ok() );

  // Empty ic_token
  let empty_token_req = HandshakeRequest
  {
    ic_token: "".to_string(),
    provider: "openai".to_string(),
    provider_key_id: Some( 1 ),
  };
  assert!( empty_token_req.validate().is_err() );

  // Empty provider
  let empty_provider_req = HandshakeRequest
  {
    ic_token: "valid_token".to_string(),
    provider: "".to_string(),
    provider_key_id: Some( 1 ),
  };
  assert!( empty_provider_req.validate().is_err() );

  // ic_token too long (DoS prevention)
  let long_token = "a".repeat( 2001 );
  let long_token_req = HandshakeRequest
  {
    ic_token: long_token,
    provider: "openai".to_string(),
    provider_key_id: Some( 1 ),
  };
  assert!( long_token_req.validate().is_err() );

  // provider too long
  let long_provider = "a".repeat( 51 );
  let long_provider_req = HandshakeRequest
  {
    ic_token: "valid_token".to_string(),
    provider: long_provider,
    provider_key_id: Some( 1 ),
  };
  assert!( long_provider_req.validate().is_err() );
}

/// Test usage report request validation
#[ test ]
fn test_usage_report_request_validation()
{
  // Valid request
  let valid_req = UsageReportRequest
  {
    lease_id: "lease_abc123".to_string(),
    request_id: "req_xyz789".to_string(),
    tokens: 1000,
    cost_usd: 0.05,
    model: "gpt-4".to_string(),
    provider: "openai".to_string(),
  };
  assert!( valid_req.validate().is_ok() );

  // Empty lease_id
  let empty_lease_req = UsageReportRequest
  {
    lease_id: "".to_string(),
    request_id: "req_xyz789".to_string(),
    tokens: 1000,
    cost_usd: 0.05,
    model: "gpt-4".to_string(),
    provider: "openai".to_string(),
  };
  assert!( empty_lease_req.validate().is_err() );

  // Negative tokens
  let negative_tokens_req = UsageReportRequest
  {
    lease_id: "lease_abc123".to_string(),
    request_id: "req_xyz789".to_string(),
    tokens: -100,
    cost_usd: 0.05,
    model: "gpt-4".to_string(),
    provider: "openai".to_string(),
  };
  assert!( negative_tokens_req.validate().is_err() );

  // Zero tokens
  let zero_tokens_req = UsageReportRequest
  {
    lease_id: "lease_abc123".to_string(),
    request_id: "req_xyz789".to_string(),
    tokens: 0,
    cost_usd: 0.05,
    model: "gpt-4".to_string(),
    provider: "openai".to_string(),
  };
  assert!( zero_tokens_req.validate().is_err() );

  // Negative cost
  let negative_cost_req = UsageReportRequest
  {
    lease_id: "lease_abc123".to_string(),
    request_id: "req_xyz789".to_string(),
    tokens: 1000,
    cost_usd: -0.05,
    model: "gpt-4".to_string(),
    provider: "openai".to_string(),
  };
  assert!( negative_cost_req.validate().is_err() );
}

/// Test budget refresh request validation
#[ test ]
fn test_budget_refresh_request_validation()
{
  // Valid request with explicit budget
  let valid_req = BudgetRefreshRequest
  {
    ic_token: "valid_token_here".to_string(),
    current_lease_id: "lease_abc123".to_string(),
    requested_budget: Some( 10.0 ),
  };
  assert!( valid_req.validate().is_ok() );

  // Valid request with default budget (None)
  let default_budget_req = BudgetRefreshRequest
  {
    ic_token: "valid_token_here".to_string(),
    current_lease_id: "lease_abc123".to_string(),
    requested_budget: None,
  };
  assert!( default_budget_req.validate().is_ok() );

  // Empty ic_token
  let empty_token_req = BudgetRefreshRequest
  {
    ic_token: "".to_string(),
    current_lease_id: "lease_abc123".to_string(),
    requested_budget: Some( 10.0 ),
  };
  assert!( empty_token_req.validate().is_err() );

  // Empty current_lease_id
  let empty_lease_req = BudgetRefreshRequest
  {
    ic_token: "valid_token_here".to_string(),
    current_lease_id: "".to_string(),
    requested_budget: Some( 10.0 ),
  };
  assert!( empty_lease_req.validate().is_err() );

  // Zero budget request
  let zero_budget_req = BudgetRefreshRequest
  {
    ic_token: "valid_token_here".to_string(),
    current_lease_id: "lease_abc123".to_string(),
    requested_budget: Some( 0.0 ),
  };
  assert!( zero_budget_req.validate().is_err() );

  // Negative budget request
  let negative_budget_req = BudgetRefreshRequest
  {
    ic_token: "valid_token_here".to_string(),
    current_lease_id: "lease_abc123".to_string(),
    requested_budget: Some( -10.0 ),
  };
  assert!( negative_budget_req.validate().is_err() );

  // Budget request too large (DoS prevention)
  let large_budget_req = BudgetRefreshRequest
  {
    ic_token: "valid_token_here".to_string(),
    current_lease_id: "lease_abc123".to_string(),
    requested_budget: Some( 1001.0 ),
  };
  assert!( large_budget_req.validate().is_err() );
}

/// Test IC Token generation and validation
#[ test ]
fn test_ic_token_lifecycle()
{
  let manager = IcTokenManager::new( "test_secret_key_12345".to_string() );

  // Create IC Token claims
  let claims = IcTokenClaims::new(
    "agent_123".to_string(),
    "budget_456".to_string(),
    vec![ "llm:call".to_string(), "data:read".to_string() ],
    None, // No expiration (long-lived)
  );

  // Validate claims
  assert!( claims.validate().is_ok() );

  // Generate IC Token JWT
  let token = manager.generate_token( &claims ).expect( "Should generate IC Token" );
  assert!( !token.is_empty() );

  // Verify IC Token
  let verified_claims = manager.verify_token( &token ).expect( "Should verify IC Token" );
  assert_eq!( verified_claims.agent_id, "agent_123" );
  assert_eq!( verified_claims.budget_id, "budget_456" );
  assert_eq!( verified_claims.permissions, vec![ "llm:call", "data:read" ] );
}

/// Test IP Token encryption and decryption
#[ test ]
fn test_ip_token_encryption()
{
  // Create 32-byte encryption key
  let key : [ u8; 32 ] = [ 0u8; 32 ]; // In production, use random key

  let crypto = IpTokenCrypto::new( &key ).expect( "Should create IP Token crypto" );

  // Test provider API key
  let provider_key = "sk-proj_test_key_abc123";

  // Encrypt
  let ip_token = crypto.encrypt( provider_key ).expect( "Should encrypt" );
  assert!( ip_token.starts_with( "AES256:" ) );

  // Decrypt
  let decrypted = crypto.decrypt( &ip_token ).expect( "Should decrypt" );
  assert_eq!( &*decrypted, provider_key );
}

/// Test IP Token format validation
#[ test ]
fn test_ip_token_format_validation()
{
  let key : [ u8; 32 ] = [ 0u8; 32 ];
  let crypto = IpTokenCrypto::new( &key ).expect( "Should create crypto" );

  // Invalid format - missing parts
  let invalid_token1 = "AES256:abc:def";
  assert!( crypto.decrypt( invalid_token1 ).is_err() );

  // Invalid format - wrong prefix
  let invalid_token2 = "INVALID:abc:def:ghi";
  assert!( crypto.decrypt( invalid_token2 ).is_err() );

  // Invalid format - bad base64
  let invalid_token3 = "AES256:!!invalid!!:!!invalid!!:!!invalid!!";
  assert!( crypto.decrypt( invalid_token3 ).is_err() );
}

/// Test IC Token expiration validation
#[ test ]
fn test_ic_token_expiration()
{
  // Create expired IC Token (expires_at in the past)
  let expired_claims = IcTokenClaims::new(
    "agent_123".to_string(),
    "budget_456".to_string(),
    vec![ "llm:call".to_string() ],
    Some( 1 ), // Expired timestamp (1 second after epoch)
  );

  // Validation should fail
  assert!( expired_claims.validate().is_err() );

  // Create long-lived IC Token (no expiration)
  let long_lived_claims = IcTokenClaims::new(
    "agent_123".to_string(),
    "budget_456".to_string(),
    vec![ "llm:call".to_string() ],
    None, // No expiration
  );

  // Validation should succeed
  assert!( long_lived_claims.validate().is_ok() );
}

/// Test IC Token agent_id format validation
#[ test ]
fn test_ic_token_agent_id_format()
{
  // Valid agent_id format
  let valid_claims = IcTokenClaims::new(
    "agent_123".to_string(),
    "budget_456".to_string(),
    vec![ "llm:call".to_string() ],
    None,
  );
  assert!( valid_claims.validate().is_ok() );

  // Invalid agent_id format (missing prefix)
  let mut invalid_claims = IcTokenClaims::new(
    "123".to_string(), // Missing "agent_" prefix
    "budget_456".to_string(),
    vec![ "llm:call".to_string() ],
    None,
  );
  assert!( invalid_claims.validate().is_err() );

  // Test issuer validation
  invalid_claims.agent_id = "agent_123".to_string();
  invalid_claims.issuer = "malicious-issuer".to_string();
  assert!( invalid_claims.validate().is_err() );
}

/// Test budget state initialization
#[ tokio::test ]
async fn test_budget_state_creation()
{
  let ic_token_secret = "test_secret_key_12345".to_string();
  let ip_token_key : [ u8; 32 ] = [ 0u8; 32 ];
  let jwt_secret = std::sync::Arc::new( iron_control_api::jwt_auth::JwtSecret::new( "test_jwt_secret".to_string() ) );
  let database_url = "sqlite::memory:";

  let state = BudgetState::new( ic_token_secret, &ip_token_key, jwt_secret, database_url ).await;
  assert!( state.is_ok(), "Should create budget state" );
}

/// Test handshake response serialization
#[ test ]
fn test_handshake_response_serialization()
{
  let response = HandshakeResponse
  {
    ip_token: "AES256:abc:def:ghi".to_string(),
    lease_id: "lease_123".to_string(),
    budget_granted: 10.0,
    budget_remaining: 90.0,
    expires_at: None,
  };

  let json = serde_json::to_string( &response ).expect( "Should serialize" );
  assert!( json.contains( "ip_token" ) );
  assert!( json.contains( "lease_id" ) );
  assert!( json.contains( "budget_granted" ) );
}

/// Test usage report response serialization
#[ test ]
fn test_usage_report_response_serialization()
{
  let response = UsageReportResponse
  {
    success: true,
    budget_remaining: 9.5,
  };

  let json = serde_json::to_string( &response ).expect( "Should serialize" );
  assert!( json.contains( "success" ) );
  assert!( json.contains( "budget_remaining" ) );
}

/// Test budget refresh response serialization
#[ test ]
fn test_budget_refresh_response_serialization()
{
  // Approved response
  let approved = BudgetRefreshResponse
  {
    status: "approved".to_string(),
    budget_granted: Some( 10.0 ),
    budget_remaining: 80.0,
    lease_id: Some( "lease_456".to_string() ),
    reason: None,
  };

  let json = serde_json::to_string( &approved ).expect( "Should serialize" );
  assert!( json.contains( "approved" ) );
  assert!( json.contains( "budget_granted" ) );

  // Denied response
  let denied = BudgetRefreshResponse
  {
    status: "denied".to_string(),
    budget_granted: None,
    budget_remaining: 0.0,
    lease_id: None,
    reason: Some( "insufficient_budget".to_string() ),
  };

  let json = serde_json::to_string( &denied ).expect( "Should serialize" );
  assert!( json.contains( "denied" ) );
  assert!( json.contains( "insufficient_budget" ) );
}
