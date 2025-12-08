//! Tokens domain tests
//!
//! This module contains all token-related tests organized by subdomain.

#[ path = "common/mod.rs" ]
mod common;

#[ path = "tokens/endpoints.rs" ]
mod endpoints;

#[ path = "tokens/validation.rs" ]
mod validation;

#[ path = "tokens/security.rs" ]
mod security;

#[ path = "tokens/corner_cases.rs" ]
mod corner_cases;

#[ path = "tokens/state_transitions.rs" ]
mod state_transitions;

#[ path = "tokens/concurrency.rs" ]
mod concurrency;

#[ path = "tokens/malformed_json.rs" ]
mod malformed_json;

#[ path = "tokens/http_methods.rs" ]
mod http_methods;

#[ path = "tokens/content_type.rs" ]
mod content_type;

#[ path = "tokens/idempotency.rs" ]
mod idempotency;

#[ path = "tokens/empty_body.rs" ]
mod empty_body;
