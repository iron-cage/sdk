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
//! 1. Extract parameters from `VerifiedCommand`
//! 2. Load data from services (async I/O)
//! 3. Call pure handler functions (validation)
//! 4. Perform state changes (if not dry-run)
//! 5. Format output using Formatter
//!
//! ## No-Mocking Strategy
//!
//! Adapters use the adapter pattern with two REAL implementations:
//! - `InMemoryAdapter`: Fast, predictable, for tests
//! - `SqlxAdapter`: Real `PostgreSQL`, for production
//!
//! ## Phase 4 Implementation
//!
//! Status: GREEN phase (implementing adapters and `InMemoryAdapter`)
//! Progress: Auth adapters (3/22), `InMemoryAdapter` complete

pub mod auth;
pub mod error;
pub mod health;
pub mod implementations;
pub mod limits;
pub mod services;
pub mod tokens;
pub mod traces;
pub mod usage;

// Control API adapters (for iron binary)
pub mod control;

// Token API adapters (for iron-token binary)
pub mod auth_adapters;
pub mod health_adapters;
pub mod keyring;
pub mod limits_adapters;
pub mod token;
pub mod token_adapters;
pub mod traces_adapters;
pub mod usage_adapters;

pub use error::{AdapterError, ServiceError};
pub use services::{
  AuthService, HealthService, HealthStatus, Limit, LimitsService, Services, StorageService, Token,
  TokenService, Tokens, Trace, TracesService, UsageRecord, UsageService,
};
