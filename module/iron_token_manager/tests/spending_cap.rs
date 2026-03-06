//! Tests for provider key spending cap management.

#![allow(missing_docs)]

mod common;

use iron_token_manager::ProviderType;

#[tokio::test]
async fn no_cap_reserve_succeeds() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  // No cap set -> reserve should always succeed
  storage.reserve_spending(key_id, 999_999).await.unwrap();
}

#[tokio::test]
async fn set_cap_and_reserve_within_limit() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage
    .set_spending_cap(key_id, Some(10_000_000))
    .await
    .unwrap();

  storage.reserve_spending(key_id, 5_000_000).await.unwrap();
}

#[tokio::test]
async fn increment_then_reserve_still_within() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage
    .set_spending_cap(key_id, Some(10_000_000))
    .await
    .unwrap();

  storage.increment_spending(key_id, 5_000_000).await.unwrap();

  // Still 5M of room left
  storage.reserve_spending(key_id, 4_000_000).await.unwrap();
}

#[tokio::test]
async fn reserve_to_exact_cap_succeeds() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.set_spending_cap(key_id, Some(1_000)).await.unwrap();

  // Reserve exactly to the cap should succeed (used + amount <= cap)
  storage.reserve_spending(key_id, 1_000).await.unwrap();

  // Any further reservation should fail
  assert!(
    storage.reserve_spending(key_id, 1).await.is_err(),
    "Over cap should fail"
  );
}

#[tokio::test]
async fn increment_beyond_cap_fails() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.set_spending_cap(key_id, Some(1_000)).await.unwrap();

  // Try to increment beyond cap
  let result = storage.increment_spending(key_id, 1_001).await;
  assert!(result.is_err(), "Exceeding cap should fail");
}

#[tokio::test]
async fn increment_without_cap_succeeds() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  // No cap set -> large increment should succeed
  storage
    .increment_spending(key_id, 999_999_999)
    .await
    .unwrap();
}

#[tokio::test]
async fn get_spending_summary_initial() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  let summary = storage.get_spending_summary(key_id).await.unwrap();
  assert_eq!(summary.used_microdollars, 0);
  assert_eq!(summary.cap_microdollars, None);
}

#[tokio::test]
async fn get_spending_summary_after_usage() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage
    .set_spending_cap(key_id, Some(10_000))
    .await
    .unwrap();
  storage.increment_spending(key_id, 3_500).await.unwrap();

  let summary = storage.get_spending_summary(key_id).await.unwrap();
  assert_eq!(summary.used_microdollars, 3_500);
  assert_eq!(summary.cap_microdollars, Some(10_000));
}

#[tokio::test]
async fn multiple_increments_accumulate() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage
    .set_spending_cap(key_id, Some(10_000))
    .await
    .unwrap();

  storage.increment_spending(key_id, 2_000).await.unwrap();
  storage.increment_spending(key_id, 3_000).await.unwrap();
  storage.increment_spending(key_id, 4_000).await.unwrap();

  let summary = storage.get_spending_summary(key_id).await.unwrap();
  assert_eq!(summary.used_microdollars, 9_000);

  // One more should succeed (9000 + 1000 = 10000 <= cap)
  storage.increment_spending(key_id, 1_000).await.unwrap();

  // But any other should fail
  let result = storage.increment_spending(key_id, 1).await;
  assert!(result.is_err(), "Over cap after accumulation");
}

#[tokio::test]
async fn remove_cap_allows_unlimited() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  // Set cap, exhaust it
  storage.set_spending_cap(key_id, Some(1_000)).await.unwrap();
  storage.increment_spending(key_id, 1_000).await.unwrap();

  // Can't spend more
  assert!(storage.increment_spending(key_id, 1).await.is_err());

  // Remove cap
  storage.set_spending_cap(key_id, None).await.unwrap();

  // Now spending should succeed
  storage.increment_spending(key_id, 1).await.unwrap();
}

#[tokio::test]
async fn spending_summary_nonexistent_key_fails() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let result = storage.get_spending_summary(99999).await;
  assert!(result.is_err(), "Nonexistent key should error");
}

#[tokio::test]
async fn reserve_nonexistent_key_fails() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let result = storage.reserve_spending(99999, 100).await;
  assert!(result.is_err(), "Nonexistent key should error");
}

#[tokio::test]
async fn adjust_spending_releases_excess() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage
    .set_spending_cap(key_id, Some(10_000))
    .await
    .unwrap();

  // Reserve 5000
  storage.reserve_spending(key_id, 5_000).await.unwrap();

  // Actual cost was only 2000 - adjust should release 3000
  storage.adjust_spending(key_id, 5_000, 2_000).await.unwrap();

  let summary = storage.get_spending_summary(key_id).await.unwrap();
  assert_eq!(
    summary.used_microdollars, 2_000,
    "Should reflect actual cost after adjust"
  );
}

#[tokio::test]
async fn adjust_spending_no_change() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.reserve_spending(key_id, 5_000).await.unwrap();

  // Actual == reserved -> no-op
  storage.adjust_spending(key_id, 5_000, 5_000).await.unwrap();

  let summary = storage.get_spending_summary(key_id).await.unwrap();
  assert_eq!(summary.used_microdollars, 5_000);
}
