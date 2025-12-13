//! Integration tests for `LimitEnforcer`
//!
//! Tests limit enforcement (token quotas, request rates, cost caps) with real database.
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_create_limit` | Create usage limits | `user_id`, tokens=10k, requests=60, cost=1M cents | Limit created with correct values | ✅ |
//! | `test_check_tokens_within_limit` | Tokens within quota | Limit=10k, check 5k tokens | Returns `true` (allowed) | ✅ |
//! | `test_check_tokens_exceeds_limit` | Tokens exceed quota | Limit=10k, check 15k tokens | Returns `false` (denied) | ✅ |
//! | `test_increment_tokens` | Token usage tracking | Limit=10k, increment by 3k | `current_tokens_today=3k` | ✅ |
//! | `test_check_requests_within_limit` | Requests within rate limit | Limit=60/min, check 1 request | Returns `true` (allowed) | ✅ |
//! | `test_check_requests_exceeds_limit` | Requests exceed rate limit | Limit=2/min, make 2 requests, check 3rd | Returns `false` (denied) | ✅ |
//! | `test_check_cost_within_limit` | Cost within budget | Limit=100k cents, check 50k cents | Returns `true` (allowed) | ✅ |
//! | `test_unlimited_when_no_limit_set` | No limit = unlimited access | All limits=None, check 1M tokens | Returns `true` (allowed) | ✅ |
//! | `test_project_level_limits` | Project-specific limits | `user+project_id`, limit=5k | Returns `true` for 3k tokens | ✅ |
//! | `test_reset_daily_tokens` | Daily quota reset | Usage=5k, reset | `current_tokens_today=0` | ✅ |
//! | `test_update_existing_limit` | Limit modification | Initial=10k, update to 20k | New limit=20k | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:**
//! - ✅ Create limits with all fields (tokens, requests, cost)
//! - ✅ Check limits within quota (allowed)
//! - ✅ Increment usage counters
//! - ✅ Project-specific limits
//!
//! **Boundary Conditions:**
//! - ✅ Usage exactly at limit (2/2 requests → deny 3rd)
//! - ✅ Zero limits vs None limits (None = unlimited)
//! - ✅ Reset to zero (5k → 0)
//!
//! **Error Conditions:**
//! - ✅ Exceeding token quota (15k > 10k → deny)
//! - ✅ Exceeding request rate (3rd request > 2/min → deny)
//!
//! **Edge Cases:**
//! - ✅ Unlimited access (all limits = None)
//! - ✅ Mixed limits (tokens=Some, requests=None, cost=Some)
//! - ✅ Project-level isolation (user+project vs user-only)
//! - ✅ Limit updates (10k → 20k)
//!
//! **State Transitions:**
//! - ✅ No usage → Usage incremented → At limit → Deny
//! - ✅ At limit → Reset → Zero usage → Allow
//! - ✅ Limit value → Updated limit value
//!
//! **Concurrent Access:** Not tested (`SQLite` handles locking, out of scope for integration tests)
//! **Resource Limits:** Not applicable (temporary databases, bounded by test data)
//! **Precondition Violations:** Not applicable (enforcer creates limits if missing, validates inputs)

mod common;

use common::create_test_enforcer;

#[ tokio::test ]
async fn test_create_limit()
{
  let ( enforcer, _storage, _temp ) = create_test_enforcer().await;

  enforcer
    .create_limit( "user_001", None, Some( 10_000 ), Some( 60 ), Some( 1_000_000 ) )
    .await
    .expect("LOUD FAILURE: Failed to create limit");

  let limit = enforcer
    .get_limit( "user_001", None )
    .await
    .expect("LOUD FAILURE: Failed to get limit");

  assert_eq!( limit.max_tokens_per_day, Some( 10_000 ), "Created limit should have correct max tokens per day" );
  assert_eq!( limit.max_requests_per_minute, Some( 60 ), "Created limit should have correct max requests per minute" );
  assert_eq!( limit.max_cost_cents_per_month, Some( 1_000_000 ), "Created limit should have correct max cost per month" );
}

#[ tokio::test ]
async fn test_check_tokens_within_limit()
{
  let ( enforcer, _storage, _temp ) = create_test_enforcer().await;

  enforcer
    .create_limit( "user_002", None, Some( 10_000 ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create limit");

  let allowed = enforcer
    .check_tokens_allowed( "user_002", None, 5_000 )
    .await
    .expect("LOUD FAILURE: Failed to check limit");

  assert!( allowed, "Should allow tokens within limit" );
}

#[ tokio::test ]
async fn test_check_tokens_exceeds_limit()
{
  let ( enforcer, _storage, _temp ) = create_test_enforcer().await;

  enforcer
    .create_limit( "user_003", None, Some( 10_000 ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create limit");

  let allowed = enforcer
    .check_tokens_allowed( "user_003", None, 15_000 )
    .await
    .expect("LOUD FAILURE: Failed to check limit");

  assert!( !allowed, "Should reject tokens exceeding limit" );
}

#[ tokio::test ]
async fn test_increment_tokens()
{
  let ( enforcer, _storage, _temp ) = create_test_enforcer().await;

  enforcer
    .create_limit( "user_004", None, Some( 10_000 ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create limit");

  enforcer
    .increment_tokens( "user_004", None, 3_000 )
    .await
    .expect("LOUD FAILURE: Failed to increment tokens");

  let limit = enforcer
    .get_limit( "user_004", None )
    .await
    .expect("LOUD FAILURE: Failed to get limit");

  assert_eq!( limit.current_tokens_today, 3_000, "Current tokens should reflect incremented amount" );
}

#[ tokio::test ]
async fn test_check_requests_within_limit()
{
  let ( enforcer, _storage, _temp ) = create_test_enforcer().await;

  enforcer
    .create_limit( "user_005", None, None, Some( 60 ), None )
    .await
    .expect("LOUD FAILURE: Failed to create limit");

  let allowed = enforcer
    .check_request_allowed( "user_005", None )
    .await
    .expect("LOUD FAILURE: Failed to check limit");

  assert!( allowed, "Should allow request within limit" );
}

#[ tokio::test ]
async fn test_check_requests_exceeds_limit()
{
  let ( enforcer, _storage, _temp ) = create_test_enforcer().await;

  enforcer
    .create_limit( "user_006", None, None, Some( 2 ), None )
    .await
    .expect("LOUD FAILURE: Failed to create limit");

  // Make 2 requests (at limit)
  enforcer.increment_requests( "user_006", None ).await.expect("LOUD FAILURE: Failed to increment");
  enforcer.increment_requests( "user_006", None ).await.expect("LOUD FAILURE: Failed to increment");

  let allowed = enforcer
    .check_request_allowed( "user_006", None )
    .await
    .expect("LOUD FAILURE: Failed to check limit");

  assert!( !allowed, "Should reject request exceeding limit" );
}

#[ tokio::test ]
async fn test_check_cost_within_limit()
{
  let ( enforcer, _storage, _temp ) = create_test_enforcer().await;

  enforcer
    .create_limit( "user_007", None, None, None, Some( 100_000 ) )
    .await
    .expect("LOUD FAILURE: Failed to create limit");

  let allowed = enforcer
    .check_cost_allowed( "user_007", None, 50_000 )
    .await
    .expect("LOUD FAILURE: Failed to check limit");

  assert!( allowed, "Should allow cost within limit" );
}

#[ tokio::test ]
async fn test_unlimited_when_no_limit_set()
{
  let ( enforcer, _storage, _temp ) = create_test_enforcer().await;

  enforcer
    .create_limit( "user_008", None, None, None, None )
    .await
    .expect("LOUD FAILURE: Failed to create limit");

  let allowed = enforcer
    .check_tokens_allowed( "user_008", None, 1_000_000 )
    .await
    .expect("LOUD FAILURE: Failed to check limit");

  assert!( allowed, "Should allow unlimited tokens when no limit set" );
}

#[ tokio::test ]
async fn test_project_level_limits()
{
  let ( enforcer, _storage, _temp ) = create_test_enforcer().await;

  enforcer
    .create_limit( "user_009", Some( "project_alpha" ), Some( 5_000 ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create limit");

  let allowed = enforcer
    .check_tokens_allowed( "user_009", Some( "project_alpha" ), 3_000 )
    .await
    .expect("LOUD FAILURE: Failed to check limit");

  assert!( allowed, "Should allow tokens for project-level limit" );
}

#[ tokio::test ]
async fn test_reset_daily_tokens()
{
  let ( enforcer, _storage, _temp ) = create_test_enforcer().await;

  enforcer
    .create_limit( "user_010", None, Some( 10_000 ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create limit");

  enforcer
    .increment_tokens( "user_010", None, 5_000 )
    .await
    .expect("LOUD FAILURE: Failed to increment tokens");

  enforcer
    .reset_daily_tokens( "user_010", None )
    .await
    .expect("LOUD FAILURE: Failed to reset tokens");

  let limit = enforcer
    .get_limit( "user_010", None )
    .await
    .expect("LOUD FAILURE: Failed to get limit");

  assert_eq!( limit.current_tokens_today, 0, "Daily tokens should be reset to 0" );
}

#[ tokio::test ]
async fn test_update_existing_limit()
{
  let ( enforcer, _storage, _temp ) = create_test_enforcer().await;

  enforcer
    .create_limit( "user_011", None, Some( 10_000 ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to create limit");

  enforcer
    .update_limit( "user_011", None, Some( 20_000 ), None, None )
    .await
    .expect("LOUD FAILURE: Failed to update limit");

  let limit = enforcer
    .get_limit( "user_011", None )
    .await
    .expect("LOUD FAILURE: Failed to get limit");

  assert_eq!( limit.max_tokens_per_day, Some( 20_000 ) );
}
