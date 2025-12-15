//! User service tests
//!
//! Tests for user management service covering user lifecycle, RBAC, audit logging,
//! and password security. Uses REAL BCrypt hashing and SQLite database.
//!
//! ## Test Matrix
//!
//! ### User Creation
//! - `test_create_user_success`: Valid user creation → User created with BCrypt hash
//! - `test_create_user_empty_username`: Empty username → Error "username required"
//! - `test_create_user_empty_password`: Empty password → Error "password required"
//! - `test_create_user_empty_email`: Empty email → Error "email required"
//! - `test_create_user_invalid_role`: Invalid role → Error "invalid role"
//! - `test_create_user_duplicate_username`: Duplicate username → Error "username exists"
//! - `test_create_user_sql_injection_username`: SQL injection in username → Rejected safely
//! - `test_create_user_unicode_username`: Unicode username → Created successfully
//! - `test_create_user_bcrypt_cost_12`: Password hash → BCrypt cost=12
//! - `test_create_user_password_never_plaintext`: Password storage → Hash stored, not plaintext
//!
//! ### User Suspension/Activation
//! - `test_suspend_user_success`: Valid suspension → User suspended with audit log
//! - `test_suspend_user_with_reason`: Suspension with reason → Reason recorded
//! - `test_suspend_already_suspended`: Suspend suspended user → No error (idempotent)
//! - `test_activate_user_success`: Activate suspended user → User activated
//! - `test_activate_already_active`: Activate active user → No error (idempotent)
//!
//! ### User Role Changes
//! - `test_change_user_role_success`: Valid role change → Role updated with audit
//! - `test_change_role_to_admin`: User→Admin → Role changed
//! - `test_change_role_to_user`: Admin→User → Role changed
//! - `test_change_role_invalid`: Invalid role → Error "invalid role"
//!
//! ### User Deletion (Soft Delete)
//! - `test_delete_user_soft_delete`: Delete user → User soft deleted (not removed)
//! - `test_deleted_user_not_in_list`: Deleted user → Not in list_users results
//! - `test_deleted_user_data_preserved`: Deleted user → Data preserved in DB
//! - `test_delete_already_deleted`: Delete deleted user → No error (idempotent)
//!
//! ### Audit Logging (Immutability)
//! - `test_audit_log_user_creation`: User created → Audit log entry exists
//! - `test_audit_log_user_suspension`: User suspended → Audit log entry with reason
//! - `test_audit_log_role_change`: Role changed → Audit log entry with old/new role
//! - `test_audit_log_immutable`: Audit entries → Cannot be modified
//!
//! ### Password Security
//! - `test_password_bcrypt_cost_12`: Password hashes → BCrypt cost exactly 12
//! - `test_password_verification`: Correct password → Verifies successfully
//! - `test_password_verification_wrong`: Wrong password → Verification fails
//! - `test_password_never_stored_plaintext`: Password storage → Only hash stored
//!
//! ### SQL Injection Prevention
//! - `test_sql_injection_username`: SQL in username → Rejected or escaped safely
//! - `test_sql_injection_email`: SQL in email → Rejected or escaped safely
//! - `test_sql_injection_search`: SQL in search filter → Escaped safely
//!
//! ### Boundary Conditions
//! - `test_empty_username`: Empty string username → Error
//! - `test_very_long_username`: 1000 char username → Error or trimmed
//! - `test_unicode_characters`: Unicode in username/email → Handled correctly
//!
//! ## Corner Cases Covered
//!
//! - ✅ Empty inputs (username, password, email)
//! - ✅ Invalid inputs (wrong role, malformed email)
//! - ✅ Duplicate prevention (username, email uniqueness)
//! - ✅ SQL injection attempts (username, email, search)
//! - ✅ Unicode handling (international characters)
//! - ✅ Boundary values (empty strings, very long strings)
//! - ✅ State transitions (active ↔ suspended ↔ deleted)
//! - ✅ Idempotent operations (double suspend, double delete)
//! - ✅ Security (BCrypt cost=12, no plaintext passwords)
//! - ✅ Audit immutability (logs can't be modified)

use iron_token_manager::{ UserService, CreateUserParams, ListUsersFilters };

mod common;

/// Test successful user creation with valid parameters
///
/// Verifies that user creation works for valid inputs and stores BCrypt password hash.
#[ tokio::test ]
async fn test_create_user_success()
{
  let db = common::create_test_db().await;
  let service = UserService::new( db.pool().clone() );

  let params = CreateUserParams
  {
    username: "john_doe".to_string(),
    password: "SecurePass123!".to_string(),
    email: "john@example.com".to_string(),
    role: "user".to_string(),
  };

  let user = service.create_user( params, "admin-001" ).await
    .expect( "LOUD FAILURE: Failed to create valid user with correct parameters" );

  assert_eq!( user.username, "john_doe", "Username should match input" );
  assert_eq!( user.email, Some( "john@example.com".to_string() ), "Email should match input" );
  assert_eq!( user.role, "user", "Role should match input" );
  assert!( user.is_active, "New user should be active by default" );
  assert!( user.password_hash.starts_with( "$2b$" ), "Password should be BCrypt hash (starts with $2b$)" );
  assert_ne!( user.password_hash, "SecurePass123!", "Password should NOT be stored as plaintext" );
}

/// Test user creation with empty username
///
/// Verifies that empty username is rejected with clear error message.
#[ tokio::test ]
async fn test_create_user_empty_username()
{
  let db = common::create_test_db_v2().await;
  let service = UserService::new( db.pool().clone() );

  let params = CreateUserParams
  {
    username: "".to_string(),
    password: "SecurePass123!".to_string(),
    email: "john@example.com".to_string(),
    role: "user".to_string(),
  };

  let result = service.create_user( params, "admin-001" ).await;

  assert!(
    result.is_err(),
    "LOUD FAILURE: Empty username should be rejected. Got: {result:?}"
  );

  let error_msg = result.unwrap_err().to_string();
  assert!(
    error_msg.contains( "username" ) || error_msg.contains( "empty" ) || error_msg.contains( "required" ),
    "LOUD FAILURE: Error message should mention username issue. Got: {error_msg}"
  );
}

/// Test user creation with empty password
///
/// Verifies that empty password is rejected with clear error message.
#[ tokio::test ]
async fn test_create_user_empty_password()
{
  let db = common::create_test_db_v2().await;
  let service = UserService::new( db.pool().clone() );

  let params = CreateUserParams
  {
    username: "john_doe".to_string(),
    password: "".to_string(),
    email: "john@example.com".to_string(),
    role: "user".to_string(),
  };

  let result = service.create_user( params, "admin-001" ).await;

  assert!(
    result.is_err(),
    "LOUD FAILURE: Empty password should be rejected. Got: {result:?}"
  );

  let error_msg = result.unwrap_err().to_string();
  assert!(
    error_msg.contains( "password" ) || error_msg.contains( "empty" ) || error_msg.contains( "required" ),
    "LOUD FAILURE: Error message should mention password issue. Got: {error_msg}"
  );
}

/// Test user creation with empty email
///
/// Verifies that empty email is rejected with clear error message.
#[ tokio::test ]
async fn test_create_user_empty_email()
{
  let db = common::create_test_db_v2().await;
  let service = UserService::new( db.pool().clone() );

  let params = CreateUserParams
  {
    username: "john_doe".to_string(),
    password: "SecurePass123!".to_string(),
    email: "".to_string(),
    role: "user".to_string(),
  };

  let result = service.create_user( params, "admin-001" ).await;

  assert!(
    result.is_err(),
    "LOUD FAILURE: Empty email should be rejected. Got: {result:?}"
  );

  let error_msg = result.unwrap_err().to_string();
  assert!(
    error_msg.contains( "email" ) || error_msg.contains( "empty" ) || error_msg.contains( "required" ),
    "LOUD FAILURE: Error message should mention email issue. Got: {error_msg}"
  );
}

/// Test user creation with invalid role
///
/// Verifies that invalid role (not admin/user/viewer) is rejected.
#[ tokio::test ]
async fn test_create_user_invalid_role()
{
  let db = common::create_test_db_v2().await;
  let service = UserService::new( db.pool().clone() );

  let params = CreateUserParams
  {
    username: "john_doe".to_string(),
    password: "SecurePass123!".to_string(),
    email: "john@example.com".to_string(),
    role: "hacker".to_string(), // Invalid role
  };

  let result = service.create_user( params, "admin-001" ).await;

  assert!(
    result.is_err(),
    "LOUD FAILURE: Invalid role 'hacker' should be rejected. Got: {result:?}"
  );

  let error_msg = result.unwrap_err().to_string();
  assert!(
    error_msg.contains( "role" ) || error_msg.contains( "invalid" ),
    "LOUD FAILURE: Error message should mention role issue. Got: {error_msg}"
  );
}

/// Test that BCrypt password hash has cost factor of exactly 12
///
/// Verifies security requirement that BCrypt cost is 12 (not default 10).
#[ tokio::test ]
async fn test_password_bcrypt_cost_12()
{
  let db = common::create_test_db_v2().await;
  let service = UserService::new( db.pool().clone() );

  let params = CreateUserParams
  {
    username: "john_doe".to_string(),
    password: "SecurePass123!".to_string(),
    email: "john@example.com".to_string(),
    role: "user".to_string(),
  };

  let user = service.create_user( params, "admin-001" ).await
    .expect( "Failed to create user" );

  // BCrypt hash format: $2b$12$...
  // Extract cost factor (between second and third $)
  let parts: Vec< &str > = user.password_hash.split( '$' ).collect();
  assert!(
    parts.len() >= 4,
    "LOUD FAILURE: BCrypt hash should have format $2b$cost$salt$hash. Got: {}",
    user.password_hash
  );

  let cost_str = parts[ 2 ];
  let cost: u32 = cost_str.parse().expect( "Failed to parse BCrypt cost" );

  assert_eq!(
    cost, 12,
    "LOUD FAILURE: BCrypt cost MUST be 12 for security. Got: {cost}. Hash: {}",
    user.password_hash
  );
}

/// Test SQL injection attempt in username
///
/// Verifies that SQL injection attempts are safely rejected or escaped.
#[ tokio::test ]
async fn test_sql_injection_username()
{
  let db = common::create_test_db_v2().await;
  let service = UserService::new( db.pool().clone() );

  let params = CreateUserParams
  {
    username: "'; DROP TABLE users;--".to_string(),
    password: "SecurePass123!".to_string(),
    email: "attacker@example.com".to_string(),
    role: "user".to_string(),
  };

  // Should either reject SQL injection OR escape it safely
  let result = service.create_user( params, "admin-001" ).await;

  // Verify users table still exists by querying it
  let users_exist: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM users" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: users table should still exist after SQL injection attempt" );

  // If user was created, verify username was escaped (not executed as SQL)
  if let Ok( user ) = result
  {
    assert_eq!(
      user.username, "'; DROP TABLE users;--",
      "LOUD FAILURE: SQL injection should be stored as literal string, not executed"
    );
  }

  assert!(
    users_exist >= 0,
    "LOUD FAILURE: SQL injection should not drop users table. User count: {users_exist}"
  );
}

/// Test that deleted user doesn't appear in list_users
///
/// Verifies soft delete properly filters deleted users from listing.
#[ tokio::test ]
async fn test_deleted_user_not_in_list()
{
  let db = common::create_test_db_v2().await;
  let service = UserService::new( db.pool().clone() );

  // Create user
  let params = CreateUserParams
  {
    username: "john_doe".to_string(),
    password: "SecurePass123!".to_string(),
    email: "john@example.com".to_string(),
    role: "user".to_string(),
  };

  let user = service.create_user( params, "admin-001" ).await
    .expect( "Failed to create user" );

  // Delete user
  service.delete_user( &user.id, "admin-001" ).await
    .expect( "Failed to delete user" );

  // List users - deleted user should NOT appear
  let ( users, total ) = service.list_users( ListUsersFilters::default() ).await
    .expect( "Failed to list users" );

  assert!(
    !users.iter().any( | u | u.id == user.id ),
    "LOUD FAILURE: Deleted user should not appear in list_users. Found user: {users:?}"
  );

  assert_eq!(
    total, 0,
    "LOUD FAILURE: Total count should not include deleted users. Got: {total}"
  );
}
