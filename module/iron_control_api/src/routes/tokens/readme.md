# tokens/ - Token Management REST API

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `handlers.rs` | Token lifecycle handlers (7 endpoints) |
| `mod.rs` | Export token management module public API |
| `shared.rs` | Token state, request/response types, validation logic |

## Directory Purpose

Token management REST API implementation (Protocol 014 compliant). Provides token CRUD operations, rotation, and validation. Uses JWT authentication, rate limiting (10 creates/min per user), and enforces max 10 active tokens per user.

## Endpoints

- POST /api/tokens - Create new API token
- GET /api/tokens - List all tokens for user
- GET /api/tokens/:id - Get specific token details
- POST /api/tokens/:id/update - Update token provider
- POST /api/tokens/:id/rotate - Rotate token (generate new value)
- DELETE /api/tokens/:id - Revoke token
- POST /api/tokens/validate - Validate token (Deliverable 1.6)
