# iron_runtime - Specification

**Module:** iron_runtime
**Layer:** 5 (Integration)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

Agent orchestrator bridging Python AI agents with Rust governance infrastructure via PyO3. Intercepts LLM calls, applies safety/cost/reliability policies, manages agent lifecycle, coordinates real-time dashboard updates.

---

## Scope

**In Scope:**
- Python-Rust FFI via PyO3 for agent execution
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

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **AgentRuntime:** Manages agent lifecycle, coordinates policies
- **PyO3 Bridge:** Exposes Rust runtime to Python agents
- **Policy Enforcer:** Applies safety/cost/reliability rules
- **Event Broadcaster:** Sends state updates via WebSocket

---

## Integration Points

**Used by:**
- iron_sdk - Python wrapper for agent protection
- iron_control_api - WebSocket server receives events

**Uses:**
- iron_runtime_state - Persists agent state
- iron_cost - Tracks budget consumption
- iron_safety - Validates LLM inputs/outputs
- iron_reliability - Circuit breaker logic

---

*For detailed implementation requirements, see spec/-archived_detailed_spec.md*
*For architecture concepts, see docs/architecture/001_execution_models.md*
