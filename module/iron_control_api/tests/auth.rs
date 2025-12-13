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

#[ path = "auth/user_name_field.rs" ]
mod user_name_field;

#[ path = "auth/refresh_token_rotation.rs" ]
mod refresh_token_rotation;

#[ path = "auth/security_comprehensive.rs" ]
mod security_comprehensive;

#[ path = "auth/sql_injection_comprehensive.rs" ]
mod sql_injection_comprehensive;

#[ path = "auth/authorization_bypass_comprehensive.rs" ]
mod authorization_bypass_comprehensive;
