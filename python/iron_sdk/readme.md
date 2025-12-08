# Iron SDK

Pythonic SDK layer for Iron Cage agent protection with decorators and framework integrations.

**Status:** Initial scaffolding (v0.1.0)
**Layer:** 5 (Integration)
**License:** Apache-2.0

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

See `iron_examples` package for 20+ runnable examples.

---

## Documentation

- **Specification:** See `spec.md` for complete technical requirements
- **API Reference:** Coming soon
- **Examples:** See `module/iron_examples/` for runnable examples

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
