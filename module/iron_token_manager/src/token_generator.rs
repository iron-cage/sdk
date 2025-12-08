//! Token generation service
//!
//! Generates cryptographically secure API tokens for user/project authentication.
//!
//! ## Known Pitfalls
//!
//! ### Hashing Algorithm Selection (Issue: issue-bcrypt-revert)
//!
//! **NEVER use BCrypt/Argon2 for cryptographically random tokens.**
//!
//! Root cause: `BCrypt` was incorrectly used for API token hashing in an earlier
//! implementation. `BCrypt` generates random salts on each hash operation, producing
//! different hashes for the same input. This broke database lookups for token
//! verification (`WHERE token_hash = hash(provided_token)`), since the same token
//! would produce different hashes each time.
//!
//! API tokens have HIGH ENTROPY (256 bits random), making `BCrypt`'s security properties
//! (slow cost, random salts) unnecessary and harmful:
//!
//! - **Slow cost:** Defends against brute-force on WEAK inputs (passwords).
//!   For 256-bit random tokens, brute-force is impossible regardless of speed.
//!
//! - **Random salt:** Prevents rainbow tables for COMMON inputs (passwords).
//!   For cryptographically random tokens, rainbow tables are impossible (no "common tokens").
//!
//! `BCrypt`'s non-determinism BREAKS functionality (database lookups) while providing
//! ZERO security benefit for high-entropy inputs. This is pure cost with no gain.
//!
//! ### Entropy-Based Algorithm Selection
//!
//! Use this decision matrix when choosing hashing algorithms:
//!
//! | Input Type | Entropy | Algorithm | Reason |
//! |------------|---------|-----------|--------|
//! | User passwords | 40-60 bits | BCrypt/Argon2 | Slow cost + random salt defend weak inputs |
//! | API tokens | 256 bits | SHA-256/SHA-512 | Deterministic for DB lookups, already unguessable |
//! | Database secrets | 128+ bits | SHA-256 (if lookup) or Argon2 (if not) | Choose based on determinism requirement |
//!
//! **Key decision criteria:**
//!
//! 1. **HIGH ENTROPY (≥128 bits):**
//!    - Use SHA-256/SHA-512 (deterministic, fast)
//!    - `BCrypt` provides no additional security
//!    - Determinism enables database indexing/lookups
//!
//! 2. **LOW ENTROPY (<100 bits):**
//!    - Use BCrypt/Argon2 (non-deterministic, slow)
//!    - Slow cost defends against brute-force
//!    - Random salt prevents rainbow tables
//!    - Non-determinism is acceptable (no DB lookups needed)
//!
//! ### Examples
//!
//! **Correct usage:**
//!
//! ```rust
//! use iron_token_manager::token_generator::TokenGenerator;
//!
//! // API token: 256 bits entropy → SHA-256
//! let generator = TokenGenerator::new();
//! let token = generator.generate();              // 32 random bytes
//! let hash = generator.hash_token( &token );     // SHA-256 (deterministic)
//! // Database: CREATE UNIQUE INDEX ON tokens(token_hash) -- works because deterministic
//! ```
//!
//! **Incorrect usage (DO NOT DO THIS):**
//!
//! ```ignore
//! // WRONG: BCrypt for API token breaks database lookups
//! let token = TokenGenerator::new().generate();
//! let hash1 = bcrypt::hash( &token, 12 )?;  // "$2b$12$randomsalt1..."
//! let hash2 = bcrypt::hash( &token, 12 )?;  // "$2b$12$randomsalt2..." (DIFFERENT!)
//! // Database lookup fails: WHERE token_hash = ? (hash changes every time)
//! ```
//!
//! ### References
//!
//! - Test: `tests/token_generator.rs::test_bcrypt_nondeterminism_breaks_token_lookup()`
//! - Spec: `pilot/spec.md` Security Architecture section
//! - Fix comment: `hash_token()` function (lines 148-153)

use rand::{ Rng, thread_rng };
use base64::{ Engine as _, engine::general_purpose };
use sha2::{ Sha256, Digest };

/// Token generator for API access
///
/// Generates cryptographically secure tokens using rand + SHA-256 hashing.
///
/// # Security Properties
///
/// - Uses `rand::thread_rng()` for cryptographic randomness
/// - Uses SHA-256 for deterministic token hashing
/// - Generates 32 random bytes (256 bits of entropy)
/// - Encodes tokens as base64 for URL-safe transmission
/// - Stores SHA-256 hashes, never plaintext tokens
///
/// # Example
///
/// ```rust
/// use iron_token_manager::token_generator::TokenGenerator;
///
/// let generator = TokenGenerator::new();
/// let token = generator.generate();
/// let hash = generator.hash_token( &token );
///
/// assert!( generator.verify_token( &token, &hash ) );
/// ```
#[ derive( Debug, Clone ) ]
pub struct TokenGenerator;

impl TokenGenerator
{
  /// Create new token generator
  #[ must_use ]
  pub fn new() -> Self
  {
    Self
  }

  /// Generate cryptographically secure random token
  ///
  /// Generates 32 random bytes using `thread_rng()` and encodes as base64.
  ///
  /// # Returns
  ///
  /// Base64-encoded token string (44 characters for 32 bytes)
  #[ must_use ]
  pub fn generate( &self ) -> String
  {
    let mut rng = thread_rng();
    let random_bytes : [ u8; 32 ] = rng.gen();
    general_purpose::STANDARD.encode( random_bytes )
  }

  /// Generate token with custom prefix
  ///
  /// # Arguments
  ///
  /// * `prefix` - Prefix to prepend to token (e.g., "iron_", "sk-")
  ///
  /// # Returns
  ///
  /// Token string starting with prefix, followed by underscore and random data
  #[ must_use ]
  pub fn generate_with_prefix( &self, prefix: &str ) -> String
  {
    format!( "{}_{}", prefix, self.generate() )
  }

  // Fix(issue-bcrypt-revert): Revert from BCrypt to SHA-256 for API token hashing
  // Root cause: BCrypt is designed for LOW-ENTROPY passwords, not cryptographically random tokens.
  //   Previous Fix(issue-003d/e) incorrectly claimed SHA-256 was vulnerable to brute-force,
  //   but this only applies to weak passwords, NOT to 256-bit random tokens.
  // Pitfall: Use BCrypt/Argon2 for PASSWORDS (user-chosen, low entropy).
  //   Use SHA-256/SHA-512 for RANDOM TOKENS (cryptographically random, high entropy).
  //   BCrypt's non-determinism breaks database lookups for token verification.

  /// Hash token using SHA-256
  ///
  /// # Arguments
  ///
  /// * `token` - Plaintext token to hash
  ///
  /// # Returns
  ///
  /// Hex-encoded SHA-256 hash (64 characters)
  ///
  /// # Security Note
  ///
  /// SHA-256 is appropriate for API token hashing because:
  /// - Tokens are cryptographically random (256 bits entropy)
  /// - Deterministic hashing enables fast database lookups
  /// - SHA-256 provides collision resistance for random inputs
  /// - No salt needed for high-entropy random values
  ///
  /// For LOW-ENTROPY passwords, use BCrypt/Argon2 instead.
  #[ must_use ]
  pub fn hash_token( &self, token: &str ) -> String
  {
    let mut hasher = Sha256::new();
    hasher.update( token.as_bytes() );
    format!( "{:x}", hasher.finalize() )
  }

  /// Verify token against stored SHA-256 hash
  ///
  /// # Arguments
  ///
  /// * `token` - Plaintext token to verify
  /// * `stored_hash` - Previously computed SHA-256 hash to check against
  ///
  /// # Returns
  ///
  /// `true` if token matches hash, `false` otherwise
  #[ must_use ]
  pub fn verify_token( &self, token: &str, stored_hash: &str ) -> bool
  {
    self.hash_token( token ) == stored_hash
  }
}

impl Default for TokenGenerator
{
  fn default() -> Self
  {
    Self::new()
  }
}
