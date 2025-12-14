//! Login rate limiting integration tests.
//!
//! Tests Protocol 007 § 3.2 rate limiting requirements via HTTP endpoints.
//!
//! ## Current Implementation Status (GAP-006)
//!
//! ✅ Rate limiter implemented (`LoginRateLimiter`)
//! ✅ Integrated into login endpoint
//! ⚠️ PILOT LIMITATION: Uses hardcoded IP `127.0.0.1` for all requests
//! ⚠️ POST-PILOT: Need real IP extraction via ConnectInfo
//! ⚠️ Missing: security_events table logging (uses tracing only)
//! ⚠️ Missing: 3-field source comments per bug-fixing workflow
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Expected (Current) | Expected (Post-Pilot) | Status |
//! |-----------|----------|-------------------|----------------------|--------|
//! | `test_rate_limit_enforced_after_5_attempts` | 5 failed logins + 6th | 6th: 429 | 6th: 429 | ⏳ |
//! | `test_rate_limit_isolated_per_ip` | Different IPs | FAILS (same IP) | IP2 gets 401 | ⚠️ SKIP |
//! | `test_rate_limit_resets_after_window` | Wait 5 min | 401 after wait | 401 after wait | ⏳ |
//! | `test_rate_limit_includes_successful_logins` | 5 success + 1 more | 6th: 429 | 6th: 429 | ⏳ |
//! | `test_rate_limit_response_format` | Check 429 response | Retry-After header | Retry-After header | ⏳ |
//!
//! ## Failure Modes
//!
//! - **All requests rate-limited**: Hardcoded IP means global limit (pilot limitation)
//! - **Tests interfere**: Shared IP counter across tests (use unique DB per test)
//! - **IP isolation test fails**: Expected - needs ConnectInfo implementation

use axum::{
  body::Body,
  extract::ConnectInfo,
  http::{Request, StatusCode},
  Router,
};
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tower::ServiceExt;

mod common;

/// Helper: Create router with auth routes
async fn create_auth_router() -> Router {
  let app_state = common::test_state::TestAppState::new().await;

  // Create test socket address (127.0.0.1:8080)
  let test_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

  Router::new()
    .route("/api/v1/auth/login", axum::routing::post(iron_control_api::routes::auth::login))
    .layer(axum::Extension(ConnectInfo(test_addr)))
    .with_state(app_state.auth)
}

/// Verifies rate limit enforced after 5 login attempts (issue-GAP-006).
///
/// ## Root Cause (issue-GAP-006)
///
/// Protocol 007 § 3.2 requires rate limiting on authentication endpoints
/// to prevent brute force attacks. Pilot implementation exists but uses
/// hardcoded IP `127.0.0.1`, which applies rate limit globally instead of
/// per-client.
///
/// Security gap allowed:
/// - **Unlimited password guessing** (without per-IP limits)
/// - **Credential stuffing** (rapid testing of leaked credentials)
/// - **Account takeover** (brute force grants full access)
/// - **Resource exhaustion** (high-volume attacks impact availability)
///
/// ## Why Not Caught Initially
///
/// Pilot implementation prioritized basic rate limiting over real IP extraction.
/// Comment in code states: "POST-PILOT: Extract real client IP from X-Forwarded-For
/// header or ConnectInfo". Rate limiter works correctly, but IP extraction was
/// deferred to post-pilot.
///
/// ## Fix Applied
///
/// Post-pilot enhancement adds real IP extraction:
///
/// **Implementation:**
/// 1. Add `ConnectInfo<SocketAddr>` to login handler signature
/// 2. Extract IP from connection: `addr.ip()`
/// 3. Use real IP instead of hardcoded `127.0.0.1`
/// 4. Add security_events table logging for rate limit violations
/// 5. Add 3-field source comments per bug-fixing workflow
///
/// **Security improvement:**
/// - Before: Global 5 attempts (all clients)
/// - After: 5 attempts per IP address
///
/// ## Prevention
///
/// All authentication endpoints require per-IP rate limiting:
/// - Extract real client IP via ConnectInfo (not X-Forwarded-For - spoofable)
/// - Log rate limit violations to security_events table
/// - Return 429 with Retry-After header
///
/// ## Pitfall to Avoid
///
/// **CRITICAL: Never use hardcoded IPs or spoofable headers for rate limiting.**
///
/// ### Mistake 1: Using hardcoded IP (pilot limitation)
/// ```rust
/// // ❌ PILOT LIMITATION - Global rate limit
/// let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
///
/// // ✅ POST-PILOT - Real per-client rate limiting
/// let client_ip = addr.ip();  // From ConnectInfo<SocketAddr>
/// ```
///
/// ### Mistake 2: Trusting X-Forwarded-For header
/// ```rust
/// // ❌ VULNERABLE - Attacker can spoof header
/// let ip = headers.get("X-Forwarded-For")?;
///
/// // ✅ SECURE - Uses actual TCP connection IP
/// ConnectInfo(addr): ConnectInfo<SocketAddr>
/// let ip = addr.ip();
/// ```
// test_kind: bug_reproducer(issue-GAP-006)
#[tokio::test]
async fn test_rate_limit_enforced_after_5_attempts() {
  let router = create_auth_router().await;

  // Create 5 failed login attempts
  for attempt in 1..=5 {
    let request = Request::builder()
      .method("POST")
      .uri("/api/v1/auth/login")
      .header("content-type", "application/json")
      .body(Body::from(
        json!({
          "email": "nonexistent@example.com",
          "password": "wrong_password"
        })
        .to_string(),
      ))
      .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();

    // First 5 should return 401 (invalid credentials)
    if status != StatusCode::UNAUTHORIZED {
      let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
      let body_text = String::from_utf8_lossy(&body_bytes);
      panic!(
        "LOUD FAILURE: Attempt {} should return 401, got {}\nResponse body: {}",
        attempt,
        status,
        body_text
      );
    }
  }

  // 6th attempt should be rate-limited (429)
  let request = Request::builder()
    .method("POST")
    .uri("/api/v1/auth/login")
    .header("content-type", "application/json")
    .body(Body::from(
      json!({
        "email": "nonexistent@example.com",
        "password": "wrong_password"
      })
      .to_string(),
    ))
    .unwrap();

  let response = router.clone().oneshot(request).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::TOO_MANY_REQUESTS,
    "LOUD FAILURE: 6th attempt should return 429 (rate limited)"
  );

  // Verify response contains retry_after
  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

  assert!(
    body["error"]["details"]["retry_after"].is_number(),
    "LOUD FAILURE: Response should include retry_after field in details"
  );

  let retry_after = body["error"]["details"]["retry_after"].as_u64().unwrap();
  assert!(
    retry_after > 0 && retry_after <= 300,
    "LOUD FAILURE: retry_after should be 1-300 seconds (5 min window), got {}",
    retry_after
  );
}

/// Verifies 429 response format matches Protocol 007 (issue-GAP-006).
///
/// Rate limit response must include:
/// - Status: 429 TOO_MANY_REQUESTS
/// - Error code: RATE_LIMIT_EXCEEDED
/// - Details: retry_after seconds
// test_kind: bug_reproducer(issue-GAP-006)
#[tokio::test]
async fn test_rate_limit_response_format() {
  let router = create_auth_router().await;

  // Exhaust rate limit
  for _ in 1..=5 {
    let request = Request::builder()
      .method("POST")
      .uri("/api/v1/auth/login")
      .header("content-type", "application/json")
      .body(Body::from(
        json!({
          "email": "test@example.com",
          "password": "wrong"
        })
        .to_string(),
      ))
      .unwrap();

    let _ = router.clone().oneshot(request).await.unwrap();
  }

  // Trigger rate limit
  let request = Request::builder()
    .method("POST")
    .uri("/api/v1/auth/login")
    .header("content-type", "application/json")
    .body(Body::from(
      json!({
        "email": "test@example.com",
        "password": "wrong"
      })
      .to_string(),
    ))
    .unwrap();

  let response = router.clone().oneshot(request).await.unwrap();

  // Verify status
  assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

  // Verify response format
  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

  // Check error code
  assert_eq!(
    body["error"]["code"].as_str().unwrap(),
    "RATE_LIMIT_EXCEEDED",
    "LOUD FAILURE: Error code should be RATE_LIMIT_EXCEEDED"
  );

  // Check message mentions retry time
  let message = body["error"]["message"].as_str().unwrap();
  assert!(
    message.contains("try again in") || message.contains("Too many"),
    "LOUD FAILURE: Message should describe rate limit: {}",
    message
  );

  // Check retry_after in details
  assert!(
    body["error"]["details"]["retry_after"].is_number(),
    "LOUD FAILURE: Details should include retry_after field"
  );
}

// NOTE: test_rate_limit_isolated_per_ip is SKIPPED in pilot
// Pilot uses hardcoded 127.0.0.1, so IP isolation cannot be tested.
// POST-PILOT: Add this test once ConnectInfo IP extraction is implemented.
//
// /// Verifies different IPs have independent rate limits (issue-GAP-006).
// ///
// /// POST-PILOT only - requires real IP extraction via ConnectInfo.
// // test_kind: bug_reproducer(issue-GAP-006)
// #[tokio::test]
// #[ignore = "Requires ConnectInfo IP extraction (post-pilot)"]
// async fn test_rate_limit_isolated_per_ip() {
//   // Test would verify:
//   // 1. IP1 exhausts 5 attempts → blocked
//   // 2. IP2 makes request → allowed (independent counter)
// }

