//! Infrastructure tests for iron_test_db
//!
//! Validates that test database infrastructure works correctly.

use iron_test_db::{ TestDatabaseBuilder, StorageMode, Migration, MigrationRegistry };

#[ tokio::test ]
async fn test_in_memory_database_creation()
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create in-memory database" );

  // Verify pool is functional
  let result: i64 = sqlx::query_scalar( "SELECT 1" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to query database" );

  assert_eq!( result, 1, "LOUD FAILURE: Query should return 1" );
}

#[ tokio::test ]
async fn test_temp_file_database_creation()
{
  let db = TestDatabaseBuilder::new()
    .temp_file()
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create temp file database" );

  // Verify pool is functional
  let result: i64 = sqlx::query_scalar( "SELECT 1" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to query database" );

  assert_eq!( result, 1, "LOUD FAILURE: Query should return 1" );

  // Verify it's file-based (not in-memory)
  assert!(
    matches!( db.storage_mode(), StorageMode::TempFile ),
    "LOUD FAILURE: Storage mode should be TempFile"
  );
}

#[ tokio::test ]
async fn test_shared_memory_database()
{
  let db1 = TestDatabaseBuilder::new()
    .shared_memory( "test_shared" )
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create shared memory database" );

  // Create table in first connection
  sqlx::query(
    "CREATE TABLE test_table ( id INTEGER PRIMARY KEY, value TEXT )"
  )
  .execute( db1.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to create table" );

  // Insert data
  sqlx::query( "INSERT INTO test_table ( id, value ) VALUES ( 1, 'shared' )" )
    .execute( db1.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to insert data" );

  // Create second connection to same shared database
  let db2 = TestDatabaseBuilder::new()
    .shared_memory( "test_shared" )
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create second shared connection" );

  // Verify data is visible in second connection
  let value: String = sqlx::query_scalar( "SELECT value FROM test_table WHERE id = 1" )
    .fetch_one( db2.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to query from second connection" );

  assert_eq!(
    value, "shared",
    "LOUD FAILURE: Data should be shared between connections"
  );
}

#[ tokio::test ]
async fn test_database_isolation_between_tests()
{
  // First database
  let db1 = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create first database" );

  sqlx::query( "CREATE TABLE test_table ( id INTEGER PRIMARY KEY )" )
    .execute( db1.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to create table in first database" );

  sqlx::query( "INSERT INTO test_table ( id ) VALUES ( 1 )" )
    .execute( db1.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to insert into first database" );

  drop( db1 );  // Explicitly drop first database

  // Second database (should be completely isolated)
  let db2 = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create second database" );

  // Table should NOT exist in second database
  let table_exists: bool = sqlx::query_scalar(
    "SELECT COUNT(*) > 0 FROM sqlite_master
     WHERE type='table' AND name='test_table'"
  )
  .fetch_one( db2.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to query schema" );

  assert!(
    !table_exists,
    "LOUD FAILURE: Second database should be isolated from first"
  );
}

#[ tokio::test ]
async fn test_foreign_keys_enabled()
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create database" );

  // Create parent and child tables
  sqlx::query(
    "CREATE TABLE parent ( id INTEGER PRIMARY KEY );
     CREATE TABLE child (
       id INTEGER PRIMARY KEY,
       parent_id INTEGER NOT NULL,
       FOREIGN KEY ( parent_id ) REFERENCES parent( id )
     )"
  )
  .execute( db.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to create tables" );

  // Attempt to insert child without parent (should fail if FK enabled)
  let result = sqlx::query( "INSERT INTO child ( id, parent_id ) VALUES ( 1, 999 )" )
    .execute( db.pool() )
    .await;

  assert!(
    result.is_err(),
    "LOUD FAILURE: Foreign key constraint should prevent orphaned child"
  );
}

#[ tokio::test ]
async fn test_migration_registry_basic()
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create database" );

  let registry = MigrationRegistry::new()
    .register( Migration {
      version: 1,
      name: "create_users".to_string(),
      sql: "CREATE TABLE users ( id INTEGER PRIMARY KEY, name TEXT NOT NULL )",
    } )
    .register( Migration {
      version: 2,
      name: "create_projects".to_string(),
      sql: "CREATE TABLE projects ( id INTEGER PRIMARY KEY, name TEXT NOT NULL )",
    } );

  // Apply migrations
  registry.apply_all( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to apply migrations" );

  // Verify tables exist
  let table_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM sqlite_master
     WHERE type='table'
     AND name IN ( 'users', 'projects' )"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to count tables" );

  assert_eq!(
    table_count, 2,
    "LOUD FAILURE: Both tables should be created"
  );

  // Verify current version
  let version = registry.current_version( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to get current version" );

  assert_eq!(
    version,
    Some( 2 ),
    "LOUD FAILURE: Current version should be 2"
  );
}

#[ tokio::test ]
async fn test_migration_registry_idempotent()
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create database" );

  let registry = MigrationRegistry::new()
    .register( Migration {
      version: 1,
      name: "create_users".to_string(),
      sql: "CREATE TABLE users ( id INTEGER PRIMARY KEY, name TEXT NOT NULL )",
    } );

  // Apply migrations first time
  registry.apply_all( db.pool() )
    .await
    .expect( "LOUD FAILURE: First migration run should succeed" );

  // Apply migrations second time (should be no-op)
  registry.apply_all( db.pool() )
    .await
    .expect( "LOUD FAILURE: Second migration run should succeed (idempotent)" );

  // Verify table only exists once (not duplicated)
  let table_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM sqlite_master
     WHERE type='table' AND name='users'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to count tables" );

  assert_eq!(
    table_count, 1,
    "LOUD FAILURE: Table should only exist once"
  );
}

#[ tokio::test ]
async fn test_wipe_all_tables_simple()
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create database" );

  // Create tables and insert data
  sqlx::query(
    "CREATE TABLE users ( id INTEGER PRIMARY KEY, name TEXT );
     CREATE TABLE projects ( id INTEGER PRIMARY KEY, name TEXT );
     INSERT INTO users ( id, name ) VALUES ( 1, 'Alice' );
     INSERT INTO projects ( id, name ) VALUES ( 1, 'Project A' )"
  )
  .execute( db.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to create tables and insert data" );

  // Verify data exists
  let user_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM users" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to count users" );

  assert_eq!( user_count, 1, "LOUD FAILURE: Should have 1 user" );

  // Wipe all tables
  db.wipe()
    .await
    .expect( "LOUD FAILURE: Failed to wipe tables" );

  // Verify tables are empty
  let user_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM users" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to count users after wipe" );

  let project_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM projects" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to count projects after wipe" );

  assert_eq!( user_count, 0, "LOUD FAILURE: Users table should be empty" );
  assert_eq!( project_count, 0, "LOUD FAILURE: Projects table should be empty" );
}

#[ tokio::test ]
async fn test_wipe_respects_foreign_keys()
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create database" );

  // Create parent-child relationship
  sqlx::query(
    "CREATE TABLE parent ( id INTEGER PRIMARY KEY, name TEXT );
     CREATE TABLE child (
       id INTEGER PRIMARY KEY,
       parent_id INTEGER NOT NULL,
       name TEXT,
       FOREIGN KEY ( parent_id ) REFERENCES parent( id ) ON DELETE CASCADE
     );
     INSERT INTO parent ( id, name ) VALUES ( 1, 'Parent' );
     INSERT INTO child ( id, parent_id, name ) VALUES ( 1, 1, 'Child' )"
  )
  .execute( db.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to create tables with FK relationship" );

  // Wipe should not violate foreign keys
  db.wipe()
    .await
    .expect( "LOUD FAILURE: Wipe should respect foreign key order" );

  // Verify both tables are empty
  let parent_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM parent" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to count parent" );

  let child_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM child" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to count child" );

  assert_eq!( parent_count, 0, "LOUD FAILURE: Parent table should be empty" );
  assert_eq!( child_count, 0, "LOUD FAILURE: Child table should be empty" );
}

#[ tokio::test ]
async fn test_pool_size_configuration()
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .pool_size( 10 )
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create database with custom pool size" );

  // Verify pool is functional
  let result: i64 = sqlx::query_scalar( "SELECT 1" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to query database" );

  assert_eq!( result, 1, "LOUD FAILURE: Query should return 1" );
}
