//! Integration tests for KeyFetcher IP Token decryption path.
//!
//! Covers the production path that had zero test coverage before the fix:
//! `KeyFetcher::new(ip_token_key)` → `get_key()` → `fetch_from_server()` → `decrypt()` → plaintext.
//!
//! ## What is tested
//!
//! | Test | Scenario | Expected |
//! |------|----------|----------|
//! | `key_fetcher_decrypts_encrypted_ip_token_from_server` | Server returns encrypted token, key present | Decrypted plaintext key returned |
//! | `key_fetcher_fails_loudly_when_ip_token_key_absent` | Server returns encrypted token, key absent | `Err` with clear message |

#![allow(missing_docs)]

use iron_runtime::llm_router::KeyFetcher;
use iron_secrets::ip_token::IpTokenCrypto;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const TEST_IP_TOKEN_KEY: [u8; 32] = [0x42u8; 32];

/// Start a minimal HTTP mock server for `POST /api/v1/agents/provider-key`.
/// Accepts one connection, discards the request body, replies with a JSON provider-key response.
async fn start_mock_provider_key_server(encrypted_token: String, provider: &str) -> String {
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let addr = listener.local_addr().unwrap();
  let provider = provider.to_string();
  tokio::spawn(async move {
    let (mut stream, _) = listener.accept().await.unwrap();
    let mut buf = [0u8; 4096];
    let _ = stream.read(&mut buf).await.unwrap_or(0);
    let body = format!(r#"{{"ip_token":"{encrypted_token}","provider":"{provider}"}}"#);
    let response = format!(
      "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
      body.len(),
      body
    );
    stream.write_all(response.as_bytes()).await.unwrap();
  });
  format!("http://127.0.0.1:{}", addr.port())
}

/// Verifies the full production decryption path end-to-end.
///
/// Previously `KeyFetcher` had zero integration test coverage — `ip_token_e2e_test.rs`
/// called `IpTokenCrypto::decrypt()` directly, bypassing `fetch_from_server()` entirely.
/// This test exercises the real path: mock server → HTTP → `fetch_from_server()` → decrypt.
#[tokio::test]
async fn key_fetcher_decrypts_encrypted_ip_token_from_server() {
  let crypto = IpTokenCrypto::new(&TEST_IP_TOKEN_KEY).unwrap();
  let plaintext_key = "sk-proj-integration-test-api-key";
  let encrypted = crypto.encrypt(plaintext_key).unwrap();

  let server_url = start_mock_provider_key_server(encrypted, "openai").await;

  let fetcher = KeyFetcher::new(
    server_url,
    "test-ic-token".to_string(),
    300,
    Some(TEST_IP_TOKEN_KEY),
  );

  let provider_key = fetcher.get_key().await.unwrap();

  assert_eq!(
    provider_key.api_key.as_str(),
    plaintext_key,
    "KeyFetcher must return the decrypted plaintext key, not the raw ciphertext"
  );
  assert_eq!(provider_key.provider, "openai");
}

/// Regression guard for issue-001: KeyFetcher must fail loudly when `IP_TOKEN_KEY` is absent.
///
/// Before the fix, `None => data.ip_token` silently used the ciphertext as the API key,
/// causing 401 errors on every subsequent LLM call. This test ensures the fixed `None` arm
/// returns `Err` immediately with a message that identifies the misconfiguration.
#[tokio::test]
async fn key_fetcher_fails_loudly_when_ip_token_key_absent() {
  let crypto = IpTokenCrypto::new(&TEST_IP_TOKEN_KEY).unwrap();
  let encrypted = crypto.encrypt("sk-proj-secret-key").unwrap();

  let server_url = start_mock_provider_key_server(encrypted, "openai").await;

  let fetcher = KeyFetcher::new(
    server_url,
    "test-ic-token".to_string(),
    300,
    None, // No IP_TOKEN_KEY — must fail loudly, not silently pass ciphertext
  );

  let result = fetcher.get_key().await;
  assert!(result.is_err(), "KeyFetcher must fail when IP_TOKEN_KEY is absent");
  assert!(
    result.unwrap_err().to_string().contains("IP_TOKEN_KEY not configured"),
    "Error message must identify the missing IP_TOKEN_KEY"
  );
}
