mod common;
use common::{ create_test_user, create_test_admin, create_test_access_token, test_state::TestAppState };
use axum::{
  Router,
  routing::post,
  http::{ StatusCode, Request, Method },
  body::Body,
};
use iron_control_api::routes::agents::{ErrorResponse};
use sqlx::SqlitePool;
use serde_json::json;
use tower::ServiceExt;

async fn create_agents_router() -> ( Router, SqlitePool, String, String, String, String )
{
  let app_state = TestAppState::new().await;
  let ( admin_id, _ ) = create_test_admin( &app_state.database ).await;
  let ( user_id, _ ) = create_test_user( &app_state.database, "regular_user@mail.com" ).await;
  let admin_token = create_test_access_token( &admin_id, "admin@admin.com", "admin", "test_jwt_secret_key_for_testing_12345" );
  let user_token = create_test_access_token( &user_id, "regular_user@mail.com", "user", "test_jwt_secret_key_for_testing_12345" );

  let router = Router::new()
    .route( "/api/agents", post( iron_control_api::routes::agents::create_agent ) )
    .with_state( app_state.clone() );

  ( router, app_state.database.clone(), admin_token, user_token, admin_id, user_id )
}

#[tokio::test]
async fn test_create_agent_validation_error() {
    let (app, _, admin_token, _, admin_id, _) = create_agents_router().await;

    let request_body = json!({
        "name": "",
        "budget": 0.0,
        "owner_id": admin_id
    });

    let response = app.oneshot(
        Request::builder()
            .method(Method::POST)
            .uri("/api/agents")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(error_response.error.code, "VALIDATION_ERROR");
    let fields = error_response.error.fields.unwrap();
    assert_eq!(fields.get("budget").unwrap(), "Must be >= 0.01");
    assert_eq!(fields.get("name").unwrap(), "Required field");
}

#[tokio::test]
async fn test_create_agent_forbidden() {
    let (app, _, _, user_token, admin_id, _) = create_agents_router().await;

    // Regular user trying to create agent for admin
    let request_body = json!({
        "name": "Test Agent",
        "budget": 100.0,
        "owner_id": admin_id 
    });

    let response = app.oneshot(
        Request::builder()
            .method(Method::POST)
            .uri("/api/agents")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", user_token))
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(error_response.error.code, "FORBIDDEN");
    assert_eq!(error_response.error.message.unwrap(), "Insufficient permissions");
}

#[tokio::test]
async fn test_create_agent_provider_not_found() {
    let (app, _, admin_token, _, admin_id, _) = create_agents_router().await;

    let request_body = json!({
        "name": "Test Agent",
        "budget": 100.0,
        "owner_id": admin_id,
        "providers": ["ip_invalid_001"]
    });

    let response = app.oneshot(
        Request::builder()
            .method(Method::POST)
            .uri("/api/agents")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(error_response.error.code, "PROVIDER_NOT_FOUND");
    assert_eq!(error_response.error.message.unwrap(), "Provider 'ip_invalid_001' does not exist");
}
