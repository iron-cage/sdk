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
- iron_cost - Budget tracking and cost calculation
- iron_safety - PII detection
- iron_reliability - Circuit breakers
- iron_telemetry - Logging infrastructure
- iron_runtime_analytics - Lock-free event recording (via `analytics` feature)

**Required External:**
- pyo3 - Python FFI
- tokio - Async runtime
- axum - HTTP server for LlmRouter proxy
- reqwest - HTTP client for forwarding requests

**Features:**
- `enabled` - Core functionality (default)
- `analytics` - Event recording via iron_runtime_analytics
- `full` - All features (`enabled` + `analytics`, **default**)

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

# ==== Mode 1: Iron Cage Server (managed keys) ====
router = LlmRouter(
    api_key="ic_xxx",           # Iron Cage token
    server_url="https://...",   # Iron Cage server
    cache_ttl_seconds=300,      # Key cache TTL (optional)
    budget=10.0,                # Budget limit in USD (optional)
)

# ==== Mode 2: Direct Provider Key (bypass server) ====
router = LlmRouter(
    provider_key="sk-xxx",      # Direct OpenAI/Anthropic key
    budget=10.0,                # Budget limit in USD (optional)
)
# Provider auto-detected: sk-ant-* → anthropic, sk-* → openai

# Properties
router.base_url      # "http://127.0.0.1:{port}/v1"
router.api_key       # Token for client auth
router.provider      # Auto-detected: "openai" or "anthropic"
router.is_running    # bool
router.budget        # Budget limit in USD (None if unlimited)
router.budget_status # (spent, limit) tuple or None

# Use with any OpenAI-compatible client
client = OpenAI(base_url=router.base_url, api_key=router.api_key)
response = client.chat.completions.create(...)

# Cost tracking
print(f"Total spent: ${router.total_spent():.6f}")

# Budget management (runtime)
router.set_budget(20.0)  # Update budget limit

# Context manager support
with LlmRouter(provider_key="sk-xxx", budget=5.0) as router:
    client = OpenAI(base_url=router.base_url, api_key=router.api_key)
    ...

router.stop()  # Explicit stop
```

**Budget Enforcement:**
- When budget is set, requests are blocked with HTTP 402 when limit exceeded
- Error response is OpenAI-compatible but distinct from rate limits:
```json
{
  "error": {
    "message": "Iron Cage budget limit exceeded. Spent: $10.50, Limit: $10.00...",
    "type": "iron_cage_budget_exceeded",
    "code": "budget_exceeded"
  }
}
```
- Use `router.set_budget()` to increase limit at runtime

**Known Limitation - Concurrent Overspend:**

Budget checking uses optimistic concurrency: check happens BEFORE request, cost added AFTER response. With concurrent requests, multiple threads can pass the budget check simultaneously, causing overspend.

Example (5 threads, $0.05 budget, 4000 tokens/request):
```
Budget: $0.05
Final spent: $0.088
OVERSPEND: $0.038 (76% over budget)
```

**Why this happens:**
1. Thread A checks budget → $0.00 < $0.05 → PASS
2. Thread B checks budget → $0.00 < $0.05 → PASS (same time)
3. Thread C checks budget → $0.00 < $0.05 → PASS (same time)
4. All 3 requests complete, each costing ~$0.008
5. Total: $0.024 spent before any thread saw updated balance

**Mitigation strategies (not yet implemented):**
- Budget reservation: reserve estimated cost before request, reconcile after
- Pessimistic locking: serialize budget checks (reduces throughput)
- Soft limits with hard cutoff: allow some overspend, hard-block at 2x limit

**Current behavior:** Best-effort enforcement. Single-threaded usage is exact. Concurrent usage may overspend proportional to (threads × cost-per-request).

**Analytics Integration:**

LlmRouter automatically records analytics events when built with `analytics` feature (default via `full`):

- **LlmRequestCompleted** - Model, tokens, cost, provider for each successful request
- **LlmRequestFailed** - Model, status code, error message for failed requests
- **RouterStarted** / **RouterStopped** - Lifecycle events with port/stats

Events are stored in lock-free `EventStore` (from `iron_runtime_analytics`) and logged to console:
```
INFO LlmRouter proxy listening on http://127.0.0.1:45297
INFO LLM request completed model=gpt-4o-mini input_tokens=11 output_tokens=1 cost_usd=0.000001
INFO LlmRouter proxy shutting down
```

**LlmRouter Rust API:**
```rust
use iron_runtime::LlmRouter;

// Mode 1: Iron Cage Server
let mut router = LlmRouter::create(api_key, server_url, cache_ttl_seconds)?;

// Mode 2: With budget
let mut router = LlmRouter::create_with_budget(api_key, server_url, ttl, 10.0)?;

// Mode 3: Direct provider key
let mut router = LlmRouter::create_with_provider_key("sk-xxx".into(), Some(10.0))?;

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
- iron_cost - Tracks budget consumption and calculates request costs
- iron_safety - Validates LLM inputs/outputs
- iron_reliability - Circuit breaker logic
- iron_telemetry - Logging infrastructure
- iron_runtime_analytics - Event recording and stats aggregation
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

**Budget E2E Tests:** `python/tests/test_budget_e2e.py`
- Budget tracking with real API calls
- Budget exceeded returns HTTP 402
- `set_budget()` unblocks after exceeded
- Concurrent budget enforcement (5 threads)
- Concurrent overspend demonstration test
- Env vars: `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` for direct mode
- Env vars: `IC_TOKEN` + `IC_SERVER` for Iron Cage mode

---

*For detailed implementation requirements, see spec/-archived_detailed_spec.md*
*For architecture concepts, see docs/architecture/001_execution_models.md*
