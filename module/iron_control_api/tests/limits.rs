//! Limits domain tests
//!
//! This module contains all limits-related tests organized by subdomain.

#[ path = "common/mod.rs" ]
mod common;

#[ path = "limits/validation.rs" ]
mod validation;

#[ path = "limits/endpoints.rs" ]
mod endpoints;

#[ path = "limits/invalid_id.rs" ]
mod invalid_id;

#[ path = "limits/malformed_json.rs" ]
mod malformed_json;

#[ path = "limits/http_methods.rs" ]
mod http_methods;

#[ path = "limits/content_type.rs" ]
mod content_type;

#[ path = "limits/idempotency.rs" ]
mod idempotency;

#[ path = "limits/empty_body.rs" ]
mod empty_body;
