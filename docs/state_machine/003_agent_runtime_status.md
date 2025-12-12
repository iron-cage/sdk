# State Machine: Agent Runtime Status

### Scope

This document describes the agent runtime execution status state machine. It defines the states an agent transitions through during its runtime lifecycle from startup to shutdown.

**In scope**: Agent states (RUNNING, STOPPED, FAILED), state transitions, startup/shutdown flow, error handling
**Out of scope**: Agent configuration and registration (see API protocols), agent budget tracking (see Budget Lease Lifecycle), agent code execution details (see implementation)

### Purpose

Agent runtime status tracking enables monitoring, debugging, and operational control of agent instances. Without defined states and transitions, runtime behavior becomes difficult to observe and manage.

**Problem Statement**:
Without explicit state machine:
- Unclear whether agent is actively executing or idle
- No distinction between clean shutdown and failure
- Difficult to implement restart logic (when can agent be restarted?)
- Monitoring systems cannot reliably detect agent health

**Solution Summary**:
State machine tracks agent lifecycle: RUNNING (actively executing) → STOPPED (clean shutdown) or FAILED (error shutdown). Clear state enables monitoring, restart logic, and debugging.

### States

**RUNNING**: Agent is actively executing
- Agent process spawned successfully
- Agent can process requests and execute tasks
- Budget can be allocated and spent
- Telemetry events logged ("agent_started")
- State persisted to StateManager

**STOPPED**: Agent has stopped cleanly
- Agent stopped via stop_agent() call
- Resources released, no active execution
- Clean shutdown (no error condition)
- Telemetry events logged ("agent_stopped")
- State can transition back to RUNNING (restart allowed)

**FAILED**: Agent terminated due to error (terminal state)
- Agent crashed or encountered fatal error
- Resources may not be fully cleaned up
- Error condition recorded
- State defined in AgentStatus enum
- Implementation status: Partial (state defined, explicit transitions TBD)

### Transitions

| From | Event | Condition | To | Action |
|------|-------|-----------|-----|--------|
| - | Spawn Agent | Valid configuration, resources available | RUNNING | Generate agent_id, create AgentState, log "agent_started", save to StateManager |
| RUNNING | Stop Agent | stop_agent() called, no active tasks | STOPPED | Log "agent_stopped", update status in StateManager |
| RUNNING | Fatal Error | Unrecoverable error, crash | FAILED | Log error, record failure reason (implementation partial) |
| STOPPED | Spawn Agent | Restart requested, resources available | RUNNING | Reuse agent_id or generate new, log "agent_started", update status |
| FAILED | (any event) | (any) | FAILED | No-op (terminal state, manual intervention required) |

**Note**: Transition from STOPPED → RUNNING (restart) is architecturally supported (state mutable) but explicit restart function may not be implemented yet.

### Invariants

**I1: Single Status per Agent**
- Agent has exactly one status at any time (RUNNING, STOPPED, or FAILED)
- Status stored in AgentState.status field
- Enforcement: StateManager saves complete AgentState atomically

**I2: Budget Spending Only When RUNNING**
- Budget can only be spent when agent status is RUNNING
- STOPPED or FAILED agents cannot incur costs
- Enforcement: Runtime checks status before processing billable operations

**I3: Telemetry Correlation**
- Status transitions logged to telemetry with agent_id
- "agent_started" event correlates with RUNNING entry
- "agent_stopped" event correlates with STOPPED entry
- Enforcement: Transition functions call iron_telemetry::log_agent_event()

**I4: State Persistence**
- All status transitions persisted to StateManager
- State survives in-memory storage (DashMap) during runtime
- SQLite persistence planned (feature flag exists)
- Enforcement: All transition functions call state.save_agent_state()

### State Diagram

```
         Spawn Agent
              |
              v
        ┌──────────┐
        │ RUNNING  │
        └──────────┘
              |
      ┌───────┴───────┐
      │               │
 Stop Agent     Fatal Error
      │               │
      v               v
┌──────────┐    ┌──────────┐
│ STOPPED  │    │  FAILED  │ (Terminal)
└──────────┘    └──────────┘
      │
 Spawn Agent
   (restart)
      │
      v
┌──────────┐
│ RUNNING  │
└──────────┘
```

### Entry/Exit Actions

**RUNNING Entry**:
- Generate agent_id (iron_types::AgentId::generate())
- Create AgentState struct (status=Running, budget_spent=0.0, pii_detections=0)
- Log telemetry event: iron_telemetry::log_agent_event(agent_id, "agent_started")
- Save state to StateManager

**RUNNING Exit**:
- Log telemetry event based on destination state ("agent_stopped" or error)
- Update AgentState with new status

**STOPPED Entry**:
- Update AgentState.status to Stopped
- Log telemetry event: iron_telemetry::log_agent_event(agent_id, "agent_stopped")
- Save updated state to StateManager
- Resources released (agent process terminated)

**FAILED Entry**:
- Update AgentState.status to Failed
- Log error details to telemetry (implementation TBD)
- Record failure reason (implementation TBD)
- Resources cleanup attempted (may be incomplete due to error)

### Error States

**Agent Spawn Failure**:
- Event: Attempt to spawn agent fails (resource exhaustion, configuration error)
- Handling: spawn_agent() returns error, no state created
- Recovery: Fix configuration/resources, retry spawn

**Concurrent Stop Operations**:
- Event: Multiple stop_agent() calls for same agent_id
- Handling: Second call finds agent already STOPPED, no-op
- Recovery: None (idempotent operation)

**State Manager Unavailable**:
- Event: StateManager.save_agent_state() fails
- Handling: State updates lost (in-memory backend unlikely to fail)
- Recovery: Agent continues running, state may be inconsistent

**Failed → Running Attempt**:
- Event: Attempt to restart agent in FAILED state
- Handling: FAILED is terminal state, restart should create new agent instance
- Recovery: Spawn new agent with new agent_id

### Implementation Notes

**Current Status** (as of codebase analysis):
- RUNNING and STOPPED fully implemented with transitions
- FAILED state defined in enum but explicit transition logic partial
- Restart (STOPPED → RUNNING) architecturally supported, implementation TBD
- State persistence: In-memory (DashMap) working, SQLite feature flag exists but operations not implemented

**Future Enhancements**:
- Explicit FAILED state transition on agent crash/error
- Restart functionality for STOPPED agents
- SQLite persistence for state durability
- State transition hooks for monitoring/alerting

### Cross-References

**Dependencies**:
- iron_types: AgentId type
- iron_telemetry: Event logging
- iron_runtime_state: StateManager, AgentState, AgentStatus

**Used By**:
- Iron Runtime: Uses state machine for agent lifecycle management
- Monitoring systems: Query agent status for health checks
- Control API: Exposes agent status via REST API

**Related**:
- State Machine 001: Budget Lease Lifecycle (./001_budget_lease_lifecycle.md) - Budget leases tied to running agents

**Implementation**:
- Source: `module/iron_runtime_state/src/lib.rs:455-460` - AgentStatus enum definition
- Source: `module/iron_runtime/src/lib.rs:53-95` - spawn_agent(), stop_agent() implementations
- Tests: `module/iron_runtime_state/tests/state_test.rs` - State management tests
- Tests: `module/iron_runtime/tests/runtime_test.rs` - Runtime lifecycle tests

**Specification**:
- Requirement: Agent lifecycle management (see iron_runtime spec)
