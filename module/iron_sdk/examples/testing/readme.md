# testing/

Testing and simulation examples for Iron Cage integration.

---

## Responsibility Table

| Entity | Responsibility | Input ’ Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| `integration_test.py` | Demonstrate end-to-end integration testing | Test scenario ’ Integration test example | Complete workflow testing, API integration validation, multi-component interaction, test assertions | NOT unit testing (’ ../patterns/), NOT budget simulation (’ budget_simulation.py), NOT mock runtime (’ mock_runtime.py) |
| `budget_simulation.py` | Show budget limit simulation and testing | Budget test scenario ’ Simulation example | Budget limit testing, cost projection, over-budget scenarios, circuit breaker triggering | NOT integration testing (’ integration_test.py), NOT mock runtime (’ mock_runtime.py), NOT real examples (’ ../langchain/, ../crewai/) |
| `mock_runtime.py` | Illustrate mock runtime for local testing | Local testing need ’ Mock implementation example | Mock runtime setup, offline development, test doubles, local simulation without API calls | NOT integration testing (’ integration_test.py), NOT budget simulation (’ budget_simulation.py), NOT real runtime (’ module/iron_runtime/) |

---

## Running Examples

```bash
# Run integration test example
python integration_test.py

# Run budget simulation example
python budget_simulation.py

# Run mock runtime example
python mock_runtime.py
```

---

**Status:** Scaffolding (files created, implementation pending)
