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
| `test_attack_taxonomy.rs` | Verify SQL injection attack vector taxonomy |
| `test_endpoint_catalog.rs` | Verify endpoint catalog completeness and accuracy |
| `test_parameter_matrix.rs` | Verify parameter-to-payload mapping matrix |
| `test_skeleton_generator.rs` | Verify test skeleton generator creates correct structure |
| `test_sql_injection_helpers.rs` | Verify SQL injection helper functions work correctly |
| `test_sql_injection_standards.rs` | Verify SQL injection testing standards completeness |
| `-generate_test_skeletons.sh` | Generate 244 test skeleton files for Phase 2 implementation |

## Directory Purpose

Tests for JWT authentication API endpoints. Covers login/refresh/logout flows, request validation, HTTP protocol compliance, and error handling for authentication operations.
