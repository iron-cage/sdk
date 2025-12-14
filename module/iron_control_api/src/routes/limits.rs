//! Limits management REST API endpoints
//!
//! Phase 4 Day 29: REST API Endpoints - Limits Management
//!
//! Endpoints:
//! - POST /api/limits - Create usage limit
//! - GET /api/limits - List all limits for user
//! - GET /api/limits/:id - Get specific limit
//! - PUT /api/limits/:id - Update limit
//! - DELETE /api/limits/:id - Delete limit

use axum::{
  extract::State,
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use crate::error::JsonPath;
use iron_token_manager::limit_enforcer::LimitEnforcer;
use serde::{ Deserialize, Serialize };
use std::sync::Arc;

/// Limits management state
#[ derive( Clone ) ]
pub struct LimitsState
{
  pub enforcer: Arc< LimitEnforcer >,
}

impl LimitsState
{
  /// Create new limits state
  ///
  /// # Errors
  ///
  /// Returns error if database connection fails
  pub async fn new( database_url: &str ) -> Result< Self, Box< dyn std::error::Error > >
  {
    let enforcer = LimitEnforcer::new( database_url ).await?;
    Ok( Self { enforcer: Arc::new( enforcer ) } )
  }
}

/// Create limit request
#[ derive( Debug, Deserialize ) ]
pub struct CreateLimitRequest
{
  pub user_id: String,
  pub project_id: Option< String >,
  pub max_tokens_per_day: Option< i64 >,
  pub max_requests_per_minute: Option< i64 >,
  pub max_cost_per_month_microdollars: Option< i64 >,
}

impl CreateLimitRequest
{
  /// Maximum safe limit value to prevent overflow.
  const MAX_SAFE_LIMIT: i64 = i64::MAX / 2;

  /// Validate a single limit value.
  ///
  /// # Arguments
  ///
  /// * `value` - Optional limit value to validate
  /// * `field_name` - Name of the field (for error messages)
  ///
  /// # Returns
  ///
  /// - Ok(()) if value is None or valid (positive and within safe range)
  /// - Err(String) with descriptive error message if validation fails
  fn validate_limit_value( value: Option< i64 >, field_name: &str ) -> Result< (), String >
  {
    if let Some( val ) = value
    {
      if val <= 0
      {
        return Err( format!( "{} must be a positive number", field_name ) );
      }

      if val > Self::MAX_SAFE_LIMIT
      {
        return Err( format!(
          "{} {} too large. Maximum: {}",
          field_name,
          val,
          Self::MAX_SAFE_LIMIT
        ) );
      }
    }

    Ok( () )
  }

  /// Validate field values (returns 400 errors)
  ///
  /// Checks that all provided field values are positive and within safe range.
  ///
  /// # Errors
  ///
  /// Returns error if any provided field is ≤ 0 or exceeds MAX_SAFE_LIMIT
  pub fn validate_values( &self ) -> Result< (), String >
  {
    Self::validate_limit_value( self.max_tokens_per_day, "max_tokens_per_day" )?;
    Self::validate_limit_value( self.max_requests_per_minute, "max_requests_per_minute" )?;
    Self::validate_limit_value( self.max_cost_per_month_microdollars, "max_cost_per_month_microdollars" )?;
    Ok( () )
  }

  /// Validate that at least one field is provided (returns 422 errors)
  ///
  /// # Errors
  ///
  /// Returns error if all fields are None (no limits specified, semantic error)
  pub fn validate_presence( &self ) -> Result< (), String >
  {
    if self.max_tokens_per_day.is_none()
      && self.max_requests_per_minute.is_none()
      && self.max_cost_per_month_microdollars.is_none()
    {
      return Err( "at least one limit must be specified (max_tokens_per_day, max_requests_per_minute, or max_cost_per_month_microdollars)".to_string() );
    }
    Ok( () )
  }

  /// Validate limit creation request
  ///
  /// # Rules
  ///
  /// - At least one limit must be specified (not all None)
  /// - All specified limits must be positive (> 0)
  /// - All specified limits must be within safe range (≤ MAX_SAFE_LIMIT)
  ///
  /// # Returns
  ///
  /// - Ok(()) if validation passes
  /// - Err(String) with error message if validation fails
  ///
  /// # Errors
  ///
  /// Returns descriptive error message for:
  /// - All limits are None (missing required data)
  /// - Any limit is ≤ 0
  /// - Any limit exceeds MAX_SAFE_LIMIT
  pub fn validate( &self ) -> Result< (), String >
  {
    self.validate_values()?;
    self.validate_presence()?;
    Ok( () )
  }
}

/// Update limit request body (for PUT /api/limits/:id)
///
/// All fields are optional to support partial updates.
/// user_id cannot be changed via update endpoint.
#[ derive( Debug, Deserialize ) ]
pub struct UpdateLimitRequest
{
  pub max_tokens_per_day: Option< i64 >,
  pub max_requests_per_minute: Option< i64 >,
  pub max_cost_per_month_microdollars: Option< i64 >,
}

impl UpdateLimitRequest
{
  /// Maximum safe limit value (same as CreateLimitRequest)
  const MAX_SAFE_LIMIT: i64 = CreateLimitRequest::MAX_SAFE_LIMIT;

  /// Validate a single limit value
  ///
  /// # Arguments
  ///
  /// * `value` - Optional limit value to validate
  /// * `field_name` - Name of the field (for error messages)
  ///
  /// # Returns
  ///
  /// - Ok(()) if value is None or valid (positive and within safe range)
  /// - Err(String) with descriptive error message if validation fails
  fn validate_limit_value( value: Option< i64 >, field_name: &str ) -> Result< (), String >
  {
    if let Some( val ) = value
    {
      if val <= 0
      {
        return Err( format!( "{} must be a positive number", field_name ) );
      }

      if val > Self::MAX_SAFE_LIMIT
      {
        return Err( format!(
          "{} {} too large. Maximum: {}",
          field_name,
          val,
          Self::MAX_SAFE_LIMIT
        ) );
      }
    }

    Ok( () )
  }

  /// Validate field values (returns 400 errors)
  ///
  /// Checks that all provided field values are positive and within safe range.
  /// Does NOT check if at least one field is provided.
  ///
  /// # Errors
  ///
  /// Returns error if any provided field is ≤ 0 or exceeds MAX_SAFE_LIMIT
  pub fn validate_values( &self ) -> Result< (), String >
  {
    Self::validate_limit_value( self.max_tokens_per_day, "max_tokens_per_day" )?;
    Self::validate_limit_value( self.max_requests_per_minute, "max_requests_per_minute" )?;
    Self::validate_limit_value( self.max_cost_per_month_microdollars, "max_cost_per_month_microdollars" )?;
    Ok( () )
  }

  /// Validate that at least one field is provided (returns 422 errors)
  ///
  /// # Errors
  ///
  /// Returns error if all fields are None (no fields to update, semantic error)
  pub fn validate_presence( &self ) -> Result< (), String >
  {
    if self.max_tokens_per_day.is_none()
      && self.max_requests_per_minute.is_none()
      && self.max_cost_per_month_microdollars.is_none()
    {
      return Err( "at least one field must be provided for update".to_string() );
    }
    Ok( () )
  }

  /// Validate update request parameters
  ///
  /// # Rules
  ///
  /// - At least one field must be provided (not all None)
  /// - All provided fields must be positive (> 0)
  /// - All provided fields must be within safe range (≤ MAX_SAFE_LIMIT)
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - All fields are None (no fields to update)
  /// - Any provided field is ≤ 0
  /// - Any provided field exceeds MAX_SAFE_LIMIT
  pub fn validate( &self ) -> Result< (), String >
  {
    self.validate_values()?;
    self.validate_presence()?;
    Ok( () )
  }
}

/// Limit response
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct LimitResponse
{
  pub id: i64,
  pub user_id: String,
  pub project_id: Option< String >,
  pub max_tokens_per_day: Option< i64 >,
  pub max_requests_per_minute: Option< i64 >,
  pub max_cost_per_month_microdollars: Option< i64 >,
  pub created_at: i64,
}

/// POST /api/limits
///
/// Create new usage limit
///
/// # Arguments
///
/// * `state` - Limits state with LimitEnforcer
/// * `request` - Limit configuration
///
/// # Returns
///
/// - 201 Created with limit response
/// - 400 Bad Request if field values are invalid (negative, overflow)
/// - 422 Unprocessable Entity if no limits specified (all-None)
/// - 500 Internal Server Error if database operation fails
pub async fn create_limit(
  State( state ): State< LimitsState >,
  Json( request ): Json< CreateLimitRequest >,
) -> impl IntoResponse
{
  // Validate field values first (returns 400)
  if let Err( validation_error ) = request.validate_values()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  // Then validate presence (returns 422)
  if let Err( validation_error ) = request.validate_presence()
  {
    return ( StatusCode::UNPROCESSABLE_ENTITY, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  // Create limit in database
  let limit_id = match state.enforcer.create_limit(
    &request.user_id,
    request.project_id.as_deref(),
    request.max_tokens_per_day,
    request.max_requests_per_minute,
    request.max_cost_per_month_microdollars,
  ).await
  {
    Ok( id ) => id,
    Err( e ) => {
      tracing::error!( "Failed to create limit: {:?}", e );
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Database operation failed"
      }) ) ).into_response();
    }
  };

  // Retrieve created limit to get full record
  let limit = match state.enforcer.get_limit_by_id( limit_id ).await
  {
    Ok( limit ) => limit,
    Err( e ) => {
      tracing::error!( "Failed to retrieve created limit: {:?}", e );
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Database operation failed"
      }) ) ).into_response();
    }
  };

  let response = LimitResponse
  {
    id: limit.id,
    user_id: limit.user_id,
    project_id: limit.project_id,
    max_tokens_per_day: limit.max_tokens_per_day,
    max_requests_per_minute: limit.max_requests_per_minute,
    max_cost_per_month_microdollars: limit.max_cost_microdollars_per_month,
    created_at: limit.created_at,
  };

  ( StatusCode::CREATED, Json( response ) ).into_response()
}

/// GET /api/limits
///
/// List all usage limits
///
/// # Arguments
///
/// * `state` - Limits state with LimitEnforcer
///
/// # Returns
///
/// - 200 OK with vector of limit responses
/// - 500 Internal Server Error if database query fails
pub async fn list_limits( State( state ): State< LimitsState > ) -> impl IntoResponse
{
  // Query all limits
  let limits = match state.enforcer.list_all_limits().await
  {
    Ok( limits ) => limits,
    Err( e ) => {
      tracing::error!( "Failed to list limits: {:?}", e );
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Database query failed"
      }) ) ).into_response();
    }
  };

  // Map to response type
  let response: Vec< LimitResponse > = limits.into_iter().map( |limit| {
    LimitResponse {
      id: limit.id,
      user_id: limit.user_id,
      project_id: limit.project_id,
      max_tokens_per_day: limit.max_tokens_per_day,
      max_requests_per_minute: limit.max_requests_per_minute,
      max_cost_per_month_microdollars: limit.max_cost_microdollars_per_month,
      created_at: limit.created_at,
    }
  } ).collect();

  ( StatusCode::OK, Json( response ) ).into_response()
}

/// GET /api/limits/:id
///
/// Get specific usage limit
///
/// # Arguments
///
/// * `state` - Limits state with LimitEnforcer
/// * `limit_id` - Limit ID
///
/// # Returns
///
/// - 200 OK with limit response
/// - 404 Not Found if limit doesn't exist
/// - 500 Internal Server Error if database query fails
pub async fn get_limit(
  State( state ): State< LimitsState >,
  JsonPath( limit_id ): JsonPath< i64 >,
) -> impl IntoResponse
{
  // Query limit by ID
  let limit = match state.enforcer.get_limit_by_id( limit_id ).await
  {
    Ok( limit ) => limit,
    Err( e ) => {
      tracing::error!( "Failed to get limit {}: {:?}", limit_id, e );
      return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
        "error": "Limit not found"
      }) ) ).into_response();
    }
  };

  let response = LimitResponse
  {
    id: limit.id,
    user_id: limit.user_id,
    project_id: limit.project_id,
    max_tokens_per_day: limit.max_tokens_per_day,
    max_requests_per_minute: limit.max_requests_per_minute,
    max_cost_per_month_microdollars: limit.max_cost_microdollars_per_month,
    created_at: limit.created_at,
  };

  ( StatusCode::OK, Json( response ) ).into_response()
}

/// PUT /api/limits/:id
///
/// Update existing usage limit
///
/// # Arguments
///
/// * `state` - Limits state with LimitEnforcer
/// * `limit_id` - Limit ID
/// * `request` - Updated limit configuration (user_id is preserved from existing record)
///
/// # Returns
///
/// - 200 OK with updated limit response
/// - 400 Bad Request if field values are invalid (negative, overflow)
/// - 422 Unprocessable Entity if no fields provided (all-None)
/// - 404 Not Found if limit doesn't exist
/// - 500 Internal Server Error if database operation fails
pub async fn update_limit(
  State( state ): State< LimitsState >,
  JsonPath( limit_id ): JsonPath< i64 >,
  Json( request ): Json< UpdateLimitRequest >,
) -> impl IntoResponse
{
  // Validate field values first (returns 400)
  if let Err( validation_error ) = request.validate_values()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  // Then validate presence (returns 422)
  if let Err( validation_error ) = request.validate_presence()
  {
    return ( StatusCode::UNPROCESSABLE_ENTITY, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  // Update limit in database
  if let Err( e ) = state.enforcer.update_limit_by_id(
    limit_id,
    request.max_tokens_per_day,
    request.max_requests_per_minute,
    request.max_cost_per_month_microdollars,
  ).await
  {
    tracing::error!( "Failed to update limit {}: {:?}", limit_id, e );
    return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
      "error": "Database operation failed"
    }) ) ).into_response();
  }

  // Retrieve updated limit
  let limit = match state.enforcer.get_limit_by_id( limit_id ).await
  {
    Ok( limit ) => limit,
    Err( e ) => {
      tracing::error!( "Failed to retrieve updated limit {}: {:?}", limit_id, e );
      return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
        "error": "Limit not found"
      }) ) ).into_response();
    }
  };

  let response = LimitResponse
  {
    id: limit.id,
    user_id: limit.user_id,
    project_id: limit.project_id,
    max_tokens_per_day: limit.max_tokens_per_day,
    max_requests_per_minute: limit.max_requests_per_minute,
    max_cost_per_month_microdollars: limit.max_cost_microdollars_per_month,
    created_at: limit.created_at,
  };

  ( StatusCode::OK, Json( response ) ).into_response()
}

/// DELETE /api/limits/:id
///
/// Delete usage limit
///
/// # Arguments
///
/// * `state` - Limits state with LimitEnforcer
/// * `limit_id` - Limit ID
///
/// # Returns
///
/// - 204 No Content on success
/// - 500 Internal Server Error if database operation fails
pub async fn delete_limit(
  State( state ): State< LimitsState >,
  JsonPath( limit_id ): JsonPath< i64 >,
) -> impl IntoResponse
{
  // Delete limit from database
  if let Err( e ) = state.enforcer.delete_limit( limit_id ).await
  {
    tracing::error!( "Failed to delete limit {}: {:?}", limit_id, e );
    return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
      "error": "Database operation failed"
    }) ) ).into_response();
  }

  StatusCode::NO_CONTENT.into_response()
}
