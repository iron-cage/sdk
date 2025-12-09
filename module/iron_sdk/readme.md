# Iron SDK

Pythonic SDK layer for Iron Cage agent protection with decorators and framework integrations.

### Scope

**Responsibilities:**
Provides a clean, Pythonic API for protecting AI agents with budget tracking, PII detection, and reliability patterns. Wraps the low-level PyO3 bindings from iron_runtime with decorators, context managers, and typed configurations for ergonomic Python usage.

**In Scope:**
- `@protect_agent` decorator for function-level protection
- Context managers (`with Budget(...)`, `with Protection(...)`)
- Typed configuration classes (BudgetConfig, SafetyConfig, ReliabilityConfig)
- Framework integrations (LangChain, CrewAI, AutoGPT)
- Async/await support for async agents
- Error handling with Python exceptions

**Out of Scope:**
- PyO3 FFI bindings (see iron_runtime)
- Budget calculation logic (see iron_cost)
- PII detection patterns (see iron_safety)
- Circuit breaker implementation (see iron_reliability)

---

## Overview

Iron SDK provides a clean, Pythonic API for protecting AI agents with budget tracking, PII detection, and reliability patterns. Instead of working with raw PyO3 bindings, developers use decorators, context managers, and typed configurations.

**Key Features:**
- `@protect_agent` decorator for function-level protection
- Context managers (`with Budget(...)`, `with Protection(...)`)
- Framework integrations (LangChain, CrewAI, AutoGPT)
- Typed configuration classes
- Async/await support

---

## Quick Start

```python
from iron_sdk import protect_agent, BudgetConfig, SafetyConfig

@protect_agent(
  budget=BudgetConfig(max_usd=50.0),
  safety=SafetyConfig(pii_detection=True),
)
def my_agent(input: str) -> str:
  # Your agent code here
  return llm.generate(input)
```

---

## Installation

```bash
pip install iron-sdk
```

**Requirements:**
- Python 3.8+
- iron-cage >=0.1.0 (automatically installed)

**Optional dependencies:**
```bash
# LangChain integration
pip install iron-sdk[langchain]

# CrewAI integration
pip install iron-sdk[crewai]

# AutoGPT integration
pip install iron-sdk[autogpt]

# All integrations
pip install iron-sdk[all]
```

---

## Examples

See `examples/` directory for 20+ runnable examples:
- `examples/langchain/` - LangChain integration examples
- `examples/crewai/` - CrewAI integration examples
- `examples/autogpt/` - AutoGPT integration examples
- `examples/patterns/` - Protection pattern examples
- `examples/raw_api/` - Direct API usage examples

Run examples:
```bash
python examples/langchain/simple_chat.py
```

---

## Documentation

- **Specification:** See `spec.md` for complete technical requirements
- **API Reference:** Coming soon
- **Examples:** See `examples/` directory for runnable examples

---

## Development Status

**Current Phase:** Initial scaffolding

**Pending Implementation:**
- Core decorator (@protect_agent)
- Context managers (Budget, Protection)
- Configuration classes
- Framework integrations (LangChain, CrewAI, AutoGPT)

---

## License

Apache-2.0 - See `license` file for details
