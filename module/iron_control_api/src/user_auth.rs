//! User authentication and password verification
//!
//! Provides functionality for:
//! - Password hash verification using bcrypt
//! - User credential validation against database
//! - User lookup by username

use sqlx::{ Pool, Sqlite, FromRow };

/// User record from database
#[ derive( Debug, Clone, FromRow ) ]
pub struct User
{
  pub id: String,
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
  pub user_id: String,
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
  user_id: &str,
  expires_at: chrono::DateTime< chrono::Utc >,
) -> Result< (), sqlx::Error >
{
  let blacklisted_at = chrono::Utc::now().timestamp();
  let expires_at = expires_at.timestamp();

  sqlx::query(
    r#"
    INSERT INTO token_blacklist (jti, user_id, blacklisted_at, expires_at) VALUES (?, ?, ?, ?)
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
    SELECT jti, user_id, blacklisted_at, expires_at FROM token_blacklist WHERE jti = ?
    "#
  )
  .bind( token )
  .fetch_optional( pool )
  .await?;

  Ok( blacklisted )
}

