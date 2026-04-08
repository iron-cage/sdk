#![allow(missing_docs)]

mod common;
use iron_token_manager::ProviderType;

#[tokio::test]
async fn create_and_get_key() {
  let (storage, _db) = common::create_test_provider_storage().await;

  let key_id = storage
    .create_key(
      ProviderType::OpenAI,
      "encrypted_data_base64",
      "nonce_base64",
      None,
      Some("Test key"),
      "user_001",
    )
    .await
    .unwrap();

  let record = storage.get_key(key_id).await.unwrap();
  assert_eq!(record.metadata.provider, ProviderType::OpenAI);
  assert_eq!(record.metadata.description, Some("Test key".to_string()));
  assert_eq!(record.metadata.user_id, "user_001");
  assert!(
    record.metadata.is_enabled,
    "Newly created key should be enabled by default"
  );
  assert_eq!(record.encrypted_api_key, "encrypted_data_base64");
  assert_eq!(record.encryption_nonce, "nonce_base64");
}

#[tokio::test]
#[allow(clippy::similar_names)]
async fn list_keys_by_user() {
  let (storage, _db) = common::create_test_provider_storage().await;

  storage
    .create_key(
      ProviderType::OpenAI,
      "enc1",
      "nonce1",
      None,
      Some("Key 1"),
      "user_001",
    )
    .await
    .unwrap();
  storage
    .create_key(
      ProviderType::Anthropic,
      "enc2",
      "nonce2",
      None,
      Some("Key 2"),
      "user_001",
    )
    .await
    .unwrap();
  storage
    .create_key(
      ProviderType::OpenAI,
      "enc3",
      "nonce3",
      None,
      Some("Key 3"),
      "user_002",
    )
    .await
    .unwrap();

  let user_a_keys = storage.list_keys("user_001").await.unwrap();
  assert_eq!(user_a_keys.len(), 2);

  let user_b_keys = storage.list_keys("user_002").await.unwrap();
  assert_eq!(user_b_keys.len(), 1);
}

#[tokio::test]
async fn enable_disable_key() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  // Initially enabled
  let meta = storage.get_key_metadata(key_id).await.unwrap();
  assert!(
    meta.is_enabled,
    "Newly created key should be enabled by default"
  );

  // Disable
  storage.update_key_fields(key_id, None, None, Some(false), None).await.unwrap();
  let meta = storage.get_key_metadata(key_id).await.unwrap();
  assert!(
    !meta.is_enabled,
    "Key should be disabled after update_key_fields with Some(false)"
  );

  // Enable again
  storage.update_key_fields(key_id, None, None, Some(true), None).await.unwrap();
  let meta = storage.get_key_metadata(key_id).await.unwrap();
  assert!(
    meta.is_enabled,
    "Key should be enabled after update_key_fields with Some(true)"
  );
}

#[tokio::test]
async fn update_balance() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  // Initially no balance
  let meta = storage.get_key_metadata(key_id).await.unwrap();
  assert!(
    meta.balance_cents.is_none(),
    "Newly created key should have no balance set"
  );

  // Update balance
  storage.update_balance(key_id, 10000).await.unwrap();
  let meta = storage.get_key_metadata(key_id).await.unwrap();
  assert_eq!(
    meta.balance_cents,
    Some(10000),
    "Balance should be updated to 10000 cents"
  );
  assert!(
    meta.balance_updated_at.is_some(),
    "Balance timestamp should be set after update"
  );
}

#[tokio::test]
async fn delete_key() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  // Delete
  storage.delete_key(key_id).await.unwrap();

  // Should fail to get
  let result = storage.get_key(key_id).await;
  assert!(result.is_err(), "Getting deleted key should fail");
}

  #[tokio::test]
async fn create_two_keys_same_provider_same_user() {
  let (storage, _db) = common::create_test_provider_storage().await;

  let id_a = storage
    .create_key(ProviderType::OpenAI, "enc_a", "nonce_a", None, Some("Key A"), "user_001")
    .await
    .unwrap();
  let id_b = storage
    .create_key(ProviderType::OpenAI, "enc_b", "nonce_b", None, Some("Key B"), "user_001")
    .await
    .unwrap();

  assert_ne!(id_a, id_b, "Two created keys must have distinct IDs");

  let record_a = storage.get_key(id_a).await.unwrap();
  let record_b = storage.get_key(id_b).await.unwrap();
  assert_eq!(record_a.metadata.user_id, "user_001");
  assert_eq!(record_b.metadata.user_id, "user_001");
  assert_eq!(record_a.metadata.provider, ProviderType::OpenAI);
  assert_eq!(record_b.metadata.provider, ProviderType::OpenAI);
}

#[tokio::test]
#[allow(clippy::similar_names)]
async fn cross_tenant_list_isolation() {
  let (storage, _db) = common::create_test_provider_storage().await;

  // user_001 has two OpenAI keys
  storage
    .create_key(ProviderType::OpenAI, "enc_a1", "nonce_a1", None, None, "user_001")
    .await
    .unwrap();
  storage
    .create_key(ProviderType::OpenAI, "enc_a2", "nonce_a2", None, None, "user_001")
    .await
    .unwrap();

  // user_002 has one OpenAI key
  storage
    .create_key(ProviderType::OpenAI, "enc_b1", "nonce_b1", None, None, "user_002")
    .await
    .unwrap();

  let user_a_keys = storage
    .get_keys_by_owner_and_provider("user_001", ProviderType::OpenAI)
    .await
    .unwrap();
  assert_eq!(
    user_a_keys.len(),
    2,
    "user_001 should see exactly their own 2 OpenAI keys"
  );

  let user_b_keys = storage
    .get_keys_by_owner_and_provider("user_002", ProviderType::OpenAI)
    .await
    .unwrap();
  assert_eq!(
    user_b_keys.len(),
    1,
    "user_002 should see exactly their own 1 OpenAI key"
  );

  // Cross-tenant isolation: no overlap between the two sets
  for id in &user_a_keys {
    assert!(
      !user_b_keys.contains(id),
      "user_002's key list must not contain user_001's key {id}"
    );
  }
}

#[tokio::test]
async fn project_assignment() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  // No assignment initially
  let assigned = storage.get_project_key("project_abc").await.unwrap();
  assert!(
    assigned.is_none(),
    "Project should have no key assigned initially"
  );

  // Assign
  storage
    .assign_to_project(key_id, "project_abc")
    .await
    .unwrap();
  let assigned = storage.get_project_key("project_abc").await.unwrap();
  assert_eq!(assigned, Some(key_id));

  // Get projects for key
  let projects = storage.get_key_projects(key_id).await.unwrap();
  assert_eq!(projects, vec!["project_abc".to_string()]);

  // Unassign
  storage
    .unassign_from_project(key_id, "project_abc")
    .await
    .unwrap();
  let assigned = storage.get_project_key("project_abc").await.unwrap();
  assert!(
    assigned.is_none(),
    "Project should have no key after unassignment"
  );
}

// ─────────────────────────────────────────────────────────────────
// Step 1 — Error paths
// ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_nonexistent_key_returns_error() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let result = storage.get_key(99999).await;
  assert!(matches!(result, Err(iron_token_manager::error::TokenError::NotFound)), "get_key for nonexistent ID must return Err(NotFound)");
}

#[tokio::test]
async fn get_nonexistent_key_metadata_returns_error() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let result = storage.get_key_metadata(99999).await;
  assert!(matches!(result, Err(iron_token_manager::error::TokenError::NotFound)), "get_key_metadata for nonexistent ID must return Err(NotFound)");
}

#[tokio::test]
async fn delete_nonexistent_key_returns_error() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let result = storage.delete_key(99999).await;
  assert!(matches!(result, Err(iron_token_manager::error::TokenError::NotFound)), "delete_key for nonexistent ID must return Err(NotFound)");
}

#[tokio::test]
async fn update_key_fields_nonexistent_returns_error() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let result = storage
    .update_key_fields(99999, Some(Some("desc")), None, None, None)
    .await;
  assert!(matches!(result, Err(iron_token_manager::error::TokenError::NotFound)), "update_key_fields for nonexistent ID must return Err(NotFound)");
}

// ─────────────────────────────────────────────────────────────────
// Step 2 — Project assignment granular tests
// ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn assign_and_retrieve_project_key() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.assign_to_project(key_id, "proj_x").await.unwrap();
  let assigned = storage.get_project_key("proj_x").await.unwrap();
  assert_eq!(assigned, Some(key_id), "Assigned key must be retrievable");
}

#[tokio::test]
async fn get_project_key_returns_none_when_unassigned() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let assigned = storage.get_project_key("proj_unassigned").await.unwrap();
  assert!(assigned.is_none(), "Project with no assignment must return None");
}

#[tokio::test]
async fn unassign_project_key() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.assign_to_project(key_id, "proj_y").await.unwrap();
  storage.unassign_from_project(key_id, "proj_y").await.unwrap();

  let assigned = storage.get_project_key("proj_y").await.unwrap();
  assert!(assigned.is_none(), "Project must have no key after unassignment");
}

#[tokio::test]
#[allow(clippy::similar_names)]
async fn get_key_projects_returns_all_assigned_projects() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.assign_to_project(key_id, "proj_a").await.unwrap();
  storage.assign_to_project(key_id, "proj_b").await.unwrap();

  let mut projects = storage.get_key_projects(key_id).await.unwrap();
  projects.sort();
  assert_eq!(projects, vec!["proj_a".to_string(), "proj_b".to_string()]);
}

#[tokio::test]
#[allow(clippy::similar_names)]
async fn get_project_key_deterministic_multiple_assignments() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_a = storage
    .create_key(ProviderType::OpenAI, "enc_a", "nonce_a", None, None, "user_001")
    .await
    .unwrap();
  let key_b = storage
    .create_key(ProviderType::OpenAI, "enc_b", "nonce_b", None, None, "user_001")
    .await
    .unwrap();

  storage.assign_to_project(key_a, "proj_det").await.unwrap();
  // Small sleep ensures key_b gets a later assigned_at timestamp
  // qqq: [Medium] timing-dependent — 2ms sleep may collide on slow CI; inject explicit assigned_at timestamps instead of relying on wall-clock ordering
  tokio::time::sleep(core::time::Duration::from_millis(2)).await;
  storage.assign_to_project(key_b, "proj_det").await.unwrap();

  let assigned = storage.get_project_key("proj_det").await.unwrap();
  assert_eq!(
    assigned,
    Some(key_b),
    "Most recently assigned key must be returned (ORDER BY assigned_at DESC)"
  );
}

// ─────────────────────────────────────────────────────────────────
// Step 3 — update_key_fields
// ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn update_key_fields_single_field() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, Some("Original"), "user_001")
    .await
    .unwrap();

  storage
    .update_key_fields(key_id, Some(Some("Updated")), None, None, None)
    .await
    .unwrap();

  let meta = storage.get_key_metadata(key_id).await.unwrap();
  assert_eq!(
    meta.description,
    Some("Updated".to_string()),
    "description must reflect new value"
  );
  assert!(meta.is_enabled, "is_enabled must not change when not specified");
}

#[tokio::test]
async fn update_key_fields_clear_nullable() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, Some("To be cleared"), "user_001")
    .await
    .unwrap();

  storage
    .update_key_fields(key_id, Some(None), None, None, None)
    .await
    .unwrap();

  let meta = storage.get_key_metadata(key_id).await.unwrap();
  assert!(meta.description.is_none(), "description must be NULL after passing Some(None)");
}

#[tokio::test]
async fn update_key_fields_multiple_fields_atomic() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, Some("Old"), "user_001")
    .await
    .unwrap();

  storage
    .update_key_fields(key_id, Some(Some("New")), None, Some(false), None)
    .await
    .unwrap();

  let meta = storage.get_key_metadata(key_id).await.unwrap();
  assert_eq!(meta.description, Some("New".to_string()));
  assert!(!meta.is_enabled, "is_enabled must be false after update");
}

#[tokio::test]
async fn update_key_fields_no_change() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, Some("Keep me"), "user_001")
    .await
    .unwrap();

  // All None — no changes requested; row must remain intact
  storage
    .update_key_fields(key_id, None, None, None, None)
    .await
    .unwrap();

  let meta = storage.get_key_metadata(key_id).await.unwrap();
  assert_eq!(meta.description, Some("Keep me".to_string()));
  assert!(meta.is_enabled);
}

// ─────────────────────────────────────────────────────────────────
// Step 4 — Spending controls
// ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn set_spending_cap_and_get_summary() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.set_spending_cap(key_id, Some(1_000_000)).await.unwrap();

  let summary = storage.get_spending_summary(key_id).await.unwrap();
  assert_eq!(summary.cap_microdollars, Some(1_000_000));
  assert_eq!(summary.used_microdollars, 0);
}

#[tokio::test]
async fn remove_spending_cap() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.set_spending_cap(key_id, Some(1_000_000)).await.unwrap();
  storage.set_spending_cap(key_id, None).await.unwrap();

  let summary = storage.get_spending_summary(key_id).await.unwrap();
  assert!(summary.cap_microdollars.is_none(), "Cap must be None after removal");
}

#[tokio::test]
async fn increment_spending_within_cap() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.set_spending_cap(key_id, Some(1_000_000)).await.unwrap();
  storage.increment_spending(key_id, 500_000).await.unwrap();

  let summary = storage.get_spending_summary(key_id).await.unwrap();
  assert_eq!(summary.used_microdollars, 500_000);
}

#[tokio::test]
async fn increment_spending_exceeds_cap() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.set_spending_cap(key_id, Some(1_000_000)).await.unwrap();
  let result = storage.increment_spending(key_id, 1_000_001).await;
  assert!(result.is_err(), "Incrementing past the cap must return Err");
}

#[tokio::test]
async fn increment_spending_no_cap() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  // No cap — any increment must succeed
  storage.increment_spending(key_id, 999_999_999).await.unwrap();

  let summary = storage.get_spending_summary(key_id).await.unwrap();
  assert_eq!(summary.used_microdollars, 999_999_999);
}

#[tokio::test]
async fn reserve_and_adjust_spending() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  // Reserve 100 microdollars; actual cost was only 80
  storage.reserve_spending(key_id, 100).await.unwrap();
  storage.adjust_spending(key_id, 100, 80).await.unwrap();

  let summary = storage.get_spending_summary(key_id).await.unwrap();
  assert_eq!(
    summary.used_microdollars,
    80,
    "After adjust, used must equal the actual cost (80)"
  );
}

#[tokio::test]
async fn adjust_spending_zero_delta_is_noop() {
  let (storage, _db) = common::create_test_provider_storage().await;
  let key_id = storage
    .create_key(ProviderType::OpenAI, "enc", "nonce", None, None, "user_001")
    .await
    .unwrap();

  storage.reserve_spending(key_id, 100).await.unwrap();
  // actual == reserved → delta is 0 → no DB write, no error
  storage.adjust_spending(key_id, 100, 100).await.unwrap();

  let summary = storage.get_spending_summary(key_id).await.unwrap();
  assert_eq!(summary.used_microdollars, 100, "Zero-delta adjust must leave used unchanged");
}
