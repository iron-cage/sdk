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
};
use serde_json::json;
use tower::ServiceExt;

/// Manual Test Gap #25: Refresh - NULL additional_budget field
///
/// # Corner Case
/// POST /api/budget/refresh with additional_budget=null
///
/// # Expected Behavior
/// 400 Bad Request "additional_budget is required"
///
/// # Risk
/// MEDIUM - Budget corruption
#[ tokio::test ]
async fn test_refresh_null_additional_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 122, 100_000_000 ).await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  // Craft request with null additional_budget
  let request_body = json!({
    "agent_id": 122,
    "additional_budget": null,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "NULL additional_budget should be rejected with 400 or 422, got: {}", response.status()
  );
}

/// Manual Test Gap #26: Refresh - Float overflow additional_budget (f64::MAX)
///
/// # Corner Case
/// POST /api/budget/refresh with additional_budget=f64::MAX
///
/// # Expected Behavior
/// 400 Bad Request
///
/// # Risk
/// MEDIUM - Budget overflow
#[ tokio::test ]
async fn test_refresh_float_overflow_f64_max()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 123, 100_000_000 ).await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  let request_body = json!({
    "agent_id": 123,
    "additional_budget": f64::MAX,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "f64::MAX additional_budget should be rejected with 400 or 422, got: {}", response.status()
  );
}

/// Manual Test Gap #26 (variant): Refresh - Float overflow additional_budget (Infinity)
///
/// # Corner Case
/// POST /api/budget/refresh with additional_budget=Infinity
///
/// # Expected Behavior
/// 400 Bad Request
///
/// # Risk
/// MEDIUM - Budget overflow
#[ tokio::test ]
async fn test_refresh_float_overflow_infinity()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 124, 100_000_000 ).await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  let request_body = json!({
    "agent_id": 124,
    "additional_budget": f64::INFINITY,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "Infinity additional_budget should be rejected with 400 or 422, got: {}", response.status()
  );
}

/// Manual Test Gap #27: Refresh - NaN additional_budget
///
/// # Corner Case
/// POST /api/budget/refresh with additional_budget=NaN
///
/// # Expected Behavior
/// 400 Bad Request
///
/// # Risk
/// MEDIUM - Budget corruption
#[ tokio::test ]
async fn test_refresh_nan_additional_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 125, 100_000_000 ).await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  let request_body = json!({
    "agent_id": 125,
    "additional_budget": f64::NAN,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "NaN additional_budget should be rejected with 400 or 422, got: {}", response.status()
  );
}
