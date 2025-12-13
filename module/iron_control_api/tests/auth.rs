//! Authentication domain tests
//!
//! This module contains all authentication-related tests organized by subdomain.

#[ path = "common/mod.rs" ]
mod common;

#[ path = "auth/login.rs" ]
mod login;

#[ path = "auth/validation.rs" ]
mod validation;

#[ path = "auth/malformed_json.rs" ]
mod malformed_json;

#[ path = "auth/http_methods.rs" ]
mod http_methods;

#[ path = "auth/content_type.rs" ]
mod content_type;

#[ path = "auth/security.rs" ]
mod security;
