//! Authorization / RBAC integration tests
//!
//! Replacement coverage for deleted `authorization_bypass_comprehensive.rs`.
//! Tests the three most critical authorization scenarios:
//!
//! 1. Developer cannot call Manager-only endpoints (vertical escalation)
//! 2. Manager cannot call Admin-only endpoints (vertical escalation)
//! 3. User A cannot access User B's resources (IDOR / horizontal escalation)
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Expected |
//! |-----------|----------|----------|
//! | `test_developer_cannot_create_provider_key` | Developer POSTs to /api/providers | 403 Forbidden |
//! | `test_manager_cannot_manage_users` | Manager POSTs to /api/users | 403 Forbidden |
//! | `test_user_cannot_access_other_users_provider_key` | User A GETs User B's key | 404 (ownership check) |

#[path = "common/mod.rs"]
mod common;

use std::sync::Arc;

use axum::{
  body::Body,
  extract::{ConnectInfo, FromRef},
  http::{Request, StatusCode},
  middleware::{self, Next},
  response::Response,
  routing::{delete, get, post, put},
  Router,
};
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use serde_json::json;
use tower::ServiceExt;

use common::test_db;
use iron_control_api::{
  jwt_auth::JwtSecret,
  rbac::PermissionChecker,
  routes::auth::{login, AuthState},
  routes::providers::{
    create_provider_key, delete_provider_key, get_provider_key, list_provider_keys,
    update_provider_key, ProvidersState,
  },
  routes::users::{create_user, UserManagementState},
};
use iron_secrets::crypto::CryptoService;
use iron_token_manager::provider_key_storage::ProviderKeyStorage;

// ---------------------------------------------------------------------------
// Test app state (combines AuthState + ProvidersState + UserManagementState)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct TestAppState {
  auth: AuthState,
  providers: ProvidersState,
  users: UserManagementState,
}

impl FromRef<TestAppState> for AuthState {
  fn from_ref(state: &TestAppState) -> Self {
    state.auth.clone()
  }
}

impl FromRef<TestAppState> for ProvidersState {
  fn from_ref(state: &TestAppState) -> Self {
    state.providers.clone()
  }
}

impl FromRef<TestAppState> for UserManagementState {
  fn from_ref(state: &TestAppState) -> Self {
    state.users.clone()
  }
}

// ---------------------------------------------------------------------------
// Middleware to inject ConnectInfo (required by login endpoint)
// ---------------------------------------------------------------------------

async fn inject_connect_info(mut request: Request<Body>, next: Next) -> Response {
  let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 54321);
  request.extensions_mut().insert(ConnectInfo(addr));
  next.run(request).await
}

// ---------------------------------------------------------------------------
// Test router factory
// ---------------------------------------------------------------------------

const TEST_JWT_SECRET: &str = "test_jwt_secret_for_rbac_tests_only";
const TEST_MASTER_KEY: [u8; 32] = [42u8; 32];

async fn create_test_router() -> Router {
  let db = test_db::create_test_db().await;
  let pool = db.pool().clone();
  // Keep db alive
  core::mem::forget(db);

  // Provider key storage (shares the same pool)
  let provider_pool = sqlx::sqlite::SqlitePoolOptions::new()
    .connect("sqlite::memory:")
    .await
    .expect("LOUD FAILURE: Failed to create provider pool");
  iron_token_manager::apply_all_migrations(&provider_pool)
    .await
    .expect("LOUD FAILURE: Failed to apply provider migrations");

  let provider_storage = Arc::new(ProviderKeyStorage::new(provider_pool));
  let crypto = CryptoService::new(&TEST_MASTER_KEY)
    .ok()
    .map(Arc::new);

  let auth_state = AuthState {
    jwt_secret: Arc::new(JwtSecret::new(TEST_JWT_SECRET.to_string())),
    db_pool: pool.clone(),
    rate_limiter: iron_control_api::rate_limiter::LoginRateLimiter::new(),
    rate_limiting_enabled: false,
  };

  let providers_state = ProvidersState {
    storage: provider_storage,
    crypto,
  };

  let user_state = UserManagementState::new(pool, Arc::new(PermissionChecker::new()));

  let state = TestAppState {
    auth: auth_state,
    providers: providers_state,
    users: user_state,
  };

  Router::new()
    // Auth
    .route("/api/auth/login", post(login))
    // Providers
    .route("/api/providers", post(create_provider_key))
    .route("/api/providers", get(list_provider_keys))
    .route("/api/providers/{id}", get(get_provider_key))
    .route("/api/providers/{id}", put(update_provider_key))
    .route("/api/providers/{id}", delete(delete_provider_key))
    // Users (admin-only)
    .route("/api/users", post(create_user))
    .with_state(state)
    .layer(middleware::from_fn(inject_connect_info))
}

// ---------------------------------------------------------------------------
// Test 1: Developer cannot create provider key (requires ManageProviderKeys)
// ---------------------------------------------------------------------------

/// Developer role lacks `ManageProviderKeys` permission.
/// POST /api/providers must return 403 Forbidden.
#[tokio::test]
async fn test_developer_cannot_create_provider_key() {
  let router = create_test_router().await;

  let developer_token = {
    let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
    jwt
      .generate_access_token("user_dev", "dev@example.com", "developer", "jti_dev")
      .expect("LOUD FAILURE: Failed to generate developer JWT")
  };

  let body = json!({
    "provider": "openai",
    "api_key": "sk-test-key-1234567890",
  });

  let request = Request::builder()
    .method("POST")
    .uri("/api/providers")
    .header("content-type", "application/json")
    .header("Authorization", format!("Bearer {developer_token}"))
    .body(Body::from(serde_json::to_string(&body).unwrap()))
    .unwrap();

  let response = router.oneshot(request).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::FORBIDDEN,
    "LOUD FAILURE: Developer must NOT be able to create provider keys (403 expected)"
  );
}

// ---------------------------------------------------------------------------
// Test 2: Manager cannot manage users (requires ManageUsers / AssignRoles)
// ---------------------------------------------------------------------------

/// Manager role lacks `ManageUsers` permission.
/// POST /api/users must return 403 Forbidden.
#[tokio::test]
async fn test_manager_cannot_create_users() {
  let router = create_test_router().await;

  let manager_token = {
    let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
    jwt
      .generate_access_token("user_mgr", "mgr@example.com", "manager", "jti_mgr")
      .expect("LOUD FAILURE: Failed to generate manager JWT")
  };

  let body = json!({
    "username": "newuser",
    "password": "securepassword123",
    "email": "new@example.com",
    "role": "developer",
  });

  let request = Request::builder()
    .method("POST")
    .uri("/api/users")
    .header("content-type", "application/json")
    .header("Authorization", format!("Bearer {manager_token}"))
    .body(Body::from(serde_json::to_string(&body).unwrap()))
    .unwrap();

  let response = router.oneshot(request).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::FORBIDDEN,
    "LOUD FAILURE: Manager must NOT be able to create users (403 expected)"
  );
}

// ---------------------------------------------------------------------------
// Test 3: IDOR - User A cannot access User B's provider key
// ---------------------------------------------------------------------------

/// User A creates a provider key. User B tries to GET it by ID.
/// The endpoint must return 404 (ownership check returns "not found").
#[tokio::test]
async fn test_user_cannot_access_other_users_provider_key() {
  let router = create_test_router().await;

  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());

  // Manager A creates a key
  let token_a = jwt
    .generate_access_token("user_alice", "alice@example.com", "manager", "jti_alice")
    .expect("LOUD FAILURE: Failed to generate manager A JWT");

  let create_body = json!({
    "provider": "openai",
    "api_key": "sk-alice-secret-key-99999",
  });

  let create_request = Request::builder()
    .method("POST")
    .uri("/api/providers")
    .header("content-type", "application/json")
    .header("Authorization", format!("Bearer {token_a}"))
    .body(Body::from(serde_json::to_string(&create_body).unwrap()))
    .unwrap();

  let create_response = router.clone().oneshot(create_request).await.unwrap();

  assert_eq!(
    create_response.status(),
    StatusCode::CREATED,
    "LOUD FAILURE: Manager A should be able to create a provider key"
  );

  // Extract the key ID from the response
  let create_body_bytes = http_body_util::BodyExt::collect(create_response.into_body())
    .await
    .expect("LOUD FAILURE: Failed to read create response body")
    .to_bytes();
  let create_json: serde_json::Value = serde_json::from_slice(&create_body_bytes)
    .expect("LOUD FAILURE: Failed to parse create response JSON");
  let key_id = create_json["id"]
    .as_i64()
    .expect("LOUD FAILURE: Response must contain 'id' field");

  // Manager B tries to access that key
  let token_b = jwt
    .generate_access_token("user_bob", "bob@example.com", "manager", "jti_bob")
    .expect("LOUD FAILURE: Failed to generate manager B JWT");

  let get_request = Request::builder()
    .method("GET")
    .uri(format!("/api/providers/{key_id}"))
    .header("Authorization", format!("Bearer {token_b}"))
    .body(Body::empty())
    .unwrap();

  let get_response = router.clone().oneshot(get_request).await.unwrap();

  assert_eq!(
    get_response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: User B must NOT be able to access User A's provider key (IDOR prevention)"
  );

  // Also verify User A CAN access their own key
  let own_request = Request::builder()
    .method("GET")
    .uri(format!("/api/providers/{key_id}"))
    .header("Authorization", format!("Bearer {token_a}"))
    .body(Body::empty())
    .unwrap();

  let own_response = router.oneshot(own_request).await.unwrap();

  assert_eq!(
    own_response.status(),
    StatusCode::OK,
    "LOUD FAILURE: User A must be able to access their own provider key"
  );
}
