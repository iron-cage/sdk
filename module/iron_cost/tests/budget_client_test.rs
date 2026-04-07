//! Integration tests for `BudgetClient` IP Token decryption path.
//!
//! Covers the production path through `BudgetClient::handshake()` → `get_provider_key()`,
//! which was previously bypassed in e2e tests that called `IpTokenCrypto::decrypt()` directly.
//!
//! ## What is tested
//!
//! | Test | Scenario | Expected |
//! |------|----------|----------|
//! | `budget_client_handshake_decrypts_encrypted_ip_token` | Server returns encrypted token, key present | Decrypted plaintext key in `get_provider_key()` |
//! | `budget_client_build_fails_loudly_when_ip_token_key_absent` | Key absent, no server needed | `Err` at `build()` with clear message |

#![cfg(feature = "budget-client")]
#![allow(missing_docs)]

use iron_cost::budget_client::BudgetClientBuilder;
use iron_secrets::ip_token::{IpTokenCrypto, IpTokenKey};
use secrecy::ExposeSecret;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const TEST_IP_TOKEN_KEY: [u8; 32] = [0x42u8; 32];

fn test_ip_token_key() -> IpTokenKey {
  IpTokenKey::try_from(TEST_IP_TOKEN_KEY).unwrap()
}

/// Start a minimal HTTP mock server for `POST /api/budget/handshake`.
/// Accepts one connection, discards the request body, replies with a JSON handshake response.
async fn start_mock_handshake_server(encrypted_token: String) -> String {
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let addr = listener.local_addr().unwrap();
  tokio::spawn(async move {
    let (mut stream, _) = listener.accept().await.unwrap();
    let mut buf = [0u8; 4096];
    let _ = stream.read(&mut buf).await.unwrap_or(0);
    let body = format!(
      r#"{{"ip_token":"{encrypted_token}","lease_id":"test-lease-001","budget_granted":10000000,"budget_remaining":10000000}}"#
    );
    let response = format!(
      "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
      body.len(),
      body
    );
    stream.write_all(response.as_bytes()).await.unwrap();
  });
  format!("http://127.0.0.1:{}", addr.port())
}

/// Verifies the full production handshake path end-to-end.
///
/// Previously e2e tests called `IpTokenCrypto::decrypt()` directly, bypassing
/// `BudgetClient::handshake()` entirely (Finding 11). This test exercises the
/// real path: mock server → HTTP → `handshake()` → `get_provider_key()` → plaintext.
#[tokio::test]
async fn budget_client_handshake_decrypts_encrypted_ip_token() {
  let crypto = IpTokenCrypto::from_slice(&TEST_IP_TOKEN_KEY).unwrap();
  let plaintext_key = "sk-ant-api03-integration-test-key";
  let encrypted = crypto.encrypt(plaintext_key).unwrap();

  let server_url = start_mock_handshake_server(encrypted).await;

  let client = BudgetClientBuilder::new()
    .server_url(server_url)
    .ic_token("test-ic-token".to_string())
    .provider("anthropic".to_string())
    .ip_token_key(test_ip_token_key())
    .build()
    .unwrap();

  client.handshake().await.unwrap();

  let provider_key = client.get_provider_key().await.unwrap();
  assert_eq!(
    provider_key.api_key.expose_secret().as_str(),
    plaintext_key,
    "BudgetClient must return the decrypted plaintext key, not the raw ciphertext"
  );
}

/// Regression guard for issue-002: `BudgetClient` must fail loudly when `IP_TOKEN_KEY` is absent.
///
/// ## Root Cause
/// `ip_token_crypto: Option<IpTokenCrypto>` in `BudgetClient` allowed the `None` arm in
/// `handshake()` to store the ciphertext verbatim as the provider API key.
/// The handshake appeared to succeed (HTTP 200) but every subsequent LLM call failed with 401.
///
/// ## Why Not Caught
/// No integration test exercised the full `BudgetClientBuilder` → `handshake()` → `get_provider_key()`
/// path without `ip_token_key`. The `ip_token_e2e_test.rs` called `IpTokenCrypto::decrypt()`
/// directly, bypassing the `handshake()` decryption arm entirely.
///
/// ## Fix Applied
/// `BudgetClient::new()` now returns `Err(BudgetClientError::IpTokenDecrypt(...))` immediately
/// when `ip_token_key` is absent, catching misconfiguration at construction time before the
/// proxy binds its port or accepts any traffic.
///
/// ## Prevention
/// Every `BudgetClientBuilder` must set `ip_token_key`. The error is surfaced at `build()`,
/// not at `handshake()` or at the first LLM call. Tests must assert `build()` fails, not `handshake()`.
///
/// ## Pitfall to Avoid
/// The previous guard was in `handshake()` — the client could be constructed successfully and
/// take a request before the misconfiguration was detected. Moving the check to `new()` ensures
/// zero requests are accepted before the error surfaces.
// test_kind: bug_reproducer(issue-002)
#[test]
fn budget_client_build_fails_loudly_when_ip_token_key_absent() {
  let result = BudgetClientBuilder::new()
    .server_url("http://not-used") // build() fails before any network I/O
    .ic_token("test-ic-token")
    .provider("openai")
    // No ip_token_key — must fail loudly at build, not silently at LLM call
    .build();

  let Err(err) = result else {
    panic!("BudgetClient::new() must fail when IP_TOKEN_KEY is absent")
  };
  assert!(
    err.to_string().contains("IP_TOKEN_KEY not configured"),
    "Error message must identify the missing IP_TOKEN_KEY"
  );
}
