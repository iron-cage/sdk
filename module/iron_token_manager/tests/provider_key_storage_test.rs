use iron_token_manager::*;

  use super::*;

  #[ tokio::test ]
  async fn create_and_get_key()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();

    let key_id = storage.create_key(
      ProviderType::OpenAI,
      "encrypted_data_base64",
      "nonce_base64",
      None,
      Some( "Test key" ),
      "user_123",
    ).await.unwrap();

    let record = storage.get_key( key_id ).await.unwrap();
    assert_eq!( record.metadata.provider, ProviderType::OpenAI );
    assert_eq!( record.metadata.description, Some( "Test key".to_string() ) );
    assert_eq!( record.metadata.user_id, "user_123" );
    assert!( record.metadata.is_enabled, "Newly created key should be enabled by default" );
    assert_eq!( record.encrypted_api_key, "encrypted_data_base64" );
    assert_eq!( record.encryption_nonce, "nonce_base64" );
  }

  #[ tokio::test ]
  #[ allow( clippy::similar_names ) ]
  async fn list_keys_by_user()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();

    storage.create_key( ProviderType::OpenAI, "enc1", "nonce1", None, Some( "Key 1" ), "user_a" ).await.unwrap();
    storage.create_key( ProviderType::Anthropic, "enc2", "nonce2", None, Some( "Key 2" ), "user_a" ).await.unwrap();
    storage.create_key( ProviderType::OpenAI, "enc3", "nonce3", None, Some( "Key 3" ), "user_b" ).await.unwrap();

    let user_a_keys = storage.list_keys( "user_a" ).await.unwrap();
    assert_eq!( user_a_keys.len(), 2 );

    let user_b_keys = storage.list_keys( "user_b" ).await.unwrap();
    assert_eq!( user_b_keys.len(), 1 );
  }

  #[ tokio::test ]
  async fn enable_disable_key()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();
    let key_id = storage.create_key( ProviderType::OpenAI, "enc", "nonce", None, None, "user" ).await.unwrap();

    // Initially enabled
    let meta = storage.get_key_metadata( key_id ).await.unwrap();
    assert!( meta.is_enabled, "Newly created key should be enabled by default" );

    // Disable
    storage.set_enabled( key_id, false ).await.unwrap();
    let meta = storage.get_key_metadata( key_id ).await.unwrap();
    assert!( !meta.is_enabled, "Key should be disabled after set_enabled(false)" );

    // Enable again
    storage.set_enabled( key_id, true ).await.unwrap();
    let meta = storage.get_key_metadata( key_id ).await.unwrap();
    assert!( meta.is_enabled, "Key should be enabled after set_enabled(true)" );
  }

  #[ tokio::test ]
  async fn update_balance()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();
    let key_id = storage.create_key( ProviderType::OpenAI, "enc", "nonce", None, None, "user" ).await.unwrap();

    // Initially no balance
    let meta = storage.get_key_metadata( key_id ).await.unwrap();
    assert!( meta.balance_cents.is_none() );

    // Update balance
    storage.update_balance( key_id, 10000 ).await.unwrap();
    let meta = storage.get_key_metadata( key_id ).await.unwrap();
    assert_eq!( meta.balance_cents, Some( 10000 ) );
    assert!( meta.balance_updated_at.is_some() );
  }

  #[ tokio::test ]
  async fn delete_key()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();
    let key_id = storage.create_key( ProviderType::OpenAI, "enc", "nonce", None, None, "user" ).await.unwrap();

    // Delete
    storage.delete_key( key_id ).await.unwrap();

    // Should fail to get
    let result = storage.get_key( key_id ).await;
    assert!( result.is_err() );
  }

  #[ tokio::test ]
  async fn project_assignment()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();
    let key_id = storage.create_key( ProviderType::OpenAI, "enc", "nonce", None, None, "user" ).await.unwrap();

    // No assignment initially
    let assigned = storage.get_project_key( "project_abc" ).await.unwrap();
    assert!( assigned.is_none() );

    // Assign
    storage.assign_to_project( key_id, "project_abc" ).await.unwrap();
    let assigned = storage.get_project_key( "project_abc" ).await.unwrap();
    assert_eq!( assigned, Some( key_id ) );

    // Get projects for key
    let projects = storage.get_key_projects( key_id ).await.unwrap();
    assert_eq!( projects, vec![ "project_abc".to_string() ] );

    // Unassign
    storage.unassign_from_project( key_id, "project_abc" ).await.unwrap();
    let assigned = storage.get_project_key( "project_abc" ).await.unwrap();
    assert!( assigned.is_none() );
  }
