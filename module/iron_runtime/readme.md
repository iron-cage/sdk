# iron_runtime

Agent orchestration and Python bridge for AI agent execution. Provides **LlmRouter** - a local proxy server for transparent LLM API key management with OpenAI and Anthropic support.

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
- Python 3.8+
- Rust toolchain (rustup)
- maturin (`pip install maturin`)

**Development Install:**

```bash
cd module/iron_runtime

# Create virtual environment
python -m venv .venv
source .venv/bin/activate  # Linux/macOS
# .venv\Scripts\activate   # Windows

# Install maturin
pip install maturin

# Build and install in development mode
maturin develop

# Verify installation
python -c "from iron_runtime import LlmRouter; print('OK')"
```

**Build Wheel:**

```bash
# Build wheel for distribution
maturin build --release

# Wheel will be in target/wheels/
ls target/wheels/
# iron_runtime-0.1.0-cp38-abi3-*.whl
```

**Install Dependencies for Testing:**

```bash
pip install pytest openai anthropic
```

### Rust Crate

```toml
[dependencies]
iron_runtime = { path = "../iron_runtime" }
```

## Quick Start (Python)

```python
import os
from iron_runtime import LlmRouter
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
from iron_runtime import LlmRouter
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
python python/examples/test_manual.py openai
python python/examples/test_manual.py anthropic
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
