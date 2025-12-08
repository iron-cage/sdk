# Iron Testing Specification

**Version:** 0.2.0
**Status:** Initial scaffolding
**Layer:** 4 (Infrastructure)
**Date:** 2025-12-07

---

## Scope

**Responsibility:**
Provides testing utilities for Iron Cage agents including pytest fixtures, mock runtime, test data generators, and assertion helpers. Enables developers to write comprehensive tests for protected agents without requiring real LLM API calls, Control Panel instances, or expensive token consumption. Supports unit testing, integration testing, and property-based testing of agent code.

**In Scope:**
- Pytest fixtures (mock_runtime, mock_budget, mock_safety)
- Mock implementations (MockRuntime, MockLLM, MockBudgetTracker)
- Test data generators (generate_valid_agent_input, generate_pii_data, generate_cost_data)
- Assertion helpers (assert_budget_not_exceeded, assert_no_pii_leaked, assert_circuit_breaker_triggered)
- Test decorators (@with_mock_runtime, @with_budget_limit)
- Async test support (async fixtures, async mocks)
- Property-based testing utilities (Hypothesis strategies)
- Test result analyzers (cost analysis, safety analysis)

**Out of Scope:**
- Production runtime (use iron_runtime)
- Example implementations (use iron_examples)
- Integration with test runners (pytest is standard)
- Code coverage tools (use pytest-cov)
- Load testing (use locust or similar)
- Real LLM integration testing (responsibility of integration tests)

## Deployment Context

This module operates identically in both Pilot and Production modes as testing utilities for developers.

**See:** [docs/deployment_packages.md](../../docs/deployment_packages.md) § Deployment Modes for deployment architecture.

**This Module (iron_testing):**
- Testing utilities for developers writing Iron Cage tests
- Not deployed in any package (development/testing only)
- Provides mocks for iron_runtime components used in both modes

---

## Dependencies

**Required:**
- iron-cage >=0.1.0 (iron_runtime for type annotations)
- pytest >=7.0.0
- pytest-asyncio >=0.21.0
- Python 3.8+

**Optional:**
- hypothesis >=6.0.0 (for property-based testing)
- faker >=18.0.0 (for test data generation)

---

## API Contract

### Pytest Fixtures

```python
import pytest
from iron_testing import mock_runtime, mock_budget, mock_safety

def test_agent_with_budget(mock_runtime, mock_budget):
  """Test agent respects budget limit."""
  mock_budget.set_limit(10.0)

  agent = MyAgent(runtime=mock_runtime)
  result = agent.run("test input")

  assert mock_budget.spent_usd < 10.0
  assert result is not None
```

### Mock Implementations

```python
from iron_testing import MockRuntime, MockLLM, MockBudgetTracker

# Create mock runtime
runtime = MockRuntime()
runtime.set_llm_response("Mocked response")

# Create mock budget tracker
budget = MockBudgetTracker(limit_usd=10.0)
budget.record_cost(model="gpt-4", tokens=100)

# Create mock LLM
llm = MockLLM(responses=["response1", "response2"])
```

### Test Data Generators

```python
from iron_testing import generators

# Generate test inputs
valid_input = generators.generate_valid_agent_input()
pii_data = generators.generate_pii_data(types=["email", "ssn", "phone"])
cost_data = generators.generate_cost_data(models=["gpt-4", "gpt-3.5"])
```

### Assertion Helpers

```python
from iron_testing import assertions

# Assert budget not exceeded
assertions.assert_budget_not_exceeded(budget, limit=10.0)

# Assert no PII leaked
assertions.assert_no_pii_leaked(output, pii_types=["email", "ssn"])

# Assert circuit breaker triggered
assertions.assert_circuit_breaker_triggered(runtime, threshold=5)
```

---

## Architecture

### Module Structure

```
iron_testing/
├── __init__.py              # Main exports
├── fixtures/                # Pytest fixtures
│   ├── __init__.py
│   ├── runtime.py           # mock_runtime fixture
│   ├── budget.py            # mock_budget fixture
│   ├── safety.py            # mock_safety fixture
│   └── async_fixtures.py    # Async fixtures
├── mocks/                   # Mock implementations
│   ├── __init__.py
│   ├── runtime.py           # MockRuntime class
│   ├── llm.py               # MockLLM class
│   ├── budget.py            # MockBudgetTracker class
│   └── safety.py            # MockSafetyChecker class
├── generators/              # Test data generators
│   ├── __init__.py
│   ├── inputs.py            # Input data generators
│   ├── pii.py               # PII data generators
│   └── costs.py             # Cost data generators
├── assertions/              # Assertion helpers
│   ├── __init__.py
│   ├── budget.py            # Budget assertions
│   ├── safety.py            # Safety assertions
│   └── reliability.py       # Reliability assertions
└── strategies/              # Hypothesis strategies
    ├── __init__.py
    └── agent_strategies.py  # Property-based testing strategies
```

---

## Development Status

**Current Phase:** Initial scaffolding (v0.1.0)

**Completed:**
- ✅ Project structure created
- ✅ pyproject.toml configured with pytest dependencies
- ✅ Package scaffolding (fixtures subdirectory)

**Pending:**
- 📋 Pytest fixtures (mock_runtime, mock_budget, mock_safety)
- 📋 Mock implementations (MockRuntime, MockLLM, MockBudgetTracker)
- 📋 Test data generators (inputs, PII, costs)
- 📋 Assertion helpers (budget, safety, reliability)
- 📋 Test decorators (@with_mock_runtime, @with_budget_limit)
- 📋 Async test support
- 📋 Hypothesis strategies for property-based testing
- 📋 Documentation with usage examples
- 📋 Unit tests for testing utilities
- 📋 Integration test examples

---

## Non-Functional Requirements

### NFR1: Usability
- **NFR1.1:** Fixtures must be auto-discoverable by pytest (conftest.py or pytest plugin)
- **NFR1.2:** Mocks must match real runtime interface exactly
- **NFR1.3:** Clear error messages when assertions fail

### NFR2: Performance
- **NFR2.1:** Mock runtime overhead <100μs per call
- **NFR2.2:** Test data generation <10ms for typical datasets
- **NFR2.3:** No network calls in unit tests (all mocked)

### NFR3: Coverage
- **NFR3.1:** Fixtures for all runtime components (budget, safety, reliability, state)
- **NFR3.2:** Generators for all test data types (inputs, PII, costs, errors)
- **NFR3.3:** Assertions for all protection features

---

## Revision History

- **2025-12-07 (v0.2.0):** Added Deployment Context - clarify development/testing-only module
- **2025-12-07 (v0.1.0):** Initial scaffolding specification

**Next Milestone:** Implement core pytest fixtures (mock_runtime, mock_budget)
