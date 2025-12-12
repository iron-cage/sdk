//! HTTP client for Token Manager API
//!
//! Provides async HTTP interface to Token Manager API REST endpoints.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use iron_cli::adapters::keyring;
//!
//! let config = TokenApiConfig::load();
//! let client = TokenApiClient::new(config);
//!
//! // GET request (requires auth)
//! let access_token = keyring::get_access_token()?;
//! let response = client.get("/api/v1/tokens", None, Some(&access_token)).await?;
//!
//! // POST request with body (requires auth)
//! let body = json!({"name": "Token1", "scope": "read:tokens"});
//! let response = client.post("/api/v1/tokens", body, Some(&access_token)).await?;
//! ```
//!
//! ## Authentication
//!
//! All requests (except login/refresh) require access token from keyring:
//! ```text
//! Authorization: Bearer <access_token>
//! ```
//!
//! On 401 response, caller should attempt token refresh using refresh_token.
//!
//! ## Error Handling
//!
//! - Network errors: Connection failures, timeouts
//! - HTTP errors: 4xx, 5xx status codes
//! - Parse errors: Invalid JSON responses
//! - Auth errors: 401 responses trigger token refresh flow

use super::TokenApiConfig;
use reqwest::{ Client, Response };
use serde_json::Value;
use std::collections::HashMap;

/// HTTP client for Token Manager API
pub struct TokenApiClient
{
  /// HTTP client (reqwest)
  client: Client,

  /// API configuration
  config: TokenApiConfig,
}

impl TokenApiClient
{
  /// Create new Token API client
  pub fn new( config: TokenApiConfig ) -> Self
  {
    let client = Client::builder()
      .timeout( config.timeout )
      .build()
      .expect( "Failed to create HTTP client" );

    Self { client, config }
  }

  /// Make GET request
  ///
  /// ## Parameters
  ///
  /// - path: API endpoint path (e.g., "/api/v1/tokens")
  /// - query_params: Optional query parameters
  /// - access_token: Optional access token (required for protected endpoints)
  ///
  /// ## Returns
  ///
  /// JSON response as serde_json::Value
  pub async fn get(
    &self,
    path: &str,
    query_params: Option<HashMap<String, String>>,
    access_token: Option<&str>,
  ) -> Result<Value, TokenApiError>
  {
    let url = format!( "{}{}", self.config.base_url, path );

    let mut request = self.client.get( &url );

    // Add authorization header if token provided
    if let Some( token ) = access_token
    {
      request = request.header( "Authorization", format!( "Bearer {}", token ) );
    }

    // Add query parameters
    if let Some( params ) = query_params
    {
      request = request.query( &params );
    }

    let response = request.send().await
      .map_err( |e| TokenApiError::NetworkError( e.to_string() ) )?;

    self.handle_response( response ).await
  }

  /// Make POST request
  ///
  /// ## Parameters
  ///
  /// - path: API endpoint path (e.g., "/api/v1/tokens")
  /// - body: JSON request body
  /// - access_token: Optional access token (required for protected endpoints)
  ///
  /// ## Returns
  ///
  /// JSON response as serde_json::Value
  pub async fn post(
    &self,
    path: &str,
    body: Value,
    access_token: Option<&str>,
  ) -> Result<Value, TokenApiError>
  {
    let url = format!( "{}{}", self.config.base_url, path );

    let mut request = self.client.post( &url )
      .json( &body );

    // Add authorization header if token provided
    if let Some( token ) = access_token
    {
      request = request.header( "Authorization", format!( "Bearer {}", token ) );
    }

    let response = request.send().await
      .map_err( |e| TokenApiError::NetworkError( e.to_string() ) )?;

    self.handle_response( response ).await
  }

  /// Make PUT request
  ///
  /// ## Parameters
  ///
  /// - path: API endpoint path (e.g., "/api/v1/tokens/{id}")
  /// - body: JSON request body
  /// - access_token: Optional access token (required for protected endpoints)
  ///
  /// ## Returns
  ///
  /// JSON response as serde_json::Value
  pub async fn put(
    &self,
    path: &str,
    body: Value,
    access_token: Option<&str>,
  ) -> Result<Value, TokenApiError>
  {
    let url = format!( "{}{}", self.config.base_url, path );

    let mut request = self.client.put( &url )
      .json( &body );

    // Add authorization header if token provided
    if let Some( token ) = access_token
    {
      request = request.header( "Authorization", format!( "Bearer {}", token ) );
    }

    let response = request.send().await
      .map_err( |e| TokenApiError::NetworkError( e.to_string() ) )?;

    self.handle_response( response ).await
  }

  /// Make DELETE request
  ///
  /// ## Parameters
  ///
  /// - path: API endpoint path (e.g., "/api/v1/tokens/{id}")
  /// - access_token: Optional access token (required for protected endpoints)
  ///
  /// ## Returns
  ///
  /// JSON response as serde_json::Value
  pub async fn delete(
    &self,
    path: &str,
    access_token: Option<&str>,
  ) -> Result<Value, TokenApiError>
  {
    let url = format!( "{}{}", self.config.base_url, path );

    let mut request = self.client.delete( &url );

    // Add authorization header if token provided
    if let Some( token ) = access_token
    {
      request = request.header( "Authorization", format!( "Bearer {}", token ) );
    }

    let response = request.send().await
      .map_err( |e| TokenApiError::NetworkError( e.to_string() ) )?;

    self.handle_response( response ).await
  }

  /// Handle HTTP response
  ///
  /// Checks status code and parses JSON body.
  async fn handle_response(
    &self,
    response: Response,
  ) -> Result<Value, TokenApiError>
  {
    let status = response.status();

    // Check for HTTP errors
    if status.is_client_error() || status.is_server_error()
    {
      let error_body = response.text().await
        .unwrap_or_else( |_| "Unknown error".to_string() );

      return Err( TokenApiError::ApiError {
        status_code: status.as_u16(),
        message: error_body,
      });
    }

    // Parse JSON response
    let json = response.json::<Value>().await
      .map_err( |e| TokenApiError::ParseError( e.to_string() ) )?;

    Ok( json )
  }
}

/// Token API errors
#[derive(Debug)]
pub enum TokenApiError
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

impl std::fmt::Display for TokenApiError
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

impl std::error::Error for TokenApiError {}

#[cfg(test)]
mod tests
{
  use super::*;

  #[test]
  fn test_client_creation()
  {
    let config = TokenApiConfig::default();
    let client = TokenApiClient::new( config );

    assert_eq!( client.config.base_url, "http://localhost:8081" );
  }

  #[test]
  fn test_client_with_custom_url()
  {
    let config = TokenApiConfig::new(
      "https://api.example.com".to_string(),
    );

    let client = TokenApiClient::new( config );

    assert_eq!( client.config.base_url, "https://api.example.com" );
  }
}
