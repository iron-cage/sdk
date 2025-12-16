# iron_runtime - Specification

**Module:** iron_runtime
**Layer:** 5 (Integration)
**Status:** Active

---

### Responsibility

Pure Rust agent orchestrator providing LlmRouter for transparent API key management and request proxying to OpenAI/Anthropic. Intercepts LLM calls, applies safety/cost/reliability policies, manages agent lifecycle.

**Note:** Python bindings are provided by `iron_sdk` module (PyPI: `iron-cage`, import: `iron_cage`).

---

### Scope

**In Scope:**
- LlmRouter - Local proxy for LLM API requests with automatic key management
- Multi-provider support (OpenAI, Anthropic) with auto-detection
- LLM call interception and policy enforcement
- Agent lifecycle management (spawn, monitor, shutdown)
- Real-time state broadcasting to dashboard
- Budget enforcement with Protocol 005 integration

**Out of Scope:**
- Python bindings (see iron_sdk module)
- Multi-agent orchestration (pilot: single agent)
- Agent sandboxing (see iron_sandbox)
- REST API endpoints (see iron_control_api)

---

### Dependencies

**Required:** iron_runtime_state, iron_cost, iron_safety, iron_reliability, iron_telemetry
**External:** tokio, axum, reqwest

**Features:**
- `enabled` - Core functionality (default)
- `analytics` - Event recording via iron_runtime_analytics
- `full` - All features (default)

---

### Core Concepts

- **LlmRouter:** Local HTTP proxy for LLM API requests. Starts on random port, accepts IC_TOKEN, fetches real API keys via agent's assigned provider key, auto-detects provider from API key format, forwards requests with real API key, tracks costs.
- **AgentRuntime:** Manages agent lifecycle, coordinates policies
- **Policy Enforcer:** Applies safety/cost/reliability rules
- **Event Broadcaster:** Sends state updates via WebSocket

---

### Integration Points

**Used by:** iron_sdk (Python bindings), iron_api, Rust applications
**Uses:** iron_runtime_state, iron_cost, iron_safety, iron_reliability, iron_telemetry, iron_runtime_analytics, Iron Cage Server (provider keys via Feature 014)

---

### Testing

- Unit tests: `tests/llm_router_test.rs` (provider detection, path stripping, model detection)
- Integration tests: `tests/llm_router_integration_test.rs` (lifecycle, feature-gated)
- Python tests: See `iron_sdk` module

---

*For architecture concepts, see docs/architecture/001_execution_models.md*
*For Python bindings, see module/iron_sdk*
