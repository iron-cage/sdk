//! IC Token security tests for Protocol 005 Budget endpoints
//!
//! Tests verify that budget endpoints properly validate IC Token authentication
//! and reject requests with expired, invalid, revoked, or cross-tenant tokens.
//!
//! # Authority
//! - Protocol 005 specification: IC Token authentication requirement
//! - Security best practices: Token expiration enforcement
//! - PR review finding H-2: /report and /return must validate IC Token
//!
//! # Test Matrix
//!
//! | Test Case | Endpoint | Token State | Expected Response |
//! |-----------|----------|-------------|-------------------|
//! | `test_handshake_expired_ic_token` | /handshake | Expired | 401 Unauthorized |
//! | `test_refresh_expired_ic_token` | /refresh | Expired | 401 Unauthorized |
//! | `test_report_rejects_missing_ic_token` | /report | Missing field | 400 Bad Request |
//! | `test_report_rejects_invalid_ic_token` | /report | Bad signature | 401 Unauthorized |
//! | `test_report_rejects_revoked_ic_token` | /report | Hash NULL | 401 Unauthorized |
//! | `test_report_rejects_cross_tenant_lease` | /report | Valid but wrong agent | 403 Forbidden |
//! | `test_return_rejects_missing_ic_token` | /return | Missing field | 400 Bad Request |
//! | `test_return_rejects_invalid_ic_token` | /return | Bad signature | 401 Unauthorized |
//! | `test_return_rejects_revoked_ic_token` | /return | Hash NULL | 401 Unauthorized |
//! | `test_return_rejects_cross_tenant_lease` | /return | Valid but wrong agent | 403 Forbidden |

mod common;

use axum::{
  body::{Body},
  http::{Request, StatusCode},
};
use common::budget::{
  create_budget_router, create_test_budget_state, seed_agent_with_budget, setup_test_db,
};
use iron_control_api::{
  ic_token::{sha256_hash, IcTokenClaims, IcTokenManager},
  routes::budget::BudgetState,
};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

/// Helper: Create expired IC Token
///
/// Generates IC Token with expiration in the past (1 hour ago)
fn create_expired_ic_token(agent_id: i64, manager: &IcTokenManager) -> String {
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("LOUD FAILURE: Time went backwards")
    .as_secs();

  let expired_time = now - 3600; // 1 hour ago

  let claims = IcTokenClaims::new(
    format!("agent_{agent_id}"),
    format!("budget_{agent_id}"),
    vec!["llm:call".to_string()],
    Some(expired_time), // Expired
  );

  manager
    .generate_token(&claims)
    .expect("LOUD FAILURE: Should generate IC Token")
}

/// E2: Handshake with expired IC Token
///
/// # Corner Case
/// POST /api/budget/handshake with IC token expired 1 hour ago
///
/// # Expected Behavior
/// - Request rejected with 401 Unauthorized
/// - Clear error message about expiration
/// - No lease created
/// - No budget reserved
///
/// # Risk
/// HIGH - Stale credentials could bypass security if not validated
///
/// # Security Impact
/// Expired tokens must be rejected to prevent:
/// - Use of stolen/leaked tokens after revocation
/// - Access with stale authorization
/// - Replay attacks with old tokens
#[tokio::test]
async fn test_handshake_expired_ic_token() {
  let pool = setup_test_db().await;
  let agent_id = 300i64;
  seed_agent_with_budget(&pool, agent_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;
  let expired_ic_token = create_expired_ic_token(agent_id, &state.ic_token_manager);
  let router = create_budget_router(state).await;

  // Attempt handshake with expired IC token
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": expired_ic_token,
            "provider": "openai"
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  // Assert: 401 Unauthorized
  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Expired IC Token should return 401 Unauthorized"
  );

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();
  let response_json: serde_json::Value =
    serde_json::from_str(&body_str).expect("LOUD FAILURE: Response should be valid JSON");

  // Assert: Error message mentions token issue
  assert!(
    response_json.get("error").is_some(),
    "LOUD FAILURE: Response should contain error message. Response: {response_json}"
  );

  // Verify: No lease created
  let lease_count: i64 =
    sqlx::query_scalar("SELECT COUNT(*) FROM budget_leases WHERE agent_id = ?")
      .bind(agent_id)
      .fetch_one(&pool)
      .await
      .expect("LOUD FAILURE: Should query lease count");

  assert_eq!(
    lease_count, 0,
    "LOUD FAILURE: No lease should be created with expired IC Token"
  );

  // Verify: Budget unchanged (no reservation)
  let budget_remaining: i64 =
    sqlx::query_scalar("SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?")
      .bind(agent_id)
      .fetch_one(&pool)
      .await
      .expect("LOUD FAILURE: Should query budget");

  assert_eq!(
    budget_remaining, 100_000_000,
    "LOUD FAILURE: Budget should be unchanged. Expected: $100, Actual: {budget_remaining}"
  );
}

/// E2b: Refresh with expired IC Token
///
/// # Corner Case
/// POST /api/budget/refresh with expired IC token
///
/// # Expected Behavior
/// - Request rejected with 401 Unauthorized
/// - No new lease created
/// - No budget reserved
///
/// # Risk
/// MEDIUM - Stale token used for budget refresh
#[tokio::test]
async fn test_refresh_expired_ic_token() {
  let pool = setup_test_db().await;
  let agent_id = 302i64;
  seed_agent_with_budget(&pool, agent_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;

  // Create initial lease with valid token
  let valid_ic_token =
    common::budget::create_ic_token(&pool, agent_id, &state.ic_token_manager).await;
  let router = create_budget_router(state.clone()).await;

  let handshake_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": valid_ic_token,
            "provider": "openai"
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  let handshake_body = axum::body::to_bytes(handshake_response.into_body(), usize::MAX)
    .await
    .unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice(&handshake_body).unwrap();
  let current_lease_id = handshake_json["lease_id"]
    .as_str()
    .expect("LOUD FAILURE: Should have lease_id");

  // Attempt refresh with EXPIRED IC token
  let expired_ic_token = create_expired_ic_token(agent_id, &state.ic_token_manager);
  let access_token =
    common::create_test_access_token("test_user", "test@example.com", "admin", "test_jwt_secret");

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/refresh")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::from(
          json!({
            "ic_token": expired_ic_token,
            "current_lease_id": current_lease_id,
            "requested_budget": 10_000_000
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  // Assert: 401 Unauthorized
  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Refresh with expired IC Token should return 401 Unauthorized"
  );

  // Verify: No new lease created (only initial handshake lease)
  let lease_count: i64 =
    sqlx::query_scalar("SELECT COUNT(*) FROM budget_leases WHERE agent_id = ?")
      .bind(agent_id)
      .fetch_one(&pool)
      .await
      .expect("LOUD FAILURE: Should query lease count");

  assert_eq!(
    lease_count, 1,
    "LOUD FAILURE: Should have only initial lease. No new lease created with expired token."
  );

  // Verify: Budget unchanged (only handshake deduction)
  let total_spent: i64 =
    sqlx::query_scalar("SELECT total_spent FROM agent_budgets WHERE agent_id = ?")
      .bind(agent_id)
      .fetch_one(&pool)
      .await
      .expect("LOUD FAILURE: Should query budget");

  assert_eq!(
    total_spent, 10_000_000,
    "LOUD FAILURE: total_spent should only include initial handshake ($10M)"
  );
}

/// JTI-1: Two tokens with same claims produce different JWT hashes
///
/// # Edge Case (Pre-jti bug)
///
/// Before jti was added, two `IcTokenClaims::new()` calls in the same second
/// with identical parameters would produce identical JWTs (same HMAC-SHA256 payload
/// → same signature → same SHA-256 hash). After regenerate, the new token's hash
/// could match the old token's hash, so the old token would still pass hash-check.
///
/// # Expected Behavior (Post-jti fix)
///
/// Each `IcTokenClaims::new()` generates a unique UUID v4 as `jti`, so even
/// with identical parameters in the same second, the JWTs and their hashes differ.
///
/// # Risk
/// HIGH - Without jti, regenerate would not invalidate old token in same-second scenario
#[tokio::test]
async fn test_jti_guarantees_unique_token_hashes() {
  let manager = IcTokenManager::new("test_secret_for_jti".to_string());

  // Generate two tokens with IDENTICAL parameters in rapid succession (same second)
  let claims_a = IcTokenClaims::new(
    "agent_500".to_string(),
    "budget_500".to_string(),
    vec!["llm:call".to_string()],
    None, // No expiration — long-lived
  );

  let claims_b = IcTokenClaims::new(
    "agent_500".to_string(),
    "budget_500".to_string(),
    vec!["llm:call".to_string()],
    None, // No expiration — long-lived
  );

  // Verify: jti (token_id) values are different
  assert_ne!(
    claims_a.token_id, claims_b.token_id,
    "LOUD FAILURE: Two IcTokenClaims::new() calls must produce different jti values"
  );

  // Generate JWTs
  let token_a = manager
    .generate_token(&claims_a)
    .expect("LOUD FAILURE: Should generate token A");
  let token_b = manager
    .generate_token(&claims_b)
    .expect("LOUD FAILURE: Should generate token B");

  // Verify: JWT strings are different
  assert_ne!(
    token_a, token_b,
    "LOUD FAILURE: JWTs with different jti must produce different token strings"
  );

  // Verify: SHA-256 hashes are different
  let hash_a = sha256_hash(&token_a);
  let hash_b = sha256_hash(&token_b);

  assert_ne!(
    hash_a, hash_b,
    "LOUD FAILURE: SHA-256 hashes of different JWTs must differ. \
     hash_a={hash_a}, hash_b={hash_b}. This would cause regenerate to fail at invalidating old token."
  );
}

/// JTI-2: After regenerate, old token rejected by hash-check (end-to-end)
///
/// # Scenario
///
/// 1. Agent receives IC Token A → hash stored in DB → handshake succeeds
/// 2. Admin calls regenerate → IC Token B generated → new hash stored in DB
/// 3. Agent tries handshake with OLD Token A → hash mismatch → 401 Unauthorized
/// 4. Agent uses NEW Token B → hash matches → 200 OK
///
/// # Risk
/// CRITICAL - This is the core scenario that Task 001 fixes
#[tokio::test]
async fn test_regenerated_token_invalidates_old_token_via_hash_check() {
  let pool = setup_test_db().await;
  let agent_id = 500i64;
  seed_agent_with_budget(&pool, agent_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;

  // Step 1: Generate Token A and store its hash (simulating initial generate)
  let token_a = common::budget::create_ic_token(&pool, agent_id, &state.ic_token_manager).await;

  // Step 1b: Verify Token A works
  let router = create_budget_router(state.clone()).await;
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": token_a.clone(),
            "provider": "openai",
            "provider_key_id": agent_id * 1000
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Token A should work before regeneration"
  );

  // Step 2: Regenerate — generate Token B and overwrite hash in DB
  // (simulating what routes/ic_token.rs::regenerate_ic_token does)
  let claims_b = IcTokenClaims::new(
    format!("agent_{agent_id}"),
    format!("budget_{agent_id}"),
    vec!["llm:call".to_string()],
    None,
  );
  let token_b = state
    .ic_token_manager
    .generate_token(&claims_b)
    .expect("LOUD FAILURE: Should generate token B");

  // Overwrite hash in DB (this is what regenerate does)
  let new_hash = sha256_hash(&token_b);
  sqlx::query("UPDATE agents SET ic_token_hash = ? WHERE id = ?")
    .bind(&new_hash)
    .bind(agent_id)
    .execute(&pool)
    .await
    .expect("LOUD FAILURE: Should update ic_token_hash");

  // Step 3: Old Token A should be REJECTED (hash mismatch)
  let router = create_budget_router(state.clone()).await;
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": token_a,
            "provider": "openai",
            "provider_key_id": agent_id * 1000
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Old Token A must be rejected after regeneration (hash mismatch)"
  );

  // Step 4: New Token B should be ACCEPTED
  let router = create_budget_router(state.clone()).await;
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": token_b,
            "provider": "openai",
            "provider_key_id": agent_id * 1000
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: New Token B must be accepted after regeneration"
  );
}

/// JTI-3: After revoke, re-issued token has different hash (not re-usable)
///
/// # Scenario
///
/// 1. Agent has Token A → hash stored → works
/// 2. Admin revokes → hash set to NULL → Token A rejected
/// 3. Admin re-issues Token B (same claims) → new hash stored
/// 4. Token A still rejected (hash mismatch, thanks to jti)
/// 5. Token B accepted
///
/// # Risk
/// HIGH - Without jti, Token A and Token B could have identical hashes
/// if generated in the same second, causing Token A to "come back to life"
#[tokio::test]
async fn test_revoke_then_reissue_old_token_stays_dead() {
  let pool = setup_test_db().await;
  let agent_id = 501i64;
  seed_agent_with_budget(&pool, agent_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;

  // Step 1: Generate Token A
  let token_a = common::budget::create_ic_token(&pool, agent_id, &state.ic_token_manager).await;
  let hash_a = sha256_hash(&token_a);

  // Step 2: Revoke — set hash to NULL
  sqlx::query("UPDATE agents SET ic_token_hash = NULL WHERE id = ?")
    .bind(agent_id)
    .execute(&pool)
    .await
    .expect("LOUD FAILURE: Should revoke token");

  // Verify Token A is rejected after revoke
  let router = create_budget_router(state.clone()).await;
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": token_a.clone(),
            "provider": "openai",
            "provider_key_id": agent_id * 1000
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Token A must be rejected after revocation"
  );

  // Step 3: Re-issue Token B with same claims
  let claims_b = IcTokenClaims::new(
    format!("agent_{agent_id}"),
    format!("budget_{agent_id}"),
    vec!["llm:call".to_string()],
    None,
  );
  let token_b = state
    .ic_token_manager
    .generate_token(&claims_b)
    .expect("LOUD FAILURE: Should generate token B");
  let hash_b = sha256_hash(&token_b);

  // Critical assertion: hashes must differ (jti guarantees this)
  assert_ne!(
    hash_a, hash_b,
    "LOUD FAILURE: Re-issued token must have different hash than revoked token. \
     Without jti, tokens generated in the same second would have identical hashes, \
     allowing revoked Token A to pass hash-check after Token B is stored."
  );

  // Store new hash
  sqlx::query("UPDATE agents SET ic_token_hash = ? WHERE id = ?")
    .bind(&hash_b)
    .bind(agent_id)
    .execute(&pool)
    .await
    .expect("LOUD FAILURE: Should store new hash");

  // Step 4: Token A must STILL be rejected (hash mismatch, not NULL)
  let router = create_budget_router(state.clone()).await;
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": token_a,
            "provider": "openai",
            "provider_key_id": agent_id * 1000
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Old Token A must remain dead after re-issue of Token B"
  );

  // Step 5: Token B must be accepted
  let router = create_budget_router(state.clone()).await;
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": token_b,
            "provider": "openai",
            "provider_key_id": agent_id * 1000
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: New Token B must be accepted"
  );
}

/// Helper: Create a lease via handshake for a test agent.
///
/// Returns the `lease_id` string.
async fn create_lease_via_handshake(
  agent_id: i64,
  ic_token: &str,
  state: &BudgetState,
) -> String {
  let router = create_budget_router(state.clone()).await;
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": ic_token,
            "provider": "openai",
            "provider_key_id": agent_id * 1000
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Handshake must succeed to create lease for test setup"
  );

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
  json["lease_id"]
    .as_str()
    .expect("LOUD FAILURE: Handshake must return lease_id")
    .to_string()
}

/// R-1: POST /api/budget/report rejects request with missing `ic_token` field
///
/// ## Root Cause
/// Before H-2 fix, /report used only `lease_id` for auth. A revoked agent could
/// continue reporting usage indefinitely using an old `lease_id`.
///
/// ## Why Not Caught Initially
/// The endpoint was designed before IC Token runtime validation existed.
/// Lease-based auth was considered sufficient.
///
/// ## Fix Applied
/// Added `ic_token` field to `UsageReportRequest` with validation.
///
/// ## Prevention
/// All Protocol 005 runtime endpoints must require IC Token as auth credential,
/// not just resource identifiers (`lease_id` is a resource handle, not a credential).
///
/// ## Pitfall to Avoid
/// Never treat resource IDs (`lease_id`, `request_id`) as authentication credentials.
/// Always require a cryptographic token for auth, even when the resource ID is secret.
#[tokio::test]
async fn test_report_rejects_missing_ic_token() {
  let pool = setup_test_db().await;
  let agent_id = 600i64;
  seed_agent_with_budget(&pool, agent_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;
  let router = create_budget_router(state).await;

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/report")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "lease_id": "some-lease-id",
            "request_id": "req-001",
            "tokens": 100,
            "cost_microdollars": 500,
            "model": "gpt-4",
            "provider": "openai"
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: /report must return 400 when ic_token field is missing"
  );
}

/// R-2: POST /api/budget/report rejects request with invalid IC Token signature
#[tokio::test]
async fn test_report_rejects_invalid_ic_token() {
  let pool = setup_test_db().await;
  let agent_id = 601i64;
  seed_agent_with_budget(&pool, agent_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;
  let router = create_budget_router(state).await;

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/report")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": "not.a.valid.jwt",
            "lease_id": "some-lease-id",
            "request_id": "req-001",
            "tokens": 100,
            "cost_microdollars": 500,
            "model": "gpt-4",
            "provider": "openai"
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: /report must return 401 for IC Token with invalid signature"
  );
}

/// R-3: POST /api/budget/report rejects request with revoked IC Token (NULL hash)
#[tokio::test]
async fn test_report_rejects_revoked_ic_token() {
  let pool = setup_test_db().await;
  let agent_id = 602i64;
  seed_agent_with_budget(&pool, agent_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;

  // Store token hash, then revoke it
  let ic_token = common::budget::create_ic_token(&pool, agent_id, &state.ic_token_manager).await;
  sqlx::query("UPDATE agents SET ic_token_hash = NULL WHERE id = ?")
    .bind(agent_id)
    .execute(&pool)
    .await
    .expect("LOUD FAILURE: Should revoke IC token");

  let router = create_budget_router(state).await;
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/report")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": ic_token,
            "lease_id": "some-lease-id",
            "request_id": "req-001",
            "tokens": 100,
            "cost_microdollars": 500,
            "model": "gpt-4",
            "provider": "openai"
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: /report must return 401 for revoked IC Token (NULL hash)"
  );
}

/// R-4: POST /api/budget/report rejects cross-tenant lease access
///
/// The intruder agent cannot report usage for a lease that belongs to the lease owner,
/// even if the intruder has a valid IC Token.
#[tokio::test]
async fn test_report_rejects_cross_tenant_lease() {
  let pool = setup_test_db().await;
  let lease_owner_id = 603i64;
  let intruder_id = 604i64;
  seed_agent_with_budget(&pool, lease_owner_id, 100_000_000).await;
  seed_agent_with_budget(&pool, intruder_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;

  // Create valid IC tokens for both agents
  let owner_token =
    common::budget::create_ic_token(&pool, lease_owner_id, &state.ic_token_manager).await;
  let intruder_token =
    common::budget::create_ic_token(&pool, intruder_id, &state.ic_token_manager).await;

  // Lease owner does handshake to get a lease
  let lease_id = create_lease_via_handshake(lease_owner_id, &owner_token, &state).await;

  // Intruder tries to report usage on the owner's lease
  let router = create_budget_router(state).await;
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/report")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": intruder_token,
            "lease_id": lease_id,
            "request_id": "req-001",
            "tokens": 100,
            "cost_microdollars": 500,
            "model": "gpt-4",
            "provider": "openai"
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::FORBIDDEN,
    "LOUD FAILURE: /report must return 403 when IC Token agent does not own the lease"
  );
}

/// RT-1: POST /api/budget/return rejects request with missing `ic_token` field
#[tokio::test]
async fn test_return_rejects_missing_ic_token() {
  let pool = setup_test_db().await;
  let agent_id = 610i64;
  seed_agent_with_budget(&pool, agent_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;
  let router = create_budget_router(state).await;

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/return")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "lease_id": "some-lease-id",
            "spent_microdollars": 500
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: /return must return 400 when ic_token field is missing"
  );
}

/// RT-2: POST /api/budget/return rejects request with invalid IC Token signature
#[tokio::test]
async fn test_return_rejects_invalid_ic_token() {
  let pool = setup_test_db().await;
  let agent_id = 611i64;
  seed_agent_with_budget(&pool, agent_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;
  let router = create_budget_router(state).await;

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/return")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": "not.a.valid.jwt",
            "lease_id": "some-lease-id",
            "spent_microdollars": 500
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: /return must return 401 for IC Token with invalid signature"
  );
}

/// RT-3: POST /api/budget/return rejects request with revoked IC Token (NULL hash)
#[tokio::test]
async fn test_return_rejects_revoked_ic_token() {
  let pool = setup_test_db().await;
  let agent_id = 612i64;
  seed_agent_with_budget(&pool, agent_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;

  let ic_token = common::budget::create_ic_token(&pool, agent_id, &state.ic_token_manager).await;
  sqlx::query("UPDATE agents SET ic_token_hash = NULL WHERE id = ?")
    .bind(agent_id)
    .execute(&pool)
    .await
    .expect("LOUD FAILURE: Should revoke IC token");

  let router = create_budget_router(state).await;
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/return")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": ic_token,
            "lease_id": "some-lease-id",
            "spent_microdollars": 500
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: /return must return 401 for revoked IC Token (NULL hash)"
  );
}

/// RT-4: POST /api/budget/return rejects cross-tenant lease access
///
/// The intruder agent cannot return a lease that belongs to the lease owner.
#[tokio::test]
async fn test_return_rejects_cross_tenant_lease() {
  let pool = setup_test_db().await;
  let lease_owner_id = 613i64;
  let intruder_id = 614i64;
  seed_agent_with_budget(&pool, lease_owner_id, 100_000_000).await;
  seed_agent_with_budget(&pool, intruder_id, 100_000_000).await;

  let state = create_test_budget_state(pool.clone()).await;

  let owner_token =
    common::budget::create_ic_token(&pool, lease_owner_id, &state.ic_token_manager).await;
  let intruder_token =
    common::budget::create_ic_token(&pool, intruder_id, &state.ic_token_manager).await;

  // Lease owner does handshake to get a lease
  let lease_id = create_lease_via_handshake(lease_owner_id, &owner_token, &state).await;

  // Intruder tries to return the owner's lease
  let router = create_budget_router(state).await;
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/return")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": intruder_token,
            "lease_id": lease_id,
            "spent_microdollars": 500
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::FORBIDDEN,
    "LOUD FAILURE: /return must return 403 when IC Token agent does not own the lease"
  );
}
