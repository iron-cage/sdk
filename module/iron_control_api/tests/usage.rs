//! Usage domain tests.
//!
//! This module contains all usage-related tests organized by endpoint.
//! Tests verify FR-8: Usage Analytics API implementation.

#[ path = "common/mod.rs" ]
mod common;

#[ path = "usage/aggregate.rs" ]
mod aggregate;

#[ path = "usage/by_project.rs" ]
mod by_project;

#[ path = "usage/by_provider.rs" ]
mod by_provider;

#[ path = "usage/path_validation.rs" ]
mod path_validation;

#[ path = "usage/persistence.rs" ]
mod persistence;
