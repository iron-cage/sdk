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
use crate::error::ValidationError;

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
      .expect( "LOUD FAILURE: Time went backwards" )
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
  pub fn validate( &self ) -> Result< (), ValidationError >
  {
    // Check issuer
    if self.issuer != "iron-control-panel"
    {
      return Err( ValidationError::InvalidValue
      {
        field: "issuer".to_string(),
        reason: format!( "expected 'iron-control-panel', got '{}'", self.issuer ),
      } );
    }

    // Check expiration
    if let Some( exp ) = self.expires_at
    {
      let now = SystemTime::now()
        .duration_since( UNIX_EPOCH )
        .expect( "LOUD FAILURE: Time went backwards" )
        .as_secs();

      if now > exp
      {
        return Err( ValidationError::Custom( "Token expired".to_string() ) );
      }
    }

    // Check agent_id format (basic validation)
    if !self.agent_id.starts_with( "agent_" )
    {
      return Err( ValidationError::InvalidFormat
      {
        field: "agent_id".to_string(),
        expected: "must start with 'agent_'".to_string(),
      } );
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
    token_data.claims.validate().map_err( |e| e.to_string() )?;

    Ok( token_data.claims )
  }
}

