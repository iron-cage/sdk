//! Shared test infrastructure for provider API tests.

#![allow(missing_docs)]

use std::sync::Arc;

use iron_control_api::{
  jwt_auth::JwtSecret,
  routes::{auth::AuthState, providers::ProvidersState},
};
use iron_secrets::crypto::CryptoService;
use iron_token_manager::provider_key_storage::ProviderKeyStorage;

#[allow(dead_code)]
pub const TEST_JWT_SECRET: &str = "test_jwt_secret_for_providers_12345";
#[allow(dead_code)]
pub const MASTER_KEY: [u8; 32] = [42u8; 32];

/// Combined state satisfying both `ProvidersState` and `AuthState` extraction.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TestProvidersAppState {
  pub providers: ProvidersState,
  pub auth: AuthState,
}

impl axum::extract::FromRef<TestProvidersAppState> for ProvidersState {
  fn from_ref(s: &TestProvidersAppState) -> Self {
    s.providers.clone()
  }
}

impl axum::extract::FromRef<TestProvidersAppState> for AuthState {
  fn from_ref(s: &TestProvidersAppState) -> Self {
    s.auth.clone()
  }
}

/// Build a test providers state backed by the given pool (crypto enabled).
#[allow(dead_code)]
pub async fn make_providers_state(pool: &sqlx::SqlitePool) -> TestProvidersAppState {
  let storage = Arc::new(ProviderKeyStorage::new(pool.clone()));
  let crypto = Arc::new(CryptoService::new(&MASTER_KEY).unwrap());
  let providers = ProvidersState { storage, crypto: Some(crypto) };
  let auth = AuthState::new(TEST_JWT_SECRET.to_string(), "sqlite::memory:", false)
    .await
    .expect("LOUD FAILURE: Failed to create test AuthState");
  TestProvidersAppState { providers, auth }
}

/// Generate an admin bearer token (has ManageProviderKeys).
#[allow(dead_code)]
pub fn bearer(user_id: &str) -> String {
  let jwt = JwtSecret::new(TEST_JWT_SECRET.to_string());
  let token = jwt
    .generate_access_token(user_id, &format!("{user_id}@example.com"), "admin", "tok_001")
    .expect("LOUD FAILURE: Failed to generate test JWT");
  format!("Bearer {token}")
}
