//! Integration tests for POST /api/v1/workspace/budget endpoint.
//!
//! `test_kind`: integration

#![allow(missing_docs)]

mod common;

use axum::{
  body::Body,
  http::{Method, Request, StatusCode},
  routing::post,
  Router,
};
use common::budget::setup_test_db;
use iron_control_api::{
  jwt_auth::JwtSecret,
  routes::{auth::AuthState, workspace::post_workspace_budget},
};
use serde_json::json;
use tower::ServiceExt;

const TEST_JWT_SECRET: &str = "test_jwt_secret_workspace_budget_99";

#[derive(Clone)]
struct TestState {
  pool: sqlx::SqlitePool,
  auth: AuthState,
}

impl axum::extract::FromRef<TestState> for sqlx::SqlitePool {
  fn from_ref(s: &TestState) -> Self {
    s.pool.clone()
  }
}

impl axum::extract::FromRef<TestState> for AuthState {
  fn from_ref(s: &TestState) -> Self {
    s.auth.clone()
  }
}

async fn setup() -> (TestState, String) {
  let pool = setup_test_db().await;

  sqlx::query(
    r"
      INSERT OR REPLACE INTO workspaces (id, name, domain, account_type)
      VALUES (1, 'Test Workspace', 'test.example.com', 'client')
    ",
  )
  .execute(&pool)
  .await
  .expect("LOUD FAILURE: Failed to seed workspace");

  let auth = AuthState::new(TEST_JWT_SECRET.to_string(), "sqlite::memory:", false)
    .await
    .expect("LOUD FAILURE: Failed to create test AuthState");

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let token = jwt
    .generate_access_token("user_1", "admin@test.com", "admin", "tok_001")
    .expect("LOUD FAILURE: Failed to generate test JWT");
  let bearer = format!("Bearer {token}");

  (TestState { pool, auth }, bearer)
}

fn build_router(state: TestState) -> Router {
  Router::new()
    .route("/api/v1/workspace/budget", post(post_workspace_budget))
    .with_state(state)
}

// Structured input ------------------------------------------------------------

#[tokio::test]
async fn test_structured_input_ok() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workspace/budget")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"amount_cents": 10000, "period": "week"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::OK);
  let body: serde_json::Value =
    serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
  assert_eq!(body["workspace_id"], 1);
  assert_eq!(body["amount_cents"], 10000);
  assert_eq!(body["period"], "week");
}

#[tokio::test]
async fn test_structured_upsert_overwrites() {
  let (state, bearer_1) = setup().await;
  let app1 = build_router(state.clone());
  let app2 = build_router(state);

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let token = jwt
    .generate_access_token("user_1", "admin@test.com", "admin", "tok_002")
    .unwrap();
  let bearer_2 = format!("Bearer {token}");

  let _ = app1
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workspace/budget")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer_1)
        .body(Body::from(
          json!({"amount_cents": 5000, "period": "day"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  let resp = app2
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workspace/budget")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer_2)
        .body(Body::from(
          json!({"amount_cents": 20000, "period": "month"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::OK);
  let body: serde_json::Value =
    serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
  assert_eq!(body["amount_cents"], 20000);
  assert_eq!(body["period"], "month");
}

// Text (freeform) input -------------------------------------------------------

#[tokio::test]
async fn test_text_input_ok() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workspace/budget")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"text": "limit all users $250/month"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::OK);
  let body: serde_json::Value =
    serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
  assert_eq!(body["amount_cents"], 25000);
  assert_eq!(body["period"], "month");
}

#[tokio::test]
async fn test_text_input_no_cap_returns_400() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workspace/budget")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"text": "default: claude-3-5-sonnet-20241022"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// Validation errors -----------------------------------------------------------

#[tokio::test]
async fn test_missing_both_inputs_returns_400() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workspace/budget")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(json!({}).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_period_returns_400() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workspace/budget")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"amount_cents": 1000, "period": "year"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_negative_amount_returns_400() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workspace/budget")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"amount_cents": -100, "period": "week"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

//  RBAC -----------------------------------------------------------------------

#[tokio::test]
async fn test_non_admin_returns_403() {
  let (state, _) = setup().await;
  let app = build_router(state);

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let token = jwt
    .generate_access_token("dev_user", "dev@test.com", "developer", "tok_dev")
    .unwrap();
  let bearer = format!("Bearer {token}");

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workspace/budget")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"amount_cents": 5000, "period": "week"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_unauthenticated_returns_401() {
  let (state, _) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workspace/budget")
        .header("Content-Type", "application/json")
        .body(Body::from(
          json!({"amount_cents": 5000, "period": "week"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// test_kind: bug_reproducer(issue-pr68-corr5)
//
// Root Cause: `post_workspace_budget` (workspace.rs:90) guards with
//   `Permission::ManageUsers`. Setting the workspace budget is an
//   IC-token-tier control, so the correct guard is `Permission::ManageIcTokens`
//   (matching ic_token.rs:94). Under the RBAC matrix (rbac.rs:91) `Manager`
//   holds `ManageIcTokens` but NOT `ManageUsers`, so a Manager is wrongly
//   rejected with 403 from a control they are entitled to use.
//
// Why Not Caught: the existing RBAC tests cover only Admin (allowed, holds all
//   permissions) and Developer (denied, holds none). Manager is the only role
//   that distinguishes `ManageUsers` from `ManageIcTokens`, and it was never
//   exercised against this endpoint.
//
// Fix Applied: change the guard from `Permission::ManageUsers` to
//   `Permission::ManageIcTokens` in `post_workspace_budget`.
//
// Prevention: when choosing an RBAC permission for an endpoint, test the
//   boundary role that holds the intended permission but not the adjacent one,
//   not just the all-or-nothing Admin/Developer roles.
//
// Pitfall to Avoid: assert on the status transition (Manager: 403 -> 200), not
//   on the 403 body text, so the test pins authorization behavior rather than
//   error wording.
#[tokio::test]
async fn test_manager_can_set_budget() {
  let (state, _) = setup().await;
  let app = build_router(state);

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let token = jwt
    .generate_access_token("mgr_user", "mgr@test.com", "manager", "tok_mgr")
    .unwrap();
  let bearer = format!("Bearer {token}");

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workspace/budget")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"amount_cents": 5000, "period": "week"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::OK,
    "Manager holds ManageIcTokens and must be allowed to set the workspace budget"
  );
}
