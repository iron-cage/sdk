//! HTTP client for Control API
//!
//! Provides async HTTP interface to Control API REST endpoints.
//!
//! ## Usage
//!
//! ```rust,ignore
//! let config = ControlApiConfig::load();
//! let client = ControlApiClient::new(config);
//!
//! // GET request
//! let response = client.get("/api/v1/agents").await?;
//!
//! // POST request with body
//! let body = json!({"name": "Agent1", "budget": 100});
//! let response = client.post("/api/v1/agents", body).await?;
//! ```
//!
//! ## Authentication
//!
//! If api_token is configured, all requests include:
//! ```text
//! Authorization: Bearer <token>
//! ```
//!
//! ## Error Handling
//!
//! - Network errors: Connection failures, timeouts
//! - HTTP errors: 4xx, 5xx status codes
//! - Parse errors: Invalid JSON responses

use super::ControlApiConfig;
use reqwest::{ Client, Response };
use serde_json::Value;
use std::collections::HashMap;

/// HTTP client for Control API
pub struct ControlApiClient
{
  /// HTTP client (reqwest)
  client: Client,

  /// API configuration
  config: ControlApiConfig,
}

impl ControlApiClient
{
  /// Create new Control API client
  pub fn new( config: ControlApiConfig ) -> Self
  {
    let client = Client::builder()
      .timeout( config.timeout )
      .build()
      .expect( "LOUD FAILURE: Failed to create HTTP client" );

    Self { client, config }
  }

  /// Make GET request
  ///
  /// ## Parameters
  ///
  /// - path: API endpoint path (e.g., "/api/v1/agents")
  /// - query_params: Optional query parameters
  ///
  /// ## Returns
  ///
  /// JSON response as serde_json::Value
  pub async fn get(
    &self,
    path: &str,
    query_params: Option<HashMap<String, String>>,
  ) -> Result<Value, ControlApiError>
  {
    let url = format!( "{}{}", self.config.base_url, path );

    let mut request = self.client.get( &url );

    // Add authorization header if token configured
    if let Some( ref token ) = self.config.api_token
    {
      request = request.header( "Authorization", format!( "Bearer {}", token ) );
    }

    // Add query parameters
    if let Some( params ) = query_params
    {
      request = request.query( &params );
    }

    let response = request.send().await
      .map_err( |e| ControlApiError::NetworkError( e.to_string() ) )?;

    self.handle_response( response ).await
  }

  /// Make POST request
  ///
  /// ## Parameters
  ///
  /// - path: API endpoint path (e.g., "/api/v1/agents")
  /// - body: JSON request body
  ///
  /// ## Returns
  ///
  /// JSON response as serde_json::Value
  pub async fn post(
    &self,
    path: &str,
    body: Value,
  ) -> Result<Value, ControlApiError>
  {
    let url = format!( "{}{}", self.config.base_url, path );

    let mut request = self.client.post( &url )
      .json( &body );

    // Add authorization header if token configured
    if let Some( ref token ) = self.config.api_token
    {
      request = request.header( "Authorization", format!( "Bearer {}", token ) );
    }

    let response = request.send().await
      .map_err( |e| ControlApiError::NetworkError( e.to_string() ) )?;

    self.handle_response( response ).await
  }

  /// Make PUT request
  ///
  /// ## Parameters
  ///
  /// - path: API endpoint path (e.g., "/api/v1/agents/{id}")
  /// - body: JSON request body
  ///
  /// ## Returns
  ///
  /// JSON response as serde_json::Value
  pub async fn put(
    &self,
    path: &str,
    body: Value,
  ) -> Result<Value, ControlApiError>
  {
    let url = format!( "{}{}", self.config.base_url, path );

    let mut request = self.client.put( &url )
      .json( &body );

    // Add authorization header if token configured
    if let Some( ref token ) = self.config.api_token
    {
      request = request.header( "Authorization", format!( "Bearer {}", token ) );
    }

    let response = request.send().await
      .map_err( |e| ControlApiError::NetworkError( e.to_string() ) )?;

    self.handle_response( response ).await
  }

  /// Make DELETE request
  ///
  /// ## Parameters
  ///
  /// - path: API endpoint path (e.g., "/api/v1/agents/{id}")
  ///
  /// ## Returns
  ///
  /// JSON response as serde_json::Value
  pub async fn delete(
    &self,
    path: &str,
  ) -> Result<Value, ControlApiError>
  {
    let url = format!( "{}{}", self.config.base_url, path );

    let mut request = self.client.delete( &url );

    // Add authorization header if token configured
    if let Some( ref token ) = self.config.api_token
    {
      request = request.header( "Authorization", format!( "Bearer {}", token ) );
    }

    let response = request.send().await
      .map_err( |e| ControlApiError::NetworkError( e.to_string() ) )?;

    self.handle_response( response ).await
  }

  /// Make PATCH request
  ///
  /// ## Parameters
  ///
  /// - path: API endpoint path (e.g., "/api/v1/budget/requests/{id}/approve")
  /// - body: JSON request body
  ///
  /// ## Returns
  ///
  /// JSON response as serde_json::Value
  pub async fn patch(
    &self,
    path: &str,
    body: Value,
  ) -> Result<Value, ControlApiError>
  {
    let url = format!( "{}{}", self.config.base_url, path );

    let mut request = self.client.patch( &url )
      .json( &body );

    // Add authorization header if token configured
    if let Some( ref token ) = self.config.api_token
    {
      request = request.header( "Authorization", format!( "Bearer {}", token ) );
    }

    let response = request.send().await
      .map_err( |e| ControlApiError::NetworkError( e.to_string() ) )?;

    self.handle_response( response ).await
  }

  /// Handle HTTP response
  ///
  /// Checks status code and parses JSON body.
  async fn handle_response(
    &self,
    response: Response,
  ) -> Result<Value, ControlApiError>
  {
    let status = response.status();

    // Check for HTTP errors
    if status.is_client_error() || status.is_server_error()
    {
      let error_body = response.text().await
        .unwrap_or_else( |_| "Unknown error".to_string() );

      return Err( ControlApiError::ApiError {
        status_code: status.as_u16(),
        message: error_body,
      });
    }

    // Handle 204 No Content (e.g., delete operations)
    if status.as_u16() == 204
    {
      return Ok( serde_json::json!({ "status": "success" }) );
    }

    // Parse JSON response
    let json = response.json::<Value>().await
      .map_err( |e| ControlApiError::ParseError( e.to_string() ) )?;

    Ok( json )
  }
}

/// Control API errors
#[derive(Debug)]
pub enum ControlApiError
{
  /// Network error (connection failure, timeout)
  NetworkError( String ),

  /// API error (4xx, 5xx status codes)
  ApiError
  {
    status_code: u16,
    message: String,
  },

  /// JSON parse error
  ParseError( String ),
}

impl std::fmt::Display for ControlApiError
{
  fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result
  {
    match self
    {
      Self::NetworkError( msg ) =>
      {
        write!( f, "Network error: {}", msg )
      }
      Self::ApiError { status_code, message } =>
      {
        write!( f, "API error ({}): {}", status_code, message )
      }
      Self::ParseError( msg ) =>
      {
        write!( f, "Parse error: {}", msg )
      }
    }
  }
}

impl std::error::Error for ControlApiError {}
