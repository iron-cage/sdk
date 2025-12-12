# common/ - Shared Test Infrastructure

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `corner_cases.rs` | Security test vectors and attack patterns |
| `database.rs` | Database test infrastructure and isolation |
| `error_format.rs` | Error response format consistency tests |
| `fixtures.rs` | Test data factories for valid fixtures |
| `mod.rs` | Module declarations and shared test utilities |
| `test_state.rs` | Test state builders for Axum application |

## Directory Purpose

Shared test infrastructure used across all test domains. Provides real (non-mocked) test utilities including database setup, JWT generation, security test vectors, and reusable fixtures.
