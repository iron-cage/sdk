# routes

## Responsibility

REST API route handlers for all iron_control_api HTTP endpoints.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `agents.rs` | Handles agent lifecycle endpoints |
| `agent_provider_key.rs` | Fetches provider keys for agents via IC Token authentication |
| `analytics.rs` | Handles analytics data endpoints (Protocol 012) |
| `auth.rs` | Handles authentication endpoints (Protocol 007) |
| `budget/` | Handles budget control endpoints (Protocol 005) |
| `health.rs` | Implements health check endpoint returning server status |
| `keys.rs` | Fetches decrypted AI provider keys by token |
| `limits.rs` | Handles usage limit CRUD endpoints |
| `mod.rs` | Declares and re-exports all route handler modules |
| `providers.rs` | Manage AI provider key CRUD operations |
| `tokens.rs` | Handles API token management endpoints |
| `traces.rs` | Handles call tracing query endpoints |
| `usage.rs` | Handles usage analytics endpoints |
| `users.rs` | Handles user account management endpoints |
| `version.rs` | API version and build metadata endpoint |
