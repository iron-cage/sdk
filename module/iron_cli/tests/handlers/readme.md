# Handler Tests

Tests for pure handler business logic validation.

## Responsibility

Test handler functions in isolation with no I/O, verifying business logic correctness.

## Files

| File | Responsibility |
|------|----------------|
| auth_handlers_test.rs | Test authentication handler logic |
| token_handlers_test.rs | Test token management handler logic |
| usage_handlers_test.rs | Test usage tracking handler logic |
| limits_handlers_test.rs | Test rate limiting handler logic |
| traces_handlers_test.rs | Test trace management handler logic |
| health_handlers_test.rs | Test health check handler logic |

## Test Strategy

**Pure Function Testing:**
- Input: `HashMap<String, String>` (simulated CLI arguments)
- Output: `Result<String, CliError>`
- No I/O operations (no mocking needed)

**Coverage:**
- Success paths with valid inputs
- Error conditions with invalid/missing inputs
- Edge cases and boundary conditions

## Test Organization

Tests organized by handler category matching src/handlers/ structure:
- Authentication (login, refresh, logout)
- Token operations (create, list, revoke, show, rotate)
- Usage tracking (show, by_project, by_provider, export)
- Rate limits (list, show, create, update, delete)
- Traces (list, show, export)
- Health (health, status)

## Running Tests

```bash
# Run all handler tests
cargo test --package iron_cli handlers::

# Run specific handler category
cargo test --package iron_cli auth_handlers_test
cargo test --package iron_cli token_handlers_test

# Run specific test
cargo test test_login_handler_success
```

## Test Principles

- **No I/O:** Handlers are pure functions, no mocking required
- **Complete coverage:** Test all success paths and error conditions
- **Isolated:** Each handler tested independently
- **Domain-focused:** Organized by business domain, not test type
