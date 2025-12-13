//! Parameter-level integration tests
//!
//! ## Purpose
//!
//! Validates all CLI parameters across all commands using real test infrastructure.
//! Tests parameter validation, format checking, and error handling end-to-end.
//!
//! ## Test Organization
//!
//! Each parameter has its own test module in `parameters/` directory:
//! - One module per parameter (e.g., `format_parameter_test.rs`)
//! - Each module tests the parameter across ALL commands that use it
//! - Cross-command consistency tests ensure uniform behavior
//!
//! ## Test Infrastructure
//!
//! All tests use real implementations (no mocking):
//! - TestServer: Real Axum HTTP server on random port
//! - TestData: Real SQLite database with SQL inserts
//! - IntegrationTestHarness: Real CLI binary execution via `cargo run`

// Test files are allowed to use println!/eprintln! for debugging
#![allow(clippy::disallowed_macros)]
//!
//! ## Coverage Target
//!
//! Total parameters: 250 across 69 commands
//! Minimum tests: 2,251 parameter-level validation tests
//!
//! Phase breakdown:
//! - Phase 0: Infrastructure (15 tests) ✅
//! - Phase 1: String parameters (850 tests target) 🔄
//! - Phase 2: Integer parameters (420 tests)
//! - Phase 3: Attribute tests (705 tests)
//! - Phase 4: Subtype tests (226 tests)

// Re-export fixtures for use in parameter tests
#[path = "fixtures/mod.rs"]
mod fixtures;

mod parameters;
