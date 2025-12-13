# State Machine: Budget Request Workflow

### Scope

This document describes the budget change request workflow state machine. It defines the states a budget request transitions through from creation to final disposition (approved/rejected/cancelled).

**In scope**: Request states (PENDING, APPROVED, REJECTED, CANCELLED), state transitions, approval/rejection workflow, optimistic locking guarantees
**Out of scope**: Budget request API protocol (see ../protocol/017_budget_requests_api.md), budget modification history tracking (see implementation), request creation validation (see spec)

### Purpose

Budget change requests require explicit approval workflow to maintain financial control and auditability. Without defined states and transitions, request processing becomes ambiguous and prone to race conditions.

**Problem Statement**:
Without explicit state machine:
- Unclear whether request can be modified or withdrawn
- No guarantee against double-approval race conditions
- Difficult to enforce "pending-only" approval constraint
- Terminal state handling undefined (can approved request be rejected?)

**Solution Summary**:
State machine enforces strict workflow: PENDING (awaiting decision) → APPROVED/REJECTED/CANCELLED (terminal). Optimistic locking prevents concurrent approval+rejection.

### States

**PENDING**: Request created, awaiting administrator decision
- Initial state after request creation
- Can be approved or rejected by administrator
- Can be cancelled by requester (state defined, implementation partial)
- Only state from which transitions are allowed

**APPROVED**: Request approved by administrator (terminal state)
- Budget change has been applied to agent's budget
- Modification recorded in budget_modification_history
- Cannot transition to any other state (immutable)
- Includes approver_id and approval timestamp

**REJECTED**: Request rejected by administrator (terminal state)
- Budget change was not applied
- No modification recorded in history
- Cannot transition to any other state (immutable)
- Includes rejection timestamp

**CANCELLED**: Request cancelled by requester (terminal state)
- Budget change was not applied
- State defined in RequestStatus enum
- Implementation status: Partial (state checks exist, cancel function TBD)
- Cannot transition to any other state (immutable)

### Transitions

| From | Event | Condition | To | Action |
|------|-------|-----------|-----|--------|
| - | Create Request | Valid budget change, requester authorized | PENDING | Store request with current_budget, requested_budget, created_at |
| PENDING | Administrator Approves | Admin has approval permission, status='pending' | APPROVED | Update agent budget, record history entry, set approver_id, updated_at |
| PENDING | Administrator Rejects | Admin has approval permission, status='pending' | REJECTED | Set updated_at, no budget change |
| PENDING | Requester Cancels | Requester owns request, status='pending' | CANCELLED | Set updated_at, no budget change (partial implementation) |
| APPROVED | (any event) | (any) | APPROVED | No-op (terminal state) |
| REJECTED | (any event) | (any) | REJECTED | No-op (terminal state) |
| CANCELLED | (any event) | (any) | CANCELLED | No-op (terminal state) |

**Optimistic Locking**: All transitions from PENDING use `WHERE status='pending'` clause and check `rows_affected()` to prevent concurrent modification races.

### Invariants

**I1: Single Pending State Ownership**
- Request can only be in one state at any time
- Concurrent approve+reject operations prevented by optimistic locking
- Enforcement: Database-level WHERE status='pending' clause + rows_affected check

**I2: Terminal State Immutability**
- Once request reaches APPROVED, REJECTED, or CANCELLED, it never transitions
- Enforcement: All transition functions validate current status, reject non-PENDING requests

**I3: Budget Modification Atomicity** (APPROVED only)
- When request approved, budget update and history recording occur atomically
- Enforcement: Database transaction wrapping status update, budget update, history insert

**I4: Status Transition Monotonicity**
- Request progresses from PENDING to exactly ONE terminal state
- No cycles, no backwards transitions
- Enforcement: Transition functions only accept status='pending', terminal states reject all transitions

**I5: Approval Authorization**
- Only authorized administrators can approve/reject requests
- Enforcement: API layer authorization checks before calling transition functions

### State Diagram

```
         Create Request
                |
                v
          ┌──────────┐
          │ PENDING  │
          └──────────┘
                |
    ┌───────────┼───────────┐
    │           │           │
Approve      Reject      Cancel
    │           │           │
    v           v           v
┌──────────┐ ┌──────────┐ ┌──────────┐
│ APPROVED │ │ REJECTED │ │CANCELLED │ (All Terminal)
└──────────┘ └──────────┘ └──────────┘
```

### Entry/Exit Actions

**PENDING Entry**:
- Store request_id, agent_id, requester_id
- Record current_budget_micros, requested_budget_micros
- Set created_at timestamp
- Set status = 'pending'

**APPROVED Entry**:
- Update agent's budget_micros to requested_budget_micros
- Insert record into budget_modification_history table
- Set approver_id from authenticated administrator
- Set updated_at timestamp
- Set status = 'approved'
- Commit transaction atomically

**REJECTED Entry**:
- Set updated_at timestamp
- Set status = 'rejected'
- No budget modification, no history entry

**CANCELLED Entry**:
- Set updated_at timestamp
- Set status = 'cancelled'
- No budget modification, no history entry
- (Implementation: Partial - state exists, cancel function TBD)

### Error States

**Concurrent Modification (Optimistic Lock Failure)**:
- Event: Two administrators attempt approve/reject simultaneously
- Handling: Second operation receives RowNotFound error (optimistic lock failed)
- Recovery: Administrator retries, sees request already processed

**Non-Pending Approval Attempt**:
- Event: Attempt to approve/reject request not in PENDING state
- Handling: Validation check returns error, operation rejected
- Recovery: None (terminal states are immutable, operation invalid)

**Transaction Failure During Approval**:
- Event: Database transaction fails during approve operation
- Handling: Entire transaction rolled back (status, budget, history all reverted)
- Recovery: Request remains in PENDING, administrator retries

**Authorization Failure**:
- Event: Non-administrator attempts approve/reject
- Handling: API layer authorization check fails before state transition
- Recovery: None (permission denied, operation invalid)

### Cross-References

**Dependencies**:
- Protocol 017: Budget Requests API (../protocol/017_budget_requests_api.md) - Defines API endpoints for request workflow

**Used By**:
- Budget Control system - Uses request workflow for budget increase requests

**Related**:
- State Machine 001: Budget Lease Lifecycle (./001_budget_lease_lifecycle.md) - Budget leases granted from approved budgets

**Implementation**:
- Source: `module/iron_token_manager/src/budget_request.rs:10-591` - RequestStatus enum and transition functions
- Tests: `module/iron_control_api/tests/budget_*.rs` - Budget request workflow tests
- API: `module/iron_control_api/src/routes/budget.rs:1579-1799` - Approve/reject endpoints

**Specification**:
- Requirement: Protocol 012: Budget Request Workflow (see docs/protocol/)
- Requirement: Protocol 017: Budget History (see docs/protocol/)
