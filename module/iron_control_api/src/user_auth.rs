//! User authentication and password verification
//!
//! Provides functionality for:
//! - Password hash verification using bcrypt
//! - User credential validation against database
//! - User lookup by username

use sqlx::{ Pool, Sqlite, FromRow, Row };

/// User record from database
#[ derive( Debug, Clone, FromRow ) ]
pub struct User
{
  pub id: i64,
  pub username: String,
  pub password_hash: String,
  pub role: String,
  pub email: String,
  pub is_active: bool,
}

/// Blacklisted token record from database
#[ derive( Debug, Clone, FromRow ) ]
pub struct BlacklistedToken
{
  pub jti: String,
  pub user_id: i64,
  pub blacklisted_at: i64,
  pub expires_at: i64,
}

/// Verify password against bcrypt hash
///
/// # Arguments
///
/// * `password` - Plain text password to verify
/// * `hash` - BCrypt hash to verify against
///
/// # Returns
///
/// `true` if password matches hash, `false` otherwise
pub fn verify_password( password: &str, hash: &str ) -> bool
{
  bcrypt::verify( password, hash ).unwrap_or( false )
}

/// Fetch user by username from database
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `username` - Username to look up
///
/// # Returns
///
/// - `Ok(Some(User))` if user found and active
/// - `Ok(None)` if user not found or inactive
/// - `Err` if database error
///
/// # Errors
///
/// Returns error if database query fails
pub async fn get_user_by_email(
  pool: &Pool< Sqlite >,
  email: &str,
) -> Result< Option< User >, sqlx::Error >
{
  let user = sqlx::query_as::<_, User>(
    r#"
    SELECT id, email, username, password_hash, role, is_active
    FROM users
    WHERE email = ? AND is_active = 1
    "#
  )
  .bind( email )
  .fetch_optional( pool )
  .await?;

  Ok( user )
}

/// Get user by ID from database
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `id` - User ID to look up
///
/// # Returns
///
/// - `Ok(Some(User))` if user found
/// - `Ok(None)` if user not found
/// - `Err` if database error
///
/// # Errors
///
/// Returns error if database query fails
pub async fn get_user_by_id(
  pool: &Pool< Sqlite >,
  id: &str,
) -> Result< Option< User >, sqlx::Error >
{
  let user = sqlx::query_as::<_, User>(
    r#"
    SELECT id, username, password_hash, role, email, is_active
    FROM users
    WHERE id = ?
    "#
  )
  .bind( id )
  .fetch_optional( pool )
  .await?;

  Ok( user )
}

/// Authenticate user with username and password
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `username` - Username to authenticate
/// * `password` - Plain text password
///
/// # Returns
///
/// - `Ok(Some(User))` if authentication successful
/// - `Ok(None)` if authentication failed (invalid credentials)
/// - `Err` if database error
///
/// # Errors
///
/// Returns error if database query fails
pub async fn authenticate_user(
  pool: &Pool< Sqlite >,
  email: &str,
  password: &str,
) -> Result< Option< User >, sqlx::Error >
{
  // Fetch user from database
  let user = match get_user_by_email( pool, email ).await?
  {
    Some( user ) => user,
    None => return Ok( None ), // User not found
  };

  // Verify password
  if verify_password( password, &user.password_hash )
  {
    Ok( Some( user ) )
  }
  else
  {
    Ok( None ) // Invalid password
  }
}

/// Add user authorization token to blacklist
/// 
/// # Arguments
/// 
/// * `pool` - Database connection pool
/// * `token` - User authorization token to blacklist
/// 
/// # Returns
/// 
/// - `Ok(())` if token added to blacklist successfully
/// - `Err` if database error
/// 
/// # Errors
/// 
/// Returns error if database query fails
pub async fn add_token_to_blacklist(
  pool: &Pool< Sqlite >,
  token: &str,
  user_id: i64,
  expires_at: chrono::DateTime< chrono::Utc >,
) -> Result< (), sqlx::Error >
{
  let blacklisted_at = chrono::Utc::now().timestamp();
  let expires_at = expires_at.timestamp();

  sqlx::query(
    r#"
    INSERT INTO blacklist (jti, user_id, blacklisted_at, expires_at) VALUES (?, ?, ?, ?)
    "#
  )
  .bind( token )
  .bind( user_id )
  .bind( blacklisted_at )
  .bind( expires_at )
  .execute( pool )
  .await?;

  Ok( () )
}

/// Check if user authorization token is blacklisted
/// 
/// # Arguments
/// 
/// * `pool` - Database connection pool
/// * `token` - User authorization token to check
/// 
/// # Returns
/// 
/// - `Ok(true)` if token is blacklisted
/// - `Ok(false)` if token is not blacklisted
/// - `Err` if database error
/// 
/// # Errors
/// 
/// Returns error if database query fails
pub async fn get_blacklisted_token(
  pool: &Pool< Sqlite >,
  token: &str,
) -> Result< Option< BlacklistedToken >, sqlx::Error >
{
  let blacklisted = sqlx::query_as(
    r#"
    SELECT jti, user_id, blacklisted_at, expires_at FROM blacklist WHERE jti = ?
    "#
  )
  .bind( token )
  .fetch_optional( pool )
  .await?;

  Ok( blacklisted )
}

#[ cfg( test ) ]
mod tests
{
  use sqlx::SqlitePool;

  use super::*;

  async fn create_test_db() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect(":memory:").await?;

    // Run migration 003 (users table)
    let migration_003 =
      include_str!("../../iron_token_manager/migrations/003_create_users_table.sql");
    sqlx::raw_sql(migration_003).execute(&pool).await?;

    // Run migration 006 (user audit log)
    let migration_006 =
      include_str!("../../iron_token_manager/migrations/006_create_user_audit_log.sql");
    sqlx::raw_sql(migration_006).execute(&pool).await?;

    Ok(pool)
  }

  #[ test ]
  fn test_verify_password_valid()
  {
    // Hash for "testpass"
    let hash = "$2b$12$zZOfQakwkynHa0mBVlSvQ.rmzFZxkkN6OelZE/bLDCY1whIW.IWf2";
    assert!( verify_password( "testpass", hash ) );
  }

  #[ test ]
  fn test_verify_password_invalid()
  {
    // Hash for "testpass"
    let hash = "$2b$12$zZOfQakwkynHa0mBVlSvQ.rmzFZxkkN6OelZE/bLDCY1whIW.IWf2";
    assert!( !verify_password( "wrongpass", hash ) );
  }

  #[ test ]
  fn test_verify_password_malformed_hash()
  {
    assert!( !verify_password( "anypass", "not-a-valid-hash" ) );
  }


#[tokio::test]
  async fn test_add_token_to_blacklist_success() {
    let pool = create_test_db().await.expect("Failed to create test database");

    let jti = "test_token_jti_123";
    let user_id = 1;
    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);

    // Add token to blacklist
    let result = add_token_to_blacklist(&pool, jti, user_id, expires_at).await;
    assert!(result.is_ok(), "Failed to add token to blacklist");

    // Verify token is in blacklist
    let blacklisted: Option<(String, i64, i64, i64)> = sqlx::query_as(
      "SELECT jti, user_id, blacklisted_at, expires_at FROM blacklist WHERE jti = ?"
    )
    .bind(jti)
    .fetch_optional(&pool)
    .await
    .expect("Failed to query blacklist");

    assert!(blacklisted.is_some(), "Token should be in blacklist");
    let (db_jti, db_user_id, db_blacklisted_at, db_expires_at) = blacklisted.unwrap();
    
    assert_eq!(db_jti, jti);
    assert_eq!(db_user_id, user_id);
    assert!(db_blacklisted_at > 0, "Blacklisted timestamp should be set");
    assert!(db_expires_at > 0, "Expiration timestamp should be set");
  }

  #[tokio::test]
  async fn test_add_token_to_blacklist_duplicate() {
    let pool = create_test_db().await.expect("Failed to create test database");

    let jti = "duplicate_token_jti";
    let user_id = 1;
    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);

    // Add token to blacklist first time
    let result1 = add_token_to_blacklist(&pool, jti, user_id, expires_at).await;
    assert!(result1.is_ok(), "First insert should succeed");

    // Try to add same token again (should fail due to unique constraint)
    let result2 = add_token_to_blacklist(&pool, jti, user_id, expires_at).await;
    assert!(result2.is_err(), "Duplicate insert should fail");
  }

  #[tokio::test]
  async fn test_add_multiple_tokens_to_blacklist() {
    let pool = create_test_db().await.expect("Failed to create test database");

    let user_id = 1;
    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);

    // Add multiple tokens
    for i in 0..5 {
      let jti = format!("token_{}", i);
      let result = add_token_to_blacklist(&pool, &jti, user_id, expires_at).await;
      assert!(result.is_ok(), "Failed to add token {} to blacklist", i);
    }

    // Verify all tokens are in blacklist
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM blacklist WHERE user_id = ?")
      .bind(user_id)
      .fetch_one(&pool)
      .await
      .expect("Failed to count blacklisted tokens");

    assert_eq!(count.0, 5, "Should have 5 tokens in blacklist");
  }

  #[tokio::test]
  async fn test_blacklist_token_expiration_timestamp() {
    let pool = create_test_db().await.expect("Failed to create test database");

    let jti = "expiry_test_token";
    let user_id = 1;
    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
    let expected_expiry = expires_at.timestamp();

    // Add token to blacklist
    add_token_to_blacklist(&pool, jti, user_id, expires_at)
      .await
      .expect("Failed to add token to blacklist");

    // Verify expiration timestamp is correct
    let (db_expires_at,): (i64,) = sqlx::query_as(
      "SELECT expires_at FROM blacklist WHERE jti = ?"
    )
    .bind(jti)
    .fetch_one(&pool)
    .await
    .expect("Failed to query blacklist");

    assert_eq!(db_expires_at, expected_expiry, "Expiration timestamp should match");
  }
}