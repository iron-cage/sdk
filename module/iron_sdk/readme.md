# Iron Cage

[![Python](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org/downloads/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](license)

Python SDK for Iron Cage AI agent protection. Provides LlmRouter and Runtime for protecting AI agents with budget tracking, safety controls, and LLM API proxying.

## Installation

```bash
uv pip install iron-cage
```

> [!IMPORTANT]
> **Requirements:** Python 3.9+ (`python --version`)


## Quick Start

```python
from iron_cage import LlmRouter
from openai import OpenAI

# Use with Iron Cage server
with LlmRouter(api_key="ic_xxx", server_url="https://api.iron-cage.io") as router:
    client = OpenAI(base_url=router.base_url, api_key=router.api_key)
    response = client.chat.completions.create(
        model="gpt-4",
        messages=[{"role": "user", "content": "Hello!"}]
    )
    print(response.choices[0].message.content)

# Or with direct provider key (for testing)
with LlmRouter(provider_key="sk-xxx", budget=10.0) as router:
    client = OpenAI(base_url=router.base_url, api_key=router.api_key)
    # ... use client
```


## Architecture

![Iron Cage Architecture - Three-Boundary Model](https://raw.githubusercontent.com/Wandalen/iron_runtime/master/asset/architecture3_1k.webp)

Iron Cage uses a two-crate architecture:

| Crate | Language | Purpose |
|-------|----------|---------|
| `iron_runtime` | Pure Rust | Core implementation - LlmRouter, AgentRuntime, policies |
| `iron_sdk` (folder) | Rust + Python | PyO3 bindings exposing iron_runtime to Python |

**Package Hierarchy:**
```
What you install:  uv pip install iron-cage
What you import:   from iron_cage import LlmRouter, Runtime
Internal:          iron_runtime (Rust crate, linked at compile time)
```


## Key Features

- **LLM Proxy**: Local HTTP proxy that intercepts OpenAI/Anthropic API requests
- **Budget Control**: Set and track spending limits in USD
- **Auto-detection**: Automatically detects provider from API key format
- **Context Manager**: Clean resource management with `with` statement
- **Type Stubs**: Full IDE support with `.pyi` files


## API Reference

### LlmRouter

```python
LlmRouter(
    api_key: str = None,           # Iron Cage API token
    server_url: str = None,        # Iron Cage server URL
    cache_ttl_seconds: int = 300,  # API key cache TTL
    budget: float = None,          # Budget limit in USD
    provider_key: str = None,      # Direct provider API key
)
```

**Properties:**
- `base_url` - URL for OpenAI client (e.g., "http://127.0.0.1:52431/v1")
- `api_key` - API key to use with client
- `port` - Port the proxy is listening on
- `provider` - Detected provider ("openai" or "anthropic")
- `is_running` - Whether proxy is running
- `budget` - Current budget limit in USD
- `budget_status` - Tuple of (spent, limit) in USD

**Methods:**
- `total_spent()` - Get total spent in USD
- `set_budget(amount_usd)` - Set budget limit
- `stop()` - Stop the proxy server

### Runtime

```python
Runtime(
    budget: float,           # Budget limit in USD
    verbose: bool = False,   # Enable verbose logging
)
```

**Properties:**
- `budget` - Budget limit
- `verbose` - Verbose setting

**Methods:**
- `start_agent(script_path)` - Start an agent
- `stop_agent(agent_id)` - Stop an agent
- `get_metrics(agent_id)` - Get agent metrics as JSON


<details>
<summary>Optional Dependencies</summary>

```bash
# LangChain integration
uv pip install iron-cage[langchain]

# CrewAI integration
uv pip install iron-cage[crewai]

# All integrations
uv pip install iron-cage[all]

# Examples dependencies
uv pip install iron-cage[examples]
```

</details>


<details>
<summary>Examples</summary>

See `examples/` directory for runnable examples:
- `examples/langchain/` - LangChain integration examples
- `examples/crewai/` - CrewAI integration examples
- `examples/raw_api/` - Direct API usage examples
- `examples/patterns/` - Protection pattern examples

Run examples:
```bash
python examples/lead_gen_agent.py
```

</details>


## Development

```bash
# Build the Python package
cd module/iron_sdk
maturin develop

# Run tests
pytest tests/
```


## Documentation

- **Specification:** See `spec.md` for complete technical requirements
- **Examples:** See `examples/` directory


## License

Apache-2.0 - See `license` file for details
