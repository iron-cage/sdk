# iron_runtime - Specification

**Module:** iron_runtime
**Layer:** 5 (Integration)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

Agent orchestrator bridging Python AI agents with Rust governance infrastructure via PyO3. Provides LlmRouter for transparent API key management and request proxying to OpenAI/Anthropic. Intercepts LLM calls, applies safety/cost/reliability policies, manages agent lifecycle, coordinates real-time dashboard updates.

---

## Scope

**In Scope:**
- Python-Rust FFI via PyO3 for agent execution
- LlmRouter - Local proxy for LLM API requests with automatic key management
- Multi-provider support (OpenAI, Anthropic) with auto-detection
- LLM call interception and policy enforcement
- Agent lifecycle management (spawn, monitor, shutdown)
- Real-time state broadcasting to dashboard
- Integration with safety, cost, reliability modules

**Out of Scope:**
- Multi-agent orchestration (pilot: single agent)
- Agent sandboxing (see iron_sandbox)
- REST API endpoints (see iron_control_api)
- Dashboard UI (see iron_dashboard)
- Policy configuration (see iron_control_api)

---

## Dependencies

**Required Modules:**
- iron_runtime_state - Agent state persistence
- iron_cost - Budget tracking
- iron_safety - PII detection
- iron_reliability - Circuit breakers
- iron_telemetry - Logging

**Required External:**
- pyo3 - Python FFI
- tokio - Async runtime
- axum - HTTP server for LlmRouter proxy
- reqwest - HTTP client for forwarding requests

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **LlmRouter:** Local HTTP proxy server for LLM API requests
  - Starts local server on random available port
  - Accepts IC_TOKEN for authentication
  - Fetches real API keys from Iron Cage server (cached)
  - Auto-detects provider from API key format (`sk-ant-*` → Anthropic, `sk-*` → OpenAI)
  - Forwards requests to provider with real API key
  - Validates provider/model mismatch with clear error messages
  - Supports both OpenAI (`Authorization: Bearer`) and Anthropic (`x-api-key`) auth headers
  - Calculates and tracks request costs using iron_cost pricing data
  - Provides `total_spent()` for cumulative cost tracking (in-memory, per session)
- **AgentRuntime:** Manages agent lifecycle, coordinates policies
- **PyO3 Bridge:** Exposes Rust runtime to Python agents
- **Policy Enforcer:** Applies safety/cost/reliability rules
- **Event Broadcaster:** Sends state updates via WebSocket

**LlmRouter Python API:**
```python
from iron_cage import LlmRouter
from openai import OpenAI

# Create router (starts local proxy)
router = LlmRouter(
    api_key="ic_xxx",           # Iron Cage token
    server_url="https://...",   # Iron Cage server
    cache_ttl_seconds=300,      # Key cache TTL (optional)
)

# Properties
router.base_url   # "http://127.0.0.1:{port}/v1"
router.api_key    # IC token (for client auth)
router.provider   # Auto-detected: "openai" or "anthropic"
router.is_running # bool

# Use with any OpenAI-compatible client
client = OpenAI(base_url=router.base_url, api_key=router.api_key)
response = client.chat.completions.create(...)

# Cost tracking (debug)
print(f"Total spent: ${router.total_spent():.6f}")

# Context manager support
with LlmRouter(api_key=token, server_url=url) as router:
    client = OpenAI(base_url=router.base_url, api_key=router.api_key)
    ...

router.stop()  # Explicit stop
```

**LlmRouter Rust API:**
```rust
use iron_runtime::LlmRouter;

let mut router = LlmRouter::create(
    api_key,
    server_url,
    cache_ttl_seconds,
)?;

let base_url = router.get_base_url();  // "http://127.0.0.1:{port}/v1"
let is_running = router.running();

router.shutdown();  // Stop the proxy
```

---

## Integration Points

**Used by:**
- iron_sdk - Python wrapper for agent protection
- iron_api - WebSocket server receives events
- Python agents - Via LlmRouter for transparent LLM access

**Uses:**
- iron_runtime_state - Persists agent state
- iron_cost - Tracks budget consumption
- iron_safety - Validates LLM inputs/outputs
- iron_reliability - Circuit breaker logic
- Iron Cage Server - Fetches provider API keys via `/api/keys`

---

## Testing

**Unit Tests:** `tests/llm_router_test.rs`
- Provider detection from API key format
- Path prefix stripping (`/openai/`, `/anthropic/`)
- Model name detection (`claude-*`, `gpt-*`, `o1-*`, `o3-*`)

**Integration Tests:** `tests/llm_router_integration_test.rs`
- Router lifecycle (start/stop)
- Provider detection at startup
- Base URL format validation
- Feature-gated: `#![cfg(feature = "integration")]`

**Python E2E Tests:** `python/tests/test_llm_router_e2e.py`
- Router starts and stops
- Context manager support
- OpenAI chat completion (provider-aware skip)
- Invalid token rejection

---

*For detailed implementation requirements, see spec/-archived_detailed_spec.md*
*For architecture concepts, see docs/architecture/001_execution_models.md*
