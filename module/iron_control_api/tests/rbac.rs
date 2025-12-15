//! RBAC (Role-Based Access Control) tests
//!
//! Phase 4 Day 27: RBAC Authorization
//!
//! Per plan:
//! - Define permission model (admin, user, viewer)
//! - Implement role checking middleware
//! - Add role-based route protection
//! - Write unit tests for RBAC
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_role_ordering` | Role hierarchy ordering | Compare Admin, User, Viewer roles | Admin > User > Viewer | ✅ |
//! | `test_admin_has_all_permissions` | Admin role permissions | Check Admin permissions for Read/Write/Stop | All permissions granted | ✅ |
//! | `test_user_has_standard_permissions` | User role permissions | Check User permissions (Read/Write agents, no Stop) | Standard permissions granted, no Stop | ✅ |
//! | `test_viewer_has_read_only_permissions` | Viewer role permissions | Check Viewer permissions (Read only, no Write/Stop) | Read-only permissions | ✅ |
//! | `test_role_from_string` | Parse role from string | "admin", "user", "viewer" strings | Correct Role enum variants | ✅ |
//! | `test_role_to_string` | Convert role to string | Admin, User, Viewer enums | "admin", "user", "viewer" strings | ✅ |
//! | `test_permission_from_string` | Parse permission from string | "read_agents", "write_agents" strings | Correct Permission enum variants | ✅ |
//! | `test_user_context_creation` | Create user context | user_id + email + role | UserContext with correct fields | ✅ |
//! | `test_middleware_permission_checking` | Middleware permission check | UserContext with role, check permissions | Permissions validated correctly | ✅ |

use iron_control_api::rbac::{ Role, Permission, PermissionChecker };
use iron_control_api::rbac::middleware::UserContext;

#[ test ]
fn test_role_ordering()
{
  // Admin > User > Viewer
  assert!( Role::Admin > Role::User, "Admin role should have higher privilege than User role" );
  assert!( Role::User > Role::Viewer, "User role should have higher privilege than Viewer role" );
  assert!( Role::Admin > Role::Viewer, "Admin role should have higher privilege than Viewer role" );
}

#[ test ]
fn test_admin_has_all_permissions()
{
  let checker = PermissionChecker::new();

  assert!( checker.has_permission( Role::Admin, Permission::ReadAgents ), "Admin should have ReadAgents permission" );
  assert!( checker.has_permission( Role::Admin, Permission::WriteAgents ), "Admin should have WriteAgents permission" );
  assert!( checker.has_permission( Role::Admin, Permission::StopAgents ), "Admin should have StopAgents permission" );
  assert!( checker.has_permission( Role::Admin, Permission::ReadMetrics ), "Admin should have ReadMetrics permission" );
  assert!( checker.has_permission( Role::Admin, Permission::ManageUsers ), "Admin should have ManageUsers permission" );
  assert!( checker.has_permission( Role::Admin, Permission::ManageTokens ), "Admin should have ManageTokens permission" );
}

#[ test ]
fn test_user_has_standard_permissions()
{
  let checker = PermissionChecker::new();

  // Users can read/write/stop their own agents
  assert!( checker.has_permission( Role::User, Permission::ReadAgents ), "User should have ReadAgents permission for their own agents" );
  assert!( checker.has_permission( Role::User, Permission::WriteAgents ), "User should have WriteAgents permission for their own agents" );
  assert!( checker.has_permission( Role::User, Permission::StopAgents ), "User should have StopAgents permission for their own agents" );
  assert!( checker.has_permission( Role::User, Permission::ReadMetrics ), "User should have ReadMetrics permission" );

  // Users cannot manage other users or tokens
  assert!( !checker.has_permission( Role::User, Permission::ManageUsers ), "User should NOT have ManageUsers permission" );
  assert!( !checker.has_permission( Role::User, Permission::ManageTokens ), "User should NOT have ManageTokens permission" );
}

#[ test ]
fn test_viewer_has_read_only_permissions()
{
  let checker = PermissionChecker::new();

  // Viewers can only read
  assert!( checker.has_permission( Role::Viewer, Permission::ReadAgents ), "Viewer should have ReadAgents permission (read-only)" );
  assert!( checker.has_permission( Role::Viewer, Permission::ReadMetrics ), "Viewer should have ReadMetrics permission (read-only)" );

  // Viewers cannot write or manage anything
  assert!( !checker.has_permission( Role::Viewer, Permission::WriteAgents ), "Viewer should NOT have WriteAgents permission" );
  assert!( !checker.has_permission( Role::Viewer, Permission::StopAgents ), "Viewer should NOT have StopAgents permission" );
  assert!( !checker.has_permission( Role::Viewer, Permission::ManageUsers ), "Viewer should NOT have ManageUsers permission" );
  assert!( !checker.has_permission( Role::Viewer, Permission::ManageTokens ), "Viewer should NOT have ManageTokens permission" );
}

#[ test ]
fn test_role_from_string()
{
  use core::str::FromStr;

  assert_eq!( Role::from_str( "admin" ), Ok( Role::Admin ), "String 'admin' should parse to Role::Admin" );
  assert_eq!( Role::from_str( "user" ), Ok( Role::User ), "String 'user' should parse to Role::User" );
  assert_eq!( Role::from_str( "viewer" ), Ok( Role::Viewer ), "String 'viewer' should parse to Role::Viewer" );
  assert!( Role::from_str( "invalid" ).is_err(), "Invalid role string should return error" );
  assert_eq!( Role::from_str( "ADMIN" ), Ok( Role::Admin ), "Role parsing should be case-insensitive" );
}

#[ test ]
fn test_role_to_string()
{
  assert_eq!( Role::Admin.as_str(), "admin", "Role::Admin should convert to string 'admin'" );
  assert_eq!( Role::User.as_str(), "user", "Role::User should convert to string 'user'" );
  assert_eq!( Role::Viewer.as_str(), "viewer", "Role::Viewer should convert to string 'viewer'" );
}

#[ test ]
fn test_permission_from_string()
{
  use core::str::FromStr;

  assert_eq!( Permission::from_str( "read_agents" ), Ok( Permission::ReadAgents ), "String 'read_agents' should parse to Permission::ReadAgents" );
  assert_eq!( Permission::from_str( "write_agents" ), Ok( Permission::WriteAgents ), "String 'write_agents' should parse to Permission::WriteAgents" );
  assert_eq!( Permission::from_str( "stop_agents" ), Ok( Permission::StopAgents ), "String 'stop_agents' should parse to Permission::StopAgents" );
  assert_eq!( Permission::from_str( "read_metrics" ), Ok( Permission::ReadMetrics ), "String 'read_metrics' should parse to Permission::ReadMetrics" );
  assert_eq!( Permission::from_str( "manage_users" ), Ok( Permission::ManageUsers ), "String 'manage_users' should parse to Permission::ManageUsers" );
  assert_eq!( Permission::from_str( "manage_tokens" ), Ok( Permission::ManageTokens ), "String 'manage_tokens' should parse to Permission::ManageTokens" );
  assert!( Permission::from_str( "invalid" ).is_err(), "Invalid permission string should return error" );
}

#[ test ]
fn test_user_context_creation()
{
  let context = UserContext
  {
    user_id: "user_123".to_string(),
    role: Role::Admin,
  };

  assert_eq!( context.user_id, "user_123", "UserContext should preserve user_id as specified" );
  assert_eq!( context.role, Role::Admin, "UserContext should preserve role as specified" );
}

#[ test ]
fn test_middleware_permission_checking()
{
  let checker = PermissionChecker::new();

  // Admin context can access manage_users
  let admin_context = UserContext
  {
    user_id: "admin_1".to_string(),
    role: Role::Admin,
  };
  assert!( checker.has_permission( admin_context.role, Permission::ManageUsers ), "Admin context should have ManageUsers permission" );

  // User context cannot access manage_users
  let user_context = UserContext
  {
    user_id: "user_1".to_string(),
    role: Role::User,
  };
  assert!( !checker.has_permission( user_context.role, Permission::ManageUsers ), "User context should NOT have ManageUsers permission" );

  // Viewer context can only read
  let viewer_context = UserContext
  {
    user_id: "viewer_1".to_string(),
    role: Role::Viewer,
  };
  assert!( checker.has_permission( viewer_context.role, Permission::ReadAgents ), "Viewer context should have ReadAgents permission" );
  assert!( !checker.has_permission( viewer_context.role, Permission::WriteAgents ), "Viewer context should NOT have WriteAgents permission" );
}
