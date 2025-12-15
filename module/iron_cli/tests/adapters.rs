//! Adapter integration tests
//!
//! This file serves as the entry point for all adapter tests.
//! Individual adapter test modules are in adapters/ subdirectory.
//!
//! Phase 4: Tests for adapter layer (unilang CLI → handlers → services)
//! Total: 110 adapter tests across 22 commands

mod adapters {
    pub mod auth_adapters_test;
    pub mod token_adapters_test;
    pub mod usage_adapters_test;
    pub mod limits_adapters_test;
    pub mod traces_adapters_test;
    pub mod health_adapters_test;
    pub mod coverage;
}
