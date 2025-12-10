//! Traces domain tests.
//!
//! This module contains all traces-related tests organized by endpoint.
//! Tests verify FR-10: Request Traces API implementation.

#[ path = "common/mod.rs" ]
mod common;

#[ path = "traces/list.rs" ]
mod list;

#[ path = "traces/get_by_id.rs" ]
mod get_by_id;

#[ path = "traces/invalid_id.rs" ]
mod invalid_id;
