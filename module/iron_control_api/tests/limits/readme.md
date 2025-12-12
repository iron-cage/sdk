# limits/ - Budget Limits Management Tests

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `content_type.rs` | Content-Type header validation (415 errors) |
| `empty_body.rs` | Empty request body rejection (422 errors) |
| `endpoints.rs` | CRUD endpoint integration tests |
| `http_methods.rs` | HTTP method validation (405 errors) |
| `idempotency.rs` | DELETE operation idempotency verification |
| `invalid_id.rs` | Invalid ID parameter handling (JSON errors) |
| `malformed_json.rs` | Malformed JSON payload handling (400 errors) |
| `validation.rs` | CreateLimitRequest input validation |

## Directory Purpose

Tests for budget limits management API endpoints (FR-9). Covers CRUD operations, input validation, HTTP protocol compliance, and error handling for user/project spending limits.
