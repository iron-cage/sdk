# iron_runtime - Specification

**Module:** iron_runtime  
**Layer:** 5 (Integration)  
**Status:** Active  

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

## Responsibility

Agent orchestrator bridging Python AI agents with Rust governance infrastructure via PyO3. Provides `LlmRouter` for transparent API key management and request proxying to OpenAI/Anthropic, intercepts LLM calls to apply safety/cost/reliability policies, and manages agent lifecycle with runtime state and telemetry.

## Scope

**In Scope**
- Python↔Rust FFI via PyO3 for agent execution
- `LlmRouter` local proxy with provider auto-detection and key translation
- LLM request interception with budget enforcement (Protocol 005), safety checks, and reliability guards
- Single-agent lifecycle management (spawn, monitor, shutdown) with state persistence
- Telemetry/analytics emission for dashboard streams

**Out of Scope**
- Multi-agent orchestration (future phase)
- Agent sandboxing (see iron_sandbox / iron_cage runtime)
- REST API endpoints and auth flows (see iron_control_api)
- Frontend control surfaces (see iron_dashboard)

## Dependencies

**Required Modules:** iron_runtime_state, iron_cost, iron_safety, iron_reliability, iron_telemetry  
**Optional Modules:** iron_runtime_analytics (analytics feature)  
**External:** pyo3, tokio, axum, reqwest, serde/serde_json, uuid, anyhow  
**Features:** `enabled` (default core), `analytics` (event recording), `full` (all)

## Core Concepts

- **LlmRouter:** Local HTTP proxy; listens on loopback, accepts IC_TOKEN, fetches provider keys via agent assignment, forwards to OpenAI/Anthropic, tracks spend, and returns translated responses.
- **AgentRuntime:** Coordinates agent lifecycle and policy enforcement across subsystems.
- **PyO3 Bridge:** Exposes `Runtime` and `LlmRouter` as the `iron_cage` Python module for SDK and tests.
- **Policy Pipeline:** Ordered safety, budget, and reliability checks applied before/after provider calls.
- **Telemetry/Analytics:** Emits structured events and state updates for dashboards and budgets.

## Integration Points

**Used by:** iron_sdk (Python agents), iron_cli_py, control plane tests, manual examples  
**Uses:** iron_runtime_state (persistence), iron_cost (pricing/budget), iron_safety (PII checks), iron_reliability (circuit breakers), iron_telemetry (logging/tracing), iron_runtime_analytics (feature-gated)  
**External Services:** Iron Cage control server for provider keys and analytics ingress

---

<<<<<<< HEAD
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
=======
Cross-references: spec/-archived_detailed_spec.md; docs/architecture/001_execution_models.md; docs/architecture/006_budget_control_protocol.md; docs/protocol/readme.md.
>>>>>>> f326cba9b63f81a68e9971089276fd64a0ba039f
