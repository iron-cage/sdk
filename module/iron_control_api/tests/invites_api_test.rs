//! Integration tests for invite link endpoints.

#![allow(missing_docs)]

mod common;

use axum::{
  body::Body,
  http::{Method, Request, StatusCode},
  routing::{get, post},
  Router,
};
use common::budget::setup_test_db;
use iron_control_api::{
  jwt_auth::JwtSecret,
  routes::{
    auth::AuthState,
    invites::{
      get_invite_preview, post_invite_accept, post_invite_approve, post_invite_generate,
      InviteState,
    },
  },
};
use serde_json::{json, Value};
use tower::ServiceExt;

const TEST_JWT_SECRET: &str = "test_jwt_secret_invites_api_test_42";
const ADMIN_USER_ID: &str = "admin_user_001";

#[derive(Clone)]
struct TestState {
  invite: InviteState,
  auth: AuthState,
}

impl axum::extract::FromRef<TestState> for InviteState {
  fn from_ref(s: &TestState) -> Self {
    s.invite.clone()
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
      INSERT OR IGNORE INTO workspaces (id, name, domain, account_type)
      VALUES (1, 'Test Corp', 'test.example.com', 'client')
    ",
  )
  .execute(&pool)
  .await
  .expect("LOUD FAILURE: Failed to seed workspace");

  let now_ms: i64 = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis()
    .try_into()
    .unwrap_or(i64::MAX);
  let password_hash = bcrypt::hash("adminpass123", 4).unwrap();
  sqlx::query(
    r"
      INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
      VALUES (?, 'test_admin', ?, 'admin@test.local', 'admin', 1, ?)
    ",
  )
  .bind(ADMIN_USER_ID)
  .bind(&password_hash)
  .bind(now_ms)
  .execute(&pool)
  .await
  .expect("LOUD FAILURE: Failed to seed admin user");

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let token = jwt
    .generate_access_token(ADMIN_USER_ID, "admin@test.local", "admin", "tok_admin_001")
    .expect("LOUD FAILURE: Failed to generate admin JWT");
  let bearer = format!("Bearer {token}");

  let auth = AuthState::new(TEST_JWT_SECRET.to_string(), "sqlite::memory:", false)
    .await
    .expect("LOUD FAILURE: Failed to create AuthState");

  let invite = InviteState {
    pool,
    jwt_secret: auth.jwt_secret.clone(),
  };

  (TestState { invite, auth }, bearer)
}

fn build_router(state: TestState) -> Router {
  Router::new()
    .route("/api/v1/invites/generate", post(post_invite_generate))
    .route("/api/v1/invites/{token}", get(get_invite_preview))
    .route("/api/v1/invites/{token}/accept", post(post_invite_accept))
    .route("/api/v1/invites/{token}/approve", post(post_invite_approve))
    .with_state(state)
}

async fn body_json(resp: axum::response::Response) -> Value {
  let bytes = axum::body::to_bytes(resp.into_body(), 65_536)
    .await
    .unwrap();
  serde_json::from_slice(&bytes).unwrap_or_default()
}

// Generate --------------------------------------------------------------------

#[tokio::test]
async fn test_generate_invite_ok() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/invites/generate")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(json!({"seats": 5}).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::CREATED);
  let body = body_json(resp).await;
  assert!(body["token"].is_string());
  assert!(body["url"].as_str().unwrap().starts_with("/join/"));
  assert_eq!(body["seats_total"], 5);
  assert!(body["expires_at"].is_null());
}

#[tokio::test]
async fn test_generate_with_expiry_ok() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/invites/generate")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"seats": 1, "expires_in_hours": 48}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::CREATED);
  let body = body_json(resp).await;
  assert!(body["expires_at"].is_number());
}

#[tokio::test]
async fn test_generate_requires_admin() {
  let (state, _) = setup().await;
  let app = build_router(state);

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let token = jwt
    .generate_access_token("dev_1", "dev@test.local", "developer", "tok_dev")
    .unwrap();
  let bearer = format!("Bearer {token}");

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/invites/generate")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(json!({"seats": 1}).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_generate_zero_seats_returns_400() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/invites/generate")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(json!({"seats": 0}).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// Preview ---------------------------------------------------------------------

#[tokio::test]
async fn test_preview_invite_ok() {
  let (state, bearer) = setup().await;
  let pool = state.invite.pool.clone();
  let app = build_router(state);

  // Generate a link first
  let gen_resp = app
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/invites/generate")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(json!({"seats": 3}).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();
  let _ = pool;
  let gen_body = body_json(gen_resp).await;
  let token = gen_body["token"].as_str().unwrap().to_string();

  let preview_resp = app
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/invites/{token}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(preview_resp.status(), StatusCode::OK);
  let body = body_json(preview_resp).await;
  assert_eq!(body["workspace_name"], "Test Corp");
  assert_eq!(body["domain"], "test.example.com");
  assert_eq!(body["seats_remaining"], 3);
}

#[tokio::test]
async fn test_preview_unknown_token_returns_404() {
  let (state, _) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/v1/invites/totally_fake_token_that_doesnt_exist")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// Accept ----------------------------------------------------------------------

#[tokio::test]
async fn test_accept_invite_ok() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let gen_body = body_json(
    app
      .clone()
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/v1/invites/generate")
          .header("Content-Type", "application/json")
          .header("Authorization", bearer)
          .body(Body::from(json!({"seats": 2}).to_string()))
          .unwrap(),
      )
      .await
      .unwrap(),
  )
  .await;

  let token = gen_body["token"].as_str().unwrap().to_string();

  let accept_resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri(format!("/api/v1/invites/{token}/accept"))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({"password": "secure_pw_123"}).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(accept_resp.status(), StatusCode::CREATED);
  let body = body_json(accept_resp).await;
  assert!(body["user_id"].is_string());
  assert!(body["access_token"].is_string());
}

#[tokio::test]
async fn test_accept_short_password_returns_400() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let gen_body = body_json(
    app
      .clone()
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/v1/invites/generate")
          .header("Content-Type", "application/json")
          .header("Authorization", bearer)
          .body(Body::from(json!({"seats": 1}).to_string()))
          .unwrap(),
      )
      .await
      .unwrap(),
  )
  .await;
  let token = gen_body["token"].as_str().unwrap().to_string();

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri(format!("/api/v1/invites/{token}/accept"))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({"password": "short"}).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_accept_unknown_token_returns_404() {
  let (state, _) = setup().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/invites/nonexistent_token/accept")
        .header("Content-Type", "application/json")
        .body(Body::from(
          json!({"password": "secure_password"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// Approve ---------------------------------------------------------------------

#[tokio::test]
async fn test_approve_pending_member_ok() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let admin_bearer_2 = format!(
    "Bearer {}",
    jwt
      .generate_access_token(ADMIN_USER_ID, "admin@test.local", "admin", "tok_admin_002")
      .unwrap()
  );

  // 1. Generate invite
  let gen_body = body_json(
    app
      .clone()
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/v1/invites/generate")
          .header("Content-Type", "application/json")
          .header("Authorization", bearer)
          .body(Body::from(json!({"seats": 5}).to_string()))
          .unwrap(),
      )
      .await
      .unwrap(),
  )
  .await;
  let token = gen_body["token"].as_str().unwrap().to_string();

  // 2. Accept invite
  let accept_body = body_json(
    app
      .clone()
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri(format!("/api/v1/invites/{token}/accept"))
          .header("Content-Type", "application/json")
          .body(Body::from(json!({"password": "member_pw_123"}).to_string()))
          .unwrap(),
      )
      .await
      .unwrap(),
  )
  .await;
  let new_user_id = accept_body["user_id"].as_str().unwrap().to_string();

  // 3. Approve pending member
  let approve_resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri(format!("/api/v1/invites/{token}/approve"))
        .header("Content-Type", "application/json")
        .header("Authorization", admin_bearer_2)
        .body(Body::from(json!({"user_id": new_user_id}).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(approve_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_approve_requires_admin() {
  let (state, _) = setup().await;
  let app = build_router(state);

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let dev_bearer = format!(
    "Bearer {}",
    jwt
      .generate_access_token("dev_user", "dev@test.local", "developer", "tok_dev_2")
      .unwrap()
  );

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/invites/some_token/approve")
        .header("Content-Type", "application/json")
        .header("Authorization", dev_bearer)
        .body(Body::from(json!({"user_id": "some_user"}).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_approve_unknown_user_returns_404() {
  let (state, bearer) = setup().await;
  let app = build_router(state);

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let admin_bearer_2 = format!(
    "Bearer {}",
    jwt
      .generate_access_token(ADMIN_USER_ID, "admin@test.local", "admin", "tok_admin_003")
      .unwrap()
  );

  // Generate invite
  let gen_body = body_json(
    app
      .clone()
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/v1/invites/generate")
          .header("Content-Type", "application/json")
          .header("Authorization", bearer)
          .body(Body::from(json!({"seats": 1}).to_string()))
          .unwrap(),
      )
      .await
      .unwrap(),
  )
  .await;
  let token = gen_body["token"].as_str().unwrap().to_string();

  // Try to approve non-existent user
  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri(format!("/api/v1/invites/{token}/approve"))
        .header("Content-Type", "application/json")
        .header("Authorization", admin_bearer_2)
        .body(Body::from(
          json!({"user_id": "nonexistent_user_id"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
