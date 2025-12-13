# Tests

Tests for iron_runtime LLM router and translation layer.

## Organization

| File | Responsibility |
|------|----------------|
| llm_router_test.rs | LLM router core functionality tests |
| llm_router_integration_test.rs | End-to-end router integration tests |
| llm_router_translator_request_test.rs | Request translation validation |
| llm_router_translator_response_test.rs | Response translation validation |
| runtime_test.rs | Runtime infrastructure tests |

## Test Categories

- **Unit Tests:** Individual component validation
- **Integration Tests:** Router + translator integration
- **Translation Tests:** Request/response format conversions

## Running Tests

```bash
# All tests
cargo nextest run

# Router tests only
cargo nextest run --test llm_router_test

# Translation tests
cargo nextest run --test llm_router_translator_request_test
cargo nextest run --test llm_router_translator_response_test

# Integration tests
cargo nextest run --test llm_router_integration_test
```

## Test Data

- Mock provider configurations in test fixtures
- Request/response translation test cases
