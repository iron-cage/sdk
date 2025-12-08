//! Adapter layer for unilang CLI
//!
//! ## Architecture
//!
//! Adapters bridge the CLI layer to business logic handlers and async services:
//!
//! ```text
//! CLI (unilang) → Adapter (async I/O) → Handler (pure logic) → Formatter (output)
//! ```
//!
//! ## Responsibilities
//!
//! 1. Extract parameters from VerifiedCommand
//! 2. Load data from services (async I/O)
//! 3. Call pure handler functions (validation)
//! 4. Perform state changes (if not dry-run)
//! 5. Format output using Formatter
//!
//! ## No-Mocking Strategy
//!
//! Adapters use the adapter pattern with two REAL implementations:
//! - InMemoryAdapter: Fast, predictable, for tests
//! - SqlxAdapter: Real PostgreSQL, for production
//!
//! ## Phase 4 Implementation
//!
//! Status: GREEN phase (implementing adapters and InMemoryAdapter)
//! Progress: Auth adapters (3/22), InMemoryAdapter complete

pub mod error;
pub mod services;
pub mod implementations;
pub mod auth;
pub mod tokens;
pub mod usage;
pub mod limits;
pub mod traces;
pub mod health;

pub use error::{ AdapterError, ServiceError };
pub use services::{ AuthService, TokenService, UsageService, LimitsService, TracesService, HealthService, StorageService, Services, Tokens, Token, UsageRecord, Limit, Trace, HealthStatus };
