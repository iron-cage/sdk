# Tests

Tests for iron_sdk Python SDK.

## Status

Currently contains placeholder test infrastructure. Test implementation pending.

## Responsibility Table

| File | Responsibility | Notes |
|------|----------------|-------|
| `test_context.py` | Test context management and agent lifecycle | Implementation pending |
| `test_decorators.py` | Test decorator functionality and API wrapping | Implementation pending |
| `test_integrations.py` | Test end-to-end integration with Control Panel | Implementation pending |

## Test Categories

- **Unit Tests:** Individual SDK component validation
- **Integration Tests:** SDK + Control Panel integration
- **API Tests:** REST API client functionality

## Running Tests

```bash
# All tests
pytest

# Specific test file
pytest tests/test_context.py

# With coverage
pytest --cov=iron_sdk --cov-report=html

# Verbose output
pytest -v
```

## Test Data

- Mock API responses
- Test agent configurations
- Fixture data for integration tests
