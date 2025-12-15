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

Cross-references: spec/-archived_detailed_spec.md; docs/architecture/001_execution_models.md; docs/architecture/006_budget_control_protocol.md; docs/protocol/readme.md.
