# Tests

Tests for iron_runtime LLM router and translation layer.

## Responsibility Table

| File | Responsibility | Input→Output | Out of Scope |
|------|----------------|--------------|--------------|
| `llm_router_test.rs` | Test LLM Router provider detection and model parsing | Model/key strings → Provider detection | NOT integration (llm_router_integration_test.rs), NOT translation (translator tests) |
| `llm_router_integration_test.rs` | Test LLM Router end-to-end integration with Iron Cage server | Real API calls → Integration validation | NOT unit logic (llm_router_test.rs), NOT translation (translator tests), NOT security (pyo3_string_safety_test.rs) |
| `llm_router_translator_request_test.rs` | Test OpenAI to Anthropic request translation | OpenAI format → Anthropic format validation | NOT response translation (translator_response_test.rs), NOT routing (llm_router_test.rs) |
| `llm_router_translator_response_test.rs` | Test Anthropic to OpenAI response translation | Anthropic format → OpenAI format validation | NOT request translation (translator_request_test.rs), NOT routing (llm_router_test.rs) |
| `pyo3_string_safety_test.rs` | Test PyO3 buffer overflow vulnerability fix (RUSTSEC-2025-0020) | Security scenarios → Vulnerability prevention | NOT routing (llm_router_test.rs), NOT translation (translator tests) |
| `runtime_test.rs` | Test Runtime creation and configuration | Runtime setup → Initialization validation | NOT routing (llm_router_test.rs), NOT integration (llm_router_integration_test.rs) |

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
