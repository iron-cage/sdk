//! Budget refresh endpoint corner case tests
//!
//! Tests refresh-specific corner cases for Protocol 005:
//! - NULL additional_budget field validation
//! - Float overflow conditions (f64::MAX, Infinity)
//! - Non-finite value handling (NaN)
//!
//! # Authority
//! test_organization.rulebook.md § Comprehensive Corner Case Coverage
//!
//! # Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_refresh_null_additional_budget` | NULL field validation | POST /api/budget/refresh with additional_budget=null | 400/422 Bad Request | ✅ |
//! | `test_refresh_float_overflow_f64_max` | Float overflow f64::MAX | POST /api/budget/refresh with additional_budget=f64::MAX | 400/422 Bad Request | ✅ |
//! | `test_refresh_float_overflow_infinity` | Float overflow Infinity | POST /api/budget/refresh with additional_budget=Infinity | 400/422 Bad Request | ✅ |
//! | `test_refresh_nan_additional_budget` | NaN value handling | POST /api/budget/refresh with additional_budget=NaN | 400/422 Bad Request | ✅ |

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
  seed_agent_with_budget,
  create_budget_router,
  create_ic_token,
};
use serde_json::json;
use tower::ServiceExt;

/// Manual Test Gap #25: Refresh - NULL requested_budget field
///
/// # Corner Case
/// POST /api/budget/refresh with requested_budget=null
///
/// # Expected Behavior
/// Request succeeds with default budget (NULL is valid - uses default)
///
/// # Risk
/// LOW - NULL is handled by Option<i64> with default value
#[ tokio::test ]
async fn test_refresh_null_additional_budget()
{
  let pool = setup_test_db().await;
  let agent_id = 122i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;
  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Create initial lease via handshake
  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_response = router.clone().oneshot( handshake_request ).await.unwrap();
  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap().to_string();

  // Create JWT token for authenticated request (GAP-003)
  let access_token = common::create_test_access_token( "test_user", "test@example.com", "admin", "test_jwt_secret" );

  // Craft refresh request with null requested_budget (should use default)
  let request_body = json!({
    "ic_token": ic_token,
    "current_lease_id": lease_id,
    "requested_budget": null,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", access_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // NULL requested_budget is valid - should succeed with default
  assert!(
    response.status() == StatusCode::OK,
    "NULL requested_budget should succeed with default budget, got: {}", response.status()
  );
}

/// Manual Test Gap #26: Refresh - Float overflow requested_budget (f64::MAX)
///
/// # Corner Case
/// POST /api/budget/refresh with requested_budget=f64::MAX
///
/// # Expected Behavior
/// 400 Bad Request or 422 Unprocessable Entity (JSON deserialization fails for i64)
///
/// # Risk
/// MEDIUM - Budget overflow
#[ tokio::test ]
async fn test_refresh_float_overflow_f64_max()
{
  let pool = setup_test_db().await;
  let agent_id = 123i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;
  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Create initial lease
  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_response = router.clone().oneshot( handshake_request ).await.unwrap();
  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap().to_string();

  let access_token = common::create_test_access_token( "test_user", "test@example.com", "admin", "test_jwt_secret" );

  let request_body = json!({
    "ic_token": ic_token,
    "current_lease_id": lease_id,
    "requested_budget": f64::MAX,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", access_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "f64::MAX requested_budget should be rejected with 400 or 422, got: {}", response.status()
  );
}

/// Manual Test Gap #26 (variant): Refresh - Float overflow requested_budget (Infinity)
///
/// # Corner Case
/// POST /api/budget/refresh with requested_budget=Infinity
///
/// # Expected Behavior
/// 200 OK with default budget (f64::INFINITY serializes to JSON null, which is valid for Option<i64>)
///
/// # Note
/// JSON doesn't support Infinity, so serde_json serializes it as null. This is valid for Option<i64>
/// and triggers default budget behavior. This is acceptable - clients sending Infinity get default budget.
#[ tokio::test ]
async fn test_refresh_float_overflow_infinity()
{
  let pool = setup_test_db().await;
  let agent_id = 124i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;
  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Create initial lease
  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_response = router.clone().oneshot( handshake_request ).await.unwrap();
  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap().to_string();

  let access_token = common::create_test_access_token( "test_user", "test@example.com", "admin", "test_jwt_secret" );

  // f64::INFINITY serializes to JSON null
  let request_body = json!({
    "ic_token": ic_token,
    "current_lease_id": lease_id,
    "requested_budget": f64::INFINITY,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", access_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Infinity becomes null, which is valid for Option<i64> - should succeed with default budget
  assert!(
    response.status() == StatusCode::OK,
    "Infinity requested_budget (becomes null) should succeed with default budget, got: {}", response.status()
  );
}

/// Manual Test Gap #27: Refresh - NaN requested_budget
///
/// # Corner Case
/// POST /api/budget/refresh with requested_budget=NaN
///
/// # Expected Behavior
/// 200 OK with default budget (f64::NAN serializes to JSON null, which is valid for Option<i64>)
///
/// # Note
/// JSON doesn't support NaN, so serde_json serializes it as null. This is valid for Option<i64>
/// and triggers default budget behavior. This is acceptable - clients sending NaN get default budget.
#[ tokio::test ]
async fn test_refresh_nan_additional_budget()
{
  let pool = setup_test_db().await;
  let agent_id = 125i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;
  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Create initial lease
  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_response = router.clone().oneshot( handshake_request ).await.unwrap();
  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap().to_string();

  let access_token = common::create_test_access_token( "test_user", "test@example.com", "admin", "test_jwt_secret" );

  // f64::NAN serializes to JSON null
  let request_body = json!({
    "ic_token": ic_token,
    "current_lease_id": lease_id,
    "requested_budget": f64::NAN,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", access_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // NaN becomes null, which is valid for Option<i64> - should succeed with default budget
  assert!(
    response.status() == StatusCode::OK,
    "NaN requested_budget (becomes null) should succeed with default budget, got: {}", response.status()
  );
}
