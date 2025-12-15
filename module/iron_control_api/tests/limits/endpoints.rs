//! Endpoint integration tests for limits management.
//!
//! Test Matrix: POST /api/limits validation integration
//!
//! | Test Case | Request | Expected Status | Expected Body |
//! |-----------|---------|-----------------|---------------|
//! | Valid single limit | tokens=1000000 | 201 Created | LimitResponse |
//! | All None | all fields None | 422 Unprocessable Entity | error message |
//! | Zero value | tokens=0 | 400 Bad Request | "positive number" |
//! | Negative value | tokens=-100 | 400 Bad Request | "positive number" |
//! | Overflow | tokens=i64::MAX | 400 Bad Request | "too large" |
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_endpoint_valid_request_accepted` | Create limit with valid request | POST /api/limits with tokens=1000000 | 201 Created, LimitResponse returned | ✅ |
//! | `test_endpoint_all_none_rejected` | Create limit with all None fields | POST /api/limits with all fields None | 422 Unprocessable Entity | ✅ |
//! | `test_endpoint_zero_value_rejected` | Create limit with zero value | POST /api/limits with tokens=0 | 400 Bad Request "positive number" | ✅ |
//! | `test_endpoint_negative_value_rejected` | Create limit with negative value | POST /api/limits with tokens=-100 | 400 Bad Request "positive number" | ✅ |
//! | `test_endpoint_overflow_rejected` | Create limit with overflow value | POST /api/limits with tokens=i64::MAX | 400 Bad Request "too large" | ✅ |
//! | `test_endpoint_valid_multiple_limits_accepted` | Create limit with multiple fields | POST /api/limits with multiple valid limits | 201 Created | ✅ |
//! | `test_endpoint_mixed_valid_invalid_rejected` | Create limit with mixed valid/invalid | POST /api/limits with valid + invalid fields | 400 Bad Request | ✅ |
//! | `test_update_limit_all_none_rejected` | Update limit with all None | PUT /api/limits/:id with all fields None | 422 Unprocessable Entity | ✅ |
//! | `test_update_limit_negative_value_rejected` | Update limit with negative value | PUT /api/limits/:id with tokens=-100 | 400 Bad Request "positive number" | ✅ |
//! | `test_update_limit_overflow_rejected` | Update limit with overflow value | PUT /api/limits/:id with tokens=i64::MAX | 400 Bad Request "too large" | ✅ |

use crate::common::{ extract_response, extract_json_response };
use iron_control_api::routes::limits::{ LimitsState, LimitResponse };
use axum::{ Router, routing::post, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;
use serde_json::json;

/// Create test router with limits routes.
async fn create_test_router() -> Router
{
  // Create limits state with in-memory database
  let limits_state = LimitsState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create limits state" );

  Router::new()
    .route( "/api/limits", post( iron_control_api::routes::limits::create_limit ) )
    .with_state( limits_state )
}

/// Test valid limit request returns 201 Created.
#[ tokio::test ]
async fn test_endpoint_valid_request_accepted()
{
  let router = create_test_router().await;

  let request_body = json!({
    "user_id": "user_test",
    "project_id": null,
    "max_tokens_per_day": 1000000,
    "max_requests_per_minute": null,
    "max_cost_per_month_microdollars": null,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "LOUD FAILURE: Valid limit request must return 201 Created"
  );

  let ( status, body ): ( StatusCode, LimitResponse ) = extract_json_response( response ).await;
  assert_eq!( status, StatusCode::CREATED );
  assert_eq!( body.user_id, "user_test" );
  assert_eq!( body.max_tokens_per_day, Some( 1000000 ) );
}

/// Test all None request returns 400 Bad Request.
#[ tokio::test ]
async fn test_endpoint_all_none_rejected()
{
  let router = create_test_router().await;

  let request_body = json!({
    "user_id": "user_test",
    "project_id": null,
    "max_tokens_per_day": null,
    "max_requests_per_minute": null,
    "max_cost_per_month_microdollars": null,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Request with all None limits must return 422 Unprocessable Entity"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::UNPROCESSABLE_ENTITY );
  assert!(
    body.contains( "at least one" ) || body.contains( "error" ),
    "LOUD FAILURE: Error response must contain descriptive message. Got: {}",
    body
  );
}

/// Test zero value returns 400 Bad Request.
#[ tokio::test ]
async fn test_endpoint_zero_value_rejected()
{
  let router = create_test_router().await;

  let request_body = json!({
    "user_id": "user_test",
    "project_id": null,
    "max_tokens_per_day": 0,
    "max_requests_per_minute": null,
    "max_cost_per_month_microdollars": null,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Zero limit value must return 400 Bad Request"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::BAD_REQUEST );
  assert!(
    body.contains( "positive" ),
    "LOUD FAILURE: Error message must indicate positive number required. Got: {}",
    body
  );
}

/// Test negative value returns 400 Bad Request.
#[ tokio::test ]
async fn test_endpoint_negative_value_rejected()
{
  let router = create_test_router().await;

  let request_body = json!({
    "user_id": "user_test",
    "project_id": null,
    "max_tokens_per_day": -100,
    "max_requests_per_minute": null,
    "max_cost_per_month_microdollars": null,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Negative limit value must return 400 Bad Request"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::BAD_REQUEST );
  assert!(
    body.contains( "positive" ),
    "LOUD FAILURE: Error message must indicate positive number required. Got: {}",
    body
  );
}

/// Test overflow value returns 400 Bad Request.
#[ tokio::test ]
async fn test_endpoint_overflow_rejected()
{
  let router = create_test_router().await;

  let request_body = json!({
    "user_id": "user_test",
    "project_id": null,
    "max_tokens_per_day": i64::MAX,
    "max_requests_per_minute": null,
    "max_cost_per_month_microdollars": null,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Overflow limit value must return 400 Bad Request"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::BAD_REQUEST );
  assert!(
    body.contains( "too large" ) || body.contains( "Maximum" ),
    "LOUD FAILURE: Error message must indicate value too large. Got: {}",
    body
  );
}

/// Test valid multiple limits accepted.
#[ tokio::test ]
async fn test_endpoint_valid_multiple_limits_accepted()
{
  let router = create_test_router().await;

  let request_body = json!({
    "user_id": "user_test",
    "project_id": "project_abc",
    "max_tokens_per_day": 1000000,
    "max_requests_per_minute": 100,
    "max_cost_per_month_microdollars": 50000,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "LOUD FAILURE: Valid request with multiple limits must return 201 Created"
  );

  let ( status, body ): ( StatusCode, LimitResponse ) = extract_json_response( response ).await;
  assert_eq!( status, StatusCode::CREATED );
  assert_eq!( body.max_tokens_per_day, Some( 1000000 ) );
  assert_eq!( body.max_requests_per_minute, Some( 100 ) );
  assert_eq!( body.max_cost_per_month_microdollars, Some( 50000 ) );
}

/// Test mixed valid/invalid rejected.
#[ tokio::test ]
async fn test_endpoint_mixed_valid_invalid_rejected()
{
  let router = create_test_router().await;

  let request_body = json!({
    "user_id": "user_test",
    "project_id": null,
    "max_tokens_per_day": 1000000,
    "max_requests_per_minute": 0,
    "max_cost_per_month_microdollars": null,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Request with mixed valid/invalid limits must return 400 Bad Request"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::BAD_REQUEST );
  assert!(
    body.contains( "positive" ),
    "LOUD FAILURE: Error message must indicate which field failed. Got: {}",
    body
  );
}

//
// PUT /api/limits/:id validation tests
//

/// Create test router with update_limit route.
async fn create_update_test_router() -> Router
{
  // Create limits state with in-memory database
  let limits_state = LimitsState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create limits state" );

  Router::new()
    .route( "/api/limits/:id", axum::routing::put( iron_control_api::routes::limits::update_limit ) )
    .with_state( limits_state )
}

/// Test update_limit rejects all None.
#[ tokio::test ]
async fn test_update_limit_all_none_rejected()
{
  let router = create_update_test_router().await;

  let request_body = json!({
    "max_tokens_per_day": null,
    "max_requests_per_minute": null,
    "max_cost_per_month_microdollars": null,
  });

  let request = Request::builder()
    .method( "PUT" )
    .uri( "/api/limits/1" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Update with all None limits must return 422 Unprocessable Entity"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::UNPROCESSABLE_ENTITY );
  assert!(
    body.contains( "at least one" ) || body.contains( "error" ),
    "LOUD FAILURE: Error response must contain descriptive message. Got: {}",
    body
  );
}

/// Test update_limit rejects negative value.
#[ tokio::test ]
async fn test_update_limit_negative_value_rejected()
{
  let router = create_update_test_router().await;

  let request_body = json!({
    "max_tokens_per_day": -100,
    "max_requests_per_minute": null,
    "max_cost_per_month_microdollars": null,
  });

  let request = Request::builder()
    .method( "PUT" )
    .uri( "/api/limits/1" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Update with negative value must return 400 Bad Request"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::BAD_REQUEST );
  assert!(
    body.contains( "positive" ),
    "LOUD FAILURE: Error message must indicate positive number required. Got: {}",
    body
  );
}

/// Test update_limit rejects overflow.
#[ tokio::test ]
async fn test_update_limit_overflow_rejected()
{
  let router = create_update_test_router().await;

  let request_body = json!({
    "max_tokens_per_day": i64::MAX,
    "max_requests_per_minute": null,
    "max_cost_per_month_microdollars": null,
  });

  let request = Request::builder()
    .method( "PUT" )
    .uri( "/api/limits/1" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Update with overflow value must return 400 Bad Request"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::BAD_REQUEST );
  assert!(
    body.contains( "too large" ) || body.contains( "Maximum" ),
    "LOUD FAILURE: Error message must indicate value too large. Got: {}",
    body
  );
}
