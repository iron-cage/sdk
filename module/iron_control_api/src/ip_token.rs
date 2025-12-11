//! IP Token (Iron Provider Token) encryption
//!
//! Protocol 005: Budget Control Protocol
//!
//! IP Tokens are AES-256-GCM encrypted provider API keys that are:
//! - Never written to disk (memory-only storage)
//! - Encrypted with runtime session key
//! - Format: AES256:{IV_base64}:{ciphertext_base64}:{auth_tag_base64}
//!
//! The runtime acts as a secure proxy, decrypting IP Tokens on-demand to make
//! LLM API calls without exposing provider credentials to the developer.

use aes_gcm::
{
  aead::{ Aead, KeyInit, OsRng },
  Aes256Gcm,
  Nonce,
};
use base64::{ Engine as _, engine::general_purpose::STANDARD };
use rand::RngCore;
use zeroize::Zeroizing;

/// Nonce size for AES-256-GCM (96 bits = 12 bytes)
const NONCE_SIZE : usize = 12;

/// Auth tag size for AES-256-GCM (128 bits = 16 bytes)
const TAG_SIZE : usize = 16;

/// Key size for AES-256 (256 bits = 32 bytes)
const KEY_SIZE : usize = 32;

/// IP Token crypto manager
///
/// Encrypts and decrypts provider API keys using AES-256-GCM.
/// Tokens are formatted as: `AES256:{IV}:{ciphertext}:{tag}`
pub struct IpTokenCrypto
{
  cipher : Aes256Gcm,
}

impl std::fmt::Debug for IpTokenCrypto
{
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    f.debug_struct( "IpTokenCrypto" )
      .field( "cipher", &"<redacted>" )
      .finish()
  }
}

impl IpTokenCrypto
{
  /// Create new IP Token crypto manager
  ///
  /// # Arguments
  ///
  /// * `key` - 32-byte encryption key
  ///
  /// # Errors
  ///
  /// Returns error if key length is invalid
  pub fn new( key : &[ u8 ] ) -> Result< Self, IpTokenError >
  {
    if key.len() != KEY_SIZE
    {
      return Err( IpTokenError::InvalidKeyLength );
    }

    let cipher = Aes256Gcm::new_from_slice( key )
      .map_err( |_| IpTokenError::InvalidKey )?;

    Ok( Self { cipher } )
  }

  /// Encrypt provider API key into IP Token format
  ///
  /// # Arguments
  ///
  /// * `provider_api_key` - Plaintext provider API key (e.g., "sk-proj_abc123...")
  ///
  /// # Returns
  ///
  /// Encrypted IP Token string in format: `AES256:{IV}:{ciphertext}:{tag}`
  ///
  /// # Errors
  ///
  /// Returns error if encryption fails
  pub fn encrypt( &self, provider_api_key : &str ) -> Result< String, IpTokenError >
  {
    // Generate random nonce (IV)
    let mut nonce_bytes = [ 0u8; NONCE_SIZE ];
    OsRng.fill_bytes( &mut nonce_bytes );
    let nonce = Nonce::from_slice( &nonce_bytes );

    // Encrypt (produces ciphertext + auth tag combined)
    let ciphertext_with_tag = self.cipher
      .encrypt( nonce, provider_api_key.as_bytes() )
      .map_err( |_| IpTokenError::EncryptionFailed )?;

    // Split ciphertext and auth tag
    // AES-GCM appends 16-byte tag to ciphertext
    if ciphertext_with_tag.len() < TAG_SIZE
    {
      return Err( IpTokenError::EncryptionFailed );
    }

    let ciphertext_len = ciphertext_with_tag.len() - TAG_SIZE;
    let ciphertext = &ciphertext_with_tag[ ..ciphertext_len ];
    let tag = &ciphertext_with_tag[ ciphertext_len.. ];

    // Encode as base64
    let iv_b64 = STANDARD.encode( nonce_bytes );
    let ciphertext_b64 = STANDARD.encode( ciphertext );
    let tag_b64 = STANDARD.encode( tag );

    // Format: AES256:{IV}:{ciphertext}:{tag}
    Ok( format!( "AES256:{iv_b64}:{ciphertext_b64}:{tag_b64}" ) )
  }

  /// Decrypt IP Token to recover provider API key
  ///
  /// # Arguments
  ///
  /// * `ip_token` - Encrypted IP Token string (format: AES256:{IV}:{ciphertext}:{tag})
  ///
  /// # Returns
  ///
  /// Decrypted provider API key (zeroized on drop)
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - Format is invalid
  /// - Base64 decoding fails
  /// - Decryption fails (wrong key or tampered data)
  /// - Plaintext is not valid UTF-8
  pub fn decrypt( &self, ip_token : &str ) -> Result< Zeroizing< String >, IpTokenError >
  {
    // Parse format: AES256:{IV}:{ciphertext}:{tag}
    let parts : Vec< &str > = ip_token.split( ':' ).collect();
    if parts.len() != 4 || parts[ 0 ] != "AES256"
    {
      return Err( IpTokenError::InvalidFormat );
    }

    // Decode base64 components
    let iv_bytes = STANDARD.decode( parts[ 1 ] )
      .map_err( |_| IpTokenError::InvalidBase64 )?;

    let ciphertext_bytes = STANDARD.decode( parts[ 2 ] )
      .map_err( |_| IpTokenError::InvalidBase64 )?;

    let tag_bytes = STANDARD.decode( parts[ 3 ] )
      .map_err( |_| IpTokenError::InvalidBase64 )?;

    // Validate lengths
    if iv_bytes.len() != NONCE_SIZE
    {
      return Err( IpTokenError::InvalidNonceLength );
    }

    if tag_bytes.len() != TAG_SIZE
    {
      return Err( IpTokenError::InvalidTagLength );
    }

    // Recombine ciphertext + tag for aes-gcm library
    let mut ciphertext_with_tag = ciphertext_bytes;
    ciphertext_with_tag.extend_from_slice( &tag_bytes );

    // Decrypt
    let nonce = Nonce::from_slice( &iv_bytes );
    let plaintext_bytes = self.cipher
      .decrypt( nonce, ciphertext_with_tag.as_ref() )
      .map_err( |_| IpTokenError::DecryptionFailed )?;

    let plaintext = String::from_utf8( plaintext_bytes )
      .map_err( |_| IpTokenError::InvalidUtf8 )?;

    Ok( Zeroizing::new( plaintext ) )
  }
}

/// IP Token operation errors
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum IpTokenError
{
  /// Invalid key length (must be 32 bytes)
  InvalidKeyLength,

  /// Invalid encryption key
  InvalidKey,

  /// Invalid IP Token format (expected: AES256:{IV}:{ciphertext}:{tag})
  InvalidFormat,

  /// Invalid base64 encoding
  InvalidBase64,

  /// Invalid nonce length (must be 12 bytes)
  InvalidNonceLength,

  /// Invalid auth tag length (must be 16 bytes)
  InvalidTagLength,

  /// Encryption failed
  EncryptionFailed,

  /// Decryption failed (wrong key or tampered data)
  DecryptionFailed,

  /// Decrypted data is not valid UTF-8
  InvalidUtf8,
}

impl std::fmt::Display for IpTokenError
{
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    match self
    {
      Self::InvalidKeyLength => write!( f, "Invalid key length: must be {} bytes", KEY_SIZE ),
      Self::InvalidKey => write!( f, "Invalid encryption key" ),
      Self::InvalidFormat => write!( f, "Invalid IP Token format (expected: AES256:{{IV}}:{{ciphertext}}:{{tag}})" ),
      Self::InvalidBase64 => write!( f, "Invalid base64 encoding" ),
      Self::InvalidNonceLength => write!( f, "Invalid nonce length: must be {} bytes", NONCE_SIZE ),
      Self::InvalidTagLength => write!( f, "Invalid auth tag length: must be {} bytes", TAG_SIZE ),
      Self::EncryptionFailed => write!( f, "Encryption failed" ),
      Self::DecryptionFailed => write!( f, "Decryption failed: wrong key or tampered data" ),
      Self::InvalidUtf8 => write!( f, "Decrypted data is not valid UTF-8" ),
    }
  }
}

impl std::error::Error for IpTokenError {}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  fn test_key() -> [ u8; KEY_SIZE ]
  {
    // Fixed test key (32 bytes)
    [ 0x42; KEY_SIZE ]
  }

  #[ test ]
  fn test_encrypt_and_decrypt()
  {
    let crypto = IpTokenCrypto::new( &test_key() ).expect( "Should create crypto" );
    let api_key = "sk-proj_abc123def456ghi789";

    // Encrypt
    let ip_token = crypto.encrypt( api_key ).expect( "Should encrypt" );
    assert!( ip_token.starts_with( "AES256:" ) );

    // Decrypt
    let decrypted = crypto.decrypt( &ip_token ).expect( "Should decrypt" );
    assert_eq!( *decrypted, api_key );
  }

  #[ test ]
  fn test_format_structure()
  {
    let crypto = IpTokenCrypto::new( &test_key() ).expect( "Should create crypto" );
    let ip_token = crypto.encrypt( "test_key_123" ).expect( "Should encrypt" );

    // Check format: AES256:{IV}:{ciphertext}:{tag}
    let parts : Vec< &str > = ip_token.split( ':' ).collect();
    assert_eq!( parts.len(), 4 );
    assert_eq!( parts[ 0 ], "AES256" );

    // IV should be 12 bytes base64 (16 characters)
    let iv_decoded = STANDARD.decode( parts[ 1 ] ).expect( "Should decode IV" );
    assert_eq!( iv_decoded.len(), NONCE_SIZE );

    // Tag should be 16 bytes base64
    let tag_decoded = STANDARD.decode( parts[ 3 ] ).expect( "Should decode tag" );
    assert_eq!( tag_decoded.len(), TAG_SIZE );
  }

  #[ test ]
  fn test_decrypt_invalid_format()
  {
    let crypto = IpTokenCrypto::new( &test_key() ).expect( "Should create crypto" );
    let result = crypto.decrypt( "invalid_format" );
    assert!( result.is_err() );
    assert_eq!( result.unwrap_err(), IpTokenError::InvalidFormat );
  }

  #[ test ]
  fn test_decrypt_wrong_key()
  {
    let crypto1 = IpTokenCrypto::new( &test_key() ).expect( "Should create crypto" );
    let ip_token = crypto1.encrypt( "test_key" ).expect( "Should encrypt" );

    let crypto2 = IpTokenCrypto::new( &[ 0x99; KEY_SIZE ] ).expect( "Should create crypto" );
    let result = crypto2.decrypt( &ip_token );

    assert!( result.is_err() );
    assert_eq!( result.unwrap_err(), IpTokenError::DecryptionFailed );
  }

  #[ test ]
  fn test_decrypt_tampered_ciphertext()
  {
    let crypto = IpTokenCrypto::new( &test_key() ).expect( "Should create crypto" );
    let ip_token = crypto.encrypt( "test_key" ).expect( "Should encrypt" );

    // Tamper with ciphertext part
    let parts : Vec< &str > = ip_token.split( ':' ).collect();
    let tampered = format!( "{}:{}:TAMPERED:{}",
      parts[ 0 ], parts[ 1 ], parts[ 3 ] );

    let result = crypto.decrypt( &tampered );
    assert!( result.is_err() );
  }

  #[ test ]
  fn test_invalid_key_length()
  {
    let result = IpTokenCrypto::new( &[ 0x42; 16 ] ); // Wrong length
    assert!( result.is_err() );
    assert_eq!( result.unwrap_err(), IpTokenError::InvalidKeyLength );
  }

  #[ test ]
  fn test_multiple_encryptions_different_nonces()
  {
    let crypto = IpTokenCrypto::new( &test_key() ).expect( "Should create crypto" );
    let api_key = "sk-proj_same_key";

    let token1 = crypto.encrypt( api_key ).expect( "Should encrypt" );
    let token2 = crypto.encrypt( api_key ).expect( "Should encrypt" );

    // Different tokens (due to random nonces)
    assert_ne!( token1, token2 );

    // But both decrypt to same value
    let dec1 = crypto.decrypt( &token1 ).expect( "Should decrypt" );
    let dec2 = crypto.decrypt( &token2 ).expect( "Should decrypt" );
    assert_eq!( *dec1, api_key );
    assert_eq!( *dec2, api_key );
  }
}
