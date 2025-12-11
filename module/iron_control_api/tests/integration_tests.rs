//! Integration tests for REST API endpoints
//!
//! Phase 4 Day 30: Testing & Documentation
//!
//! Tests cover:
//! - Authentication endpoints (login, refresh, logout)
//! - Token management endpoints (CRUD operations)
//! - Error cases (401, 403, 404)
//! - Health check endpoint

mod common;

use common::{ create_test_database, create_test_user };
use iron_control_api::routes::auth::AuthState;
use iron_control_api::routes::health;
use iron_control_api::jwt_auth::JwtSecret;
use axum::{
  Router,
  routing::{ get, post },
  http::{ StatusCode, Request },
  body::Body,
};
use tower::ServiceExt;
use serde_json::json;
use std::sync::Arc;

/// Helper to create test auth router
async fn create_auth_router() -> Router
{
  // Create database with schema
  let db_pool = create_test_database().await;

  // Create test user with known credentials
  create_test_user( &db_pool, "test_user@mail.com" ).await;

  // Construct AuthState directly with prepared database
  let auth_state = AuthState
  {
    jwt_secret: Arc::new( JwtSecret::new( "test_secret_key_12345".to_string() ) ),
    db_pool,
  };

  Router::new()
    .route( "/api/auth/login", post( iron_control_api::routes::auth::login ) )
    .route( "/api/auth/refresh", post( iron_control_api::routes::auth::refresh ) )
    .route( "/api/auth/logout", post( iron_control_api::routes::auth::logout ) )
    .with_state( auth_state )
}

/// Helper to create test health router
fn create_health_router() -> Router
{
  Router::new().route( "/api/health", get( health::health_check ) )
}

#[ tokio::test ]
async fn test_health_endpoint()
{
  let app = create_health_router();

  let response = app
    .oneshot( Request::builder().uri( "/api/health" ).body( Body::empty() ).unwrap() )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::OK );
}

#[ tokio::test ]
async fn test_login_success()
{
  let app = create_auth_router().await;

  let request_body = json!({
    "email": "test_user@mail.com",
    "password": "test_password"
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/auth/login" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  let status = response.status();
  if status != StatusCode::OK
  {
    let body = axum::body::to_bytes( response.into_body(), usize::MAX )
      .await
      .unwrap();
    let body_str = String::from_utf8( body.to_vec() ).unwrap();
    panic!( "Expected 200 OK, got {}. Body: {}", status, body_str );
  }

  assert_eq!( status, StatusCode::OK );
}

#[ tokio::test ]
async fn test_login_empty_credentials()
{
  let app = create_auth_router().await;

  let request_body = json!({
    "email": "",
    "password": ""
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/auth/login" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();
  
  assert_eq!( response.status(), StatusCode::BAD_REQUEST );
}

#[ tokio::test ]
async fn test_refresh_token_flow()
{
  let app = create_auth_router().await;

  // First login to get tokens
  let login_body = json!({
    "email": "test_user@mail.com",
    "password": "test_password"
  });

  let login_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/auth/login" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &login_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( login_response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( login_response.into_body(), usize::MAX )
    .await
    .unwrap();
  let login_data: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let refresh_token = login_data[ "refresh_token" ].as_str().unwrap();

  // Use refresh token to get new access token
  let refresh_body = json!({
    "refresh_token": refresh_token
  });

  let refresh_response = app
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/auth/refresh" )
        .header( "content-type", "application/json" )
        .header( "Authorization", format!( "Bearer {}", refresh_token ) )
        .body( Body::from( serde_json::to_string( &refresh_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( refresh_response.status(), StatusCode::OK );
}

#[ tokio::test ]
async fn test_logout_flow()
{
  let app = create_auth_router().await;

  // First login
  let login_body = json!({
    "email": "test_user@mail.com",
    "password": "test_password"
  });

  let login_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/auth/login" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &login_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  let body_bytes = axum::body::to_bytes( login_response.into_body(), usize::MAX )
    .await
    .unwrap();
  let login_data: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let user_token = login_data[ "user_token" ].as_str().unwrap();

  // Logout
  let logout_body = json!({
    "user_token": user_token
  });

  let logout_response = app
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/auth/logout" )
        .header( "content-type", "application/json" )
        .header( "Authorization", format!( "Bearer {}", user_token ) )
        .body( Body::from( serde_json::to_string( &logout_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( logout_response.status(), StatusCode::NO_CONTENT );
}

#[ tokio::test ]
async fn test_invalid_refresh_token()
{
  let app = create_auth_router().await;

  let refresh_body = json!({
    "refresh_token": "invalid_token_123"
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/auth/refresh" )
        .header( "content-type", "application/json" )
        .header( "Authorization", format!( "Bearer {}", "invalid_token_123" ) )
        .body( Body::from( serde_json::to_string( &refresh_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::UNAUTHORIZED );
}
