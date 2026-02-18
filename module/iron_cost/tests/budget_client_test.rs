//! Integration tests for BudgetClient IP Token decryption path.
//!
//! Covers the production path through `BudgetClient::handshake()` → `get_provider_key()`,
//! which was previously bypassed in e2e tests that called `IpTokenCrypto::decrypt()` directly.
//!
//! ## What is tested
//!
//! | Test | Scenario | Expected |
//! |------|----------|----------|
//! | `budget_client_handshake_decrypts_encrypted_ip_token` | Server returns encrypted token, key present | Decrypted plaintext key in `get_provider_key()` |
//! | `budget_client_handshake_fails_loudly_when_ip_token_key_absent` | Server returns encrypted token, key absent | `Err` with clear message |

#![cfg(feature = "budget-client")]
#![allow(missing_docs)]

use iron_cost::budget_client::BudgetClientBuilder;
use iron_secrets::ip_token::IpTokenCrypto;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const TEST_IP_TOKEN_KEY: [u8; 32] = [0x42u8; 32];

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
  let crypto = IpTokenCrypto::new(&TEST_IP_TOKEN_KEY).unwrap();
  let plaintext_key = "sk-ant-api03-integration-test-key";
  let encrypted = crypto.encrypt(plaintext_key).unwrap();

  let server_url = start_mock_handshake_server(encrypted).await;

  let client = BudgetClientBuilder::new()
    .server_url(server_url)
    .ic_token("test-ic-token".to_string())
    .provider("anthropic".to_string())
    .ip_token_key(TEST_IP_TOKEN_KEY)
    .build()
    .unwrap();

  client.handshake().await.unwrap();

  let provider_key = client.get_provider_key().await.unwrap();
  assert_eq!(
    provider_key.api_key.as_str(),
    plaintext_key,
    "BudgetClient must return the decrypted plaintext key, not the raw ciphertext"
  );
}

/// Regression guard for issue-002: BudgetClient must fail loudly when `IP_TOKEN_KEY` is absent.
///
/// Before the fix, `None => data.ip_token` silently stored the ciphertext as the API key —
/// handshake returned Ok but every subsequent LLM call failed with 401.
/// This test ensures the fixed `None` arm returns `Err` at handshake time.
#[tokio::test]
async fn budget_client_handshake_fails_loudly_when_ip_token_key_absent() {
  let crypto = IpTokenCrypto::new(&TEST_IP_TOKEN_KEY).unwrap();
  let encrypted = crypto.encrypt("sk-secret-key").unwrap();

  let server_url = start_mock_handshake_server(encrypted).await;

  let client = BudgetClientBuilder::new()
    .server_url(server_url)
    .ic_token("test-ic-token".to_string())
    .provider("openai".to_string())
    // No ip_token_key — must fail loudly at handshake, not silently at LLM call
    .build()
    .unwrap();

  let result = client.handshake().await;
  assert!(result.is_err(), "Handshake must fail when IP_TOKEN_KEY is absent");
  assert!(
    result.unwrap_err().to_string().contains("IP_TOKEN_KEY not configured"),
    "Error message must identify the missing IP_TOKEN_KEY"
  );
}
