//! Tests for the Role RBAC type.

use core::str::FromStr;

use iron_types::Role;

#[test]
fn parse_admin() {
  assert_eq!(Role::from_str("admin").unwrap(), Role::Admin);
}

#[test]
fn parse_manager() {
  assert_eq!(Role::from_str("manager").unwrap(), Role::Manager);
}

#[test]
fn parse_developer() {
  assert_eq!(Role::from_str("developer").unwrap(), Role::Developer);
}

#[test]
fn parse_case_insensitive() {
  assert_eq!(Role::from_str("Admin").unwrap(), Role::Admin);
  assert_eq!(Role::from_str("ADMIN").unwrap(), Role::Admin);
  assert_eq!(Role::from_str("Manager").unwrap(), Role::Manager);
  assert_eq!(Role::from_str("DEVELOPER").unwrap(), Role::Developer);
}

#[test]
fn parse_invalid_role_returns_error() {
  let result = Role::from_str("superuser");
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("superuser"));
}

#[test]
fn as_str_round_trip() {
  for role in [Role::Admin, Role::Manager, Role::Developer] {
    let s = role.as_str();
    let parsed = Role::from_str(s).unwrap();
    assert_eq!(parsed, role);
  }
}

#[test]
fn display_matches_as_str() {
  for role in [Role::Admin, Role::Manager, Role::Developer] {
    assert_eq!(role.to_string(), role.as_str());
  }
}

#[test]
fn ordering_developer_less_than_manager() {
  assert!(Role::Developer < Role::Manager);
}

#[test]
fn ordering_manager_less_than_admin() {
  assert!(Role::Manager < Role::Admin);
}

#[test]
fn ordering_developer_less_than_admin() {
  assert!(Role::Developer < Role::Admin);
}

#[test]
fn admin_is_max_privilege() {
  let roles = [Role::Developer, Role::Admin, Role::Manager];
  let max = roles.iter().max().unwrap();
  assert_eq!(*max, Role::Admin);
}

#[test]
fn serde_round_trip() {
  for role in [Role::Admin, Role::Manager, Role::Developer] {
    let json = serde_json::to_string(&role).unwrap();
    let parsed: Role = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, role);
  }
}
