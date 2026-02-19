//! Token management REST API endpoints
//!
//! Phase 4 Day 29: REST API Endpoints - Token Management
//!
//! # Module Organization
//!
//! - `shared`: Token state, request/response types, validation logic
//! - `handlers`: Token lifecycle handlers (7 endpoints)
//!
//! # Endpoints
//!
//! - POST /api/tokens - Create new API token (Protocol 014 compliant)
//! - GET /api/tokens - List all tokens for user
//! - GET /api/tokens/:id - Get specific token details
//! - POST /api/tokens/:id/update - Update token provider
//! - POST /api/tokens/:id/rotate - Rotate token (generate new value)
//! - DELETE /api/tokens/:id - Revoke token
//! - POST /api/tokens/validate - Validate token (Deliverable 1.6)
//!
//! # Protocol 014 Compliance
//!
//! Token creation follows Protocol 014:
//! - JWT authentication required (user_id from claims, not request body)
//! - Rate limiting: 10 creates/min per user
//! - Token limit: Max 10 active tokens per user
//! - Audit logging for all operations
//!
//! # Backward Compatibility
//!
//! Supports legacy request format with user_id in request body for existing tests.
//! Once tests are migrated, legacy support can be removed.

mod shared;
mod handlers;

// Re-export shared types and state
pub use shared::{
  TokenState,
  CreateTokenRequest,
  UpdateTokenRequest,
  ValidateTokenRequest,
  CreateTokenResponse,
  TokenListItem,
  ValidateTokenResponse,
};

// Re-export all handler functions
pub use handlers::{
  create_token,
  list_tokens,
  get_token,
  update_token,
  rotate_token,
  revoke_token,
  validate_token,
};
