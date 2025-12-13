# Tests

Tests for iron_sdk Python SDK.

## Organization

| File | Responsibility |
|------|----------------|
| test_context.py | Context management and agent lifecycle tests |
| test_decorators.py | Decorator functionality and API wrapping tests |
| test_integrations.py | End-to-end integration tests with Control Panel |

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
