//! Iron CLI Library
//!
//! Unilang-based CLI for LLM token management.
//!
//! ## Architecture
//!
//! - Handlers: Pure business logic (no I/O)
//! - Adapters: Async I/O bridge
//! - Services: Service trait definitions
//! - Formatters: Universal output formatting
//! - Config: Hierarchical configuration system

pub mod handlers;
pub mod formatting;
pub mod adapters;
pub mod config;
