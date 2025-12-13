//! LLM cost tracking and budget management for Iron Runtime.
//!
//! Provides precise cost calculation, budget enforcement, and pricing management
//! for LLM API requests. Uses integer arithmetic (microdollars) to avoid
//! floating-point precision errors in financial calculations.
//!
//! # Purpose
//!
//! This crate handles all cost-related operations for Iron Runtime's LLM proxy:
//! - Pre-request budget validation and reservation
//! - Real-time cost tracking during streaming
//! - Post-request cost reconciliation
//! - Pricing lookup from LiteLLM database
//! - Token estimation for cost prediction
//!
//! # Precision Guarantees
//!
//! All monetary values use **microdollars** (1 USD = 1,000,000 micros) stored as `u64`.
//! Per-token costs are stored as "microdollars per million tokens" to maintain integer
//! precision for small values:
//!
//! ```text
//! Example: GPT-4 input cost
//! $0.00003/token = $30/M tokens = 30,000,000 micros/M tokens
//! ```
//!
//! This approach eliminates floating-point rounding errors in cost calculations
//! while maintaining precision to fractions of a cent.
//!
//! # Key Types
//!
//! - [`budget::CostController`] - Thread-safe budget management with atomic operations
//! - [`budget::Reservation`] - RAII handle for reserved budget (commit or auto-cancel)
//! - [`pricing::PricingManager`] - Lock-free pricing database from LiteLLM JSON
//! - [`pricing::Model`] - Per-model pricing with max token limits
//! - [`converter`] - Conversion utilities between USD and microdollars
//! - [`token_estimation`] - Estimate input tokens from request JSON
//!
//! # Public API
//!
//! ## Budget Management
//!
//! ```rust,no_run
//! use iron_cost::budget::CostController;
//!
//! // Create controller with $10 budget (10 million microdollars)
//! let controller = CostController::new(10_000_000);
//!
//! // Reserve budget for max possible cost ($0.50 = 500k microdollars)
//! let reservation = controller.reserve(500_000)?;
//!
//! // After actual usage known, commit actual cost ($0.32 = 320k microdollars)
//! controller.commit(reservation, 320_000);
//! // Unused $0.18 automatically returned to budget
//! # Ok::<(), iron_cost::error::CostError>(())
//! ```
//!
//! ## Pricing Lookup
//!
//! ```rust,no_run
//! use iron_cost::pricing::PricingManager;
//!
//! let pricing = PricingManager::new()?;
//!
//! if let Some(model) = pricing.get("gpt-4") {
//!   let cost = model.calculate_cost(1000, 500); // input/output tokens
//!   println!("Cost: ${:.6}", cost);
//! }
//! # Ok::<(), String>(())
//! ```
//!
//! ## Request Cost Estimation
//!
//! ```rust,no_run
//! use iron_cost::pricing::PricingManager;
//!
//! let pricing = PricingManager::new()?;
//! let request_body = br#"{"model": "gpt-4", "messages": [...]}"#;
//!
//! // Estimate max cost based on max_tokens in request
//! if let Some(max_cost_micros) = pricing.estimate_max_cost(request_body) {
//!   println!("Max cost: {} microdollars", max_cost_micros);
//! }
//! # Ok::<(), String>(())
//! ```
//!
//! # Feature Flags
//!
//! - `budget-client` - Enable HTTP client for centralized budget service
//!
//! # Architecture
//!
//! Cost tracking follows a three-phase workflow:
//!
//! 1. **Pre-Request**: Estimate max cost, reserve budget atomically
//! 2. **Streaming**: Track actual token usage in real-time
//! 3. **Post-Request**: Commit actual cost, return unused reservation
//!
//! Budget operations use atomic compare-and-swap to prevent race conditions
//! in concurrent request handling. See [`budget::CostController`] for TOCTOU
//! race prevention strategies.

pub mod budget;
#[cfg(feature = "budget-client")]
pub mod budget_client;
pub mod converter;
pub mod error;
pub mod token_estimation;
pub mod pricing;
