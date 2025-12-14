//! Shared types and state for token management endpoints
//!
//! Contains token state, request/response types, and validation logic.

use iron_token_manager::storage::TokenStorage;
use iron_token_manager::token_generator::TokenGenerator;
use serde::{ Deserialize, Serialize };
use std::sync::Arc;
use crate::error::ValidationError;

/// Token management state
#[ derive( Clone ) ]
pub struct TokenState
{
  pub storage: Arc< TokenStorage >,
  pub generator: Arc< TokenGenerator >,
}

impl TokenState
{
  /// Create new token state
  ///
  /// # Errors
  ///
  /// Returns error if database connection fails
  pub async fn new( database_url: &str ) -> Result< Self, Box< dyn std::error::Error > >
  {
    let storage = TokenStorage::new( database_url ).await?;
    Ok( Self {
      storage: Arc::new( storage ),
      generator: Arc::new( TokenGenerator::new() ),
    } )
  }
}

/// Create token request (Protocol 014 compliant with backward compatibility)
///
/// Per Protocol 014: user_id comes from JWT authentication, not request body
///
/// # Formats Supported
///
/// **Protocol 014 format (preferred):**
/// - `name`: required, 1-100 chars
/// - `description`: optional, max 500 chars
/// - `user_id`: from JWT (not in request body)
///
/// **Legacy format (backward compatibility):**
/// - `user_id`: in request body
/// - `project_id`: optional
/// - `description`: optional (used as token name in database)
#[ derive( Debug, Deserialize ) ]
pub struct CreateTokenRequest
{
  // Protocol 014 field - optional for backward compatibility with legacy tests
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  #[ serde( default ) ]
  pub name: Option< String >,

  pub description: Option< String >,

  // Legacy fields kept for backward compatibility with existing tests
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  #[ serde( default ) ]
  pub user_id: Option< String >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  #[ serde( default ) ]
  pub project_id: Option< String >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  #[ serde( default ) ]
  pub agent_id: Option< i64 >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  #[ serde( default ) ]
  pub provider: Option< String >,
}

impl CreateTokenRequest
{
  // Fix(issue-001): Prevent DoS via unlimited string validation
  // Root cause: Accepted unbounded external input without resource limits
  // Pitfall: Always validate input length before processing to prevent resource exhaustion

  // Fix(issue-max-user-id-length): Align validation with migration 013 schema constraint
  // Root cause: Validation allowed 500 chars but migration 013 reduced user_id to 255 to match users.id FK target
  // Pitfall: Validation constants must match database CHECK constraints to prevent insertion failures

  /// Maximum user_id length (DoS protection). Must match migration 013 CHECK constraint (255 chars, aligned with users.id).
  const MAX_USER_ID_LENGTH: usize = 255;

  /// Maximum project_id length (DoS protection). Must match database CHECK constraint.
  const MAX_PROJECT_ID_LENGTH: usize = 500;

  /// Maximum name length (Protocol 014: 1-100 chars)
  const MAX_NAME_LENGTH: usize = 100;

  /// Maximum description length (Protocol 014: max 500 chars)
  const MAX_DESCRIPTION_LENGTH: usize = 500;

  /// Validate token creation request (Protocol 014 compliant with backward compatibility).
  ///
  /// **Protocol 014 format:** Validates `name` (required, 1-100 chars) when provided
  /// **Legacy format:** Validates `user_id` (required, non-empty) when `name` not provided
  ///
  /// Both formats validate `description` (optional, max 500 chars) and `project_id` (optional, non-empty).
  pub fn validate( &self ) -> Result< (), ValidationError >
  {
    // Protocol 014 validation: If `name` is provided, validate it
    if let Some( ref name ) = self.name
    {
      // Validate name is not empty (Protocol 014 requirement)
      if name.trim().is_empty()
      {
        return Err( ValidationError::MissingField( "name".to_string() ) );
      }

      // Validate name length (Protocol 014: 1-100 chars)
      if name.len() > Self::MAX_NAME_LENGTH
      {
        return Err( ValidationError::TooLong
        {
          field: "name".to_string(),
          max_length: Self::MAX_NAME_LENGTH,
        } );
      }

      // Fix(issue-002): Prevent NULL byte injection causing C string termination attacks
      // Root cause: Accepted NULL bytes in strings passed to C/FFI libraries and database drivers
      // Pitfall: Always validate against control characters when interacting with C libraries or databases using C drivers

      // Validate name doesnt contain NULL bytes
      if name.contains( '\0' )
      {
        return Err( ValidationError::InvalidCharacter
        {
          field: "name".to_string(),
          character: "NULL".to_string(),
        } );
      }
    }

    // Validate description if provided (Protocol 014: max 500 chars)
    if let Some( ref description ) = self.description
    {
      if description.len() > Self::MAX_DESCRIPTION_LENGTH
      {
        return Err( ValidationError::TooLong
        {
          field: "description".to_string(),
          max_length: Self::MAX_DESCRIPTION_LENGTH,
        } );
      }

      // Validate description doesnt contain NULL bytes
      if description.contains( '\0' )
      {
        return Err( ValidationError::InvalidCharacter
        {
          field: "description".to_string(),
          character: "NULL".to_string(),
        } );
      }
    }

    // Legacy validation for backward compatibility with existing tests
    // These validations apply when legacy format is used (no `name` field)

    if let Some( ref user_id ) = self.user_id
    {
      if user_id.trim().is_empty()
      {
        return Err( ValidationError::MissingField( "user_id".to_string() ) );
      }

      if user_id.len() > Self::MAX_USER_ID_LENGTH
      {
        return Err( ValidationError::TooLong
        {
          field: "user_id".to_string(),
          max_length: Self::MAX_USER_ID_LENGTH,
        } );
      }

      if user_id.contains( '\0' )
      {
        return Err( ValidationError::InvalidCharacter
        {
          field: "user_id".to_string(),
          character: "NULL".to_string(),
        } );
      }
    }

    if let Some( ref project_id ) = self.project_id
    {
      if project_id.trim().is_empty()
      {
        return Err( ValidationError::MissingField( "project_id".to_string() ) );
      }

      if project_id.len() > Self::MAX_PROJECT_ID_LENGTH
      {
        return Err( ValidationError::TooLong
        {
          field: "project_id".to_string(),
          max_length: Self::MAX_PROJECT_ID_LENGTH,
        } );
      }

      if project_id.contains( '\0' )
      {
        return Err( ValidationError::InvalidCharacter
        {
          field: "project_id".to_string(),
          character: "NULL".to_string(),
        } );
      }
    }

    Ok( () )
  }
}

/// Update token request
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct UpdateTokenRequest
{
  pub provider: String,
}

impl UpdateTokenRequest
{
  /// Maximum length of provider (DoS protection)
  const MAX_PROVIDER_LENGTH: usize = 64;

  pub fn validate( &self ) -> Result< (), ValidationError >
  {
    // Validate provider if provided
    if self.provider.trim().is_empty()
    {
      return Err( ValidationError::MissingField( "provider".to_string() ) );
    }

    // Validate provider length (DoS protection)
    if self.provider.len() > Self::MAX_PROVIDER_LENGTH
    {
      return Err( ValidationError::TooLong
      {
        field: "provider".to_string(),
        max_length: Self::MAX_PROVIDER_LENGTH,
      } );
    }

    // Validate provider doesnt contain NULL bytes
    if self.provider.contains( '\0' )
    {
      return Err( ValidationError::InvalidCharacter
      {
        field: "provider".to_string(),
        character: "NULL".to_string(),
      } );
    }

    Ok( () )
  }
}

/// Validate token request (Deliverable 1.6)
#[ derive( Debug, Deserialize ) ]
pub struct ValidateTokenRequest
{
  pub token: String,
}

/// Validate token response (Deliverable 1.6)
#[ derive( Debug, Serialize ) ]
pub struct ValidateTokenResponse
{
  pub valid: bool,
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub user_id: Option< String >,
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub project_id: Option< String >,
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub token_id: Option< i64 >,
}

/// Create token response
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct CreateTokenResponse
{
  pub id: i64,
  pub token: String,
  pub user_id: String,
  pub project_id: Option< String >,
  pub description: Option< String >,
  pub agent_id: Option< i64 >,
  pub provider: Option< String >,
  pub created_at: i64,
}

/// Token list item
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct TokenListItem
{
  pub id: i64,
  pub user_id: String,
  pub project_id: Option< String >,
  pub description: Option< String >,
  pub agent_id: Option< i64 >,
  pub provider: Option< String >,
  pub created_at: i64,
  pub last_used_at: Option< i64 >,
  pub is_active: bool,
}
