//! Authorization Bypass Fix - Malformed agent_id Validation
//!
//! # Security Context
//!
//! **Vulnerability:** Authorization bypass via malformed agent_id
//! **Severity:** CRITICAL
//! **CVE:** Internal (not publicly disclosed)
//! **Location:** `routes/budget/handshake.rs:143`
//!
//! ## Vulnerability Details
//!
//! The Budget Control API handshake endpoint parses agent_id from IC Tokens.
//! When parsing fails (malformed agent_id), the code used `.unwrap_or(1)`,
//! defaulting to agent_id=1. This allowed attackers to bypass authorization:
//!
//! ```rust
//! // VULNERABLE CODE (before fix):
//! let agent_id: i64 = match agent_id_str.strip_prefix("agent_") {
//!   Some(id_part) => {
//!     id_part.parse::<i64>().unwrap_or(1)  // ← CRITICAL: defaults to 1
//!   }
//!   None => return BadRequest
//! };
//! ```
//!
//! ### Attack Vector
//!
//! 1. Create IC Token with `agent_id = "agent_INVALID"` (non-numeric)
//! 2. Send handshake request with this IC Token
//! 3. Parsing fails → defaults to `agent_id=1`
//! 4. Attacker now uses agent_id=1's budget (authorization bypass)
//!
//! ### Impact
//!
//! - Unauthorized budget access
//! - Budget exhaustion for victim agent_id=1
//! - Billing fraud
//! - No audit trail (requests logged under wrong agent_id)
//!
//! ## Fix Applied
//!
//! Replace `.unwrap_or(1)` with explicit error handling that rejects
//! malformed input:
//!
//! ```rust
//! // FIXED CODE (after fix):
//! let agent_id: i64 = match agent_id_str.strip_prefix("agent_") {
//!   Some(id_part) => {
//!     match id_part.parse::<i64>() {
//!       Ok(id) if id > 0 => id,  // Valid positive ID
//!       Ok(_) => return BadRequest("agent_id must be positive"),
//!       Err(_) => return BadRequest("agent_id must be numeric"),
//!     }
//!   }
//!   None => return BadRequest("agent_id missing prefix")
//! };
//! ```
//!
//! ## Test Strategy
//!
//! These tests verify that malformed agent_id inputs are rejected with
//! 400 Bad Request, NOT defaulted to agent_id=1.
//!
//! ### Test Matrix
//!
//! | Test | agent_id Value | Parse Result | Expected HTTP | Expected agent_id |
//! |------|----------------|--------------|---------------|-------------------|
//! | alphabetic | agent_INVALID | Parse error | 400 | None (rejected) |
//! | special chars | agent_!!!@@@### | Parse error | 400 | None (rejected) |
//! | overflow | agent_99999999999999999999 | Parse error | 400 | None (rejected) |
//! | negative | agent_-1 | Parse success (negative) | 400 | None (rejected) |
//! | zero | agent_0 | Parse success (zero) | 400 | None (rejected) |
//! | valid | agent_42 | Parse success (positive) | 200/403 | 42 (accepted) |

use axum::{body::Body, http::{Request, StatusCode}};
use iron_control_api::ic_token::IcTokenClaims;
use serde_json::json;
use tower::ServiceExt;

mod common;

/// Test helper: Create IC Token with custom agent_id (including malformed)
///
/// Unlike the standard `create_ic_token` helper which uses numeric agent_id,
/// this helper allows creating IC Tokens with malformed agent_id strings
/// (alphabetic, special chars, etc.) for security testing.
fn create_ic_token_with_agent_id(
  agent_id_value: &str,
  ic_token_manager: &iron_control_api::ic_token::IcTokenManager,
) -> String
{
  let claims = IcTokenClaims::new(
    agent_id_value.to_string(),  // Allows malformed values (agent_INVALID, etc.)
    format!("budget_{}", agent_id_value),
    vec!["llm:call".to_string()],
    None,
  );

  ic_token_manager
    .generate_token(&claims)
    .expect("LOUD FAILURE: Should generate IC Token")
}

/// **Test 1:** Malformed agent_id (alphabetic characters) must be rejected
///
/// **Attack Vector:** agent_id="agent_INVALID" (parse fails)
/// **Expected:** 400 Bad Request (NOT default to agent_id=1)
///
/// This is the primary attack vector from the vulnerability analysis.
#[tokio::test]
async fn test_malformed_agent_id_alphabetic()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state(pool.clone()).await;

  // Create agent_id=101 with budget (the target of bypass)
  // Note: Use agent_id > 100 to avoid conflicts with seeded data (per Fix(issue-concurrency-001))
  common::budget::seed_agent_with_budget(&pool, 101, 100_000_000).await;

  // Create IC Token with MALFORMED agent_id (alphabetic)
  let ic_token = create_ic_token_with_agent_id("agent_INVALID", &state.ic_token_manager);

  let app = common::budget::create_budget_router(state).await;

  let request = Request::builder()
    .method("POST")
    .uri("/api/budget/handshake")
    .header("content-type", "application/json")
    .body(Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai"
      })
      .to_string(),
    ))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  // CRITICAL: Must reject, NOT default to agent_id=1
  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Malformed agent_id (alphabetic) MUST return 400 Bad Request, not default to agent_id=1"
  );

  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
  let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();

  assert!(
    body_text.contains("numeric") || body_text.contains("Invalid agent_id"),
    "Error message should indicate numeric requirement: {}",
    body_text
  );
}

/// **Test 2:** Malformed agent_id (special characters) must be rejected
///
/// **Attack Vector:** agent_id="agent_!!!@@@###" (parse fails)
/// **Expected:** 400 Bad Request
#[tokio::test]
async fn test_malformed_agent_id_special_chars()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state(pool.clone()).await;
  common::budget::seed_agent_with_budget(&pool, 102, 100_000_000).await;

  let ic_token = create_ic_token_with_agent_id("agent_!!!@@@###", &state.ic_token_manager);
  let app = common::budget::create_budget_router(state).await;

  let request = Request::builder()
    .method("POST")
    .uri("/api/budget/handshake")
    .header("content-type", "application/json")
    .body(Body::from(
      json!({"ic_token": ic_token, "provider": "openai"}).to_string(),
    ))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Malformed agent_id (special chars) MUST return 400 Bad Request"
  );
}

/// **Test 3:** Integer overflow agent_id must be rejected
///
/// **Attack Vector:** agent_id="agent_99999999999999999999" (overflow i64)
/// **Expected:** 400 Bad Request
#[tokio::test]
async fn test_malformed_agent_id_overflow()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state(pool.clone()).await;
  common::budget::seed_agent_with_budget(&pool, 103, 100_000_000).await;

  // i64::MAX is 9223372036854775807 (19 digits)
  // This value has 20 digits and will overflow
  let ic_token = create_ic_token_with_agent_id("agent_99999999999999999999", &state.ic_token_manager);
  let app = common::budget::create_budget_router(state).await;

  let request = Request::builder()
    .method("POST")
    .uri("/api/budget/handshake")
    .header("content-type", "application/json")
    .body(Body::from(
      json!({"ic_token": ic_token, "provider": "openai"}).to_string(),
    ))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Overflow agent_id MUST return 400 Bad Request"
  );
}

/// **Test 4:** Negative agent_id must be rejected
///
/// **Attack Vector:** agent_id="agent_-1" (parses as -1)
/// **Expected:** 400 Bad Request (database IDs must be positive)
#[tokio::test]
async fn test_malformed_agent_id_negative()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state(pool.clone()).await;
  common::budget::seed_agent_with_budget(&pool, 104, 100_000_000).await;

  let ic_token = create_ic_token_with_agent_id("agent_-1", &state.ic_token_manager);
  let app = common::budget::create_budget_router(state).await;

  let request = Request::builder()
    .method("POST")
    .uri("/api/budget/handshake")
    .header("content-type", "application/json")
    .body(Body::from(
      json!({"ic_token": ic_token, "provider": "openai"}).to_string(),
    ))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Negative agent_id MUST return 400 Bad Request"
  );

  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
  let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();

  assert!(
    body_text.contains("positive") || body_text.contains("Invalid agent_id"),
    "Error message should indicate positive requirement: {}",
    body_text
  );
}

/// **Test 5:** Zero agent_id must be rejected
///
/// **Attack Vector:** agent_id="agent_0" (parses as 0)
/// **Expected:** 400 Bad Request (database IDs start at 1)
#[tokio::test]
async fn test_malformed_agent_id_zero()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state(pool.clone()).await;
  common::budget::seed_agent_with_budget(&pool, 105, 100_000_000).await;

  let ic_token = create_ic_token_with_agent_id("agent_0", &state.ic_token_manager);
  let app = common::budget::create_budget_router(state).await;

  let request = Request::builder()
    .method("POST")
    .uri("/api/budget/handshake")
    .header("content-type", "application/json")
    .body(Body::from(
      json!({"ic_token": ic_token, "provider": "openai"}).to_string(),
    ))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Zero agent_id MUST return 400 Bad Request"
  );
}

/// **Test 6:** Valid agent_id must be accepted (positive control)
///
/// **Valid Input:** agent_id="agent_42" (parses as 42)
/// **Expected:** NOT 400 (either 200 success or 403 forbidden if no budget)
///
/// This test verifies that the fix doesn't break valid agent_id inputs.
#[tokio::test]
async fn test_valid_agent_id()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state(pool.clone()).await;
  common::budget::seed_agent_with_budget(&pool, 106, 100_000_000).await;

  let ic_token = create_ic_token_with_agent_id("agent_106", &state.ic_token_manager);
  let app = common::budget::create_budget_router(state).await;

  let request = Request::builder()
    .method("POST")
    .uri("/api/budget/handshake")
    .header("content-type", "application/json")
    .body(Body::from(
      json!({"ic_token": ic_token, "provider": "openai"}).to_string(),
    ))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  // Must NOT be 400 Bad Request (valid input)
  // Acceptable: 200 OK, 403 Forbidden (budget exhausted), 401 Unauthorized (token invalid)
  assert_ne!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Valid agent_id MUST NOT return 400 Bad Request"
  );
}

// ## Fix Documentation
//
// **Fix(authorization-bypass-handshake):** Reject malformed agent_id instead of defaulting to 1
//
// **Root Cause:** Code used `.unwrap_or(1)` when parsing agent_id from IC Token,
// defaulting to agent_id=1 on parse failure. This allowed attackers to bypass
// authorization by sending malformed agent_id values (alphabetic, special chars,
// overflow, etc.), which would parse fail and default to using agent_id=1's budget.
//
// **Why Not Caught:** No existing tests sent malformed agent_id values to handshake
// endpoint. Test coverage focused on valid positive integers (agent_1, agent_42, etc.)
// and missing agent_id (None case). Parse failure path with `.unwrap_or(1)` was never
// exercised, so authorization bypass went undetected. Security testing didn't include
// input validation fuzzing for authorization-critical fields.
//
// **Pitfall:** Never use fallback values for security-critical parsing. Always
// reject invalid input with explicit error responses. Using `.unwrap_or()` for
// authorization data is a critical anti-pattern:
// 1. Silently accepts malformed input (no audit trail)
// 2. Creates authorization bypass when fallback is privileged
// 3. Billing fraud (requests billed to wrong agent)
// 4. No error visibility (attacker knows exploit works)
//
// **Prevention:**
// - Use explicit match on parse Result
// - Reject with 400 Bad Request on any parse failure
// - Validate parsed value (positive for database IDs)
// - Never use `.unwrap_or()` for security-critical fields
// - Test all malformed input cases (alphabetic, special chars, overflow, negative, zero)
//
// ## Verification
//
// ```bash
// # Run this test suite
// cargo nextest run --package iron_control_api handshake_malformed_agent_id_test
// # Expected: All 6 tests pass
//
// # Run full security test suite
// cargo nextest run --package iron_control_api budget_security
// # Expected: All tests pass
// ```
