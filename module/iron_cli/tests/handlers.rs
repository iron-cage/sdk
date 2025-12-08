//! Handler integration tests
//!
//! This file serves as the entry point for all handler tests.
//! Individual handler test modules are in handlers/ subdirectory.
//!
//! Total: 100 test cases across 6 handler categories

mod handlers {
    pub mod auth_handlers_test;
    pub mod token_handlers_test;
    pub mod usage_handlers_test;
    pub mod limits_handlers_test;
    pub mod traces_handlers_test;
    pub mod health_handlers_test;
}
