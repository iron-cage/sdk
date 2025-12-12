//! User management command handlers
//!
//! Pure functions for user create, list, get, suspend, activate, delete,
//! role change, and password reset operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use super::validation::{ validate_non_empty, validate_non_negative_integer };

/// Handle .users.create command
///
/// Validates user creation parameters.
///
/// ## Parameters
///
/// Required:
/// - username: String (non-empty, max 255 chars)
/// - password: String (min 8 chars, max 1000 chars)
/// - email: String (non-empty, contains @, max 255 chars)
/// - role: String (viewer|user|admin)
pub fn create_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let username = params
    .get("username")
    .ok_or(CliError::MissingParameter("username"))?;

  let password = params
    .get("password")
    .ok_or(CliError::MissingParameter("password"))?;

  let email = params
    .get("email")
    .ok_or(CliError::MissingParameter("email"))?;

  let role = params
    .get("role")
    .ok_or(CliError::MissingParameter("role"))?;

  // Validate username
  validate_non_empty(username, "username")?;
  if username.len() > 255
  {
    return Err(CliError::InvalidParameter {
      param: "username",
      reason: "cannot exceed 255 characters",
    });
  }

  // Validate password
  if password.len() < 8
  {
    return Err(CliError::InvalidParameter {
      param: "password",
      reason: "must be at least 8 characters",
    });
  }
  if password.len() > 1000
  {
    return Err(CliError::InvalidParameter {
      param: "password",
      reason: "cannot exceed 1000 characters",
    });
  }

  // Validate email
  validate_non_empty(email, "email")?;
  if !email.contains('@')
  {
    return Err(CliError::InvalidParameter {
      param: "email",
      reason: "must contain @ symbol",
    });
  }
  if email.len() > 255
  {
    return Err(CliError::InvalidParameter {
      param: "email",
      reason: "cannot exceed 255 characters",
    });
  }

  // Validate role
  validate_user_role(role)?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!(
    "User '{}' created successfully with role '{}'",
    username, role
  ))
}

/// Handle .users.list command
///
/// Validates user listing parameters.
///
/// ## Parameters
///
/// All optional:
/// - role: String (viewer|user|admin)
/// - is_active: String (true|false)
/// - search: String (username or email search)
/// - page: String (integer, default 1)
/// - page_size: String (integer, 1-100, default 20)
pub fn list_users_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate optional role filter
  if let Some(role) = params.get("role")
  {
    validate_user_role(role)?;
  }

  // Validate optional is_active filter
  if let Some(is_active) = params.get("is_active")
  {
    validate_boolean(is_active, "is_active")?;
  }

  // Validate optional page
  if let Some(page_str) = params.get("page")
  {
    let page = validate_non_negative_integer(page_str, "page")?;
    if page < 1
    {
      return Err(CliError::InvalidParameter {
        param: "page",
        reason: "must be at least 1",
      });
    }
  }

  // Validate optional page_size
  if let Some(page_size_str) = params.get("page_size")
  {
    let page_size = validate_non_negative_integer(page_size_str, "page_size")?;
    if !(1..=100).contains(&page_size)
    {
      return Err(CliError::InvalidParameter {
        param: "page_size",
        reason: "must be between 1 and 100",
      });
    }
  }

  // Placeholder response - actual API call handled by adapter
  Ok("User list retrieved successfully".to_string())
}

/// Handle .users.get command
///
/// Validates user retrieval parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
pub fn get_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("User {} retrieved successfully", user_id))
}

/// Handle .users.suspend command
///
/// Validates user suspension parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
///
/// Optional:
/// - reason: String (suspension reason)
pub fn suspend_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("User {} suspended successfully", user_id))
}

/// Handle .users.activate command
///
/// Validates user activation parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
pub fn activate_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("User {} activated successfully", user_id))
}

/// Handle .users.delete command
///
/// Validates user deletion parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
pub fn delete_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("User {} deleted successfully", user_id))
}

/// Handle .users.change_role command
///
/// Validates user role change parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
/// - role: String (viewer|user|admin)
pub fn change_user_role_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Validate required role
  let role = params
    .get("role")
    .ok_or(CliError::MissingParameter("role"))?;

  validate_user_role(role)?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("User {} role changed to '{}' successfully", user_id, role))
}

/// Handle .users.reset_password command
///
/// Validates password reset parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
/// - new_password: String (min 8 chars, max 1000 chars)
/// - force_change: String (true|false)
pub fn reset_password_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Validate required new_password
  let new_password = params
    .get("new_password")
    .ok_or(CliError::MissingParameter("new_password"))?;

  if new_password.len() < 8
  {
    return Err(CliError::InvalidParameter {
      param: "new_password",
      reason: "must be at least 8 characters",
    });
  }
  if new_password.len() > 1000
  {
    return Err(CliError::InvalidParameter {
      param: "new_password",
      reason: "cannot exceed 1000 characters",
    });
  }

  // Validate required force_change
  let force_change_str = params
    .get("force_change")
    .ok_or(CliError::MissingParameter("force_change"))?;

  validate_boolean(force_change_str, "force_change")?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("Password reset for user {} successfully", user_id))
}

// Helper validation functions

/// Validates that role is one of: viewer, user, admin
fn validate_user_role(role: &str) -> Result<(), CliError>
{
  match role
  {
    "viewer" | "user" | "admin" => Ok(()),
    _ =>
    {
      Err(CliError::InvalidParameter {
        param: "role",
        reason: "must be one of: viewer, user, admin",
      })
    }
  }
}

/// Validates that a string is a boolean (true|false)
fn validate_boolean(value: &str, param_name: &'static str) -> Result<(), CliError>
{
  match value
  {
    "true" | "false" => Ok(()),
    _ =>
    {
      Err(CliError::InvalidParameter {
        param: param_name,
        reason: "must be 'true' or 'false'",
      })
    }
  }
}

#[cfg(test)]
mod tests
{
  use super::*;

  // create_user_handler tests

  #[test]
  fn test_create_user_valid()
  {
    let mut params = HashMap::new();
    params.insert("username".to_string(), "john_doe".to_string());
    params.insert("password".to_string(), "SecurePass123!".to_string());
    params.insert("email".to_string(), "john@example.com".to_string());
    params.insert("role".to_string(), "user".to_string());

    assert!(create_user_handler(&params).is_ok());
  }

  #[test]
  fn test_create_user_missing_username()
  {
    let mut params = HashMap::new();
    params.insert("password".to_string(), "SecurePass123!".to_string());
    params.insert("email".to_string(), "john@example.com".to_string());
    params.insert("role".to_string(), "user".to_string());

    assert!(matches!(
      create_user_handler(&params),
      Err(CliError::MissingParameter("username"))
    ));
  }

  #[test]
  fn test_create_user_empty_username()
  {
    let mut params = HashMap::new();
    params.insert("username".to_string(), "".to_string());
    params.insert("password".to_string(), "SecurePass123!".to_string());
    params.insert("email".to_string(), "john@example.com".to_string());
    params.insert("role".to_string(), "user".to_string());

    assert!(matches!(
      create_user_handler(&params),
      Err(CliError::InvalidParameter { param: "username", .. })
    ));
  }

  #[test]
  fn test_create_user_password_too_short()
  {
    let mut params = HashMap::new();
    params.insert("username".to_string(), "john_doe".to_string());
    params.insert("password".to_string(), "short".to_string());
    params.insert("email".to_string(), "john@example.com".to_string());
    params.insert("role".to_string(), "user".to_string());

    assert!(matches!(
      create_user_handler(&params),
      Err(CliError::InvalidParameter { param: "password", .. })
    ));
  }

  #[test]
  fn test_create_user_invalid_email()
  {
    let mut params = HashMap::new();
    params.insert("username".to_string(), "john_doe".to_string());
    params.insert("password".to_string(), "SecurePass123!".to_string());
    params.insert("email".to_string(), "invalid-email".to_string());
    params.insert("role".to_string(), "user".to_string());

    assert!(matches!(
      create_user_handler(&params),
      Err(CliError::InvalidParameter { param: "email", .. })
    ));
  }

  #[test]
  fn test_create_user_invalid_role()
  {
    let mut params = HashMap::new();
    params.insert("username".to_string(), "john_doe".to_string());
    params.insert("password".to_string(), "SecurePass123!".to_string());
    params.insert("email".to_string(), "john@example.com".to_string());
    params.insert("role".to_string(), "invalid".to_string());

    assert!(matches!(
      create_user_handler(&params),
      Err(CliError::InvalidParameter { param: "role", .. })
    ));
  }

  // list_users_handler tests

  #[test]
  fn test_list_users_no_filters()
  {
    let params = HashMap::new();
    assert!(list_users_handler(&params).is_ok());
  }

  #[test]
  fn test_list_users_with_role_filter()
  {
    let mut params = HashMap::new();
    params.insert("role".to_string(), "admin".to_string());

    assert!(list_users_handler(&params).is_ok());
  }

  #[test]
  fn test_list_users_invalid_role()
  {
    let mut params = HashMap::new();
    params.insert("role".to_string(), "invalid".to_string());

    assert!(matches!(
      list_users_handler(&params),
      Err(CliError::InvalidParameter { param: "role", .. })
    ));
  }

  #[test]
  fn test_list_users_invalid_is_active()
  {
    let mut params = HashMap::new();
    params.insert("is_active".to_string(), "maybe".to_string());

    assert!(matches!(
      list_users_handler(&params),
      Err(CliError::InvalidParameter { param: "is_active", .. })
    ));
  }

  #[test]
  fn test_list_users_invalid_page_size()
  {
    let mut params = HashMap::new();
    params.insert("page_size".to_string(), "200".to_string());

    assert!(matches!(
      list_users_handler(&params),
      Err(CliError::InvalidParameter { param: "page_size", .. })
    ));
  }

  // get_user_handler tests

  #[test]
  fn test_get_user_valid()
  {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "123".to_string());

    assert!(get_user_handler(&params).is_ok());
  }

  #[test]
  fn test_get_user_missing_id()
  {
    let params = HashMap::new();

    assert!(matches!(
      get_user_handler(&params),
      Err(CliError::MissingParameter("user_id"))
    ));
  }

  #[test]
  fn test_get_user_invalid_id()
  {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "invalid".to_string());

    assert!(matches!(
      get_user_handler(&params),
      Err(CliError::InvalidParameter { param: "user_id", .. })
    ));
  }

  // suspend_user_handler tests

  #[test]
  fn test_suspend_user_valid()
  {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "123".to_string());

    assert!(suspend_user_handler(&params).is_ok());
  }

  #[test]
  fn test_suspend_user_with_reason()
  {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "123".to_string());
    params.insert("reason".to_string(), "Policy violation".to_string());

    assert!(suspend_user_handler(&params).is_ok());
  }

  // activate_user_handler tests

  #[test]
  fn test_activate_user_valid()
  {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "123".to_string());

    assert!(activate_user_handler(&params).is_ok());
  }

  // delete_user_handler tests

  #[test]
  fn test_delete_user_valid()
  {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "123".to_string());

    assert!(delete_user_handler(&params).is_ok());
  }

  // change_user_role_handler tests

  #[test]
  fn test_change_role_valid()
  {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "123".to_string());
    params.insert("role".to_string(), "admin".to_string());

    assert!(change_user_role_handler(&params).is_ok());
  }

  #[test]
  fn test_change_role_invalid_role()
  {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "123".to_string());
    params.insert("role".to_string(), "superadmin".to_string());

    assert!(matches!(
      change_user_role_handler(&params),
      Err(CliError::InvalidParameter { param: "role", .. })
    ));
  }

  // reset_password_handler tests

  #[test]
  fn test_reset_password_valid()
  {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "123".to_string());
    params.insert("new_password".to_string(), "NewPass123!".to_string());
    params.insert("force_change".to_string(), "true".to_string());

    assert!(reset_password_handler(&params).is_ok());
  }

  #[test]
  fn test_reset_password_too_short()
  {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "123".to_string());
    params.insert("new_password".to_string(), "short".to_string());
    params.insert("force_change".to_string(), "true".to_string());

    assert!(matches!(
      reset_password_handler(&params),
      Err(CliError::InvalidParameter { param: "new_password", .. })
    ));
  }

  #[test]
  fn test_reset_password_invalid_force_change()
  {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "123".to_string());
    params.insert("new_password".to_string(), "NewPass123!".to_string());
    params.insert("force_change".to_string(), "maybe".to_string());

    assert!(matches!(
      reset_password_handler(&params),
      Err(CliError::InvalidParameter { param: "force_change", .. })
    ));
  }

  // Helper validation tests

  #[test]
  fn test_validate_user_role_valid()
  {
    assert!(validate_user_role("viewer").is_ok());
    assert!(validate_user_role("user").is_ok());
    assert!(validate_user_role("admin").is_ok());
  }

  #[test]
  fn test_validate_user_role_invalid()
  {
    assert!(validate_user_role("superuser").is_err());
    assert!(validate_user_role("").is_err());
  }

  #[test]
  fn test_validate_boolean_valid()
  {
    assert!(validate_boolean("true", "param").is_ok());
    assert!(validate_boolean("false", "param").is_ok());
  }

  #[test]
  fn test_validate_boolean_invalid()
  {
    assert!(validate_boolean("yes", "param").is_err());
    assert!(validate_boolean("1", "param").is_err());
  }
}
