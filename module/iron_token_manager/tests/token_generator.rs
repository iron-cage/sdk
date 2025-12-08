//! Token generator tests
//!
//! Tests for cryptographically secure token generation.
//! Uses REAL cryptographic functions (no mocks).
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_generate_token_returns_non_empty_string` | Token generation produces output | `generate()` | Non-empty string | ✅ |
//! | `test_generate_token_has_minimum_length` | Token meets minimum security length | `generate()` | ≥32 characters | ✅ |
//! | `test_generate_token_produces_unique_tokens` | Tokens are unique (100 iterations) | 100 x `generate()` | All unique, no duplicates | ✅ |
//! | `test_generate_token_uses_base64_encoding` | Token uses URL-safe encoding | `generate()` | Base64 chars only (A-Za-z0-9+/=) | ✅ |
//! | `test_generate_token_has_sufficient_entropy` | No predictable patterns | 10 x `generate()` | No token substring of another | ✅ |
//! | `test_generate_with_prefix_includes_prefix` | Prefix support works | `generate_with_prefix("iron")` | Starts with "iron", longer than prefix | ✅ |
//! | `test_hash_token_produces_sha256_hash` | SHA-256 hashing produces correct format | `hash_token("test_token")` | 64 hex characters | ✅ |
//! | `test_hash_token_is_deterministic` | Same input → same hash | 2 x `hash_token("same")` | Identical hashes | ✅ |
//! | `test_hash_token_different_tokens_produce_different_hashes` | Different inputs → different hashes | `hash_token("a")` vs `hash_token("b")` | Different hashes | ✅ |
//! | `test_verify_token_validates_correct_hash` | Verification succeeds for valid hash | `verify_token(token, correct_hash)` | Returns `true` | ✅ |
//! | `test_verify_token_rejects_wrong_hash` | Verification fails for invalid hash | `verify_token(token, wrong_hash)` | Returns `false` | ✅ |
//! | `test_bcrypt_nondeterminism_breaks_token_lookup` | Bug reproducer: `BCrypt` non-determinism breaks DB lookups (issue-bcrypt-revert) | `hash_token("token")` called twice | Same hash both times (deterministic) | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:**
//! - ✅ Token generation returns non-empty string
//! - ✅ Prefix support for custom token formats
//! - ✅ Hash verification for valid hashes
//!
//! **Boundary Conditions:**
//! - ✅ Minimum token length (32 chars)
//! - ✅ Hash format (exactly 64 hex chars)
//!
//! **Error Conditions:**
//! - ✅ Wrong hash verification (returns `false`, not panic)
//!
//! **Edge Cases:**
//! - ✅ Token uniqueness (100 iterations, no collisions)
//! - ✅ Entropy verification (no predictable patterns)
//! - ✅ Deterministic hashing (same input → same output)
//! - ✅ Base64 encoding validation
//!
//! **State Transitions:** N/A (stateless functions)
//! **Concurrent Access:** Not tested (stateless, thread-safe by design)
//! **Resource Limits:** Not applicable (bounded memory usage)
//! **Precondition Violations:** None (all functions work with any input)

use iron_token_manager::token_generator::TokenGenerator;
use std::collections::HashSet;

#[ test ]
fn test_generate_token_returns_non_empty_string()
{
  let generator = TokenGenerator::new();
  let token = generator.generate();

  assert!( !token.is_empty(), "Generated token should not be empty" );
}

#[ test ]
fn test_generate_token_has_minimum_length()
{
  let generator = TokenGenerator::new();
  let token = generator.generate();

  // Token should be at least 32 characters (for security)
  assert!( token.len() >= 32, "Token should be at least 32 characters, got {}", token.len() );
}

#[ test ]
fn test_generate_token_produces_unique_tokens()
{
  let generator = TokenGenerator::new();
  let mut tokens = HashSet::new();

  // Generate 100 tokens and verify all are unique
  for _ in 0..100
  {
    let token = generator.generate();
    assert!( tokens.insert( token.clone() ), "Generated duplicate token: {token}" );
  }

  assert_eq!( tokens.len(), 100, "Expected 100 unique tokens" );
}

#[ test ]
fn test_generate_token_uses_base64_encoding()
{
  let generator = TokenGenerator::new();
  let token = generator.generate();

  // Base64 characters: A-Z, a-z, 0-9, +, /, =
  let is_base64 = token.chars().all( |c|
    c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
  );

  assert!( is_base64, "Token should be base64 encoded, got: {token}" );
}

#[ test ]
fn test_hash_token_produces_sha256_hash()
{
  let generator = TokenGenerator::new();
  let token = "test_token_12345";
  let hash = generator.hash_token( token );

  // SHA-256 produces 64 hex characters
  assert_eq!( hash.len(), 64, "SHA-256 hash should be 64 hex characters" );

  // Verify hex encoding
  let is_hex = hash.chars().all( |c| c.is_ascii_hexdigit() );
  assert!( is_hex, "Hash should be hex encoded" );
}

#[ test ]
fn test_hash_token_is_deterministic()
{
  let generator = TokenGenerator::new();
  let token = "deterministic_test_token";

  let hash1 = generator.hash_token( token );
  let hash2 = generator.hash_token( token );

  assert_eq!( hash1, hash2, "Same token should produce same hash" );
}

#[ test ]
fn test_hash_token_different_tokens_produce_different_hashes()
{
  let generator = TokenGenerator::new();

  let hash1 = generator.hash_token( "token1" );
  let hash2 = generator.hash_token( "token2" );

  assert_ne!( hash1, hash2, "Different tokens should produce different hashes" );
}

#[ test ]
fn test_generate_with_prefix_includes_prefix()
{
  let generator = TokenGenerator::new();
  let prefix = "iron";
  let token = generator.generate_with_prefix( prefix );

  assert!( token.starts_with( prefix ), "Token should start with prefix '{prefix}', got: {token}" );
  assert!( token.len() > prefix.len(), "Token should be longer than just the prefix" );
}

#[ test ]
fn test_verify_token_validates_correct_hash()
{
  let generator = TokenGenerator::new();
  let token = generator.generate();
  let hash = generator.hash_token( &token );

  let is_valid = generator.verify_token( &token, &hash );
  assert!( is_valid, "Token should verify against its own hash" );
}

#[ test ]
fn test_verify_token_rejects_wrong_hash()
{
  let generator = TokenGenerator::new();
  let token = generator.generate();
  let wrong_hash = "0".repeat( 64 );  // Invalid hash

  let is_valid = generator.verify_token( &token, &wrong_hash );
  assert!( !is_valid, "Token should not verify against wrong hash" );
}

#[ test ]
fn test_generate_token_has_sufficient_entropy()
{
  let generator = TokenGenerator::new();

  // Generate multiple tokens and verify they don't share common patterns
  let tokens : Vec< String > = ( 0..10 ).map( |_| generator.generate() ).collect();

  // No token should be a substring of another (indicating poor randomness)
  for i in 0..tokens.len()
  {
    for j in 0..tokens.len()
    {
      if i != j
      {
        assert!(
          !tokens[ i ].contains( &tokens[ j ] ),
          "Token {i} contains token {j} - insufficient entropy"
        );
      }
    }
  }
}

/// Reproduces database lookup failure when using `BCrypt` for token verification (issue-bcrypt-revert).
///
/// ## Root Cause
///
/// `BCrypt` was incorrectly used for hashing API tokens. `BCrypt` generates random
/// salts on each hash operation, producing different hashes for the same input:
///
/// ```text
/// bcrypt::hash("token", 12) → "$2b$12$randomsalt1..."
/// bcrypt::hash("token", 12) → "$2b$12$randomsalt2..." (DIFFERENT!)
/// ```
///
/// This broke the fundamental contract for token verification:
/// - **Database Lookup:** Token hashes stored as unique index in database
/// - **Verification:** Lookup `WHERE token_hash = hash(provided_token)`
/// - **`BCrypt` Problem:** Same token produces different hash each time → lookup fails
///
/// API tokens have HIGH ENTROPY (256 bits random), making `BCrypt`'s slow cost and
/// random salts unnecessary. `BCrypt` is designed for LOW-ENTROPY passwords, where:
/// - Slow cost defends against brute-force on weak passwords
/// - Random salt prevents rainbow tables
///
/// For cryptographically random tokens (≥128 bits entropy), these protections are
/// redundant. SHA-256 provides:
/// - **Deterministic:** Same input always produces same hash (enables DB lookup)
/// - **Fast:** No artificial slowdown needed (token already unguessable)
/// - **Collision-resistant:** Cryptographically secure for high-entropy inputs
///
/// ## Why Not Caught Initially
///
/// 1. **Incomplete Test Coverage:** Early tests only verified hash format (60 chars
///    for `BCrypt`, 64 chars for SHA-256), not the determinism requirement.
///
/// 2. **Missing Integration Tests:** No tests exercised the full lifecycle:
///    `generate() → hash() → store in DB → retrieve from DB → verify()`
///    The non-determinism only manifests when attempting database lookups.
///
/// 3. **Incorrect Assumption:** Previous fix (issue-003d/e) incorrectly claimed
///    SHA-256 was "vulnerable to brute-force." This is only true for LOW-ENTROPY
///    inputs (passwords), not for 256-bit random tokens.
///
/// 4. **Insufficient Documentation:** Entropy-based decision criteria for choosing
///    hashing algorithms was not documented in specification or module docs.
///
/// ## Fix Applied
///
/// 1. **Reverted to SHA-256:** Changed `token_generator.rs` from `BCrypt` back to SHA-256
///    - Removed `BCrypt` dependency from `Cargo.toml`
///    - Updated hash implementation to use `sha2::Sha256`
///
/// 2. **Added Determinism Tests:** Created `test_hash_token_is_deterministic()` to
///    verify same input produces same hash (prevents future `BCrypt` reintroduction)
///
/// 3. **Documented Fix:** Added Fix(issue-bcrypt-revert) comment in source explaining
///    root cause and entropy-based decision criteria
///
/// 4. **Specification Update:** Added Security Architecture section to `spec.md`
///    with prohibited patterns: "MUST NOT use `BCrypt` for API tokens"
///
/// ## Prevention
///
/// 1. **Entropy-Based Selection:**
///    - HIGH ENTROPY (≥128 bits): SHA-256/SHA-512 (deterministic, fast)
///    - LOW ENTROPY (<100 bits): `BCrypt`/Argon2 (non-deterministic, slow)
///
/// 2. **Database Lookup Requirement:** If hash must support `WHERE hash = ?`
///    queries, algorithm MUST be deterministic (rules out `BCrypt`/Argon2)
///
/// 3. **Test Requirements:**
///    - Hash format tests (verify correct output length/encoding)
///    - Determinism tests (verify same input → same output)
///    - Integration tests (generate → hash → verify full cycle)
///
/// 4. **Specification Enforcement:** Document MUST NOT clauses in spec.md for
///    prohibited algorithm/use-case combinations
///
/// ## Pitfall to Avoid
///
/// **NEVER use `BCrypt`/Argon2 for cryptographically random tokens.**
///
/// Common mistake: "`BCrypt` is more secure than SHA-256, so use it everywhere"
///
/// Reality: `BCrypt`'s security comes from:
/// - Slow cost (defends against brute-force on WEAK inputs)
/// - Random salt (prevents rainbow tables for COMMON inputs)
///
/// For 256-bit random tokens (2^256 possibilities), these protections provide ZERO
/// additional security:
/// - Brute-force impossible regardless of speed (would take longer than universe age)
/// - Rainbow tables impossible (no "common tokens" to pre-compute)
///
/// `BCrypt`'s non-determinism BREAKS functionality (database lookups) while providing
/// no security benefit. This is pure cost with no gain.
///
/// **Correct usage:**
/// - User passwords (40-60 bits entropy) → `BCrypt`/Argon2 (non-deterministic OK)
/// - API tokens (256 bits entropy) → SHA-256/SHA-512 (deterministic required)
/// - Database secrets (128+ bits entropy) → SHA-256 if lookup needed, Argon2 if not
///
// test_kind: bug_reproducer(issue-bcrypt-revert)
#[ test ]
fn test_bcrypt_nondeterminism_breaks_token_lookup()
{
  let generator = TokenGenerator::new();
  let token = "test_token_12345";

  // Verify deterministic hashing (same input always produces same output)
  let hash1 = generator.hash_token( token );
  let hash2 = generator.hash_token( token );

  assert_eq!(
    hash1, hash2,
    "Token hashing must be deterministic for database lookups. \
     `BCrypt` would fail this test (random salt → different hashes)."
  );

  // Verify SHA-256 format (64 hex chars, not `BCrypt`'s 60-char format)
  assert_eq!(
    hash1.len(), 64,
    "SHA-256 produces 64-char hex hash. `BCrypt` produces 60-char format \
     (e.g., $2b$12$...), which would fail this assertion."
  );

  // Verify hex encoding (SHA-256), not `BCrypt`'s base64-like encoding
  let is_hex = hash1.chars().all( |c| c.is_ascii_hexdigit() );
  assert!(
    is_hex,
    "SHA-256 uses hex encoding (0-9a-f). `BCrypt` uses base64-like encoding \
     with $ delimiters, which would fail this assertion."
  );

  // Verify verification works (relies on determinism)
  let is_valid = generator.verify_token( token, &hash1 );
  assert!(
    is_valid,
    "Verification must succeed for correct hash. `BCrypt`'s non-determinism \
     would break this when hash is retrieved from database (hash(token) != stored_hash)."
  );
}
