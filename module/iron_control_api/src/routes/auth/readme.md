# auth/ - Authentication REST API (Protocol 007)

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `handlers.rs` | Authentication endpoint handlers (4 endpoints) |
| `mod.rs` | Export authentication module public API |
| `shared.rs` | Authentication state and request/response types |

## Directory Purpose

Authentication REST API implementation (Protocol 007). Provides User login/logout, User Token refresh, and validation. Uses JWT (HS256) with bcrypt password hashing, rate limiting, and token blacklisting.

## Endpoints

- POST /api/v1/auth/login - User login (email/password → User Token)
- POST /api/v1/auth/logout - User logout (invalidate User Token)
- POST /api/v1/auth/refresh - User Token refresh (extend expiration)
- POST /api/v1/auth/validate - User Token validation (check if valid)
