//! RBAC (Role-Based Access Control) tests
//!
//! Phase 4 Day 27: RBAC Authorization
//!
//! Per plan:
//! - Define permission model (admin, user, viewer)
//! - Implement role checking middleware
//! - Add role-based route protection
//! - Write unit tests for RBAC

use iron_api::rbac::{ Role, Permission, PermissionChecker };
use iron_api::rbac::middleware::UserContext;

#[ test ]
fn test_role_ordering()
{
  // Admin > User > Viewer
  assert!( Role::Admin > Role::User );
  assert!( Role::User > Role::Viewer );
  assert!( Role::Admin > Role::Viewer );
}

#[ test ]
fn test_admin_has_all_permissions()
{
  let checker = PermissionChecker::new();

  assert!( checker.has_permission( Role::Admin, Permission::ReadAgents ) );
  assert!( checker.has_permission( Role::Admin, Permission::WriteAgents ) );
  assert!( checker.has_permission( Role::Admin, Permission::StopAgents ) );
  assert!( checker.has_permission( Role::Admin, Permission::ReadMetrics ) );
  assert!( checker.has_permission( Role::Admin, Permission::ManageUsers ) );
  assert!( checker.has_permission( Role::Admin, Permission::ManageTokens ) );
}

#[ test ]
fn test_user_has_standard_permissions()
{
  let checker = PermissionChecker::new();

  // Users can read/write/stop their own agents
  assert!( checker.has_permission( Role::User, Permission::ReadAgents ) );
  assert!( checker.has_permission( Role::User, Permission::WriteAgents ) );
  assert!( checker.has_permission( Role::User, Permission::StopAgents ) );
  assert!( checker.has_permission( Role::User, Permission::ReadMetrics ) );

  // Users cannot manage other users or tokens
  assert!( !checker.has_permission( Role::User, Permission::ManageUsers ) );
  assert!( !checker.has_permission( Role::User, Permission::ManageTokens ) );
}

#[ test ]
fn test_viewer_has_read_only_permissions()
{
  let checker = PermissionChecker::new();

  // Viewers can only read
  assert!( checker.has_permission( Role::Viewer, Permission::ReadAgents ) );
  assert!( checker.has_permission( Role::Viewer, Permission::ReadMetrics ) );

  // Viewers cannot write or manage anything
  assert!( !checker.has_permission( Role::Viewer, Permission::WriteAgents ) );
  assert!( !checker.has_permission( Role::Viewer, Permission::StopAgents ) );
  assert!( !checker.has_permission( Role::Viewer, Permission::ManageUsers ) );
  assert!( !checker.has_permission( Role::Viewer, Permission::ManageTokens ) );
}

#[ test ]
fn test_role_from_string()
{
  use core::str::FromStr;

  assert_eq!( Role::from_str( "admin" ), Ok( Role::Admin ) );
  assert_eq!( Role::from_str( "user" ), Ok( Role::User ) );
  assert_eq!( Role::from_str( "viewer" ), Ok( Role::Viewer ) );
  assert!( Role::from_str( "invalid" ).is_err() );
  assert_eq!( Role::from_str( "ADMIN" ), Ok( Role::Admin ) ); // Case insensitive
}

#[ test ]
fn test_role_to_string()
{
  assert_eq!( Role::Admin.as_str(), "admin" );
  assert_eq!( Role::User.as_str(), "user" );
  assert_eq!( Role::Viewer.as_str(), "viewer" );
}

#[ test ]
fn test_permission_from_string()
{
  use core::str::FromStr;

  assert_eq!( Permission::from_str( "read_agents" ), Ok( Permission::ReadAgents ) );
  assert_eq!( Permission::from_str( "write_agents" ), Ok( Permission::WriteAgents ) );
  assert_eq!( Permission::from_str( "stop_agents" ), Ok( Permission::StopAgents ) );
  assert_eq!( Permission::from_str( "read_metrics" ), Ok( Permission::ReadMetrics ) );
  assert_eq!( Permission::from_str( "manage_users" ), Ok( Permission::ManageUsers ) );
  assert_eq!( Permission::from_str( "manage_tokens" ), Ok( Permission::ManageTokens ) );
  assert!( Permission::from_str( "invalid" ).is_err() );
}

#[ test ]
fn test_user_context_creation()
{
  let context = UserContext
  {
    user_id: "user_123".to_string(),
    role: Role::Admin,
  };

  assert_eq!( context.user_id, "user_123" );
  assert_eq!( context.role, Role::Admin );
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
  assert!( checker.has_permission( admin_context.role, Permission::ManageUsers ) );

  // User context cannot access manage_users
  let user_context = UserContext
  {
    user_id: "user_1".to_string(),
    role: Role::User,
  };
  assert!( !checker.has_permission( user_context.role, Permission::ManageUsers ) );

  // Viewer context can only read
  let viewer_context = UserContext
  {
    user_id: "viewer_1".to_string(),
    role: Role::Viewer,
  };
  assert!( checker.has_permission( viewer_context.role, Permission::ReadAgents ) );
  assert!( !checker.has_permission( viewer_context.role, Permission::WriteAgents ) );
}
