# iron_runtime - Specification

**Module:** iron_runtime
**Layer:** 5 (Integration)
**Status:** Active

---

### Responsibility

Agent orchestrator bridging Python AI agents with Rust governance infrastructure via PyO3. Provides LlmRouter for transparent API key management and request proxying to OpenAI/Anthropic. Intercepts LLM calls, applies safety/cost/reliability policies, manages agent lifecycle.

---

### Scope

**In Scope:**
- Python-Rust FFI via PyO3 for agent execution
- LlmRouter - Local proxy for LLM API requests with automatic key management
- Multi-provider support (OpenAI, Anthropic) with auto-detection
- LLM call interception and policy enforcement
- Agent lifecycle management (spawn, monitor, shutdown)
- Real-time state broadcasting to dashboard
- Budget enforcement with Protocol 005 integration

**Out of Scope:**
- Multi-agent orchestration (pilot: single agent)
- Agent sandboxing (see iron_sandbox)
- REST API endpoints (see iron_control_api)

---

### Dependencies

**Required:** iron_runtime_state, iron_cost, iron_safety, iron_reliability, iron_telemetry
**External:** pyo3, tokio, axum, reqwest

**Features:**
- `enabled` - Core functionality (default)
- `analytics` - Event recording via iron_runtime_analytics
- `full` - All features (default)

---

### Core Concepts

- **LlmRouter:** Local HTTP proxy for LLM API requests. Starts on random port, accepts IC_TOKEN, fetches real API keys via agent's assigned provider key, auto-detects provider from API key format, forwards requests with real API key, tracks costs.
- **AgentRuntime:** Manages agent lifecycle, coordinates policies
- **PyO3 Bridge:** Exposes Rust runtime to Python agents
- **Policy Enforcer:** Applies safety/cost/reliability rules
- **Event Broadcaster:** Sends state updates via WebSocket

---

### Integration Points

**Used by:** iron_sdk, iron_api, Python agents
**Uses:** iron_runtime_state, iron_cost, iron_safety, iron_reliability, iron_telemetry, iron_runtime_analytics, Iron Cage Server (provider keys via Feature 014)

---

### Testing

- Unit tests: `tests/llm_router_test.rs` (provider detection, path stripping, model detection)
- Integration tests: `tests/llm_router_integration_test.rs` (lifecycle, feature-gated)
- Python E2E: `python/tests/test_llm_router_e2e.py`, `test_budget_e2e.py`, `test_analytics_*.py`

---

*For detailed specification, see spec/-archived_detailed_spec.md*
*For architecture concepts, see docs/architecture/001_execution_models.md*
