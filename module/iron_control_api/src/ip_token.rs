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

