# auth/ - Authentication API Tests

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `content_type.rs` | Content-Type header validation tests |
| `http_methods.rs` | HTTP method validation tests |
| `login.rs` | Login endpoint integration tests |
| `malformed_json.rs` | Malformed JSON payload handling tests |
| `validation.rs` | Auth request validation (LoginRequest, RefreshRequest, LogoutRequest) |

## Directory Purpose

Tests for JWT authentication API endpoints. Covers login/refresh/logout flows, request validation, HTTP protocol compliance, and error handling for authentication operations.
