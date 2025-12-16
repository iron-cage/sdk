# Iron SDK

Pythonic SDK layer for Iron Cage agent protection with decorators and framework integrations.

[![Documentation](https://img.shields.io/badge/docs-ironcage.ai-E5E7EB.svg)](https://ironcage.ai/docs)

> [!WARNING]
> **Development Status:** Initial scaffolding - Core features pending implementation


## Installation

```bash
pip install iron-sdk
```

> [!IMPORTANT]
> **Requirements:** Python 3.9+ (`python --version`)

**About Dependencies:**

The `iron-cage` package (containing the Rust runtime) is automatically installed as a dependency - you never need to install or interact with it directly.

**Package Hierarchy:**
```
What you install:  pip install iron-sdk
What you import:   from iron_sdk import protect_agent
Automatic (internal): iron-cage (Rust runtime, auto-installed)
Internal (never seen): iron_runtime (Rust crate)
```


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


## Architecture

![Iron Cage Architecture - Three-Boundary Model](https://raw.githubusercontent.com/Wandalen/iron_runtime/master/asset/architecture3_1k.webp)

**Visual Guide:**
- **Left (Developer Zone):** Agent, iron_sdk, Runtime (Safety/Cost/Audit), Gateway - 100% local
- **Middle (Management Plane):** Control Panel - NOT in data path
- **Right (Provider Zone):** LLM provider receives only prompts with IP Token

See [root readme](../../readme.md) for detailed architecture explanation.


## Key Features

- `@protect_agent` decorator for function-level protection
- Context managers (`with Budget(...)`, `with Protection(...)`)
- Framework integrations (LangChain, CrewAI, AutoGPT)
- Typed configuration classes
- Async/await support


<details>
<summary>Optional Dependencies</summary>

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

</details>


<details>
<summary>Examples</summary>

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

</details>


<details>
<summary>Development Status & Roadmap</summary>

**Current Phase:** Initial scaffolding

**Pending Implementation:**
- Core decorator (@protect_agent)
- Context managers (Budget, Protection)
- Configuration classes
- Framework integrations (LangChain, CrewAI, AutoGPT)

</details>


<details>
<summary>Scope & Boundaries</summary>

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

</details>


## Documentation

- **Specification:** See `spec.md` for complete technical requirements
- **API Reference:** Coming soon
- **Examples:** See `examples/` directory for runnable examples


## License

Apache-2.0 - See `license` file for details
