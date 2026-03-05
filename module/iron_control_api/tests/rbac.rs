//! RBAC (Role-Based Access Control) tests
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Expected |
//! |-----------|----------|----------|
//! | `test_role_ordering` | Role hierarchy | Admin > Manager > Developer |
//! | `test_admin_has_all_permissions` | Admin permissions | All permissions granted |
//! | `test_manager_permissions` | Manager permissions | Keys/tokens/agents, no users/roles |
//! | `test_developer_has_read_only_permissions` | Developer permissions | Read-only access |
//! | `test_role_from_string` | Parse canonical names | Correct Role enum variants |
//! | `test_role_from_string_legacy` | Parse legacy names | User->Manager, Viewer->Developer |
//! | `test_role_to_string` | Convert to string | Canonical names |
//! | `test_permission_from_string` | Parse permission names | Correct Permission variants |
//! | `test_user_context_creation` | Create user context | UserContext with correct fields |
//! | `test_middleware_permission_checking` | Middleware check | Permissions validated correctly |

use iron_control_api::rbac::{middleware::UserContext, Permission, PermissionChecker, Role};

#[test]
fn test_role_ordering() {
  assert!(
    Role::Admin > Role::Manager,
    "Admin role should have higher privilege than Manager role"
  );
  assert!(
    Role::Manager > Role::Developer,
    "Manager role should have higher privilege than Developer role"
  );
  assert!(
    Role::Admin > Role::Developer,
    "Admin role should have higher privilege than Developer role"
  );
}

#[test]
fn test_admin_has_all_permissions() {
  let checker = PermissionChecker::new();

  assert!(checker.has_permission(Role::Admin, Permission::ReadAgents));
  assert!(checker.has_permission(Role::Admin, Permission::WriteAgents));
  assert!(checker.has_permission(Role::Admin, Permission::ManageProviderKeys));
  assert!(checker.has_permission(Role::Admin, Permission::ManageIcTokens));
  assert!(checker.has_permission(Role::Admin, Permission::ManageUsers));
  assert!(checker.has_permission(Role::Admin, Permission::AssignRoles));
  assert!(checker.has_permission(Role::Admin, Permission::ReadMetrics));
}

#[test]
fn test_manager_permissions() {
  let checker = PermissionChecker::new();

  // Manager CAN do:
  assert!(checker.has_permission(Role::Manager, Permission::ReadAgents));
  assert!(checker.has_permission(Role::Manager, Permission::WriteAgents));
  assert!(checker.has_permission(Role::Manager, Permission::ManageProviderKeys));
  assert!(checker.has_permission(Role::Manager, Permission::ManageIcTokens));
  assert!(checker.has_permission(Role::Manager, Permission::ReadMetrics));

  // Manager CANNOT do:
  assert!(
    !checker.has_permission(Role::Manager, Permission::ManageUsers),
    "Manager should NOT have ManageUsers permission"
  );
  assert!(
    !checker.has_permission(Role::Manager, Permission::AssignRoles),
    "Manager should NOT have AssignRoles permission"
  );
}

#[test]
fn test_developer_has_read_only_permissions() {
  let checker = PermissionChecker::new();

  // Developer CAN do:
  assert!(checker.has_permission(Role::Developer, Permission::ReadAgents));
  assert!(checker.has_permission(Role::Developer, Permission::ReadMetrics));

  // Developer CANNOT do:
  assert!(!checker.has_permission(Role::Developer, Permission::WriteAgents));
  assert!(!checker.has_permission(Role::Developer, Permission::ManageProviderKeys));
  assert!(!checker.has_permission(Role::Developer, Permission::ManageIcTokens));
  assert!(!checker.has_permission(Role::Developer, Permission::ManageUsers));
  assert!(!checker.has_permission(Role::Developer, Permission::AssignRoles));
}

#[test]
fn test_role_from_string() {
  use core::str::FromStr;

  assert_eq!(Role::from_str("admin"), Ok(Role::Admin));
  assert_eq!(Role::from_str("manager"), Ok(Role::Manager));
  assert_eq!(Role::from_str("developer"), Ok(Role::Developer));
  assert!(Role::from_str("invalid").is_err());
  // Case-insensitive
  assert_eq!(Role::from_str("ADMIN"), Ok(Role::Admin));
  assert_eq!(Role::from_str("manager"), Ok(Role::Manager));
  assert_eq!(Role::from_str("DEVELOPER"), Ok(Role::Developer));
}

#[test]
fn test_role_to_string() {
  assert_eq!(Role::Admin.as_str(), "admin");
  assert_eq!(Role::Manager.as_str(), "manager");
  assert_eq!(Role::Developer.as_str(), "developer");
}

#[test]
fn test_permission_from_string() {
  use core::str::FromStr;

  assert_eq!(
    Permission::from_str("read_agents"),
    Ok(Permission::ReadAgents)
  );
  assert_eq!(
    Permission::from_str("write_agents"),
    Ok(Permission::WriteAgents)
  );
  assert_eq!(
    Permission::from_str("manage_provider_keys"),
    Ok(Permission::ManageProviderKeys)
  );
  assert_eq!(
    Permission::from_str("manage_ic_tokens"),
    Ok(Permission::ManageIcTokens)
  );
  assert_eq!(
    Permission::from_str("manage_users"),
    Ok(Permission::ManageUsers)
  );
  assert_eq!(
    Permission::from_str("assign_roles"),
    Ok(Permission::AssignRoles)
  );
  assert_eq!(
    Permission::from_str("read_metrics"),
    Ok(Permission::ReadMetrics)
  );
  assert!(Permission::from_str("invalid").is_err());
}

#[test]
fn test_user_context_creation() {
  let context = UserContext {
    user_id: "user_123".to_string(),
    role: Role::Admin,
  };

  assert_eq!(context.user_id, "user_123");
  assert_eq!(context.role, Role::Admin);
}

#[test]
fn test_middleware_permission_checking() {
  let checker = PermissionChecker::new();

  // Admin can manage users and assign roles
  let admin_context = UserContext {
    user_id: "admin_1".to_string(),
    role: Role::Admin,
  };
  assert!(checker.has_permission(admin_context.role, Permission::ManageUsers));
  assert!(checker.has_permission(admin_context.role, Permission::AssignRoles));

  // Manager can manage keys but not assign roles
  let manager_context = UserContext {
    user_id: "su_1".to_string(),
    role: Role::Manager,
  };
  assert!(checker.has_permission(manager_context.role, Permission::ManageProviderKeys));
  assert!(!checker.has_permission(manager_context.role, Permission::AssignRoles));

  // Developer can only read
  let dev_context = UserContext {
    user_id: "dev_1".to_string(),
    role: Role::Developer,
  };
  assert!(checker.has_permission(dev_context.role, Permission::ReadAgents));
  assert!(!checker.has_permission(dev_context.role, Permission::WriteAgents));
  assert!(!checker.has_permission(dev_context.role, Permission::ManageProviderKeys));
}
