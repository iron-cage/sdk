//! Test fixtures module
//!
//! Provides real test infrastructure (no mocking):
//! - TestServer: Real Axum HTTP server
//! - TestData: Real database fixtures
//! - IntegrationTestHarness: Real CLI execution

pub mod test_data;
pub mod test_harness;
pub mod test_server;

pub use test_data::TestData;
pub use test_harness::IntegrationTestHarness;
pub use test_server::TestServer;
