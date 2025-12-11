//! User management service
//!
//! Provides admin-only operations for user lifecycle management: create, suspend,
//! activate, delete, role changes, and password resets. All operations are audited.

use core::fmt::Write as _;
use sqlx::{ SqlitePool, Row };
use crate::error::Result;
use tracing::error;

/// User data returned from database
#[ derive( Debug, Clone ) ]
pub struct User
{
  /// Database ID
  pub id: String,
  /// Username (unique)
  pub username: String,
  /// Email address (unique, optional)
  pub email: Option< String >,
  /// `BCrypt` password hash
  pub password_hash: String,
  /// User role (user, `super_user`, admin)
  pub role: String,
  /// Account active status
  pub is_active: bool,
  /// Creation timestamp (milliseconds since epoch)
  pub created_at: i64,
  /// Last login timestamp (milliseconds since epoch)
  pub last_login: Option< i64 >,
  /// Suspension timestamp (milliseconds since epoch)
  pub suspended_at: Option< i64 >,
  /// Admin who suspended this user
  pub suspended_by: Option< String >,
  /// Deletion timestamp (milliseconds since epoch, soft delete)
  pub deleted_at: Option< i64 >,
  /// Admin who deleted this user
  pub deleted_by: Option< String >,
  /// Force password change on next login
  pub force_password_change: bool,
}

/// User creation parameters
#[ derive( Debug, Clone ) ]
pub struct CreateUserParams
{
  /// Username (3-50 chars, alphanumeric + underscore)
  pub username: String,
  /// Password (will be hashed with `BCrypt`)
  pub password: String,
  /// Email address (must be unique)
  pub email: String,
  /// Role (user, `super_user`, admin)
  pub role: String,
}

/// User listing filters
#[ derive( Debug, Clone, Default ) ]
pub struct ListUsersFilters
{
  /// Filter by role
  pub role: Option< String >,
  /// Filter by active status
  pub is_active: Option< bool >,
  /// Search by username or email (partial match)
  pub search: Option< String >,
  /// Results limit (default 50, max 100)
  pub limit: Option< i64 >,
  /// Pagination offset (default 0)
  pub offset: Option< i64 >,
}

/// User statistics (agents, tokens, spending)
#[ derive( Debug, Clone ) ]
pub struct UserStatistics
{
  /// Number of active agents
  pub active_agents: i64,
  /// Total API tokens
  pub total_tokens: i64,
  /// Total spending in cents
  pub total_spending_cents: i64,
}

/// Audit log entry for user management operations
#[ derive( Debug, Clone ) ]
pub struct UserAuditLog
{
  /// Audit log ID
  pub id: i64,
  /// Operation type
  pub operation: String,
  /// Target user ID
  pub target_user_id: String,
  /// Admin who performed operation
  pub performed_by: String,
  /// Operation timestamp (milliseconds since epoch)
  pub timestamp: i64,
  /// Previous state (JSON)
  pub previous_state: Option< String >,
  /// New state (JSON)
  pub new_state: Option< String >,
  /// Optional reason
  pub reason: Option< String >,
}

/// User management service
///
/// Handles all admin-only user lifecycle operations with automatic audit logging.
#[ derive( Debug, Clone ) ]
pub struct UserService
{
  pool: SqlitePool,
}

impl UserService
{
  /// Create new user service
  ///
  /// # Arguments
  ///
  /// * `pool` - Database connection pool
  #[ must_use ]
  pub fn new( pool: SqlitePool ) -> Self
  {
    Self { pool }
  }

  /// Create new user account
  ///
  /// # Arguments
  ///
  /// * `params` - User creation parameters
  /// * `admin_id` - ID of admin creating the user
  ///
  /// # Returns
  ///
  /// Created user with generated ID
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - Username already exists (unique constraint violation)
  /// - Email already exists (unique constraint violation)
  /// - Password hashing fails
  /// - Database insert fails
  pub async fn create_user( &self, params: CreateUserParams, admin_id: &str ) -> Result< User >
  {
    // Hash password with BCrypt
    let password_hash = bcrypt::hash( &params.password, bcrypt::DEFAULT_COST )
      .map_err( |e| { error!( "Error hashing password: {}", e ); crate::error::TokenError } )?;

    let now_ms = current_time_ms();

    let mut user_prefix = "user_".to_string();
    let user_id = uuid::Uuid::new_v4().to_string();
    user_prefix.push_str( &user_id );

    // Insert user
    let result = sqlx::query(
      "INSERT INTO users (id, username, password_hash, email, role, is_active, created_at) \
       VALUES ($1, $2, $3, $4, $5, 1, $6)"
    )
    .bind( &user_prefix )
    .bind( &params.username )
    .bind( &password_hash )
    .bind( &params.email )
    .bind( &params.role )
    .bind( now_ms )
    .execute( &self.pool )
    .await
    .map_err( |e| { error!( "Error creating user: {}", e ); crate::error::TokenError } )?;

    let user_id = user_prefix;

    // Audit log
    self.log_audit(
      "create",
      &user_id,
      admin_id,
      None,
      Some( serde_json::json!( {
        "username": params.username,
        "email": params.email,
        "role": params.role,
      } ).to_string() ),
      None,
    ).await?;

    // Return created user
    self.get_user_by_id( &user_id ).await
  }

  /// List users with optional filters
  ///
  /// # Arguments
  ///
  /// * `filters` - Optional filters for role, active status, search
  ///
  /// # Returns
  ///
  /// Vector of users matching filters and total count
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn list_users( &self, filters: ListUsersFilters ) -> Result< ( Vec< User >, i64 ) >
  {
    let limit = filters.limit.unwrap_or( 50 ).min( 100 );
    let offset = filters.offset.unwrap_or( 0 );

    // Build query with filters
    let mut query = String::from(
      "SELECT id, username, email, password_hash, role, is_active, created_at, \
       last_login, suspended_at, suspended_by, deleted_at, deleted_by, force_password_change \
       FROM users WHERE 1=1"
    );
    let mut count_query = String::from( "SELECT COUNT(*) FROM users WHERE 1=1" );

    if let Some( ref role ) = filters.role
    {
      let _ = write!( &mut query, " AND role = '{role}'" );
      let _ = write!( &mut count_query, " AND role = '{role}'" );
    }

    if let Some( is_active ) = filters.is_active
    {
      let active_val = i32::from( is_active );
      let _ = write!( &mut query, " AND is_active = {active_val}" );
      let _ = write!( &mut count_query, " AND is_active = {active_val}" );
    }

    if let Some( ref search ) = filters.search
    {
      let _ = write!( &mut query, " AND (username LIKE '%{search}%' OR email LIKE '%{search}%')" );
      let _ = write!( &mut count_query, " AND (username LIKE '%{search}%' OR email LIKE '%{search}%')" );
    }

    let _ = write!( &mut query, " ORDER BY created_at DESC LIMIT {limit} OFFSET {offset}" );

    // Get total count
    let total : i64 = sqlx::query_scalar( &count_query )
      .fetch_one( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;

    // Get users
    let rows = sqlx::raw_sql( &query )
      .fetch_all( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;

    let users = rows.iter().map( |row| User {
      id: row.get( "id" ),
      username: row.get( "username" ),
      email: row.get( "email" ),
      password_hash: row.get( "password_hash" ),
      role: row.get( "role" ),
      is_active: row.get::< i64, _ >( "is_active" ) != 0,
      created_at: row.get( "created_at" ),
      last_login: row.get( "last_login" ),
      suspended_at: row.get( "suspended_at" ),
      suspended_by: row.get( "suspended_by" ),
      deleted_at: row.get( "deleted_at" ),
      deleted_by: row.get( "deleted_by" ),
      force_password_change: row.get::< i64, _ >( "force_password_change" ) != 0,
    } ).collect();

    Ok( ( users, total ) )
  }

  /// Get user by ID
  ///
  /// # Arguments
  ///
  /// * `user_id` - User database ID
  ///
  /// # Returns
  ///
  /// User data
  ///
  /// # Errors
  ///
  /// Returns error if user not found or database query fails
  pub async fn get_user_by_id( &self, user_id: &str ) -> Result< User >
  {
    let row = sqlx::query(
      "SELECT id, username, email, password_hash, role, is_active, created_at, \
       last_login, suspended_at, suspended_by, deleted_at, deleted_by, force_password_change \
       FROM users WHERE id = $1"
    )
    .bind( user_id )
    .fetch_one( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( User {
      id: row.get( "id" ),
      username: row.get( "username" ),
      email: row.get( "email" ),
      password_hash: row.get( "password_hash" ),
      role: row.get( "role" ),
      is_active: row.get::< i64, _ >( "is_active" ) != 0,
      created_at: row.get( "created_at" ),
      last_login: row.get( "last_login" ),
      suspended_at: row.get( "suspended_at" ),
      suspended_by: row.get( "suspended_by" ),
      deleted_at: row.get( "deleted_at" ),
      deleted_by: row.get( "deleted_by" ),
      force_password_change: row.get::< i64, _ >( "force_password_change" ) != 0,
    } )
  }

  /// Suspend user account
  ///
  /// # Arguments
  ///
  /// * `user_id` - User to suspend
  /// * `admin_id` - Admin performing suspension
  /// * `reason` - Optional reason for suspension
  ///
  /// # Returns
  ///
  /// Updated user
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - User not found
  /// - User already suspended
  /// - Database update fails
  pub async fn suspend_user( &self, user_id: &str, admin_id: &str, reason: Option< String > ) -> Result< User >
  {
    let now_ms = current_time_ms();

    // Get current user state
    let user = self.get_user_by_id( user_id ).await?;

    // Check if already suspended
    if !user.is_active
    {
      return Err( crate::error::TokenError );
    }

    // Suspend user
    sqlx::query(
      "UPDATE users SET is_active = 0, suspended_at = $1, suspended_by = $2 WHERE id = $3"
    )
    .bind( now_ms )
    .bind( admin_id )
    .bind( user_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    // Audit log
    self.log_audit(
      "suspend",
      user_id,
      admin_id,
      Some( serde_json::json!( { "is_active": true } ).to_string() ),
      Some( serde_json::json!( { "is_active": false } ).to_string() ),
      reason,
    ).await?;

    self.get_user_by_id( user_id ).await
  }

  /// Activate user account
  ///
  /// # Arguments
  ///
  /// * `user_id` - User to activate
  /// * `admin_id` - Admin performing activation
  ///
  /// # Returns
  ///
  /// Updated user
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - User not found
  /// - User already active
  /// - Database update fails
  pub async fn activate_user( &self, user_id: &str, admin_id: &str ) -> Result< User >
  {
    // Get current user state
    let user = self.get_user_by_id( user_id ).await?;

    // Check if already active
    if user.is_active
    {
      return Err( crate::error::TokenError );
    }

    // Activate user
    sqlx::query(
      "UPDATE users SET is_active = 1, suspended_at = NULL, suspended_by = NULL WHERE id = $1"
    )
    .bind( user_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    // Audit log
    self.log_audit(
      "activate",
      user_id,
      admin_id,
      Some( serde_json::json!( { "is_active": false } ).to_string() ),
      Some( serde_json::json!( { "is_active": true } ).to_string() ),
      None,
    ).await?;

    self.get_user_by_id( user_id ).await
  }

  /// Delete user (soft delete)
  ///
  /// # Arguments
  ///
  /// * `user_id` - User to delete
  /// * `admin_id` - Admin performing deletion
  ///
  /// # Returns
  ///
  /// Updated user with `deleted_at` timestamp
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - User not found
  /// - Trying to delete self
  /// - Database update fails
  pub async fn delete_user( &self, user_id: &str, admin_id: &str ) -> Result< User >
  {
    // Prevent deleting self
    if user_id == admin_id
    {
      return Err( crate::error::TokenError );
    }

    let now_ms = current_time_ms();

    // Soft delete user
    sqlx::query(
      "UPDATE users SET is_active = 0, deleted_at = $1, deleted_by = $2 WHERE id = $3"
    )
    .bind( now_ms )
    .bind( admin_id )
    .bind( user_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    // Audit log
    self.log_audit(
      "delete",
      user_id,
      admin_id,
      None,
      Some( serde_json::json!( { "deleted": true } ).to_string() ),
      None,
    ).await?;

    self.get_user_by_id( user_id ).await
  }

  /// Change user role
  ///
  /// # Arguments
  ///
  /// * `user_id` - User to change role
  /// * `admin_id` - Admin performing role change
  /// * `new_role` - New role (user, `super_user`, admin)
  ///
  /// # Returns
  ///
  /// Updated user
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - User not found
  /// - Trying to change own role
  /// - Database update fails
  pub async fn change_user_role( &self, user_id: &str, admin_id: &str, new_role: String ) -> Result< User >
  {
    // Prevent changing own role
    if user_id == admin_id
    {
      return Err( crate::error::TokenError );
    }

    // Get current user state
    let user = self.get_user_by_id( user_id ).await?;
    let old_role = user.role.clone();

    // Update role
    sqlx::query( "UPDATE users SET role = $1 WHERE id = $2" )
      .bind( &new_role )
      .bind( user_id )
      .execute( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;

    // Audit log
    self.log_audit(
      "role_change",
      user_id,
      admin_id,
      Some( serde_json::json!( { "role": old_role } ).to_string() ),
      Some( serde_json::json!( { "role": &new_role } ).to_string() ),
      None,
    ).await?;

    self.get_user_by_id( user_id ).await
  }

  /// Reset user password
  ///
  /// # Arguments
  ///
  /// * `user_id` - User to reset password
  /// * `admin_id` - Admin performing reset
  /// * `new_password` - New temporary password
  /// * `force_change` - Force password change on next login
  ///
  /// # Returns
  ///
  /// Updated user
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - User not found
  /// - Password hashing fails
  /// - Database update fails
  pub async fn reset_password(
    &self,
    user_id: &str,
    admin_id: &str,
    new_password: String,
    force_change: bool,
  ) -> Result< User >
  {
    // Hash new password
    let password_hash = bcrypt::hash( &new_password, bcrypt::DEFAULT_COST )
      .map_err( |_| crate::error::TokenError )?;

    let force_change_val = i32::from( force_change );

    // Update password
    sqlx::query(
      "UPDATE users SET password_hash = $1, force_password_change = $2 WHERE id = $3"
    )
    .bind( &password_hash )
    .bind( force_change_val )
    .bind( user_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    // Audit log
    self.log_audit(
      "password_reset",
      user_id,
      admin_id,
      None,
      Some( serde_json::json!( { "force_change": force_change } ).to_string() ),
      None,
    ).await?;

    self.get_user_by_id( user_id ).await
  }

  /// Log audit entry for user management operation
  ///
  /// # Arguments
  ///
  /// * `operation` - Operation type
  /// * `target_user_id` - User affected
  /// * `performed_by` - Admin performing operation
  /// * `previous_state` - Previous state (JSON)
  /// * `new_state` - New state (JSON)
  /// * `reason` - Optional reason
  ///
  /// # Errors
  ///
  /// Returns error if database insert fails
  async fn log_audit(
    &self,
    operation: &str,
    target_user_id: &str,
    performed_by: &str,
    previous_state: Option< String >,
    new_state: Option< String >,
    reason: Option< String >,
  ) -> Result< () >
  {
    let now_ms = current_time_ms();

    sqlx::query(
      "INSERT INTO user_audit_log \
       (operation, target_user_id, performed_by, timestamp, previous_state, new_state, reason) \
       VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind( operation )
    .bind( target_user_id )
    .bind( performed_by )
    .bind( now_ms )
    .bind( previous_state )
    .bind( new_state )
    .bind( reason )
    .execute( &self.pool )
    .await
    .map_err( |e| { error!( "Error logging audit: {}", e ); crate::error::TokenError } )?;

    Ok( () )
  }

  /// Get database pool for test verification
  ///
  /// **Warning:** Test-only method for accessing internal state
  #[ must_use ]
  pub fn pool( &self ) -> &SqlitePool
  {
    &self.pool
  }
}

/// Get current time in milliseconds since UNIX epoch
#[ allow( clippy::cast_possible_truncation ) ]
fn current_time_ms() -> i64
{
  std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "Time went backwards" )
    .as_millis() as i64
}
