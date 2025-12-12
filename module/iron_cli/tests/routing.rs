//! Routing correctness tests
//!
//! This file serves as the entry point for routing correctness tests.
//! Individual routing test modules are in routing/ subdirectory.
//!
//! Verifies that all CLI commands route to correct adapter functions
//! and that no routes call orphaned adapters (adapters without valid API endpoints).

mod routing
{
  pub mod correctness;
}
