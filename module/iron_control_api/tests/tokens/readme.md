# Token API Tests

Token management endpoint tests organized by test concern.

## Organization

Tests are organized by concern (validation, concurrency, security, etc.) rather than by endpoint, allowing focused test suites that validate specific quality attributes across all token endpoints.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `concurrency.rs` | Test concurrent token operations |
| `content_type.rs` | Test Content-Type header validation |
| `corner_cases.rs` | Test token creation corner cases |
| `empty_body.rs` | Test empty request body handling |
| `endpoints.rs` | Test token endpoint integration |
| `http_methods.rs` | Test HTTP method validation |
| `idempotency.rs` | Test token operation idempotency |
| `malformed_json.rs` | Test malformed JSON handling |
| `security.rs` | Test token security boundaries |
| `state_transitions.rs` | Test token state transitions |
| `validation.rs` | Test token request validation |

## Coverage

All tests validate Protocol 014 (API Tokens API) compliance with comprehensive coverage across:
- Input validation (validation.rs, content_type.rs, malformed_json.rs, empty_body.rs)
- Security (security.rs)
- Concurrency (concurrency.rs)
- State management (state_transitions.rs)
- Integration (endpoints.rs)
- Protocol compliance (http_methods.rs, idempotency.rs)
- Edge cases (corner_cases.rs)
