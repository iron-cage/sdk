# src

## Responsibility

Core library implementation for iron_control_api REST API and WebSocket server.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `error.rs` | Define custom error types for API responses |
| `ic_token.rs` | Validate IC Token JWT claims (Protocol 005) |
| `ip_token.rs` | Encrypt provider API keys using AES-256-GCM |
| `jwt_auth.rs` | Authenticate requests using JWT tokens |
| `lib.rs` | Declare library public API exports |
| `rbac.rs` | Enforce role-based access control permissions |
| `token_auth.rs` | Authenticate external API access via tokens |
| `user_auth.rs` | Verify user passwords using bcrypt |

## Notes

This directory contains core authentication, authorization, and security primitives. Route handlers (in `routes/`) and middleware (in `middleware/`) build upon these modules.
