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

use jsonwebtoken::{ decode, encode, DecodingKey, EncodingKey, Header, Validation };
use serde::{ Deserialize, Serialize };
use std::time::{ SystemTime, UNIX_EPOCH };

/// IC Token JWT claims
///
/// Per Protocol 005 specification, IC Tokens contain:
/// - agent_id: Unique agent identifier (format: agent_<id>)
/// - budget_id: Links to budget allocation
/// - issued_at: Token creation time (Unix timestamp seconds)
/// - expires_at: Optional expiration (null for long-lived tokens)
/// - issuer: Must be "iron-control-panel"
/// - permissions: Array of allowed operations
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct IcTokenClaims
{
  /// Agent identifier (format: agent_<id>)
  pub agent_id: String,

  /// Budget allocation identifier
  pub budget_id: String,

  /// Token creation time (Unix timestamp, seconds)
  #[ serde( rename = "iat" ) ]
  pub issued_at: u64,

  /// Optional expiration time (Unix timestamp, seconds)
  /// null = long-lived, no auto-expiration
  #[ serde( rename = "exp", skip_serializing_if = "Option::is_none" ) ]
  pub expires_at: Option< u64 >,

  /// Token issuer (must be "iron-control-panel")
  #[ serde( rename = "iss" ) ]
  pub issuer: String,

  /// Allowed operations (e.g., ["llm:call", "data:read"])
  pub permissions: Vec< String >,
}

impl IcTokenClaims
{
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
  #[ must_use ]
  pub fn new(
    agent_id: String,
    budget_id: String,
    permissions: Vec< String >,
    expires_at: Option< u64 >,
  ) -> Self
  {
    let now = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .expect( "Time went backwards" )
      .as_secs();

    Self {
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
  /// - If expires_at is set, token hasnt expired
  /// - agent_id format is valid
  ///
  /// # Errors
  ///
  /// Returns error if validation fails
  pub fn validate( &self ) -> Result< (), String >
  {
    // Check issuer
    if self.issuer != "iron-control-panel"
    {
      return Err( format!( "Invalid issuer: {}", self.issuer ) );
    }

    // Check expiration
    if let Some( exp ) = self.expires_at
    {
      let now = SystemTime::now()
        .duration_since( UNIX_EPOCH )
        .expect( "Time went backwards" )
        .as_secs();

      if now > exp
      {
        return Err( "Token expired".to_string() );
      }
    }

    // Check agent_id format (basic validation)
    if !self.agent_id.starts_with( "agent_" )
    {
      return Err( format!( "Invalid agent_id format: {}", self.agent_id ) );
    }

    Ok( () )
  }
}

/// IC Token manager for generating and validating IC Tokens
pub struct IcTokenManager
{
  secret: String,
}

impl IcTokenManager
{
  /// Create new IC Token manager
  ///
  /// # Arguments
  ///
  /// * `secret` - Secret key for signing JWTs (should be from environment)
  #[ must_use ]
  pub fn new( secret: String ) -> Self
  {
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
  pub fn generate_token( &self, claims: &IcTokenClaims ) -> Result< String, jsonwebtoken::errors::Error >
  {
    encode( &Header::default(), claims, &EncodingKey::from_secret( self.secret.as_bytes() ) )
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
  pub fn verify_token( &self, token: &str ) -> Result< IcTokenClaims, String >
  {
    // Create custom validation that doesnt require exp claim
    // IC Tokens can have expires_at=null for long-lived tokens
    let mut validation = Validation::default();
    validation.required_spec_claims.clear(); // Dont require standard claims
    validation.validate_exp = false; // Manual expiration check in validate()

    // Decode JWT
    let token_data = decode::< IcTokenClaims >(
      token,
      &DecodingKey::from_secret( self.secret.as_bytes() ),
      &validation,
    )
    .map_err( |e| format!( "JWT decode error: {e}" ) )?;

    // Validate claims
    token_data.claims.validate()?;

    Ok( token_data.claims )
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn test_create_ic_token_claims()
  {
    let claims = IcTokenClaims::new(
      "agent_abc123".to_string(),
      "budget_xyz789".to_string(),
      vec![ "llm:call".to_string() ],
      None,
    );

    assert_eq!( claims.agent_id, "agent_abc123" );
    assert_eq!( claims.budget_id, "budget_xyz789" );
    assert_eq!( claims.issuer, "iron-control-panel" );
    assert_eq!( claims.permissions, vec![ "llm:call" ] );
    assert!( claims.expires_at.is_none() );
    assert!( claims.issued_at > 0 );
  }

  #[ test ]
  fn test_validate_ic_token_claims_success()
  {
    let claims = IcTokenClaims::new(
      "agent_abc123".to_string(),
      "budget_xyz789".to_string(),
      vec![ "llm:call".to_string() ],
      None,
    );

    assert!( claims.validate().is_ok() );
  }

  #[ test ]
  fn test_validate_ic_token_claims_invalid_issuer()
  {
    let mut claims = IcTokenClaims::new(
      "agent_abc123".to_string(),
      "budget_xyz789".to_string(),
      vec![ "llm:call".to_string() ],
      None,
    );

    claims.issuer = "evil-hacker".to_string();
    let result = claims.validate();
    assert!( result.is_err() );
    assert!( result.unwrap_err().contains( "Invalid issuer" ) );
  }

  #[ test ]
  fn test_validate_ic_token_claims_invalid_agent_id_format()
  {
    let claims = IcTokenClaims::new(
      "invalid_format".to_string(),
      "budget_xyz789".to_string(),
      vec![ "llm:call".to_string() ],
      None,
    );

    let result = claims.validate();
    assert!( result.is_err() );
    assert!( result.unwrap_err().contains( "Invalid agent_id format" ) );
  }

  #[ test ]
  fn test_validate_ic_token_claims_expired()
  {
    let past_timestamp = 1000000000; // Far in the past
    let claims = IcTokenClaims::new(
      "agent_abc123".to_string(),
      "budget_xyz789".to_string(),
      vec![ "llm:call".to_string() ],
      Some( past_timestamp ),
    );

    let result = claims.validate();
    assert!( result.is_err() );
    assert!( result.unwrap_err().contains( "Token expired" ) );
  }

  #[ test ]
  fn test_generate_and_verify_ic_token()
  {
    let manager = IcTokenManager::new( "test_secret_key_12345".to_string() );

    let claims = IcTokenClaims::new(
      "agent_abc123".to_string(),
      "budget_xyz789".to_string(),
      vec![ "llm:call".to_string(), "data:read".to_string() ],
      None,
    );

    // Generate token
    let token = manager.generate_token( &claims ).expect( "Should generate token" );
    assert!( !token.is_empty() );

    // Verify token
    let verified_claims = manager.verify_token( &token ).expect( "Should verify token" );
    assert_eq!( verified_claims.agent_id, "agent_abc123" );
    assert_eq!( verified_claims.budget_id, "budget_xyz789" );
    assert_eq!( verified_claims.permissions.len(), 2 );
  }

  #[ test ]
  fn test_verify_invalid_token()
  {
    let manager = IcTokenManager::new( "test_secret_key_12345".to_string() );
    let result = manager.verify_token( "invalid.token.here" );
    assert!( result.is_err() );
  }

  #[ test ]
  fn test_verify_token_wrong_secret()
  {
    let manager1 = IcTokenManager::new( "secret_1".to_string() );
    let manager2 = IcTokenManager::new( "secret_2".to_string() );

    let claims = IcTokenClaims::new(
      "agent_abc123".to_string(),
      "budget_xyz789".to_string(),
      vec![ "llm:call".to_string() ],
      None,
    );

    let token = manager1.generate_token( &claims ).expect( "Should generate" );
    let result = manager2.verify_token( &token );

    assert!( result.is_err() );
  }

  #[ test ]
  fn test_verify_token_with_expiration()
  {
    let manager = IcTokenManager::new( "test_secret_key_12345".to_string() );

    // Create token that expires in 1 hour
    let future_timestamp = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .unwrap()
      .as_secs()
      + 3600;

    let claims = IcTokenClaims::new(
      "agent_abc123".to_string(),
      "budget_xyz789".to_string(),
      vec![ "llm:call".to_string() ],
      Some( future_timestamp ),
    );

    let token = manager.generate_token( &claims ).expect( "Should generate" );
    let verified_claims = manager.verify_token( &token ).expect( "Should verify" );

    assert_eq!( verified_claims.agent_id, "agent_abc123" );
  }
}
