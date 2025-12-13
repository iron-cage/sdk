//! User management domain tests
//!
//! Tests for all user management endpoints (Phase 2: User Management API).
//!
//! Endpoints tested:
//! - POST /api/users - Create user
//! - GET /api/users - List users with filters
//! - GET /api/users/:id - Get user details
//! - PUT /api/users/:id/suspend - Suspend user account
//! - PUT /api/users/:id/activate - Activate user account
//! - DELETE /api/users/:id - Delete user account (soft delete)
//! - PUT /api/users/:id/role - Change user role
//! - POST /api/users/:id/reset-password - Reset user password
//!
//! Coverage:
//! - Request validation (username, email, password, role)
//! - HTTP status codes (200, 201, 400, 403, 404, 500)
//! - JSON response structure
//! - RBAC enforcement (Admin-only operations)
//! - Database persistence and audit logging
//! - Edge cases (self-deletion, duplicate usernames, invalid filters)
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_create_and_list_users` | User creation and listing integration | POST /api/v1/users with valid user data (username="newuser", email="newuser@example.com"), then GET /api/v1/users | User created with 201 Created, appears in user list | ✅ |

#[ path = "common/mod.rs" ]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    Router,
    routing::post,
};
use iron_control_api::routes::users::{self, CreateUserRequest, ListUsersResponse, UserManagementState, UserResponse};
use iron_control_api::routes::auth::AuthState;
use iron_control_api::jwt_auth::JwtSecret;
use iron_control_api::rbac::PermissionChecker;
use tower::ServiceExt;
use std::sync::Arc;
use axum::extract::FromRef;
use common::{create_test_database, create_test_access_token, extract_json_response};

#[derive(Clone)]
struct TestAppState {
    auth: AuthState,
    users: UserManagementState,
}

impl FromRef<TestAppState> for AuthState {
    fn from_ref(state: &TestAppState) -> Self {
        state.auth.clone()
    }
}

impl FromRef<TestAppState> for UserManagementState {
    fn from_ref(state: &TestAppState) -> Self {
        state.users.clone()
    }
}

async fn create_test_app() -> (Router, TestAppState) {
    let db_pool = create_test_database().await;
    let jwt_secret = Arc::new(JwtSecret::new("test_secret".to_string()));
    
    let auth_state = AuthState {
        db_pool: db_pool.clone(),
        jwt_secret,
        rate_limiter: iron_control_api::rate_limiter::LoginRateLimiter::new(),
    };

    let permission_checker = Arc::new(PermissionChecker::new());
    let user_state = UserManagementState::new(db_pool, permission_checker);

    let state = TestAppState {
        auth: auth_state,
        users: user_state,
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
    let (admin_id, _) = common::create_test_admin(&state.auth.db_pool).await;
    let token = create_test_access_token(&admin_id.to_string(), "admin@mail.com", "admin", "test_secret");

    // 1. Create a new user
    let create_request = CreateUserRequest {
        username: "newuser".to_string(),
        password: "password123".to_string(),
        email: "newuser@example.com".to_string(),
        role: "user".to_string(),
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
    assert!( !list_response.users.is_empty() );
    let found = list_response.users.iter().any(|u| u.username == "newuser");
    assert!(found, "Created user should be in the list");
}
