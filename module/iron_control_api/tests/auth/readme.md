# auth/ - Authentication API Tests

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `content_type.rs` | Content-Type header validation tests |
| `http_methods.rs` | HTTP method validation tests |
| `login.rs` | Login endpoint integration tests |
| `malformed_json.rs` | Malformed JSON payload handling tests |
| `refresh_token_rotation.rs` | Refresh token rotation security tests |
| `security.rs` | Security audit logging and basic rate limiting tests |
| `authorization_bypass_comprehensive.rs` | Authorization bypass prevention (vertical/horizontal escalation, IDOR, RBAC) |
| `security_comprehensive.rs` | Comprehensive security tests (brute force, timing, JWT, sessions) |
| `sql_injection_comprehensive.rs` | Comprehensive SQL injection tests (30+ attack vectors) |
| `user_name_field.rs` | Username field validation tests |
| `validation.rs` | Auth request validation (LoginRequest, RefreshRequest, LogoutRequest) |

## Directory Purpose

Tests for JWT authentication API endpoints. Covers login/refresh/logout flows, request validation, HTTP protocol compliance, and error handling for authentication operations.
