# routes

## Responsibility

REST API route handlers for all iron_control_api HTTP endpoints.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `agents.rs` | Implements agent registration and management endpoints |
| `analytics.rs` | Implements analytics endpoints for event ingestion and spending queries (Protocol 012) |
| `auth.rs` | Implements authentication endpoints for login, logout, token refresh, validation (Protocol 007) |
| `budget.rs` | Implements Budget Control Protocol endpoints for handshake, usage reporting, budget refresh (Protocol 005) |
| `health.rs` | Implements health check endpoint returning server status |
| `keys.rs` | Implements endpoints for fetching decrypted AI provider keys by token |
| `limits.rs` | Implements CRUD endpoints for usage limit management |
| `mod.rs` | Declares and re-exports all route handler modules |
| `providers.rs` | Manage AI provider key CRUD operations |
| `tokens.rs` | Implements API token CRUD endpoints with rotation and revocation |
| `traces.rs` | Implements call tracing query endpoints for API request history |
| `usage.rs` | Implements usage analytics aggregation endpoints by project and provider |
| `users.rs` | Implements user management endpoints for account administration |
