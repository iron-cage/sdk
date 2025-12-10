//! User management integration tests

#[path = "common/mod.rs"]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    Router,
    routing::post,
};
use iron_control_api::routes::users::{self, CreateUserRequest, UserResponse, ListUsersResponse};
use iron_control_api::routes::auth::AuthState;
use iron_control_api::jwt_auth::JwtSecret;
use tower::ServiceExt;
use std::sync::Arc;
use common::{create_test_database, create_test_access_token, extract_json_response};

async fn create_test_app() -> (Router, AuthState) {
    let db_pool = create_test_database().await;
    let jwt_secret = Arc::new(JwtSecret::new("test_secret".to_string()));
    
    let state = AuthState {
        db_pool,
        jwt_secret,
    };

    let router = Router::new()
        .route("/api/v1/users", post(users::create_user).get(users::list_users))
        .with_state(state.clone());

    (router, state)
}

#[tokio::test]
async fn test_create_and_list_users() {
    let (router, state) = create_test_app().await;

    // Create admin user for auth
    let (admin_id, _) = common::create_test_user(&state.db_pool, "admin").await;
    // Update role to admin
    sqlx::query("UPDATE users SET role = 'admin' WHERE id = ?")
        .bind(admin_id)
        .execute(&state.db_pool)
        .await
        .unwrap();

    let token = create_test_access_token("admin", "admin", "test_secret");

    // 1. Create a new user
    let create_request = CreateUserRequest {
        username: "newuser".to_string(),
        password: "password123".to_string(),
        email: "newuser@example.com".to_string(),
        role: Some("user".to_string()),
    };

    let response = router.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/users")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&create_request).unwrap()))
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    
    let (_status, user_response): (StatusCode, UserResponse) = extract_json_response(response).await;
    assert_eq!(user_response.username, "newuser");
    assert_eq!(user_response.email.as_deref(), Some("newuser@example.com"));

    // 2. List users
    let response = router.oneshot(
        Request::builder()
            .method("GET")
            .uri("/api/v1/users")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let (_status, list_response): (StatusCode, ListUsersResponse) = extract_json_response(response).await;
    assert!(list_response.users.len() >= 1);
    let found = list_response.users.iter().any(|u| u.username == "newuser");
    assert!(found, "Created user should be in the list");
}
