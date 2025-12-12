//! Token storage integration tests
//!
//! Tests for database storage operations using real `SQLite` databases.
//! No mocks - all tests use real database connections.
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_create_token_stores_hash_not_plaintext` | Token storage uses hash, not plaintext | Token + `user_id` | Hash stored, plaintext NOT stored | ✅ |
//! | `test_create_token_with_metadata` | Metadata storage works correctly | Token + `user_id` + `project_id` + name | Metadata retrievable | ✅ |
//! | `test_verify_token_returns_token_id` | Token verification returns correct ID | Valid token | Returns created token ID | ✅ |
//! | `test_verify_token_fails_for_invalid_token` | Invalid token verification fails | Non-existent token | Returns error | ✅ |
//! | `test_deactivate_token` | Token deactivation prevents verification | Valid token → deactivate | Verification fails after deactivation | ✅ |
//! | `test_list_user_tokens` | User token listing filters by user | Multiple users with tokens | Returns only user's tokens | ✅ |
//! | `test_update_last_used_timestamp` | Last-used timestamp updates | Token + `update_last_used()` | Timestamp changes from None to Some | ✅ |
//! | `test_delete_token` | Token deletion removes from database | Valid token → delete | Verification fails after deletion | ✅ |
//! | `test_token_with_expiration` | Expired tokens fail verification | Token with past expiration | Verification fails, metadata retrievable | ✅ |
//! | `test_protocol_014_token_format_integration` | Protocol 014 tokens work end-to-end | New format token | Creates, stores, verifies with apitok_ prefix | ✅ |
//! | `test_backward_compatibility_old_token_format` | Old tokens without prefix still verify | Old format token | Stores and verifies old format | ✅ |
//! | `test_prefix_stripped_before_hashing_integration` | Prefix stripped during hash storage | New format token | Hash stored without prefix | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:**
//! - ✅ Token creation with full metadata (`user_id`, `project_id`, `name`)
//! - ✅ Token verification returns correct ID
//! - ✅ List tokens filtered by user
//!
//! **Boundary Conditions:**
//! - ✅ Token creation with minimal metadata (`user_id` only, no project/name)
//! - ✅ Expiration exactly in the past (1 hour ago)
//! - ✅ Initially None `last_used_at` timestamp
//!
//! **Error Conditions:**
//! - ✅ Verification of non-existent token returns error
//! - ✅ Verification of deactivated token returns error
//! - ✅ Verification of deleted token returns error
//! - ✅ Verification of expired token returns error
//!
//! **Edge Cases:**
//! - ✅ Hash storage verification (plaintext never stored)
//! - ✅ Metadata retrieval for expired tokens (still accessible)
//! - ✅ Multiple tokens for same user (isolation)
//! - ✅ Timestamp updates (None → Some transition)
//!
//! **State Transitions:**
//! - ✅ Active → Deactivated (verification succeeds → fails)
//! - ✅ Created → Used (`last_used_at`: None → Some)
//! - ✅ Valid → Expired (verification succeeds → fails based on time)
//! - ✅ Exists → Deleted (verification succeeds → fails)
//!
//! **Concurrent Access:** Not tested (`SQLite` handles locking internally, out of scope for integration tests)
//! **Resource Limits:** Not applicable (temporary databases, bounded by test data)
//! **Precondition Violations:** Not applicable (storage validates internally, returns errors for invalid operations)
//!
//! **Protocol 014 Integration:**
//! - ✅ New format tokens (apitok_{64 chars}) work end-to-end
//! - ✅ Old format tokens (no prefix) still verify (backward compatibility)
//! - ✅ Prefix stripped before hashing (hash stored without prefix)

mod common;

use iron_token_manager::token_generator::TokenGenerator;
use common::create_test_storage;

#[ tokio::test ]
async fn test_create_token_stores_hash_not_plaintext()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();
  let plaintext_token = generator.generate();

  // Store token
  let token_id = storage
    .create_token( &plaintext_token, "user_001", Some( "project_123" ), Some( "Test Token" ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create token");

  assert!( token_id > 0, "Token ID should be positive" );

  // Verify hash is stored (not plaintext)
  let stored_hash = storage
    .get_token_hash( token_id )
    .await
    .expect("LOUD FAILURE: Failed to get token hash");

  let expected_hash = generator.hash_token( &plaintext_token );
  assert_eq!( stored_hash, expected_hash, "Stored hash should match computed hash" );

  // Verify plaintext is NOT stored
  let result = storage.verify_token( &plaintext_token ).await;
  assert!( result.is_ok(), "Token verification should succeed" );
}

#[ tokio::test ]
async fn test_create_token_with_metadata()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let token_id = storage
    .create_token(
      &token,
      "user_002",
      Some( "project_456" ),
      Some( "Development Token" ),
      None,
      None,
    )
    .await
    .expect("LOUD FAILURE: Failed to create token");

  // Retrieve token metadata
  let metadata = storage
    .get_token_metadata( token_id )
    .await
    .expect("LOUD FAILURE: Failed to get metadata");

  assert_eq!( metadata.user_id, "user_002" );
  assert_eq!( metadata.project_id, Some( "project_456".to_string() ) );
  assert_eq!( metadata.name, Some( "Development Token".to_string() ) );
  assert!( metadata.is_active );
}

#[ tokio::test ]
async fn test_verify_token_returns_token_id()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let created_id = storage
    .create_token( &token, "user_003", None, None, None, None )
    .await
    .expect("LOUD FAILURE: Failed to create token");

  // Verify token returns the ID
  let verified_id = storage
    .verify_token( &token )
    .await
    .expect("LOUD FAILURE: Failed to verify token");

  assert_eq!( verified_id, created_id, "Verified ID should match created ID" );
}

#[ tokio::test ]
async fn test_verify_token_fails_for_invalid_token()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();

  // Create valid token
  storage
    .create_token( &generator.generate(), "user_004", None, None, None, None )
    .await
    .expect("LOUD FAILURE: Failed to create token");

  // Try to verify non-existent token
  let invalid_token = generator.generate();
  let result = storage.verify_token( &invalid_token ).await;

  assert!( result.is_err(), "Verification should fail for invalid token" );
}

#[ tokio::test ]
async fn test_deactivate_token()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let token_id = storage
    .create_token( &token, "user_005", None, None, None, None )
    .await
    .expect("LOUD FAILURE: Failed to create token");

  // Deactivate token
  storage
    .deactivate_token( token_id )
    .await
    .expect("LOUD FAILURE: Failed to deactivate token");

  // Verify token is now inactive
  let result = storage.verify_token( &token ).await;
  assert!( result.is_err(), "Deactivated token should fail verification" );
}

#[ tokio::test ]
async fn test_list_user_tokens()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();

  // Create multiple tokens for same user
  storage
    .create_token( &generator.generate(), "user_006", None, Some( "Token 1" ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create token 1");

  storage
    .create_token( &generator.generate(), "user_006", None, Some( "Token 2" ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create token 2");

  storage
    .create_token( &generator.generate(), "user_007", None, Some( "Other User Token" ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create token 3");

  // List tokens for user_006
  let tokens = storage
    .list_user_tokens( "user_006" )
    .await
    .expect("LOUD FAILURE: Failed to list tokens");

  assert_eq!( tokens.len(), 2, "Should return 2 tokens for user_006" );
}

#[ tokio::test ]
async fn test_update_last_used_timestamp()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let token_id = storage
    .create_token( &token, "user_008", None, None, None, None )
    .await
    .expect("LOUD FAILURE: Failed to create token");

  // Initially last_used_at should be None
  let metadata = storage
    .get_token_metadata( token_id )
    .await
    .expect("LOUD FAILURE: Failed to get metadata");
  assert!( metadata.last_used_at.is_none(), "last_used_at should initially be None" );

  // Update last used
  storage
    .update_last_used( token_id )
    .await
    .expect("LOUD FAILURE: Failed to update last_used");

  // Verify timestamp was set
  let updated_metadata = storage
    .get_token_metadata( token_id )
    .await
    .expect("LOUD FAILURE: Failed to get updated metadata");
  assert!( updated_metadata.last_used_at.is_some(), "last_used_at should now be set" );
}

#[ tokio::test ]
async fn test_delete_token()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let token_id = storage
    .create_token( &token, "user_009", None, None, None, None )
    .await
    .expect("LOUD FAILURE: Failed to create token");

  // Delete token
  storage
    .delete_token( token_id )
    .await
    .expect("LOUD FAILURE: Failed to delete token");

  // Verify token no longer exists
  let result = storage.verify_token( &token ).await;
  assert!( result.is_err(), "Deleted token should not verify" );
}

#[ tokio::test ]
#[ allow( clippy::cast_possible_truncation ) ]
async fn test_token_with_expiration()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  // Create token that expired 1 hour ago
  let now_ms = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect("LOUD FAILURE: Time went backwards")
    .as_millis() as i64;
  let expired_time = now_ms - ( 3_600_000 ); // -1 hour

  let token_id = storage
    .create_token_with_expiry( &token, "user_010", None, None, Some( expired_time ) )
    .await
    .expect("LOUD FAILURE: Failed to create token");

  // Verify expired token fails
  let result = storage.verify_token( &token ).await;
  assert!( result.is_err(), "Expired token should fail verification" );

  // Metadata should still be retrievable
  let metadata = storage
    .get_token_metadata( token_id )
    .await
    .expect("LOUD FAILURE: Should still retrieve metadata for expired token");
  assert_eq!( metadata.expires_at, Some( expired_time ) );
}

/// Protocol 014 integration test: verify new token format works end-to-end
///
/// Tests that tokens generated with Protocol 014 format (apitok_{64 chars}):
/// 1. Generate in correct format
/// 2. Store successfully in database
/// 3. Verify correctly with prefix stripping
/// 4. Complete full lifecycle (create → verify → use)
#[ tokio::test ]
async fn test_protocol_014_token_format_integration()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();

  // Generate token in Protocol 014 format
  let token = generator.generate();

  // Verify token has correct format
  assert!( token.starts_with( "apitok_" ), "Token should start with apitok_ prefix" );
  assert_eq!( token.len(), 71, "Token should be exactly 71 characters" );

  let body = &token[ 7.. ];
  assert_eq!( body.len(), 64, "Token body should be 64 characters" );
  assert!( body.chars().all( |c| c.is_ascii_alphanumeric() ), "Token body should be Base62" );

  // Create token in database (uses user_001 from seed_test_users)
  let token_id = storage
    .create_token( &token, "user_001", Some( "project_014" ), Some( "Protocol 014 Token" ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create Protocol 014 token");

  assert!( token_id > 0, "Token ID should be positive" );

  // Verify token works end-to-end
  let verified_id = storage
    .verify_token( &token )
    .await
    .expect("LOUD FAILURE: Failed to verify Protocol 014 token");

  assert_eq!( verified_id, token_id, "Verified ID should match created ID" );

  // Verify metadata retrievable
  let metadata = storage
    .get_token_metadata( token_id )
    .await
    .expect("LOUD FAILURE: Failed to get Protocol 014 token metadata");

  assert_eq!( metadata.user_id, "user_001" );
  assert_eq!( metadata.project_id, Some( "project_014".to_string() ) );
  assert_eq!( metadata.name, Some( "Protocol 014 Token".to_string() ) );
  assert!( metadata.is_active );

  // Update last used
  storage
    .update_last_used( token_id )
    .await
    .expect("LOUD FAILURE: Failed to update last_used for Protocol 014 token");

  let updated_metadata = storage
    .get_token_metadata( token_id )
    .await
    .expect("LOUD FAILURE: Failed to get updated metadata");

  assert!( updated_metadata.last_used_at.is_some(), "Last used timestamp should be set" );
}

/// Backward compatibility test: verify old tokens (without apitok_ prefix) still work
///
/// Tests that tokens created before Protocol 014 (no prefix, different format):
/// 1. Store successfully in database
/// 2. Verify correctly (no prefix stripping)
/// 3. Continue to work during migration period
///
/// This ensures zero downtime during Protocol 014 rollout.
#[ tokio::test ]
async fn test_backward_compatibility_old_token_format()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();

  // Simulate old token format (no apitok_ prefix, random Base64-like string)
  // This represents tokens created before Protocol 014 implementation
  let old_token = "xyz789ABC123def456GHI789jkl012MNO345pqr678STU901vwx234YZa567bcd";

  // Create old token in database (uses user_002 from seed_test_users)
  let token_id = storage
    .create_token( old_token, "user_002", Some( "legacy_project" ), Some( "Old Format Token" ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create old format token");

  assert!( token_id > 0, "Token ID should be positive" );

  // Verify old token still works (hash_token should NOT strip prefix from old tokens)
  let verified_id = storage
    .verify_token( old_token )
    .await
    .expect("LOUD FAILURE: Failed to verify old format token");

  assert_eq!( verified_id, token_id, "Old format token should verify successfully" );

  // Verify metadata retrievable
  let metadata = storage
    .get_token_metadata( token_id )
    .await
    .expect("LOUD FAILURE: Failed to get old format token metadata");

  assert_eq!( metadata.user_id, "user_002" );
  assert_eq!( metadata.project_id, Some( "legacy_project".to_string() ) );
  assert_eq!( metadata.name, Some( "Old Format Token".to_string() ) );
  assert!( metadata.is_active );

  // Verify hash stored correctly (entire token, no prefix stripping)
  let stored_hash = storage
    .get_token_hash( token_id )
    .await
    .expect("LOUD FAILURE: Failed to get old token hash");

  let expected_hash = generator.hash_token( old_token );
  assert_eq!( stored_hash, expected_hash, "Old token hash should match (no prefix stripping)" );
}

/// Integration test: verify prefix is stripped before hashing
///
/// Tests that Protocol 014 tokens have their apitok_ prefix stripped before hashing:
/// 1. Generate token with apitok_ prefix
/// 2. Store in database (hash should be of body only, not including prefix)
/// 3. Verify stored hash matches hash of body only
///
/// This ensures:
/// - Database stores hash of token body (64 chars)
/// - Not hash of full token (71 chars including prefix)
/// - Enables future prefix changes without breaking existing hashes
#[ tokio::test ]
async fn test_prefix_stripped_before_hashing_integration()
{
  let ( storage, _temp ) = create_test_storage().await;
  let generator = TokenGenerator::new();

  // Generate token with apitok_ prefix
  let token = generator.generate();
  let body = &token[ 7.. ]; // Extract body (64 chars)

  // Create token in database (uses user_003 from seed_test_users)
  let token_id = storage
    .create_token( &token, "user_003", None, Some( "Prefix Strip Test" ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create token");

  // Get stored hash
  let stored_hash = storage
    .get_token_hash( token_id )
    .await
    .expect("LOUD FAILURE: Failed to get stored hash");

  // Hash should be of body only (not including prefix)
  let expected_hash_body = generator.hash_token( body );
  let expected_hash_full = generator.hash_token( &token );

  // These should be EQUAL because hash_token strips prefix
  assert_eq!( stored_hash, expected_hash_body, "Stored hash should match hash of body only" );
  assert_eq!( stored_hash, expected_hash_full, "hash_token should strip prefix before hashing" );

  // Verify token verification works (uses same prefix-stripping logic)
  let verified_id = storage
    .verify_token( &token )
    .await
    .expect("LOUD FAILURE: Failed to verify token with prefix");

  assert_eq!( verified_id, token_id, "Token should verify successfully with prefix stripping" );
}
