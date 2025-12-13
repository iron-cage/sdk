#![allow(missing_docs)]

use iron_secrets::crypto::{ CryptoService, CryptoError, EncryptedSecret, mask_api_key, KEY_SIZE };

fn test_key() -> [ u8; KEY_SIZE ]
{
  [ 0x42u8; KEY_SIZE ]
}

#[ test ]
fn encrypt_decrypt_roundtrip()
{
  let crypto = CryptoService::new( &test_key() ).unwrap();
  let plaintext = "sk-proj-test-api-key-12345";

  let encrypted = crypto.encrypt( plaintext ).unwrap();
  let decrypted = crypto.decrypt( &encrypted ).unwrap();

  assert_eq!( &*decrypted, plaintext, "Decrypted text should match original plaintext" );
}

#[ test ]
fn different_encryptions_have_different_nonces()
{
  let crypto = CryptoService::new( &test_key() ).unwrap();
  let plaintext = "sk-proj-test";

  let encrypted1 = crypto.encrypt( plaintext ).unwrap();
  let encrypted2 = crypto.encrypt( plaintext ).unwrap();

  assert_ne!( encrypted1.nonce, encrypted2.nonce, "Each encryption should use different nonce" );
  assert_ne!( encrypted1.ciphertext, encrypted2.ciphertext, "Different nonces should produce different ciphertexts" );
}

#[ test ]
fn tampered_ciphertext_fails_decryption()
{
  let crypto = CryptoService::new( &test_key() ).unwrap();
  let plaintext = "sk-proj-test";

  let mut encrypted = crypto.encrypt( plaintext ).unwrap();
  encrypted.ciphertext[ 0 ] ^= 0xFF; // Tamper with ciphertext

  let result = crypto.decrypt( &encrypted );
  assert!( matches!( result, Err( CryptoError::DecryptionFailed ) ) );
}

#[ test ]
fn wrong_key_fails_decryption()
{
  let crypto1 = CryptoService::new( &[ 0x42u8; KEY_SIZE ] ).unwrap();
  let crypto2 = CryptoService::new( &[ 0x43u8; KEY_SIZE ] ).unwrap();
  let plaintext = "sk-proj-test";

  let encrypted = crypto1.encrypt( plaintext ).unwrap();
  let result = crypto2.decrypt( &encrypted );

  assert!( matches!( result, Err( CryptoError::DecryptionFailed ) ) );
}

#[ test ]
fn base64_roundtrip()
{
  let crypto = CryptoService::new( &test_key() ).unwrap();
  let plaintext = "sk-proj-test";

  let encrypted = crypto.encrypt( plaintext ).unwrap();
  let ciphertext_b64 = encrypted.ciphertext_base64();
  let nonce_b64 = encrypted.nonce_base64();

  let restored = EncryptedSecret::from_base64( &ciphertext_b64, &nonce_b64 ).unwrap();
  let decrypted = crypto.decrypt( &restored ).unwrap();

  assert_eq!( &*decrypted, plaintext, "Base64 roundtrip should preserve plaintext" );
}

#[ test ]
fn mask_short_key()
{
  assert_eq!( mask_api_key( "short" ), "***", "Short keys should be fully masked" );
  assert_eq!( mask_api_key( "12345678" ), "***", "8-character keys should be fully masked" );
}

#[ test ]
fn mask_long_key()
{
  assert_eq!( mask_api_key( "sk-proj-abc123xyz" ), "sk-p...xyz", "Long keys should show prefix and suffix" );
  assert_eq!( mask_api_key( "sk-ant-api-key-12345" ), "sk-a...345", "API keys should preserve recognizable prefix" );
}
