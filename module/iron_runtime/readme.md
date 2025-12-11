# iron_runtime

**Audience:** Platform contributors developing the iron_runtime Rust crate
**End Users:** See [iron_sdk documentation](../iron_sdk/readme.md) - just `pip install iron-sdk`

Agent orchestration and Python bridge for AI agent execution. Provides **LlmRouter** - a local proxy server for transparent LLM API key management with OpenAI and Anthropic support.

**Package Flow:** This Rust crate → builds to iron-cage PyPI wheel → auto-installed by iron-sdk

### Scope

**Responsibilities:**
Bridges Python AI agents with Rust-based safety, cost, and reliability infrastructure via PyO3. Provides LlmRouter for transparent API key management and request proxying. Manages agent lifecycle (spawn, monitor, shutdown), intercepts LLM calls for policy enforcement, coordinates tokio async runtime, and provides WebSocket server for real-time dashboard updates.

**In Scope:**
- Python-Rust FFI via PyO3 (agent execution bridge)
- LlmRouter - Local proxy for LLM API requests
- Multi-provider support (OpenAI, Anthropic) with auto-detection
- Agent lifecycle management (spawn, monitor, shutdown)
- LLM call interception and policy enforcement
- Tokio async runtime coordination
- WebSocket server for dashboard real-time updates
- Configuration management (CLI args to RuntimeConfig)
- Single-agent execution model

**Out of Scope:**
- REST API endpoints (see iron_control_api)
- PII detection logic (see iron_safety)
- Cost calculation (see iron_cost)
- Circuit breaker patterns (see iron_reliability)
- Token management (see iron_token_manager)
- State persistence (see iron_runtime_state)
- Multi-agent orchestration (future)
- Distributed runtime (future)

## Installation

### Python Library (pip)

The Python library is built using [maturin](https://github.com/PyO3/maturin) and PyO3.

**Prerequisites:**
- Python 3.9+
- Rust toolchain (rustup)
- uv package manager (`curl -LsSf https://astral.sh/uv/install.sh | sh`)

**Development Install:**

```bash
cd module/iron_runtime

# Install dependencies and setup environment
uv sync  # Automatically creates .venv and installs all dev dependencies

# Build and install in development mode
uv run maturin develop

# Verify installation
uv run python -c "from iron_cage import LlmRouter; print('OK')"
```

**Build Wheel:**

```bash
# Build wheel for distribution
uv run maturin build --release

# Wheel will be in target/wheels/
ls target/wheels/
# iron_cage-0.1.0-cp38-abi3-*.whl
```

### Rust Crate

```toml
[dependencies]
iron_runtime = { path = "../iron_runtime" }
```

## Quick Start (Python)

```python
import os
from iron_cage import LlmRouter
from openai import OpenAI

# Set environment variables
# export IC_TOKEN=your_iron_cage_token
# export IC_SERVER=https://your-iron-cage-server.com

router = LlmRouter(
    api_key=os.environ["IC_TOKEN"],
    server_url=os.environ["IC_SERVER"],
)

# Use with any OpenAI-compatible client
client = OpenAI(base_url=router.base_url, api_key=router.api_key)

response = client.chat.completions.create(
    model="gpt-4o-mini",
    messages=[{"role": "user", "content": "Hello!"}],
)
print(response.choices[0].message.content)

router.stop()
```

**With Anthropic:**

```python
from iron_cage import LlmRouter
from anthropic import Anthropic

router = LlmRouter(api_key=ic_token, server_url=ic_server)

# Anthropic API doesn't use /v1 suffix
client = Anthropic(
    base_url=router.base_url.replace("/v1", ""),
    api_key=router.api_key,
)

response = client.messages.create(
    model="claude-sonnet-4-20250514",
    max_tokens=100,
    messages=[{"role": "user", "content": "Hello!"}],
)
print(response.content[0].text)

router.stop()
```

**Gateway Mode (OpenAI client for Claude):**

Use the same OpenAI client for both OpenAI and Claude models - just change the model name:

```python
from iron_cage import LlmRouter
from openai import OpenAI

router = LlmRouter(api_key=ic_token, server_url=ic_server)
client = OpenAI(base_url=router.base_url, api_key=router.api_key)

# Same client works for both providers!
response = client.chat.completions.create(
    model="claude-sonnet-4-20250514",  # Claude model with OpenAI client!
    messages=[
        {"role": "system", "content": "You are helpful."},
        {"role": "user", "content": "Hello!"}
    ],
    max_tokens=100
)
print(response.choices[0].message.content)  # OpenAI format response

router.stop()
```

The router automatically:
1. Detects Claude model → routes to Anthropic API
2. Translates request (OpenAI → Anthropic format)
3. Translates response (Anthropic → OpenAI format)

**Context Manager:**

```python
with LlmRouter(api_key=token, server_url=url) as router:
    client = OpenAI(base_url=router.base_url, api_key=router.api_key)
    # ... use client
# Router automatically stops on exit
```

## API Reference

### LlmRouter

Local HTTP proxy server for LLM API requests with automatic key management.

**Constructor:**
```python
LlmRouter(
    api_key: str,           # Iron Cage token (IC_TOKEN)
    server_url: str,        # Iron Cage server URL
    cache_ttl_seconds: int = 300,  # Key cache TTL
)
```

**Properties:**
| Property | Type | Description |
|----------|------|-------------|
| `base_url` | `str` | Proxy URL for OpenAI client (`http://127.0.0.1:{port}/v1`) |
| `api_key` | `str` | IC token for client authentication |
| `port` | `int` | Port the proxy is listening on |
| `provider` | `str` | Auto-detected provider (`"openai"` or `"anthropic"`) |
| `is_running` | `bool` | Whether the proxy is running |

**Methods:**
| Method | Description |
|--------|-------------|
| `stop()` | Stop the proxy server |
| `__enter__()` / `__exit__()` | Context manager support |

**Provider Auto-Detection:**
- API keys starting with `sk-ant-` → Anthropic
- All other `sk-*` keys → OpenAI

## Testing

```bash
cd module/iron_runtime

# Run Rust tests
cargo test

# Run Python tests (requires IC_TOKEN and IC_SERVER)
export IC_TOKEN=your_token
export IC_SERVER=http://localhost:3000
python -m pytest python/tests/ -v

# Manual testing
python python/examples/test_manual.py openai     # Test OpenAI API
python python/examples/test_manual.py anthropic  # Test Anthropic API
python python/examples/test_manual.py gateway    # Test OpenAI client → Claude
```

## Example (Rust)

```rust
use iron_runtime::LlmRouter;

// Create router
let mut router = LlmRouter::create(
    api_key.to_string(),
    server_url.to_string(),
    300,  // cache TTL
)?;

let base_url = router.get_base_url();
println!("Proxy running at: {}", base_url);

// Use with HTTP client...

router.shutdown();
```

## License

Apache-2.0
