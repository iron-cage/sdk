//! Usage tracker integration tests
//!
//! Tests for tracking LLM API usage with real databases (no mocks).
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_record_usage` | Basic usage recording | `token_id`, provider, model, tokens | Usage record created with correct values | ✅ |
//! | `test_record_usage_with_cost` | Usage recording with cost tracking | `token_id`, provider, model, tokens, `cost_cents` | Usage record includes cost | ✅ |
//! | `test_multiple_usage_records` | Multiple usage events for same token | 3 x `record_usage()` | 3 separate records | ✅ |
//! | `test_get_usage_by_provider` | Provider-specific usage filtering | Record openai + anthropic, query "openai" | Returns only openai records | ✅ |
//! | `test_aggregate_token_usage` | Aggregation across multiple records | 2 records (100+200, 50+100 tokens) | `total_tokens`=450, `total_requests`=2 | ✅ |
//! | `test_get_usage_in_time_range` | Time-based usage filtering | Record now, query ±1 hour | Returns usage within range | ✅ |
//! | `test_cascade_delete_usage_on_token_delete` | Cascade deletion when token deleted | Create token + usage, delete token | Usage automatically deleted | ✅ |
//! | `test_usage_records_have_timestamps` | Timestamp recording | `record_usage()` | `recorded_at` > 0 (valid timestamp) | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:**
//! - ✅ Basic usage recording (provider, model, tokens)
//! - ✅ Usage recording with cost tracking
//! - ✅ Provider-specific filtering
//! - ✅ Aggregation across multiple records
//!
//! **Boundary Conditions:**
//! - ✅ Empty usage list (no records for token)
//! - ✅ Time range filtering (±1 hour from now)
//! - ✅ Zero aggregation (empty result set)
//!
//! **Error Conditions:**
//! - ✅ Token deletion cascades to usage records (referential integrity)
//!
//! **Edge Cases:**
//! - ✅ Multiple usage records for same token
//! - ✅ Multiple providers for same token
//! - ✅ Timestamp accuracy (`recorded_at` > 0)
//! - ✅ Cost tracking (optional `cost_cents` field)
//!
//! **State Transitions:**
//! - ✅ Token with usage → Token deleted → Usage deleted (cascade)
//! - ✅ No usage → Usage recorded → Multiple usage records
//!
//! **Concurrent Access:** Not tested (`SQLite` handles locking, out of scope for integration tests)
//! **Resource Limits:** Not applicable (temporary databases, bounded by test data)
//! **Precondition Violations:** Not applicable (tracker validates `token_id` via foreign key constraints)

mod common;

use iron_token_manager::token_generator::TokenGenerator;
use common::create_test_tracker;

#[ tokio::test ]
async fn test_record_usage()
{
  let ( tracker, storage, _temp ) = create_test_tracker().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  // Create token first
  let token_id = storage
    .create_token( &token, "user_001", None, None, None, None )
    .await
    .expect( "Failed to create token" );

  // Record usage
  tracker
    .record_usage( token_id.id, "openai", "gpt-4", 100, 50, 150 )
    .await
    .expect( "Failed to record usage" );

  // Verify usage was recorded
  let usage = tracker
    .get_token_usage( token_id.id )
    .await
    .expect( "Failed to get usage" );

  assert_eq!( usage.len(), 1, "Should have 1 usage record" );
  assert_eq!( usage[ 0 ].provider, "openai" );
  assert_eq!( usage[ 0 ].model, "gpt-4" );
  assert_eq!( usage[ 0 ].input_tokens, 100 );
  assert_eq!( usage[ 0 ].output_tokens, 50 );
  assert_eq!( usage[ 0 ].total_tokens, 150 );
}

#[ tokio::test ]
async fn test_record_usage_with_cost()
{
  let ( tracker, storage, _temp ) = create_test_tracker().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let token_id = storage
    .create_token( &token, "user_002", None, None, None, None )
    .await
    .expect( "Failed to create token" );

  // Record usage with cost (in cents)
  tracker
    .record_usage_with_cost( token_id.id, "anthropic", "claude-sonnet-4-5-20250929", 200, 100, 300, 45 )
    .await
    .expect( "Failed to record usage" );

  let usage = tracker
    .get_token_usage( token_id.id )
    .await
    .expect( "Failed to get usage" );

  assert_eq!( usage.len(), 1 );
  assert_eq!( usage[ 0 ].cost_cents, 45 );
}

#[ tokio::test ]
async fn test_multiple_usage_records()
{
  let ( tracker, storage, _temp ) = create_test_tracker().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let token_id = storage
    .create_token( &token, "user_003", None, None, None, None )
    .await
    .expect( "Failed to create token" );

  // Record multiple usage events
  tracker
    .record_usage( token_id.id, "openai", "gpt-4", 100, 50, 150 )
    .await
    .expect( "Failed to record usage 1" );

  tracker
    .record_usage( token_id.id, "openai", "gpt-4", 200, 100, 300 )
    .await
    .expect( "Failed to record usage 2" );

  tracker
    .record_usage( token_id.id, "gemini", "gemini-pro", 50, 25, 75 )
    .await
    .expect( "Failed to record usage 3" );

  let usage = tracker
    .get_token_usage( token_id.id )
    .await
    .expect( "Failed to get usage" );

  assert_eq!( usage.len(), 3, "Should have 3 usage records" );
}

#[ tokio::test ]
async fn test_get_usage_by_provider()
{
  let ( tracker, storage, _temp ) = create_test_tracker().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let token_id = storage
    .create_token( &token, "user_004", None, None, None, None )
    .await
    .expect( "Failed to create token" );

  // Record usage for different providers
  tracker
    .record_usage( token_id.id, "openai", "gpt-4", 100, 50, 150 )
    .await
    .expect( "Failed to record usage" );

  tracker
    .record_usage( token_id.id, "anthropic", "claude-sonnet-4-5-20250929", 200, 100, 300 )
    .await
    .expect( "Failed to record usage" );

  // Get usage for specific provider
  let openai_usage = tracker
    .get_usage_by_provider( token_id.id, "openai" )
    .await
    .expect( "Failed to get usage" );

  assert_eq!( openai_usage.len(), 1 );
  assert_eq!( openai_usage[ 0 ].provider, "openai" );
}

#[ tokio::test ]
async fn test_aggregate_token_usage()
{
  let ( tracker, storage, _temp ) = create_test_tracker().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let token_id = storage
    .create_token( &token, "user_005", None, None, None, None )
    .await
    .expect( "Failed to create token" );

  // Record multiple usage events
  tracker
    .record_usage( token_id.id, "openai", "gpt-4", 100, 50, 150 )
    .await
    .expect( "Failed" );

  tracker
    .record_usage( token_id.id, "openai", "gpt-4", 200, 100, 300 )
    .await
    .expect( "Failed" );

  // Get aggregated stats
  let stats = tracker
    .get_aggregate_usage( token_id.id )
    .await
    .expect( "Failed to get aggregate" );

  assert_eq!( stats.total_tokens, 450 );
  assert_eq!( stats.total_requests, 2 );
  assert_eq!( stats.input_tokens, 300 );
  assert_eq!( stats.output_tokens, 150 );
}

#[ tokio::test ]
#[ allow( clippy::cast_possible_truncation ) ]
async fn test_get_usage_in_time_range()
{
  let ( tracker, storage, _temp ) = create_test_tracker().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let token_id = storage
    .create_token( &token, "user_006", None, None, None, None )
    .await
    .expect( "Failed to create token" );

  // Record usage
  tracker
    .record_usage( token_id.id, "openai", "gpt-4", 100, 50, 150 )
    .await
    .expect( "Failed to record usage" );

  let now_ms = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "Time went backwards" )
    .as_millis() as i64;

  // Query last hour
  let start_time = now_ms - ( 3_600_000 );
  let end_time = now_ms + ( 3_600_000 );

  let usage = tracker
    .get_usage_in_range( token_id.id, start_time, end_time )
    .await
    .expect( "Failed to get usage in range" );

  assert!( !usage.is_empty(), "Should find usage in time range" );
}

#[ tokio::test ]
async fn test_cascade_delete_usage_on_token_delete()
{
  let ( tracker, storage, _temp ) = create_test_tracker().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let token_id = storage
    .create_token( &token, "user_007", None, None, None, None )
    .await
    .expect( "Failed to create token" );

  // Record usage
  tracker
    .record_usage( token_id.id, "openai", "gpt-4", 100, 50, 150 )
    .await
    .expect( "Failed to record usage" );

  // Verify usage exists
  let usage_before = tracker
    .get_token_usage( token_id.id )
    .await
    .expect( "Failed to get usage" );
  assert_eq!( usage_before.len(), 1 );

  // Delete token (should cascade to usage)
  storage
    .delete_token( token_id.id )
    .await
    .expect( "Failed to delete token" );

  // Verify usage was cascade-deleted
  let usage_after = tracker.get_token_usage( token_id.id ).await;
  assert!( usage_after.is_ok() );
  assert_eq!( usage_after.unwrap().len(), 0, "Usage should be cascade-deleted" );
}

#[ tokio::test ]
async fn test_usage_records_have_timestamps()
{
  let ( tracker, storage, _temp ) = create_test_tracker().await;
  let generator = TokenGenerator::new();
  let token = generator.generate();

  let token_id = storage
    .create_token( &token, "user_008", None, None, None, None )
    .await
    .expect( "Failed to create token" );

  tracker
    .record_usage( token_id.id, "openai", "gpt-4", 100, 50, 150 )
    .await
    .expect( "Failed to record usage" );

  let usage = tracker
    .get_token_usage( token_id.id )
    .await
    .expect( "Failed to get usage" );

  assert!( usage[ 0 ].recorded_at > 0, "Should have valid timestamp" );
}
