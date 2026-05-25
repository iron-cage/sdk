//! Integration tests for POST /freeform/company and POST /freeform/providers handlers.
//!
//! `test_kind`: integration

#![allow(missing_docs)]

mod common;

use std::sync::Arc;

use axum::{
  body::Body,
  http::{Method, Request, StatusCode},
  routing::post,
  Router,
};
use common::budget::setup_test_db;
use iron_control_api::{
  jwt_auth::JwtSecret,
  routes::{
    auth::AuthState,
    freeform::{post_company, post_providers, FreeformState},
  },
};
use iron_secrets::crypto::CryptoService;
use iron_token_manager::provider_key_storage::ProviderKeyStorage;
use serde_json::{json, Value};
use tower::ServiceExt;

const TEST_JWT_SECRET: &str = "test_jwt_secret_freeform_api_test_77";
const MASTER_KEY: [u8; 32] = [7u8; 32];

#[derive(Clone)]
struct TestState {
  freeform: FreeformState,
  auth: AuthState,
}

impl axum::extract::FromRef<TestState> for FreeformState {
  fn from_ref(s: &TestState) -> Self {
    s.freeform.clone()
  }
}

impl axum::extract::FromRef<TestState> for AuthState {
  fn from_ref(s: &TestState) -> Self {
    s.auth.clone()
  }
}

async fn setup_with_crypto() -> (TestState, String) {
  let pool = setup_test_db().await;

  let crypto = Arc::new(CryptoService::new(&MASTER_KEY).unwrap());
  let storage = Arc::new(ProviderKeyStorage::new(pool.clone()));
  let auth = AuthState::new(TEST_JWT_SECRET.to_string(), "sqlite::memory:", false)
    .await
    .unwrap();

  let freeform = FreeformState {
    pool,
    provider_storage: storage,
    crypto: Some(crypto),
  };

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let token = jwt
    .generate_access_token("admin_1", "admin@test.local", "admin", "tok_001")
    .unwrap();

  (TestState { freeform, auth }, format!("Bearer {token}"))
}

async fn setup_no_crypto() -> (TestState, String) {
  let pool = setup_test_db().await;
  let storage = Arc::new(ProviderKeyStorage::new(pool.clone()));
  let auth = AuthState::new(TEST_JWT_SECRET.to_string(), "sqlite::memory:", false)
    .await
    .unwrap();

  let freeform = FreeformState {
    pool,
    provider_storage: storage,
    crypto: None,
  };

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let token = jwt
    .generate_access_token("admin_1", "admin@test.local", "admin", "tok_002")
    .unwrap();

  (TestState { freeform, auth }, format!("Bearer {token}"))
}

fn dev_bearer() -> String {
  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let token = jwt
    .generate_access_token("dev_1", "dev@test.local", "developer", "tok_dev")
    .unwrap();
  format!("Bearer {token}")
}

fn build_router(state: TestState) -> Router {
  Router::new()
    .route("/api/v1/freeform/company", post(post_company))
    .route("/api/v1/freeform/providers", post(post_providers))
    .with_state(state)
}

async fn body_json(resp: axum::response::Response) -> Value {
  let bytes = axum::body::to_bytes(resp.into_body(), 65_536)
    .await
    .unwrap();
  serde_json::from_slice(&bytes).unwrap_or_default()
}

// POST /freeform/company ------------------------------------------------------

#[tokio::test]
async fn test_company_ok() {
  let (state, bearer) = setup_with_crypto().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/freeform/company")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"text": "Acme Corp, acme.example.com, Client"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::OK);
  let body = body_json(resp).await;
  assert_eq!(body["name"], "Acme Corp");
  assert_eq!(body["domain"], "acme.example.com");
  assert_eq!(body["account_type"], "client");
  assert_eq!(body["id"], 1);
}

#[tokio::test]
async fn test_company_upsert_updates_fields() {
  let (state, bearer) = setup_with_crypto().await;
  let app = build_router(state);

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let bearer2 = format!(
    "Bearer {}",
    jwt
      .generate_access_token("admin_1", "admin@test.local", "admin", "tok_003")
      .unwrap()
  );

  let _ = app
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/freeform/company")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"text": "Old Corp, old.com, Client"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/freeform/company")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer2)
        .body(Body::from(
          json!({"text": "New Corp, new.com, Internal"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::OK);
  let body = body_json(resp).await;
  assert_eq!(body["name"], "New Corp");
  assert_eq!(body["account_type"], "internal");
}

#[tokio::test]
async fn test_company_requires_admin() {
  let (state, _) = setup_with_crypto().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/freeform/company")
        .header("Content-Type", "application/json")
        .header("Authorization", dev_bearer())
        .body(Body::from(
          json!({"text": "Acme, acme.com, Client"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_company_bad_format_returns_400() {
  let (state, bearer) = setup_with_crypto().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/freeform/company")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(json!({"text": "just one field"}).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_company_unknown_account_type_returns_400() {
  let (state, bearer) = setup_with_crypto().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/freeform/company")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"text": "Acme, acme.com, Partner"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// POST /freeform/providers ----------------------------------------------------

#[tokio::test]
async fn test_providers_ok() {
  let (state, bearer) = setup_with_crypto().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/freeform/providers")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"text": "openai: sk-testkey000\nanthropic: sk-ant-testkey000"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::OK);
  let body = body_json(resp).await;
  let applied = body["applied"].as_array().unwrap();
  assert_eq!(applied.len(), 2);
  assert!(applied.iter().any(|e| e["provider"] == "openai"));
  assert!(applied.iter().any(|e| e["provider"] == "anthropic"));
}

#[tokio::test]
async fn test_providers_requires_admin() {
  let (state, _) = setup_with_crypto().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/freeform/providers")
        .header("Content-Type", "application/json")
        .header("Authorization", dev_bearer())
        .body(Body::from(
          json!({"text": "openai: sk-testkey000"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_providers_no_crypto_returns_503() {
  let (state, bearer) = setup_no_crypto().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/freeform/providers")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"text": "openai: sk-testkey000"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_providers_bad_format_returns_400() {
  let (state, bearer) = setup_with_crypto().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/freeform/providers")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"text": "this is not a provider line"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_providers_unsupported_provider_returns_400() {
  let (state, bearer) = setup_with_crypto().await;
  let app = build_router(state);

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/freeform/providers")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(
          json!({"text": "mistral: sk-testkey000"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
