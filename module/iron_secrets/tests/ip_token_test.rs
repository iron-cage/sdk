#![allow(missing_docs)]

use iron_secrets::ip_token::{IpTokenCrypto, IpTokenError};
use secrecy::ExposeSecret;

const KEY_SIZE: usize = 32;

fn test_key() -> [u8; KEY_SIZE] {
  [0x42u8; KEY_SIZE]
}

fn other_key() -> [u8; KEY_SIZE] {
  [0x43u8; KEY_SIZE]
}

#[test]
fn encrypt_decrypt_roundtrip() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let plaintext = "sk-proj-test-api-key-12345";

  let ip_token = crypto.encrypt(plaintext).unwrap();
  let decrypted = crypto.decrypt(&ip_token).unwrap();

  assert_eq!(decrypted.expose_secret().as_str(), plaintext);
}

#[test]
fn encrypt_decrypt_empty_string() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();

  let ip_token = crypto.encrypt("").unwrap();
  let decrypted = crypto.decrypt(&ip_token).unwrap();

  assert_eq!(decrypted.expose_secret().as_str(), "");
}

#[test]
fn encrypt_decrypt_long_key() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let long_key = "sk-proj-".to_string() + &"a".repeat(500);

  let ip_token = crypto.encrypt(&long_key).unwrap();
  let decrypted = crypto.decrypt(&ip_token).unwrap();

  assert_eq!(decrypted.expose_secret().as_str(), long_key);
}

#[test]
fn encrypted_token_has_aes256_prefix() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let ip_token = crypto.encrypt("sk-test").unwrap();

  assert!(
    ip_token.starts_with("AES256:"),
    "Token should start with AES256: prefix"
  );
}

#[test]
fn encrypted_token_has_four_colon_separated_parts() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let ip_token = crypto.encrypt("sk-test").unwrap();

  let parts: Vec<&str> = ip_token.split(':').collect();
  assert_eq!(
    parts.len(),
    4,
    "Token should have 4 parts: prefix:iv:ciphertext:tag"
  );
}

#[test]
fn different_encryptions_produce_different_tokens() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let plaintext = "sk-test";

  let token1 = crypto.encrypt(plaintext).unwrap();
  let token2 = crypto.encrypt(plaintext).unwrap();

  assert_ne!(token1, token2, "Each encryption should use different nonce");
}

#[test]
fn wrong_key_fails_decryption() {
  let crypto1 = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let crypto2 = IpTokenCrypto::from_slice(&other_key()).unwrap();

  let ip_token = crypto1.encrypt("sk-secret").unwrap();
  let result = crypto2.decrypt(&ip_token);

  assert_eq!(result.unwrap_err(), IpTokenError::DecryptionFailed);
}

#[test]
fn tampered_ciphertext_fails_decryption() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let ip_token = crypto.encrypt("sk-test").unwrap();

  // Flip a character in the ciphertext (third part)
  let mut parts: Vec<String> = ip_token.split(':').map(String::from).collect();
  let mut bytes = parts[2].clone().into_bytes();
  if !bytes.is_empty() {
    bytes[0] ^= 0xFF;
  }
  parts[2] = String::from_utf8(bytes).unwrap_or_default();
  let tampered = parts.join(":");

  let result = crypto.decrypt(&tampered);
  assert!(
    result.is_err(),
    "Tampered ciphertext should fail decryption"
  );
}

#[test]
fn tampered_tag_fails_decryption() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let ip_token = crypto.encrypt("sk-test").unwrap();

  // Flip a character in the auth tag (fourth part)
  let mut parts: Vec<String> = ip_token.split(':').map(String::from).collect();
  let mut bytes = parts[3].clone().into_bytes();
  if !bytes.is_empty() {
    bytes[0] ^= 0xFF;
  }
  parts[3] = String::from_utf8(bytes).unwrap_or_default();
  let tampered = parts.join(":");

  let result = crypto.decrypt(&tampered);
  assert!(result.is_err(), "Tampered auth tag should fail decryption");
}

#[test]
fn invalid_prefix_returns_error() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let result = crypto.decrypt("INVALID:abc:def:ghi");

  assert_eq!(result.unwrap_err(), IpTokenError::InvalidFormat);
}

#[test]
fn missing_parts_returns_error() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();

  assert_eq!(
    crypto.decrypt("AES256:abc:def").unwrap_err(),
    IpTokenError::InvalidFormat
  );
  assert_eq!(
    crypto.decrypt("AES256:abc").unwrap_err(),
    IpTokenError::InvalidFormat
  );
  assert_eq!(
    crypto.decrypt("AES256").unwrap_err(),
    IpTokenError::InvalidFormat
  );
  assert_eq!(crypto.decrypt("").unwrap_err(), IpTokenError::InvalidFormat);
}

#[test]
fn invalid_base64_returns_error() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let result = crypto.decrypt("AES256:!!!:!!!:!!!");

  assert_eq!(result.unwrap_err(), IpTokenError::InvalidBase64);
}

#[test]
fn garbage_string_returns_error() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let result = crypto.decrypt("totally-not-an-ip-token");

  assert_eq!(result.unwrap_err(), IpTokenError::InvalidFormat);
}

#[test]
fn short_key_returns_error() {
  let err = IpTokenCrypto::from_slice(&[0u8; 16]).unwrap_err();
  assert_eq!(err, IpTokenError::InvalidKeyLength);
}

#[test]
fn long_key_returns_error() {
  let err = IpTokenCrypto::from_slice(&[0u8; 64]).unwrap_err();
  assert_eq!(err, IpTokenError::InvalidKeyLength);
}

#[test]
fn empty_key_returns_error() {
  let err = IpTokenCrypto::from_slice(&[]).unwrap_err();
  assert_eq!(err, IpTokenError::InvalidKeyLength);
}

/// Reproduces Bug: encrypted IP Token passed verbatim as provider API key (issue-001).
///
/// ## Root Cause
/// `key_fetcher.rs` had `None => data.ip_token` in the decryption match arm.
/// When `IP_TOKEN_KEY` env var was absent, `ip_token_crypto` was `None`, causing the
/// `AES256:{IV}:{ciphertext}:{tag}` string to be forwarded directly to OpenAI/Anthropic
/// as the provider API key.
///
/// ## Why Not Caught Initially
/// The encryption and decryption paths were tested in isolation via unit tests.
/// No test verified the shape of an encrypted token against valid provider key formats,
/// so the silent fallback (`None => raw_ciphertext`) went undetected.
///
/// ## Fix Applied
/// The `None` arm in `key_fetcher.rs` now returns `Err(LlmRouterError::KeyFetch(...))`.
/// See `key_fetcher.rs` Fix(issue-001) comment at the match site.
///
/// ## Prevention
/// Any conditional decryption path must fail-loud when the key is absent.
/// Never use `None => raw_value` for security-critical data — use `None => Err(...)`.
///
/// ## Pitfall to Avoid
/// `Option<Crypto>` with a `None => value` fallback is a fail-open pattern.
/// The encrypted string is syntactically valid UTF-8, so the compiler will not warn —
/// the bug produces a runtime 401, not a type error.
// test_kind: bug_reproducer(issue-001)
#[test]
fn encrypted_token_is_not_a_valid_provider_api_key() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let ip_token = crypto.encrypt("sk-proj-real-openai-key").unwrap();

  // The ciphertext starts with "AES256:" — not a valid provider key format.
  // Before the fix this string was sent to OpenAI/Anthropic, causing 401.
  assert!(
    ip_token.starts_with("AES256:"),
    "Encrypted token must have AES256: prefix, confirming it is ciphertext, not an API key"
  );
  assert!(
    !ip_token.starts_with("sk-"),
    "Encrypted token must NOT look like a provider API key"
  );

  // The only correct action is to decrypt — never use the raw token as a key.
  let plaintext = crypto.decrypt(&ip_token).unwrap();
  assert_eq!(
    plaintext.expose_secret().as_str(),
    "sk-proj-real-openai-key"
  );
}

/// Reproduces Bug: missing `IP_TOKEN_KEY` caused silent ciphertext passthrough (issue-002).
///
/// ## Root Cause
/// `budget_client.rs` had `None => data.ip_token` in the decryption match arm.
/// When `IP_TOKEN_KEY` was absent, the handshake appeared to succeed but the agent
/// received `AES256:{IV}:{ciphertext}:{tag}` as its API key, failing every LLM call.
///
/// ## Why Not Caught Initially
/// The handshake returned HTTP 200 — the failure only surfaced on the first LLM request
/// (401 from the provider). No test verified that the decrypted key is structurally
/// different from the encrypted token, so the ticking-time-bomb went undetected.
///
/// ## Fix Applied
/// The `None` arm in `budget_client.rs` now returns `Err(BudgetClientError::IpTokenDecrypt(...))`.
/// See `budget_client.rs` Fix(issue-002) comment at the match site.
///
/// ## Prevention
/// Validate that the decrypted result does not contain the encryption prefix.
/// Fail at startup (or at handshake time) — not silently at the first LLM call.
///
/// ## Pitfall to Avoid
/// A successful handshake HTTP 200 does not guarantee the API key is usable.
/// Decrypt and validate the key format immediately after receiving it.
// test_kind: bug_reproducer(issue-002)
#[test]
fn decrypted_key_does_not_contain_ciphertext_prefix() {
  let crypto = IpTokenCrypto::from_slice(&test_key()).unwrap();
  let original_key = "sk-ant-api03-real-anthropic-key";

  let ip_token = crypto.encrypt(original_key).unwrap();
  let decrypted = crypto.decrypt(&ip_token).unwrap();

  // After correct decryption — no AES256: prefix, the plaintext key is recovered.
  // Before the fix, the `None` fallback arm bypassed this step entirely.
  assert!(
    !decrypted.expose_secret().as_str().starts_with("AES256:"),
    "Decrypted key must NOT contain the ciphertext prefix — it must be the plaintext key"
  );
  assert_eq!(
    decrypted.expose_secret().as_str(),
    original_key,
    "Decrypted key must exactly match the original plaintext key"
  );
}
