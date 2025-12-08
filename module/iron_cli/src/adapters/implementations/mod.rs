//! Adapter implementations
//!
//! Multiple REAL implementations following the adapter pattern:
//! - InMemoryAdapter: Fast, for tests only (compile_error! guard enforced)
//! - HttpAdapter: Production API client
//!
//! Note: Integration tests (tests/ directory) require feature="test-adapter"
//! because they compile as separate crates without automatic cfg(test).
//! The compile_error! in in_memory.rs prevents production use.

pub mod http;

#[ cfg( any( test, feature = "test-adapter" ) ) ]
pub mod in_memory;

pub use http::HttpAdapter;

#[ cfg( any( test, feature = "test-adapter" ) ) ]
pub use in_memory::InMemoryAdapter;
