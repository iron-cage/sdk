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
  pub id: i64,
  pub username: String,
  pub password_hash: String,
  pub role: String,
  pub is_active: bool,
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
pub async fn get_user_by_username(
  pool: &Pool< Sqlite >,
  username: &str,
) -> Result< Option< User >, sqlx::Error >
{
  let user = sqlx::query_as::<_, User>(
    r#"
    SELECT id, username, password_hash, role, is_active
    FROM users
    WHERE username = ? AND is_active = 1
    "#
  )
  .bind( username )
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
  username: &str,
  password: &str,
) -> Result< Option< User >, sqlx::Error >
{
  // Fetch user from database
  let user = match get_user_by_username( pool, username ).await?
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

#[ cfg( test ) ]
mod tests
{
  use super::*;

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
}
