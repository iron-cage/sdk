# Iron Testing

Testing utilities for Iron Cage agents with pytest fixtures, mocks, and assertion helpers.

**Status:** Initial scaffolding (v0.1.0)
**Layer:** 4 (Infrastructure)
**License:** Apache-2.0

---

## Overview

Iron Testing provides comprehensive testing utilities for Iron Cage agents, enabling developers to write unit and integration tests without requiring real LLM API calls or expensive token consumption. Includes pytest fixtures, mock implementations, test data generators, and assertion helpers.

**Key Features:**
- **Pytest Fixtures** - Auto-discoverable fixtures for runtime, budget, safety
- **Mock Implementations** - MockRuntime, MockLLM, MockBudgetTracker
- **Test Data Generators** - Generate valid inputs, PII data, cost data
- **Assertion Helpers** - Budget, safety, and reliability assertions
- **Async Support** - Async fixtures and mocks for async agents

---

## Quick Start

```python
import pytest
from iron_testing import mock_runtime, mock_budget

def test_agent_with_budget(mock_runtime, mock_budget):
  """Test agent respects budget limit."""
  mock_budget.set_limit(10.0)

  agent = MyAgent(runtime=mock_runtime)
  result = agent.run("test input")

  assert mock_budget.spent_usd < 10.0
  assert result is not None
```

---

## Installation

```bash
# Base package
pip install iron-testing

# With property-based testing
pip install iron-testing[hypothesis]

# With test data generation
pip install iron-testing[faker]
```

**Requirements:**
- Python 3.8+
- pytest >=7.0.0
- pytest-asyncio >=0.21.0

---

## Available Utilities

### Pytest Fixtures

- `mock_runtime` - Mock Iron Cage runtime
- `mock_budget` - Mock budget tracker
- `mock_safety` - Mock safety checker
- `mock_llm` - Mock LLM for testing agents

### Mock Implementations

- `MockRuntime` - Simulates runtime behavior
- `MockLLM` - Simulates LLM responses
- `MockBudgetTracker` - Tracks test costs
- `MockSafetyChecker` - Simulates PII detection

### Test Data Generators

- `generate_valid_agent_input()` - Valid agent inputs
- `generate_pii_data()` - PII test data (email, SSN, phone)
- `generate_cost_data()` - Cost tracking test data

### Assertion Helpers

- `assert_budget_not_exceeded()` - Budget limit assertions
- `assert_no_pii_leaked()` - PII detection assertions
- `assert_circuit_breaker_triggered()` - Reliability assertions

---

## Usage Examples

```python
from iron_testing import MockRuntime, generators, assertions

# Create mock runtime
runtime = MockRuntime()
runtime.set_llm_response("Mocked response")

# Generate test data
pii_data = generators.generate_pii_data(types=["email", "ssn"])
valid_input = generators.generate_valid_agent_input()

# Run tests with assertions
result = agent.run(valid_input)
assertions.assert_no_pii_leaked(result, pii_types=["email", "ssn"])
```

---

## Documentation

- **Specification:** See `spec.md` for complete requirements
- **API Reference:** Coming soon
- **Testing Patterns:** See examples in iron_examples

---

## Development Status

**Current Phase:** Initial scaffolding

**Pending Implementation:**
- Pytest fixtures (runtime, budget, safety)
- Mock implementations
- Test data generators
- Assertion helpers
- Async test support
- Hypothesis strategies

---

## License

Apache-2.0 - See `license` file for details
