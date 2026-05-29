//! Authentication REST API endpoints - Protocol 007 Implementation
//!
//! **Status:** Specification
//! **Version:** 1.0.0
//! **Last Updated:** 2025-12-10
//!
//! REST API endpoints for User authentication and User Token lifecycle management.
//!
//! # Module Organization
//!
//! - `shared`: Authentication state, request/response types
//! - `handlers`: Authentication handlers (4 endpoints)
//! - `magic_link`: Magic-link sign-in handlers (Task 029 Slide 1)
//!
//! # Endpoints
//!
//! - POST /api/v1/auth/login - User login (email/password → User Token)
//! - POST /api/v1/auth/logout - User logout (invalidate User Token)
//! - POST /api/v1/auth/refresh - User Token refresh (extend expiration)
//! - POST /api/v1/auth/validate - User Token validation (check if valid)
//! - POST /api/v1/auth/magic-link/send - issue a one-time magic link (Task 029 Slide 1)
//! - POST /api/v1/auth/magic-link/verify - redeem a magic link, return a session
//!
//! # Token Types
//!
//! - **User Token (JWT)**: For Control Panel access (30 days)
//! - **NOT IC Token**: IC Tokens are for agents (see Protocol 005)
//!
//! # Security
//!
//! - JWT signed with HS256 (HMAC SHA-256)
//! - Password hashing with bcrypt (cost factor 12)
//! - Rate limiting: 5 attempts per 5 minutes per IP
//! - Token blacklisting for logout
//! - Account lockout after 10 failed attempts

mod handlers;
mod magic_link;
mod shared;

// Re-export shared types and state
pub use shared::{
  AuthState, ErrorDetail, ErrorResponse, LoginRequest, LoginResponse, RefreshResponse, UserInfo,
  ValidateResponse,
};

// Re-export all handler functions
pub use handlers::{login, logout, refresh, validate};
pub use magic_link::{magic_link_send, magic_link_verify};
