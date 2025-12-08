# Iron SDK Specification

**Version:** 0.2.0
**Status:** Initial scaffolding
**Layer:** 5 (Integration)
**Date:** 2025-12-07

---

## Scope

**Responsibility:**
Provides Pythonic SDK layer for Iron Cage agent protection with decorators, context managers, and framework integrations (LangChain, CrewAI, AutoGPT). Wraps iron_runtime with ergonomic Python API including @protect_agent decorator, budget context managers, and typed configuration classes. Enables Python developers to add agent protection without learning Rust FFI details.

**In Scope:**
- @protect_agent decorator for function-level protection
- Context managers for resource management (with Budget(...), with Protection(...))
- LangChain integration (callbacks, chains, agents)
- CrewAI integration (crew protection, task wrapping)
- AutoGPT integration (plugin system, command wrapping)
- Typed configuration classes (BudgetConfig, SafetyConfig, ReliabilityConfig)
- Pythonic error handling with custom exception hierarchy
- Async/await support for async agents

**Out of Scope:**
- Core runtime functionality (use iron_runtime Rust crate)
- OS-level sandboxing (use iron_sandbox)
- Example implementations (use iron_examples)
- Testing utilities (use iron_testing)
- CLI functionality (use iron_cli_py)
- Direct PyO3 FFI (hidden behind Pythonic API)

## Deployment Context

Iron Cage supports two deployment modes. This module operates identically in both modes as it's part of Agent Runtime only.

**See:** [docs/deployment_packages.md](../../docs/deployment_packages.md) § Deployment Modes for deployment architecture.

**This Module (iron_sdk):**
- **Both Modes:** Part of Agent Runtime package (PyPI), runs on developer machines
- Wraps iron_runtime with Pythonic API (decorators, context managers)
- Not included in Control Panel package (dashboard uses REST API directly)

---

## Dependencies

**Required:**
- iron-cage >=0.1.0 (iron_runtime package from PyPI)
- Python 3.8+

**Optional:**
- langchain >=0.1.0 (for LangChain integration)
- crewai >=0.1.0 (for CrewAI integration)
- autogpt >=0.1.0 (for AutoGPT integration)

---

## API Contract

### Decorator API

```python
from iron_sdk import protect_agent, BudgetConfig, SafetyConfig

@protect_agent(
  budget=BudgetConfig(max_usd=50.0),
  safety=SafetyConfig(pii_detection=True),
)
def my_agent(input: str) -> str:
  # Agent implementation
  return result
```

### Context Manager API

```python
from iron_sdk import Budget, Protection

with Budget(max_usd=10.0) as budget:
  with Protection(pii_detection=True) as protection:
    result = agent.run(task)
    print(f"Cost: ${budget.spent_usd}")
```

### Framework Integration API

```python
# LangChain
from iron_sdk.langchain import IronCallbackHandler, ProtectedChain

callback = IronCallbackHandler(budget_usd=25.0)
chain = ProtectedChain(llm=llm, budget=callback)

# CrewAI
from iron_sdk.crewai import ProtectedCrew

crew = ProtectedCrew(
  agents=[agent1, agent2],
  budget_usd=100.0,
  pii_detection=True
)

# AutoGPT
from iron_sdk.autogpt import ProtectedPlugin

plugin = ProtectedPlugin(budget_usd=50.0)
```

---

## Architecture

### Module Structure

```
iron_sdk/
├── __init__.py           # Main exports: protect_agent, Budget, Protection
├── decorators.py         # @protect_agent decorator implementation
├── context_managers.py   # Budget, Protection context managers
├── config.py            # Configuration classes (BudgetConfig, SafetyConfig, etc.)
├── exceptions.py         # Custom exception hierarchy
├── langchain/           # LangChain integration
│   ├── __init__.py
│   ├── callbacks.py     # IronCallbackHandler
│   └── chains.py        # ProtectedChain
├── crewai/              # CrewAI integration
│   ├── __init__.py
│   └── crew.py          # ProtectedCrew
└── autogpt/             # AutoGPT integration
    ├── __init__.py
    └── plugin.py        # ProtectedPlugin
```

---

## Development Status

**Current Phase:** Initial scaffolding (v0.1.0)

**Completed:**
- ✅ Project structure created
- ✅ pyproject.toml configured with dependencies
- ✅ Package scaffolding (module directories)

**Pending:**
- 📋 Core decorator implementation (@protect_agent)
- 📋 Context manager implementations (Budget, Protection)
- 📋 Configuration classes (BudgetConfig, SafetyConfig, ReliabilityConfig)
- 📋 LangChain integration (callbacks, chains)
- 📋 CrewAI integration (crew protection)
- 📋 AutoGPT integration (plugin system)
- 📋 Exception hierarchy design
- 📋 Async/await support
- 📋 Unit tests with pytest
- 📋 Integration tests with real frameworks

---

## Non-Functional Requirements

### NFR1: Ergonomics
- **NFR1.1:** API must be Pythonic (follow PEP 8, PEP 484 type hints)
- **NFR1.2:** Zero boilerplate for common use cases (single decorator for full protection)
- **NFR1.3:** Clear error messages with actionable guidance

### NFR2: Performance
- **NFR2.1:** Decorator overhead <1ms per invocation
- **NFR2.2:** Context manager overhead <0.5ms per entry/exit

### NFR3: Compatibility
- **NFR3.1:** Python 3.8+ support
- **NFR3.2:** LangChain 0.1+ compatibility
- **NFR3.3:** CrewAI 0.1+ compatibility
- **NFR3.4:** AutoGPT 0.1+ compatibility

---

## Revision History

- **2025-12-07 (v0.2.0):** Added Deployment Context - clarify Agent Runtime-only module
- **2025-12-07 (v0.1.0):** Initial scaffolding specification

**Next Milestone:** Implement core decorator and context managers
