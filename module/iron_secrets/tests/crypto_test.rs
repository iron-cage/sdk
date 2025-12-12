use iron_secrets::*;

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

  assert_eq!( &*decrypted, plaintext );
}

#[ test ]
fn different_encryptions_have_different_nonces()
{
  let crypto = CryptoService::new( &test_key() ).unwrap();
  let plaintext = "sk-proj-test";

  let encrypted1 = crypto.encrypt( plaintext ).unwrap();
  let encrypted2 = crypto.encrypt( plaintext ).unwrap();

  assert_ne!( encrypted1.nonce, encrypted2.nonce );
  assert_ne!( encrypted1.ciphertext, encrypted2.ciphertext );
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

  assert_eq!( &*decrypted, plaintext );
}

#[ test ]
fn mask_short_key()
{
  assert_eq!( mask_api_key( "short" ), "***" );
  assert_eq!( mask_api_key( "12345678" ), "***" );
}

#[ test ]
fn mask_long_key()
{
  assert_eq!( mask_api_key( "sk-proj-abc123xyz" ), "sk-p...xyz" );
  assert_eq!( mask_api_key( "sk-ant-api-key-12345" ), "sk-a...345" );
}
