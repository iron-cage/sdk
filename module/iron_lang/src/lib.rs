//! # `iron_lang`
//!
//! Protocol for AI agents to safely access and manipulate data sources.
//!
//! This crate provides a type-safe, performant communication protocol that enables
//! AI agents (like Claude, GPT-4, or custom models) to read, write, and query data
//! from various sources including databases, filesystems, APIs, caches, and object storage.
//!
//! ## Features
//!
//! - **Type-safe protocol**: Rust types with compile-time guarantees
//! - **NDJSON transport**: Newline-Delimited JSON over STDIN/STDOUT
//! - **Multiple data sources**: SQL, files, HTTP, cache, object storage
//! - **Authentication**: Built-in auth with multiple credential types
//! - **Observability**: Logging and metrics built into protocol
//!
//! ## Usage
//!
//! ```rust,ignore
//! use iron_lang::protocol::{ IronMessage, ReadMessage, ReadOperation, SqlQuery };
//! use iron_lang::protocol::new_request_id;
//!
//! // Create a READ message for SQL query
//! let msg = IronMessage::Read( ReadMessage
//! {
//!   request_id : new_request_id(),
//!   source : "production_db".to_string(),
//!   operation : ReadOperation::Sql( SqlQuery
//!   {
//!     query : "SELECT * FROM users".to_string(),
//!     parameters : None,
//!   }),
//!   options : None,
//! });
//!
//! // Serialize to NDJSON
//! let json = serde_json::to_string( &msg )?;
//! ```
//!
//! ## Protocol Messages
//!
//! The protocol defines 9 message types:
//!
//! - **READ**: Request data from source (SQL, files, HTTP, cache, objects)
//! - **WRITE**: Write data to destination
//! - **QUERY**: Query metadata (tables, files, keys)
//! - **SCHEMA**: Request schema information
//! - **AUTH**: Authenticate agent
//! - **ACK**: Acknowledge successful operation
//! - **ERROR**: Report operation failure
//! - **LOG**: Diagnostic logging
//! - **METRICS**: Performance metrics
//!
//! ## Architecture
//!
//! ```text
//! Agent                     Runtime                    Connector
//!   |                          |                          |
//!   |---READ (SQL query)------>|                          |
//!   |                          |---execute_read()-------->|
//!   |                          |                   [query DB]
//!   |                          |<--rows-------------------|
//!   |<--ACK (result data)------|                          |
//! ```

#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(missing_docs)]
#![allow(clippy::doc_markdown)]

/// Protocol message type definitions and serialization.
///
/// Contains all message types for the IronLang protocol including READ, WRITE,
/// QUERY, SCHEMA, AUTH, ACK, ERROR, LOG, and METRICS messages.
#[cfg(feature = "enabled")]
pub mod protocol;

/// Message processing runtime engine.
///
/// Provides the core runtime for processing messages over STDIN/STDOUT,
/// managing request/response lifecycle, and coordinating with connectors.
#[cfg(feature = "enabled")]
pub mod runtime;

/// Connector trait and implementations.
///
/// Defines the Connector trait that all data source connectors must implement,
/// along with common connector utilities and connection pooling.
#[cfg(feature = "enabled")]
pub mod connectors;

/// Authentication and authorization.
///
/// Handles agent authentication, credential validation, session management,
/// and role-based access control (RBAC).
#[cfg(feature = "enabled")]
pub mod auth;

/// Request routing and dispatch.
///
/// Routes incoming requests to appropriate connectors based on source configuration,
/// manages connector lifecycle, and handles connection pooling.
#[cfg(feature = "enabled")]
pub mod router;
