//! Integration tests for providers API: RBAC, validation, happy paths, ownership.
//!
//! Test Matrix:
//! | Test Name                          | Purpose                    | Verification       |
//! |------------------------------------|----------------------------|--------------------|
//! | list_…_requires_manage_permission  | RBAC: dev cannot list      | 403 Forbidden      |
//! | get_…_requires_manage_permission   | RBAC: dev cannot get       | 403 Forbidden      |
//! | update_…_requires_manage_perm      | RBAC: dev cannot update    | 403 Forbidden      |
//! | delete_…_requires_manage_perm      | RBAC: dev cannot delete    | 403 Forbidden      |
//! | assign_requires_manage_permission  | RBAC: dev cannot assign    | 403 Forbidden      |
//! | unassign_requires_manage_perm      | RBAC: dev cannot unassign  | 403 Forbidden      |
//! | create_rejects_empty_api_key       | Empty api_key rejected     | 400 Bad Request    |
//! | create_rejects_api_key_too_long    | 501-char api_key rejected  | 400 Bad Request    |
//! | create_rejects_null_byte_in_key    | NULL byte in key rejected  | 400 Bad Request    |
//! | create_rejects_invalid_provider    | Unknown provider rejected  | 400 Bad Request    |
//! | create_rejects_description_long    | 501-char desc rejected     | 400 Bad Request    |
//! | create_disabled_without_master_key | No crypto → 503            | 503 Unavailable    |
//! | update_provider_key_success        | Update desc & is_enabled   | 200 OK w/ updates  |
//! | delete_provider_key_success        | Delete then GET → 404      | 204 then 404       |
//! | assign_and_unassign_project_prov   | Assign then unassign       | 200 then 204       |
//! | get_…_returns_404_for_wrong_owner  | Wrong owner GET            | 404 Not Found      |
//! | update_…_404_for_wrong_owner       | Wrong owner PUT            | 404 Not Found      |
//! | delete_…_404_for_wrong_owner       | Wrong owner DELETE         | 404 Not Found      |
//! | unassign_403_for_wrong_owner       | Non-owner unassign         | 403 Forbidden      |
//! | cross_tenant_project_assign_reject | Cross-tenant assign        | 404 Not Found      |

#![allow(missing_docs)]

mod common;

use std::sync::Arc;

use axum::{
  body::Body,
  http::{Method, Request, StatusCode},
  routing::{delete, get, post, put},
  Router,
};
use common::budget::setup_test_db;
use common::providers::{bearer, make_providers_state, TestProvidersAppState, TEST_JWT_SECRET};
use iron_control_api::{
  jwt_auth::JwtSecret,
  routes::{
    auth::AuthState,
    providers::{
      assign_provider_to_project, create_provider_key, delete_provider_key, get_provider_key,
      list_provider_keys, unassign_provider_from_project, update_provider_key, ProvidersState,
    },
  },
};
use iron_token_manager::provider_key_storage::{ProviderKeyStorage, ProviderType};
use serde_json::json;
use tower::ServiceExt;

// ─────────────────────────────────────────────────────────────────
// Constants & shared state builders
// ─────────────────────────────────────────────────────────────────

async fn make_providers_state_no_crypto(pool: &sqlx::SqlitePool) -> TestProvidersAppState {
  let storage = Arc::new(ProviderKeyStorage::new(pool.clone()));
  let providers = ProvidersState { storage, crypto: None };
  let auth = AuthState::new(TEST_JWT_SECRET.to_string(), "sqlite::memory:", false)
    .await
    .expect("LOUD FAILURE: Failed to create test AuthState");
  TestProvidersAppState { providers, auth }
}

/// Build a router with all 7 provider handler routes
fn build_full_router(state: TestProvidersAppState) -> Router {
  Router::new()
    .route("/api/v1/providers", post(create_provider_key))
    .route("/api/v1/providers", get(list_provider_keys))
    .route("/api/v1/providers/{id}", get(get_provider_key))
    .route("/api/v1/providers/{id}", put(update_provider_key))
    .route("/api/v1/providers/{id}", delete(delete_provider_key))
    .route("/api/v1/projects/{id}/provider", post(assign_provider_to_project))
    .route("/api/v1/projects/{id}/provider", delete(unassign_provider_from_project))
    .with_state(state)
}

/// Developer bearer token (lacks ManageProviderKeys)
fn bearer_developer(user_id: &str) -> String {
  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let token = jwt
    .generate_access_token(user_id, &format!("{user_id}@example.com"), "developer", "tok_002")
    .expect("LOUD FAILURE: Failed to generate developer JWT");
  format!("Bearer {token}")
}

// ─────────────────────────────────────────────────────────────────
// RBAC tests — developer role blocked from all provider endpoints
// ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_provider_keys_requires_manage_permission() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;

  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/v1/providers")
        .header("authorization", bearer_developer("user_a"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::FORBIDDEN,
    "Developer role must not be allowed to list provider keys"
  );
}

#[tokio::test]
async fn get_provider_key_requires_manage_permission() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_a")
    .await
    .unwrap();

  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/providers/{key_id}"))
        .header("authorization", bearer_developer("user_a"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::FORBIDDEN,
    "Developer role must not be allowed to get provider key details"
  );
}

#[tokio::test]
async fn update_provider_key_requires_manage_permission() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_a")
    .await
    .unwrap();

  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/v1/providers/{key_id}"))
        .header("content-type", "application/json")
        .header("authorization", bearer_developer("user_a"))
        .body(Body::from(r#"{"description":"new"}"#))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::FORBIDDEN,
    "Developer role must not be allowed to update provider keys"
  );
}

#[tokio::test]
async fn delete_provider_key_requires_manage_permission() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_a")
    .await
    .unwrap();

  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/v1/providers/{key_id}"))
        .header("authorization", bearer_developer("user_a"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::FORBIDDEN,
    "Developer role must not be allowed to delete provider keys"
  );
}

#[tokio::test]
async fn assign_requires_manage_permission() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_a")
    .await
    .unwrap();

  let body = json!({ "provider_key_id": key_id });
  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/projects/proj_rbac/provider")
        .header("content-type", "application/json")
        .header("authorization", bearer_developer("user_a"))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::FORBIDDEN,
    "Developer role must not be allowed to assign provider keys"
  );
}

#[tokio::test]
async fn unassign_requires_manage_permission() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;

  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/projects/proj_rbac/provider")
        .header("authorization", bearer_developer("user_a"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::FORBIDDEN,
    "Developer role must not be allowed to unassign provider keys"
  );
}

// ─────────────────────────────────────────────────────────────────
// Validation tests — create_provider_key
// ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_rejects_empty_api_key() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;

  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/providers")
        .header("content-type", "application/json")
        .header("authorization", bearer("user_val"))
        .body(Body::from(
          serde_json::to_string(&json!({ "provider": "openai", "api_key": "" })).unwrap(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Empty api_key must be rejected");
}

#[tokio::test]
async fn create_rejects_api_key_too_long() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;

  let body = json!({ "provider": "openai", "api_key": "x".repeat(501) });
  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/providers")
        .header("content-type", "application/json")
        .header("authorization", bearer("user_val"))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "501-char api_key must be rejected");
}

#[tokio::test]
async fn create_rejects_null_byte_in_api_key() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;

  let body = json!({ "provider": "openai", "api_key": "sk-\0abc" });
  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/providers")
        .header("content-type", "application/json")
        .header("authorization", bearer("user_val"))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::BAD_REQUEST,
    "NULL byte in api_key must be rejected"
  );
}

#[tokio::test]
async fn create_rejects_invalid_provider() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;

  let body = json!({ "provider": "mistral", "api_key": "sk-valid-key-0000000000000000" });
  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/providers")
        .header("content-type", "application/json")
        .header("authorization", bearer("user_val"))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Unknown provider must be rejected");
}

#[tokio::test]
async fn create_rejects_description_too_long() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;

  let body = json!({
    "provider": "openai",
    "api_key": "sk-valid-key-0000000000000000",
    "description": "x".repeat(501),
  });
  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/providers")
        .header("content-type", "application/json")
        .header("authorization", bearer("user_val"))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "501-char description must be rejected");
}

#[tokio::test]
async fn create_disabled_without_master_key() {
  let pool = setup_test_db().await;
  let state = make_providers_state_no_crypto(&pool).await;

  let body = json!({ "provider": "openai", "api_key": "sk-valid-key-0000000000000000" });
  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/providers")
        .header("content-type", "application/json")
        .header("authorization", bearer("user_val"))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::SERVICE_UNAVAILABLE,
    "Missing crypto must return 503"
  );
}

// ─────────────────────────────────────────────────────────────────
// Happy paths — update, delete, assign/unassign
// ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn update_provider_key_success() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, Some("Before"), "owner_upd")
    .await
    .unwrap();

  let body = json!({ "description": "After", "is_enabled": false });
  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/v1/providers/{key_id}"))
        .header("content-type", "application/json")
        .header("authorization", bearer("owner_upd"))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::OK, "Valid PUT must return 200");
  let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
  let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
  assert_eq!(json["description"].as_str().unwrap(), "After");
  assert!(!json["is_enabled"].as_bool().unwrap());
}

#[tokio::test]
async fn delete_provider_key_success() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "owner_del")
    .await
    .unwrap();

  let delete_resp = build_full_router(state.clone())
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/v1/providers/{key_id}"))
        .header("authorization", bearer("owner_del"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT, "DELETE must return 204");

  // Subsequent GET must return 404
  let get_resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/providers/{key_id}"))
        .header("authorization", bearer("owner_del"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    get_resp.status(),
    StatusCode::NOT_FOUND,
    "Deleted key must return 404 on subsequent GET"
  );
}

#[tokio::test]
async fn assign_and_unassign_project_provider() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "owner_assign")
    .await
    .unwrap();

  // Assign
  let assign_body = json!({ "provider_key_id": key_id });
  let assign_resp = build_full_router(state.clone())
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/projects/proj_happy/provider")
        .header("content-type", "application/json")
        .header("authorization", bearer("owner_assign"))
        .body(Body::from(serde_json::to_string(&assign_body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(assign_resp.status(), StatusCode::OK, "Assign must return 200");

  // Unassign
  let unassign_resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/projects/proj_happy/provider")
        .header("authorization", bearer("owner_assign"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(unassign_resp.status(), StatusCode::NO_CONTENT, "Unassign must return 204");
}

// ─────────────────────────────────────────────────────────────────
// Ownership enforcement — wrong owner gets 404 / 403
// ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_provider_key_returns_404_for_wrong_owner() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_a")
    .await
    .unwrap();

  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/providers/{key_id}"))
        .header("authorization", bearer("user_b"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::NOT_FOUND,
    "Wrong owner must receive 404 on GET"
  );
}

#[tokio::test]
async fn update_provider_key_returns_404_for_wrong_owner() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_a")
    .await
    .unwrap();

  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/v1/providers/{key_id}"))
        .header("content-type", "application/json")
        .header("authorization", bearer("user_b"))
        .body(Body::from(r#"{"description":"sneaky"}"#))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::NOT_FOUND,
    "Wrong owner must receive 404 on PUT"
  );
}

#[tokio::test]
async fn delete_provider_key_returns_404_for_wrong_owner() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_a")
    .await
    .unwrap();

  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/v1/providers/{key_id}"))
        .header("authorization", bearer("user_b"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::NOT_FOUND,
    "Wrong owner must receive 404 on DELETE"
  );
}

#[tokio::test]
async fn unassign_returns_403_for_wrong_owner() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_a")
    .await
    .unwrap();

  // user_a assigns key to project
  state
    .providers
    .storage
    .assign_to_project(key_id, "proj_ownership")
    .await
    .unwrap();

  // user_b tries to unassign — owns neither the key nor the assignment
  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/projects/proj_ownership/provider")
        .header("authorization", bearer("user_b"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::FORBIDDEN,
    "Non-owner trying to unassign must receive 403"
  );
}

/// User A cannot assign their provider key to user B's project.
///
/// The assign endpoint verifies project ownership via api_tokens; if the caller has
/// no token for the target project, the assignment is rejected with 404.
#[tokio::test]
async fn cross_tenant_project_assignment_rejected() {
  let pool = setup_test_db().await;
  let state = make_providers_state(&pool).await;

  // user_a creates a provider key
  let key_id = state
    .providers
    .storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_a")
    .await
    .unwrap();

  // Seed user_b's project: insert an api_token row that belongs to user_b / project_b
  let now_ms = chrono::Utc::now().timestamp_millis();
  sqlx::query(
    "INSERT INTO api_tokens (user_id, project_id, token_hash, is_active, created_at) \
     VALUES (?, ?, ?, 1, ?)",
  )
  .bind("user_b")
  .bind("project_b")
  .bind("hash_b")
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  // user_a tries to assign their key to user_b's project — must be rejected
  let resp = build_full_router(state)
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/projects/project_b/provider")
        .header("content-type", "application/json")
        .header("authorization", bearer("user_a"))
        .body(Body::from(
          serde_json::to_string(&json!({ "provider_key_id": key_id })).unwrap(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  // The project exists (user_b owns it) but user_a is not the owner → 404
  assert_eq!(
    resp.status(),
    StatusCode::NOT_FOUND,
    "User A must not be able to assign their key to user B's project"
  );
}
