//! Integration tests for proxy startup validation.
//!
//! Covers the startup guard in `run_proxy()` that rejects misconfiguration
//! before the server binds to a port.
//!
//! ## What is tested
//!
//! | Test | Scenario | Expected |
//! |------|----------|----------|
//! | `run_proxy_fails_loudly_when_ip_token_key_absent` | Neither `provider_key` nor `ip_token_key` set | `Err` with clear message before bind |

#![allow(missing_docs)]

use iron_runtime::llm_router::{run_proxy, ProxyConfig};
use std::sync::Arc;
use tokio::sync::oneshot;

#[cfg(feature = "analytics")]
use iron_runtime_analytics::EventStore;

/// Regression guard for the missing `run_proxy` startup validation coverage gap.
///
/// ## Root Cause
/// `run_proxy()` gained a startup validation guard (commit 6fdd453) that returns
/// `Err` when both `provider_key` and `ip_token_key` are absent.  However, no
/// test was added to exercise this path, leaving it invisible to regressions.
///
/// ## Why Not Caught
/// `llm_router_test.rs` only tests utility functions (`detect_provider_from_key`,
/// `strip_provider_prefix`, `detect_provider_from_model`).  The `run_proxy`
/// startup path was not exercised by any existing test file.
///
/// ## Fix Applied
/// This test exercises the `run_proxy()` early-return guard directly, confirming
/// it fires before any network I/O (the shutdown channel is never sent, yet the
/// function still returns before blocking on a TCP bind).
///
/// ## Prevention
/// Every structural guard that rejects misconfiguration at startup must have a
/// corresponding test.  `BudgetClient` and `KeyFetcher` each have one;
/// `run_proxy` now completes the set.
///
/// ## Pitfall to Avoid
/// The guard fires only when `provider_key.is_none() && ip_token_key.is_none()`.
/// Setting either field to `Some(...)` bypasses it.  Tests must set both to
/// `None` to exercise this specific failure mode.
// test_kind: bug_reproducer(issue-proxy-001)
#[tokio::test]
async fn run_proxy_fails_loudly_when_ip_token_key_absent() {
  let config = ProxyConfig {
    port: 0,
    ic_token: "test-ic-token".to_string(),
    server_url: "http://127.0.0.1:1".to_string(),
    cache_ttl_seconds: 300,
    cost_controller: None,
    provider_key: None, // No static key
    ip_token_key: None, // No IP_TOKEN_KEY — must fail loudly before bind
    #[cfg(feature = "analytics")]
    event_store: Arc::new(EventStore::new()),
    #[cfg(feature = "analytics")]
    agent_id: None,
    #[cfg(feature = "analytics")]
    provider_id: None,
  };

  let (_tx, rx) = oneshot::channel();
  let result = run_proxy(config, rx).await;

  assert!(
    result.is_err(),
    "run_proxy must fail when both provider_key and ip_token_key are absent"
  );
  assert!(
    result
      .unwrap_err()
      .to_string()
      .contains("IP_TOKEN_KEY must be configured"),
    "Error must identify the missing IP_TOKEN_KEY"
  );
}
