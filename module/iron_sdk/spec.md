# iron_sdk - Specification

**Module:** iron_sdk (source folder)
**PyPI Package:** iron-cage
**Python Module:** iron_cage
**Layer:** 5 (Integration)
**Status:** Active

---

## Responsibility

Python SDK for Iron Cage AI agent protection. Provides PyO3 bindings for `iron_runtime`, exposing LlmRouter and Runtime to Python agents. Enables budget tracking, safety controls, and LLM API proxying for Python AI frameworks (LangChain, CrewAI, AutoGPT).

---

## Installation

```bash
uv pip install iron-cage
```

```python
from iron_cage import LlmRouter

with LlmRouter(provider_key="sk-xxx", budget=10.0) as router:
    client = OpenAI(base_url=router.base_url, api_key=router.api_key)
    response = client.chat.completions.create(
        model="gpt-4",
        messages=[{"role": "user", "content": "Hello!"}]
    )
```

---

## Scope

**In Scope:**
- PyO3 bindings for iron_runtime (LlmRouter, Runtime)
- Context manager support (`with LlmRouter(...) as router:`)
- Simple top-level imports (`from iron_cage import LlmRouter`)
- Type stubs for IDE support (`.pyi` files)
- Python tests and examples

**Out of Scope:**
- Core runtime functionality (see iron_runtime)
- OS-level sandboxing (see iron_sandbox)
- CLI functionality (see iron_cli_py)

---

## Dependencies

**Required Modules:**
- iron_runtime - Rust crate dependency (provides core LlmRouter, AgentRuntime)

**Required External:**
- pyo3 - Python bindings
- Python 3.9+

**Optional:**
- langchain - LangChain integration examples
- crewai - CrewAI integration examples
- autogpt - AutoGPT integration examples

---

## Core Concepts

**Key Components:**
- **LlmRouter:** PyO3 wrapper for iron_runtime::llm_router::LlmRouter
- **Runtime:** PyO3 wrapper for iron_runtime::AgentRuntime
- **Context Manager:** Automatic cleanup via `__enter__`/`__exit__`

---

## Integration Points

**Used by:**
- Python developers - AI agents using SDK

**Uses:**
- iron_runtime - Core Rust implementation (via Rust crate dependency)

---

## Testing

- Python tests: `tests/test_*.py`
- E2E tests: `tests/test_llm_router_e2e.py`, `tests/test_budget_e2e.py`
- Integration tests: `tests/test_analytics_*.py`

---

*For examples, see examples/ directory*
