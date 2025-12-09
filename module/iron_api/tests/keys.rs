//! Keys API tests (FR-12: Key Fetch API).
//!
//! This module contains tests for the /api/keys endpoint.
//! Tests verify secure key retrieval with API token authentication.

#[ path = "common/mod.rs" ]
mod common;

#[ path = "keys/endpoints.rs" ]
mod endpoints;

#[ path = "keys/security.rs" ]
mod security;
