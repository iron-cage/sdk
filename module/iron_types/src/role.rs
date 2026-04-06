//! RBAC role definitions shared across Iron Cage modules.
//!
//! This module provides the canonical [`Role`] enum used by both
//! `iron_control_api` (permission checking) and `iron_token_manager`
//! (user creation/validation).

use core::str::FromStr;

use serde::{Deserialize, Serialize};

/// User role in the RBAC system.
///
/// Roles are ordered by privilege level: Admin > Manager > Developer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum Role {
  /// Read-only access + own-token inspection
  Developer = 0,
  /// Key/token management without role assignment
  Manager = 1,
  /// Full administrative access
  Admin = 2,
}

impl Role {
  /// Convert role to its canonical string representation (stored in DB/JWT).
  #[must_use]
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Admin => "admin",
      Self::Manager => "manager",
      Self::Developer => "developer",
    }
  }
}

impl FromStr for Role {
  type Err = String;

  /// Parse role from string (case-insensitive).
  ///
  /// # Errors
  ///
  /// Returns error if role name is not recognized.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "admin" => Ok(Self::Admin),
      "manager" => Ok(Self::Manager),
      "developer" => Ok(Self::Developer),
      _ => Err(format!("Invalid role: {s}")),
    }
  }
}

impl core::fmt::Display for Role {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.as_str())
  }
}
