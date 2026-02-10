#![allow(missing_docs)]

use iron_secrets::ip_token::{IpTokenCrypto, IpTokenError};

const KEY_SIZE: usize = 32;

fn test_key() -> [u8; KEY_SIZE] {
  [0x42u8; KEY_SIZE]
}

fn other_key() -> [u8; KEY_SIZE] {
  [0x43u8; KEY_SIZE]
}

#[test]
fn encrypt_decrypt_roundtrip() {
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();
  let plaintext = "sk-proj-test-api-key-12345";

  let ip_token = crypto.encrypt(plaintext).unwrap();
  let decrypted = crypto.decrypt(&ip_token).unwrap();

  assert_eq!(decrypted.as_str(), plaintext);
}

#[test]
fn encrypt_decrypt_empty_string() {
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();

  let ip_token = crypto.encrypt("").unwrap();
  let decrypted = crypto.decrypt(&ip_token).unwrap();

  assert_eq!(decrypted.as_str(), "");
}

#[test]
fn encrypt_decrypt_long_key() {
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();
  let long_key = "sk-proj-".to_string() + &"a".repeat(500);

  let ip_token = crypto.encrypt(&long_key).unwrap();
  let decrypted = crypto.decrypt(&ip_token).unwrap();

  assert_eq!(decrypted.as_str(), long_key);
}

#[test]
fn encrypted_token_has_aes256_prefix() {
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();
  let ip_token = crypto.encrypt("sk-test").unwrap();

  assert!(
    ip_token.starts_with("AES256:"),
    "Token should start with AES256: prefix"
  );
}

#[test]
fn encrypted_token_has_four_colon_separated_parts() {
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();
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
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();
  let plaintext = "sk-test";

  let token1 = crypto.encrypt(plaintext).unwrap();
  let token2 = crypto.encrypt(plaintext).unwrap();

  assert_ne!(token1, token2, "Each encryption should use different nonce");
}

#[test]
fn wrong_key_fails_decryption() {
  let crypto1 = IpTokenCrypto::new(&test_key()).unwrap();
  let crypto2 = IpTokenCrypto::new(&other_key()).unwrap();

  let ip_token = crypto1.encrypt("sk-secret").unwrap();
  let result = crypto2.decrypt(&ip_token);

  assert_eq!(result, Err(IpTokenError::DecryptionFailed));
}

#[test]
fn tampered_ciphertext_fails_decryption() {
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();
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
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();
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
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();
  let result = crypto.decrypt("INVALID:abc:def:ghi");

  assert_eq!(result, Err(IpTokenError::InvalidFormat));
}

#[test]
fn missing_parts_returns_error() {
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();

  assert_eq!(
    crypto.decrypt("AES256:abc:def"),
    Err(IpTokenError::InvalidFormat)
  );
  assert_eq!(
    crypto.decrypt("AES256:abc"),
    Err(IpTokenError::InvalidFormat)
  );
  assert_eq!(crypto.decrypt("AES256"), Err(IpTokenError::InvalidFormat));
  assert_eq!(crypto.decrypt(""), Err(IpTokenError::InvalidFormat));
}

#[test]
fn invalid_base64_returns_error() {
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();
  let result = crypto.decrypt("AES256:!!!:!!!:!!!");

  assert_eq!(result, Err(IpTokenError::InvalidBase64));
}

#[test]
fn garbage_string_returns_error() {
  let crypto = IpTokenCrypto::new(&test_key()).unwrap();
  let result = crypto.decrypt("totally-not-an-ip-token");

  assert_eq!(result, Err(IpTokenError::InvalidFormat));
}

#[test]
fn short_key_returns_error() {
  let err = IpTokenCrypto::new(&[0u8; 16]).unwrap_err();
  assert_eq!(err, IpTokenError::InvalidKeyLength);
}

#[test]
fn long_key_returns_error() {
  let err = IpTokenCrypto::new(&[0u8; 64]).unwrap_err();
  assert_eq!(err, IpTokenError::InvalidKeyLength);
}

#[test]
fn empty_key_returns_error() {
  let err = IpTokenCrypto::new(&[]).unwrap_err();
  assert_eq!(err, IpTokenError::InvalidKeyLength);
}
