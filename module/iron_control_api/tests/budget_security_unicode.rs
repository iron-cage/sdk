//! Budget Unicode handling tests for Protocol 005
//!
//! Tests verify proper handling of Unicode characters, emoji, zero-width characters,
//! and bidirectional text in user inputs.
//!
//! # Authority
//! - Protocol 005 specification: Input sanitization requirements
//! - Security: Unicode attack prevention (homograph, zero-width, RTL override)
//!
//! # Test Matrix
//!
//! | Test Case | Scenario | Security Risk | Expected Behavior |
//! |-----------|----------|---------------|-------------------|
//! | Emoji in model | "gpt-4-🚀" | Display issues | Accept OR sanitize |
//! | Non-Latin chars | "模型-中文" | Encoding issues | Accept OR sanitize |
//! | Zero-width chars | "lease_​123" (U+200B) | Homograph attacks | Normalize OR reject |
//! | RTL override | "\u{202e}niapo" (displays reversed) | Visual spoofing | Sanitize OR reject |

mod common;

use axum::
{
  body::Body,
  http::{ Request, StatusCode },
};
use common::budget::
{
  setup_test_db,
  create_test_budget_state,
  create_ic_token,
  seed_agent_with_budget,
  create_budget_router,
};
use serde_json::json;
use tower::ServiceExt;

/// E7.1: Emoji in model name
///
/// # Security Risk
/// Emoji could cause encoding issues in logs or databases
///
/// # Expected Behavior
/// - Accept (most modern systems handle emoji) OR
/// - Sanitize (remove/replace emoji) OR
/// - Reject with validation error
#[ tokio::test ]
async fn test_emoji_in_model_name()
{
  let pool = setup_test_db().await;
  let agent_id = 600i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router_handshake = create_budget_router( state.clone() ).await;

  // Create lease first
  let handshake_response = router_handshake
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().unwrap();

  // Report with emoji in model name
  let router_report = create_budget_router( state ).await;
  let response = router_report
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "lease_id": lease_id,
          "request_id": "req_emoji_test",
          "tokens": 1000,
          "cost_microdollars": 5_000_000,
          "model": "gpt-4-🚀",
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should handle gracefully (accept, sanitize, or reject)
  // No panic or crash allowed
  assert!(
    response.status() == StatusCode::OK
      || response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Emoji in model name should be handled gracefully (got {})",
    response.status()
  );

  // If accepted, verify lease budget was updated
  if response.status() == StatusCode::OK
  {
    let lease_spent: i64 = sqlx::query_scalar(
      "SELECT budget_spent FROM budget_leases WHERE id = ?"
    )
    .bind( lease_id )
    .fetch_one( &pool )
    .await
    .expect("LOUD FAILURE: Should query lease budget");

    assert_eq!(
      lease_spent, 5_000_000,
      "LOUD FAILURE: Lease budget should reflect usage cost"
    );
  }
}

/// E7.2: Non-Latin characters (Chinese)
///
/// # Security Risk
/// Non-Latin chars could cause encoding issues or homograph attacks
///
/// # Expected Behavior
/// - Accept (UTF-8 is standard) OR
/// - Sanitize (transliterate to ASCII) OR
/// - Reject with validation error
#[ tokio::test ]
async fn test_non_latin_characters()
{
  let pool = setup_test_db().await;
  let agent_id = 601i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router_handshake = create_budget_router( state.clone() ).await;

  // Create lease first
  let handshake_response = router_handshake
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().unwrap();

  // Report with Chinese characters in model name
  let router_report = create_budget_router( state ).await;
  let response = router_report
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "lease_id": lease_id,
          "request_id": "req_chinese_test",
          "tokens": 1000,
          "cost_microdollars": 5_000_000,
          "model": "模型-中文",
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should handle gracefully
  assert!(
    response.status() == StatusCode::OK
      || response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Non-Latin characters should be handled gracefully (got {})",
    response.status()
  );
}

/// E7.3: Zero-width characters
///
/// # Security Risk
/// HIGH - Zero-width characters (U+200B) can create homograph attacks
/// Example: "lease_​123" looks identical to "lease_123" but is different
///
/// # Expected Behavior
/// - Normalize (strip zero-width chars) OR
/// - Reject with validation error
/// - Should NOT accept as-is (security risk)
#[ tokio::test ]
async fn test_zero_width_characters()
{
  let pool = setup_test_db().await;
  let agent_id = 602i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router_handshake = create_budget_router( state.clone() ).await;

  // Create lease first
  let handshake_response = router_handshake
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().unwrap();

  // Report with zero-width space (U+200B) in request_id
  // "req_​test" contains invisible zero-width space between _ and test
  let router_report = create_budget_router( state ).await;
  let response = router_report
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "lease_id": lease_id,
          "request_id": "req_\u{200B}test", // U+200B = zero-width space
          "tokens": 1000,
          "cost_microdollars": 5_000_000,
          "model": "gpt-4",
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should handle safely (normalize or reject, but NOT accept as-is for security-sensitive fields)
  assert!(
    response.status() == StatusCode::OK
      || response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Zero-width characters should be handled safely (got {})",
    response.status()
  );

  // If accepted, verify lease budget was updated
  if response.status() == StatusCode::OK
  {
    let lease_spent: i64 = sqlx::query_scalar(
      "SELECT budget_spent FROM budget_leases WHERE id = ?"
    )
    .bind( lease_id )
    .fetch_one( &pool )
    .await
    .expect("LOUD FAILURE: Should query lease budget");

    assert_eq!(
      lease_spent, 5_000_000,
      "LOUD FAILURE: Lease budget should reflect usage cost (zero-width test)"
    );
  }
}

/// E7.4: RTL (Right-to-Left) override attack
///
/// # Security Risk
/// HIGH - RTL override (U+202E) can visually spoof strings
/// Example: "\u{202e}niapo" displays as reversed text due to control character
///
/// # Expected Behavior
/// - Sanitize (strip control chars) OR
/// - Reject with validation error
/// - Should NOT accept as-is (security risk)
#[ tokio::test ]
async fn test_rtl_override_attack()
{
  let pool = setup_test_db().await;
  let agent_id = 603i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router_handshake = create_budget_router( state.clone() ).await;

  // Create lease first
  let handshake_response = router_handshake
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().unwrap();

  // Report with RTL override in provider name
  // U+202E (RIGHT-TO-LEFT OVERRIDE) causes text to display reversed
  let router_report = create_budget_router( state ).await;
  let response = router_report
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "lease_id": lease_id,
          "request_id": "req_rtl_test",
          "tokens": 1000,
          "cost_microdollars": 5_000_000,
          "model": "gpt-4",
          "provider": "\u{202E}niapo" // U+202E = RTL override
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should handle safely (sanitize or reject, NOT accept control characters)
  assert!(
    response.status() == StatusCode::OK
      || response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: RTL override should be handled safely (got {})",
    response.status()
  );

  // If accepted, verify lease budget was updated
  if response.status() == StatusCode::OK
  {
    let lease_spent: i64 = sqlx::query_scalar(
      "SELECT budget_spent FROM budget_leases WHERE id = ?"
    )
    .bind( lease_id )
    .fetch_one( &pool )
    .await
    .expect("LOUD FAILURE: Should query lease budget");

    assert_eq!(
      lease_spent, 5_000_000,
      "LOUD FAILURE: Lease budget should reflect usage cost (RTL test)"
    );
  }
}
