mod common;
use common::{ create_test_user, create_test_admin, create_test_access_token, test_state::TestAppState };
use axum::{
  Router,
  routing::get,
  http::{ StatusCode, Request, Method },
  body::Body,
};
use iron_control_api::routes::agents::{ErrorResponse};
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn create_agents_router() -> ( Router, SqlitePool, String, String, String, String )
{
  let app_state = TestAppState::new().await;
  let ( admin_id, _ ) = create_test_admin( &app_state.database ).await;
  let ( user_id, _ ) = create_test_user( &app_state.database, "regular_user@mail.com" ).await;
  let admin_token = create_test_access_token( &admin_id, "admin@admin.com", "admin", "test_jwt_secret_key_for_testing_12345" );
  let user_token = create_test_access_token( &user_id, "regular_user@mail.com", "user", "test_jwt_secret_key_for_testing_12345" );

  let router = Router::new()
    .route( "/api/agents", get( iron_control_api::routes::agents::list_agents ) )
    .with_state( app_state.clone() );

  ( router, app_state.database.clone(), admin_token, user_token, admin_id, user_id )
}

#[tokio::test]
async fn test_list_agents_validation_error() {
    let (app, _, admin_token, _, _, _) = create_agents_router().await;

    // Test invalid page
    let response = app.clone().oneshot(
        Request::builder()
            .method(Method::GET)
            .uri("/api/agents?page=0")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(error_response.error.code, "VALIDATION_ERROR");
    let fields = error_response.error.fields.as_ref().unwrap();
    assert_eq!(fields.get("page").unwrap(), "Must be >= 1");

    // Test invalid per_page
    let response = app.clone().oneshot(
        Request::builder()
            .method(Method::GET)
            .uri("/api/agents?per_page=101")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(error_response.error.code, "VALIDATION_ERROR");
    let fields = error_response.error.fields.as_ref().unwrap();
    assert_eq!(fields.get("per_page").unwrap(), "Must be between 1 and 100");

    // Test invalid sort
    let response = app.clone().oneshot(
        Request::builder()
            .method(Method::GET)
            .uri("/api/agents?sort=invalid_field")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(error_response.error.code, "VALIDATION_ERROR");
    let fields = error_response.error.fields.as_ref().unwrap();
    assert_eq!(fields.get("sort").unwrap(), "Invalid sort field (allowed: name, budget, created_at)");
}
