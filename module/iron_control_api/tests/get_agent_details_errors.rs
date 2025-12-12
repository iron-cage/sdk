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
    .route( "/api/agents/:id/details", get( iron_control_api::routes::agents::get_agent_details ) )
    .with_state( app_state.clone() );

  ( router, app_state.database.clone(), admin_token, user_token, admin_id, user_id )
}

#[tokio::test]
async fn test_get_agent_details_not_found_error() {
    let (app, _, admin_token, _, _, _) = create_agents_router().await;

    let response = app.oneshot(
        Request::builder()
            .method(Method::GET)
            .uri("/api/agents/agent_invalid/details")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(error_response.error.code, "AGENT_NOT_FOUND");
    assert_eq!(error_response.error.message.unwrap(), "Agent 'agent_invalid' does not exist");
}

#[tokio::test]
async fn test_get_agent_details_forbidden_error() {
    let (app, pool, _, user_token, admin_id, _) = create_agents_router().await;

    // Create agent owned by admin
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)")
        .bind("agent_admin")
        .bind("Admin Agent")
        .bind("[]")
        .bind(now)
        .bind(&admin_id)
        .execute(&pool)
        .await
        .unwrap();

    // Regular user tries to access admin's agent
    let response = app.oneshot(
        Request::builder()
            .method(Method::GET)
            .uri("/api/agents/agent_admin/details")
            .header("authorization", format!("Bearer {}", user_token))
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(error_response.error.code, "FORBIDDEN");
    assert_eq!(error_response.error.message.unwrap(), "Insufficient permissions");
}
