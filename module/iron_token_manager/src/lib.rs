//! # `iron_token_manager`
//!
//! Token management system for LLM inference provider API access control.
//!
//! Provides secure token generation, usage tracking, limit enforcement, and
//! call tracing for multi-tenant `SaaS` deployments of Iron Cage platform.
//!
//! ## Architecture
//!
//! This crate manages the full lifecycle of API tokens that customers use to
//! access Iron Cage services. It tracks which users/projects make which LLM
//! calls, enforces hard limits, and provides usage analytics.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use iron_token_manager::TokenManager;
//! 
//! let manager = TokenManager::new("db.sqlite").await?;
//!
//! // Generate token for user/project
//! let token = manager.generate_token("user_123", Some("project_456")).await?;
//!
//! // Track usage
//! manager.record_usage(&token.id, "openai", "gpt-4", 100, 50).await?;
//!
//! // Check limits
//! if manager.check_limit(&token.id).await? {
//!     // Allowed
//! } else {
//!     // Limit exceeded
//! }
//! ```
//!
//! ## Compliance
//!
//! Follows specification: `/home/user1/pro/lib/wip_iron/iron_cage/dev/task/backlog/001_implement_llm_token_management_dashboard_and_backend.md`
//!
//! ## Features
//!
//! - **`enabled`** (default) - Core token management functionality
//! - **`full`** - All features enabled
//!
//! ## Known Pitfalls
//!
//! ### Token Security
//!
//! **Issue**: Generated tokens must be cryptographically secure and never logged in plaintext.
//!
//! **Prevention**:
//! 1. Use `rand::thread_rng()` with proper seeding
//! 2. Store `BCrypt` hashes (cost=12), NEVER plaintext tokens
//! 3. Never log tokens in tracing output
//!
//! ### Hash Algorithm Choice (CRITICAL)
//!
//! **Issue**: Using fast hash algorithms (SHA-256, MD5) for token storage enables brute-force
//! attacks if database is compromised. SHA-256 is designed for speed (integrity checking),
//! not for secret hashing. GPU attacks can test billions of SHA-256 hashes per second.
//!
//! **Why Critical**: Database compromise (SQL injection, backup leak, insider threat) would
//! expose all active tokens, allowing attackers to impersonate any user indefinitely.
//!
//! **Prevention**:
//! 1. **ALWAYS use `BCrypt`/Argon2/scrypt for secrets** (adaptive work factor, intentionally slow)
//! 2. **NEVER use SHA-256/MD5/SHA-512 for passwords or tokens** (too fast, rainbow tables)
//! 3. `BCrypt` cost parameter >= 10 (cost=12 recommended as of 2025)
//! 4. Test hash format in database verification tests (see tests/tokens/corner_cases.rs:P0-9, P0-10)
//!
//! **History**: This vulnerability (issue-003d/e) was discovered via TDD test implementation
//! (`test_create_token_uses_bcrypt_hash`). Original implementation used `SHA-256`, migrated to
//! `BCrypt` in Phase 1. See `-layer6_retrofit_verification_report.md` for details.
//!
//! ### Rate Limiting Accuracy
//!
//! **Issue**: Token bucket algorithm may allow bursts slightly above configured rate.
//!
//! **Why**: Governor crate uses token bucket, which permits burst traffic up to bucket capacity.
//!
//! **Prevention**: Configure bucket size appropriately for use case.

#![cfg_attr(not(feature = "enabled"), allow(unused))]
#![warn(missing_docs)]

#[cfg(feature = "enabled")]
pub mod error;

#[cfg(feature = "enabled")]
pub mod token_generator;

#[cfg(feature = "enabled")]
pub mod usage_tracker;

#[cfg(feature = "enabled")]
pub mod limit_enforcer;

#[cfg(feature = "enabled")]
pub mod storage;

#[cfg(feature = "enabled")]
pub mod migrations;

#[cfg(feature = "enabled")]
pub mod config;

#[cfg(feature = "enabled")]
pub mod seed;

#[cfg(feature = "enabled")]
pub mod provider_adapter;

#[cfg(feature = "enabled")]
pub mod rate_limiter;

#[cfg(feature = "enabled")]
pub mod cost_calculator;

#[cfg(feature = "enabled")]
pub mod trace_storage;

#[cfg(feature = "enabled")]
pub mod provider_key_storage;

#[cfg(feature = "enabled")]
pub mod user_service;
