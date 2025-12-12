# State Machine: Budget Lease Lifecycle

### Scope

This document describes the client-side budget lease lifecycle state machine. It defines the states a budget lease transitions through from creation to termination, the events triggering transitions, and invariants maintained across the lifecycle.

**In scope**: Lease states (ACTIVE, EXPIRED, CLOSED, REVOKED), state transitions, transition triggers, state invariants, error states
**Out of scope**: Budget control message protocol (see ../protocol/005_budget_control_protocol.md), lease manager implementation details (see source code), token encryption (see ../protocol/005_budget_control_protocol.md § Token Exchange)

### Purpose

Budget leases require explicit lifecycle management to ensure proper resource cleanup, prevent budget leaks, and maintain system integrity. Without defined states and transitions, lease management becomes ad-hoc and error-prone.

**Problem Statement**:
Without explicit state machine:
- Unclear when lease can be used (active vs expired vs closed)
- No guarantee of proper cleanup (budget leaks possible)
- Difficult to reason about concurrent operations (multiple refresh attempts)
- Error states undefined (what happens when lease expires during operation?)

**Solution Summary**:
State machine enforces strict lifecycle: ACTIVE (usable) → EXPIRED (renewal needed) → CLOSED (terminal). Invariants prevent invalid transitions (e.g., cannot activate closed lease).

### States

**ACTIVE**: Lease is valid and can be used for LLM operations
- Budget lease has non-zero remaining budget
- Lease has not expired (current_time < expires_at)
- Lease has not been explicitly closed
- IC Token and IP Token are valid

**EXPIRED**: Lease has reached expiration time but not yet closed
- current_time >= expires_at
- Lease must be refreshed to continue operations
- Cannot be used for new LLM operations
- Automatic transition to CLOSED if not refreshed within grace period

**CLOSED**: Lease has been explicitly terminated (terminal state)
- All resources released
- Remaining budget returned to budget pool
- Cannot transition to any other state (immutable)
- Occurs via explicit return or cleanup

**REVOKED**: Lease has been invalidated by server (terminal state)
- Budget exceeded, policy violation, or manual revocation
- All operations fail immediately
- Cannot be refreshed or reactivated
- Requires new handshake to obtain new lease

### Transitions

| From | Event | Condition | To | Action |
|------|-------|-----------|-----|--------|
| - | Handshake Success | Budget available, policy satisfied | ACTIVE | Store IC token, IP token, budget_granted, expires_at |
| ACTIVE | Expiration Time Reached | current_time >= expires_at | EXPIRED | Mark lease expired, block new operations |
| ACTIVE | Explicit Close | User initiates close | CLOSED | Return unused budget, invalidate tokens |
| ACTIVE | Budget Exhausted | budget_spent >= budget_granted | EXPIRED | Mark lease expired, trigger refresh |
| ACTIVE | Server Revocation | Policy violation, manual revoke | REVOKED | Invalidate tokens, record revocation reason |
| EXPIRED | Refresh Success | Refresh within grace period | ACTIVE | Update budget_granted, expires_at, reset timer |
| EXPIRED | Grace Period Exceeded | No refresh within grace period | CLOSED | Return unused budget, cleanup resources |
| EXPIRED | Explicit Close | User initiates close | CLOSED | Return unused budget, invalidate tokens |
| CLOSED | (any event) | (any) | CLOSED | No-op (terminal state) |
| REVOKED | (any event) | (any) | REVOKED | No-op (terminal state) |

**Grace Period**: 60 seconds after expiration before automatic CLOSED transition

### Invariants

**I1: Single Active Lease per Agent**
- At most one lease in ACTIVE state per agent_id at any time
- Handshake while ACTIVE lease exists returns error
- Enforcement: Check active lease before creating new lease

**I2: Budget Conservation**
- For all leases: budget_spent <= budget_granted
- Enforcement: Block operations when budget_spent >= budget_granted, transition to EXPIRED

**I3: Terminal State Immutability**
- Once lease reaches CLOSED or REVOKED, it never transitions to another state
- Enforcement: All state transition functions check current state, reject transitions from CLOSED/REVOKED

**I4: Token Validity in ACTIVE**
- If lease is ACTIVE, IC token and IP token are valid (not expired, not revoked)
- Enforcement: Validate tokens before marking lease ACTIVE

**I5: Monotonic Budget Consumption**
- budget_spent never decreases (except on explicit reset via new handshake)
- Enforcement: Only increment budget_spent, never decrement

### State Diagram

```
              Handshake
                  |
                  v
            ┌─────────┐
            │ ACTIVE  │<─────┐
            └─────────┘      │
                  │          │
     ┌────────────┼──────────┴────────────┐
     │            │                       │
 Expiration   Exhaustion            Refresh Success
     │            │                       │
     v            v                       │
┌─────────┐  ┌─────────┐                 │
│ EXPIRED │  │ EXPIRED │─────────────────┘
└─────────┘  └─────────┘
     │            │
     │      Grace Period
     │        Exceeded
     │            │
     v            v
┌─────────┐  ┌─────────┐
│ CLOSED  │  │ CLOSED  │ (Terminal)
└─────────┘  └─────────┘

     Server Revocation
            |
            v
      ┌─────────┐
      │ REVOKED │ (Terminal)
      └─────────┘
```

### Entry/Exit Actions

**ACTIVE Entry**:
- Store IC token, IP token in secure storage
- Set budget_granted, budget_spent = 0
- Start expiration timer (expires_at)
- Log lease activation

**ACTIVE Exit**:
- Stop expiration timer
- Log final budget consumption

**EXPIRED Entry**:
- Block new LLM operations
- Start grace period timer (60s)
- Log lease expiration

**CLOSED Entry**:
- Calculate unused budget (budget_granted - budget_spent)
- Return unused budget to budget pool via API
- Invalidate IC token, IP token
- Clean up local lease state
- Log lease closure with final budget stats

**REVOKED Entry**:
- Invalidate IC token, IP token immediately
- Log revocation reason
- Alert user of revocation

### Error States

**Invalid State Transition Attempt**:
- Event: Attempt to transition from CLOSED/REVOKED
- Handling: Reject transition, log error, return error to caller
- Recovery: None (terminal states are immutable)

**Refresh Failure**:
- Event: Refresh API call fails (network error, server rejection)
- Handling: Remain in EXPIRED state, retry refresh
- Recovery: Exponential backoff retry up to grace period limit

**Token Invalidation During ACTIVE**:
- Event: IC/IP token becomes invalid while lease is ACTIVE
- Handling: Immediate transition to REVOKED
- Recovery: User must initiate new handshake

### Cross-References

**Dependencies**:
- Protocol 005: Budget Control Protocol (../protocol/005_budget_control_protocol.md) - Defines message protocol for lease operations

**Used By**:
- (To be added as integrations are documented)

**Related**:
- (To be added as related state machines are documented)

**Implementation**:
- Source: `module/iron_token_manager/src/lease_manager.rs:1-358` - Lease state machine implementation
- Tests: `module/iron_control_api/tests/budget_concurrency.rs` - Concurrent lease lifecycle tests

**Specification**:
- Requirement: `module/iron_control_api/spec.md` § Budget Control - Lease lifecycle requirements
