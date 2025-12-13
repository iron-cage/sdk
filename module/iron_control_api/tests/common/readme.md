# common/ - Shared Test Infrastructure

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `budget.rs` | Budget test infrastructure for Protocol 005 |
| `corner_cases.rs` | Security test vectors and attack patterns |
| `error_format.rs` | Error response format consistency tests |
| `fixtures.rs` | Test data factories for valid fixtures |
| `mod.rs` | Module declarations and shared test utilities |
| `test_db.rs` | Test database creation using iron_test_db |
| `test_state.rs` | Test state builders for Axum application |

## Directory Purpose

Shared test infrastructure used across all test domains. Provides real (non-mocked) test utilities including database setup, JWT generation, security test vectors, and reusable fixtures.
