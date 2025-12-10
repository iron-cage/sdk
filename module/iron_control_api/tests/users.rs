//! User management domain tests
//!
//! Tests for all user management endpoints (Phase 2: User Management API).
//!
//! Endpoints tested:
//! - POST /api/users - Create user
//! - GET /api/users - List users with filters
//! - GET /api/users/:id - Get user details
//! - PUT /api/users/:id/suspend - Suspend user account
//! - PUT /api/users/:id/activate - Activate user account
//! - DELETE /api/users/:id - Delete user account (soft delete)
//! - PUT /api/users/:id/role - Change user role
//! - POST /api/users/:id/reset-password - Reset user password
//!
//! Coverage:
//! - Request validation (username, email, password, role)
//! - HTTP status codes (200, 201, 400, 403, 404, 500)
//! - JSON response structure
//! - RBAC enforcement (Admin-only operations)
//! - Database persistence and audit logging
//! - Edge cases (self-deletion, duplicate usernames, invalid filters)

#[ path = "common/mod.rs" ]
mod common;

#[ path = "users/endpoints.rs" ]
mod endpoints;

#[ path = "users/debug_test.rs" ]
mod debug_test;
