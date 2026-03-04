//! Tests for provider key spending cap management.

#![allow(missing_docs)]

mod common;

use iron_token_manager::ProviderType;

#[tokio::test]
async fn no_cap_check_returns_true() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  // No cap set -> always within limit
  let within = storage.check_spending_cap(key_id).await.unwrap();
  assert!(within, "No cap set should return true");
}

#[tokio::test]
async fn set_cap_and_check_within_limit() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage
    .set_spending_cap(key_id, Some(10_000_000))
    .await
    .unwrap();

  let within = storage.check_spending_cap(key_id).await.unwrap();
  assert!(within, "Fresh key with cap should be within limit");
}

#[tokio::test]
async fn increment_then_check_still_within() {
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

  let within = storage.check_spending_cap(key_id).await.unwrap();
  assert!(within, "Used 5M of 10M cap should be within limit");
}

#[tokio::test]
async fn increment_to_exact_cap_still_within() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.set_spending_cap(key_id, Some(1_000)).await.unwrap();

  // Increment exactly to the cap
  storage.increment_spending(key_id, 1_000).await.unwrap();

  // used == cap -> check_spending_cap returns false (used < cap is the check)
  let within = storage.check_spending_cap(key_id).await.unwrap();
  assert!(
    !within,
    "used == cap should return false (strict less-than)"
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

  let within = storage.check_spending_cap(key_id).await.unwrap();
  assert!(within, "No cap -> always within limit");
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

  let within = storage.check_spending_cap(key_id).await.unwrap();
  assert!(within, "No cap -> within limit");
}

#[tokio::test]
async fn spending_summary_nonexistent_key_fails() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let result = storage.get_spending_summary(99999).await;
  assert!(result.is_err(), "Nonexistent key should error");
}

#[tokio::test]
async fn check_spending_cap_nonexistent_key_fails() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let result = storage.check_spending_cap(99999).await;
  assert!(result.is_err(), "Nonexistent key should error");
}
