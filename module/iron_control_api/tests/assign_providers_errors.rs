mod common;
use common::{ create_test_user, create_test_admin, create_test_access_token, test_state::TestAppState };
use axum::{
  Router,
  routing::put,
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
    .route( "/api/agents/:id/providers", put( iron_control_api::routes::agents::assign_providers_to_agent ) )
    .with_state( app_state.clone() );

  ( router, app_state.database.clone(), admin_token, user_token, admin_id, user_id )
}

#[tokio::test]
async fn test_assign_providers_validation_error() {
    let (app, pool, admin_token, _, admin_id, _) = create_agents_router().await;

    // Create agent
    let now = chrono::Utc::now().timestamp_millis();
    let result = sqlx::query("INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)")
        .bind("Test Agent")
        .bind("[]")
        .bind(now)
        .bind(&admin_id)
        .execute(&pool)
        .await
        .unwrap();

    let agent_id = result.last_insert_rowid();

    let request_body = json!({
        "providers": []
    });

    let response = app.oneshot(
        Request::builder()
            .method(Method::PUT)
            .uri(format!("/api/agents/{}/providers", agent_id))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(error_response.error.code, "VALIDATION_ERROR");
    assert_eq!(error_response.error.message.unwrap(), "providers field is required");
}

#[tokio::test]
async fn test_assign_providers_invalid_provider() {
    let (app, pool, admin_token, _, admin_id, _) = create_agents_router().await;

    // Create agent
    let now = chrono::Utc::now().timestamp_millis();
    let result = sqlx::query("INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)")
        .bind("Test Agent")
        .bind("[]")
        .bind(now)
        .bind(&admin_id)
        .execute(&pool)
        .await
        .unwrap();

    let agent_id = result.last_insert_rowid();

    let request_body = json!({
        "providers": ["ip_unknown_999"]
    });

    let response = app.oneshot(
        Request::builder()
            .method(Method::PUT)
            .uri(format!("/api/agents/{}/providers", agent_id))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(error_response.error.code, "INVALID_PROVIDER_ID");
    assert_eq!(error_response.error.message.unwrap(), "Provider not found: ip_unknown_999");
}

#[tokio::test]
async fn test_assign_providers_forbidden() {
    let (app, pool, _, user_token, admin_id, _) = create_agents_router().await;

    // Create agent owned by admin
    let now = chrono::Utc::now().timestamp_millis();
    let result = sqlx::query("INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)")
        .bind("Admin Agent")
        .bind("[]")
        .bind(now)
        .bind(&admin_id)
        .execute(&pool)
        .await
        .unwrap();

    let agent_id = result.last_insert_rowid();

    let request_body = json!({
        "providers": ["1"]
    });

    let response = app.oneshot(
        Request::builder()
            .method(Method::PUT)
            .uri(format!("/api/agents/{}/providers", agent_id))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", user_token))
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(error_response.error.code, "FORBIDDEN");
    assert_eq!(error_response.error.message.unwrap(), "Insufficient permissions: You can only modify providers for your own agents");
}

#[tokio::test]
async fn test_assign_providers_not_found() {
    let (app, _, admin_token, _, _, _) = create_agents_router().await;

    let request_body = json!({
        "providers": ["1"]
    });

    let response = app.oneshot(
        Request::builder()
            .method(Method::PUT)
            .uri("/api/agents/12345/providers")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(error_response.error.code, "AGENT_NOT_FOUND");
    assert_eq!(error_response.error.message.unwrap(), "Agent not found: 12345");
}
