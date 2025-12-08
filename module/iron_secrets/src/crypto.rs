//! Cryptographic operations for secret encryption/decryption
//!
//! Uses AES-256-GCM (AEAD) for authenticated encryption.
//! Master key loaded from environment variable `IRON_SECRETS_MASTER_KEY`.

use aes_gcm::
{
  aead::{ Aead, KeyInit, OsRng },
  Aes256Gcm,
  Nonce,
};
use rand::RngCore;
use zeroize::Zeroizing;

/// Nonce size for AES-256-GCM (96 bits = 12 bytes)
pub const NONCE_SIZE : usize = 12;

/// Key size for AES-256 (256 bits = 32 bytes)
pub const KEY_SIZE : usize = 32;

/// Environment variable name for master key
pub const MASTER_KEY_ENV_VAR : &str = "IRON_SECRETS_MASTER_KEY";

/// Encryption result containing ciphertext and nonce
#[ derive( Debug, Clone ) ]
pub struct EncryptedSecret
{
  /// Encrypted data (ciphertext + auth tag)
  pub ciphertext : Vec< u8 >,
  /// 12-byte nonce used for encryption
  pub nonce : [ u8; NONCE_SIZE ],
}

impl EncryptedSecret
{
  /// Encode ciphertext as base64 string
  pub fn ciphertext_base64( &self ) -> String
  {
    use base64::{ Engine as _, engine::general_purpose::STANDARD };
    STANDARD.encode( &self.ciphertext )
  }

  /// Encode nonce as base64 string
  pub fn nonce_base64( &self ) -> String
  {
    use base64::{ Engine as _, engine::general_purpose::STANDARD };
    STANDARD.encode( &self.nonce )
  }

  /// Decode from base64 strings
  pub fn from_base64( ciphertext_b64 : &str, nonce_b64 : &str ) -> Result< Self, CryptoError >
  {
    use base64::{ Engine as _, engine::general_purpose::STANDARD };

    let ciphertext = STANDARD.decode( ciphertext_b64 )
      .map_err( |_| CryptoError::InvalidBase64 )?;

    let nonce_vec = STANDARD.decode( nonce_b64 )
      .map_err( |_| CryptoError::InvalidBase64 )?;

    if nonce_vec.len() != NONCE_SIZE
    {
      return Err( CryptoError::InvalidNonceLength );
    }

    let mut nonce = [ 0u8; NONCE_SIZE ];
    nonce.copy_from_slice( &nonce_vec );

    Ok( Self { ciphertext, nonce } )
  }
}

/// Cryptographic service for encrypting/decrypting secrets
pub struct CryptoService
{
  cipher : Aes256Gcm,
}

impl std::fmt::Debug for CryptoService
{
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    f.debug_struct( "CryptoService" )
      .field( "cipher", &"<redacted>" )
      .finish()
  }
}

impl CryptoService
{
  /// Create new crypto service with master key
  ///
  /// # Arguments
  ///
  /// * `master_key` - 32-byte master key
  ///
  /// # Errors
  ///
  /// Returns error if master key is invalid length
  pub fn new( master_key : &[ u8 ] ) -> Result< Self, CryptoError >
  {
    if master_key.len() != KEY_SIZE
    {
      return Err( CryptoError::InvalidKeyLength );
    }

    let cipher = Aes256Gcm::new_from_slice( master_key )
      .map_err( |_| CryptoError::InvalidKey )?;

    Ok( Self { cipher } )
  }

  /// Create from environment variable IRON_SECRETS_MASTER_KEY
  ///
  /// # Errors
  ///
  /// Returns error if environment variable not set or invalid
  pub fn from_env() -> Result< Self, CryptoError >
  {
    use base64::{ Engine as _, engine::general_purpose::STANDARD };

    let master_key_b64 = std::env::var( MASTER_KEY_ENV_VAR )
      .map_err( |_| CryptoError::MasterKeyNotSet )?;

    let master_key = STANDARD.decode( &master_key_b64 )
      .map_err( |_| CryptoError::InvalidBase64 )?;

    Self::new( &master_key )
  }

  /// Encrypt plaintext secret
  ///
  /// # Arguments
  ///
  /// * `plaintext` - Secret value to encrypt
  ///
  /// # Returns
  ///
  /// Encrypted secret with random nonce
  pub fn encrypt( &self, plaintext : &str ) -> Result< EncryptedSecret, CryptoError >
  {
    // Generate random nonce
    let mut nonce_bytes = [ 0u8; NONCE_SIZE ];
    OsRng.fill_bytes( &mut nonce_bytes );
    let nonce = Nonce::from_slice( &nonce_bytes );

    // Encrypt
    let ciphertext = self.cipher
      .encrypt( nonce, plaintext.as_bytes() )
      .map_err( |_| CryptoError::EncryptionFailed )?;

    Ok( EncryptedSecret
    {
      ciphertext,
      nonce : nonce_bytes,
    })
  }

  /// Decrypt ciphertext
  ///
  /// # Arguments
  ///
  /// * `encrypted` - Encrypted secret (ciphertext + nonce)
  ///
  /// # Returns
  ///
  /// Decrypted plaintext (zeroized on drop)
  pub fn decrypt( &self, encrypted : &EncryptedSecret ) -> Result< Zeroizing< String >, CryptoError >
  {
    let nonce = Nonce::from_slice( &encrypted.nonce );

    let plaintext_bytes = self.cipher
      .decrypt( nonce, encrypted.ciphertext.as_ref() )
      .map_err( |_| CryptoError::DecryptionFailed )?;

    let plaintext = String::from_utf8( plaintext_bytes )
      .map_err( |_| CryptoError::InvalidUtf8 )?;

    Ok( Zeroizing::new( plaintext ) )
  }
}

/// Crypto operation errors
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum CryptoError
{
  /// Master key environment variable not set
  MasterKeyNotSet,
  /// Master key must be 32 bytes
  InvalidKeyLength,
  /// Failed to parse master key
  InvalidKey,
  /// Invalid base64 encoding
  InvalidBase64,
  /// Nonce must be 12 bytes
  InvalidNonceLength,
  /// AES-GCM encryption failed
  EncryptionFailed,
  /// AES-GCM decryption failed (wrong key or tampered ciphertext)
  DecryptionFailed,
  /// Decrypted data is not valid UTF-8
  InvalidUtf8,
}

impl std::fmt::Display for CryptoError
{
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    match self
    {
      Self::MasterKeyNotSet => write!( f, "Master key not set: environment variable {} not found", MASTER_KEY_ENV_VAR ),
      Self::InvalidKeyLength => write!( f, "Invalid key length: master key must be {} bytes", KEY_SIZE ),
      Self::InvalidKey => write!( f, "Invalid master key" ),
      Self::InvalidBase64 => write!( f, "Invalid base64 encoding" ),
      Self::InvalidNonceLength => write!( f, "Invalid nonce length: must be {} bytes", NONCE_SIZE ),
      Self::EncryptionFailed => write!( f, "Encryption failed" ),
      Self::DecryptionFailed => write!( f, "Decryption failed: wrong key or tampered ciphertext" ),
      Self::InvalidUtf8 => write!( f, "Decrypted data is not valid UTF-8" ),
    }
  }
}

impl std::error::Error for CryptoError {}

/// Mask an API key for display (never show full key)
///
/// # Rules
/// - len <= 8: "***"
/// - len > 8: "first4...last3"
pub fn mask_api_key( key : &str ) -> String
{
  let len = key.len();

  if len <= 8
  {
    return "***".to_string();
  }

  let prefix = &key[ ..4 ];
  let suffix = &key[ len - 3.. ];
  format!( "{}...{}", prefix, suffix )
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

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
}
