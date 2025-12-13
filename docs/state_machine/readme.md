# State Machine Documentation

Documentation for all state machines in the Iron Cage Runtime system.

## Scope

This directory documents state machines using dimension-based organization. Each state machine has unique identifier (NNN format) and standardized documentation structure.

**In scope**: State definitions, transition rules, invariants, state diagrams
**Out of scope**: Message protocols (see ../protocol/), API contracts (see ../api/)

## Organization

State machines are documented individually with unique identifiers:
- `001_budget_lease_lifecycle.md` - Budget lease lifecycle
- `002_budget_request_workflow.md` - Budget request approval workflow
- `003_agent_runtime_status.md` - Agent runtime execution status

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `001_budget_lease_lifecycle.md` | Document budget lease lifecycle states and transitions |
| `002_budget_request_workflow.md` | Document budget request approval workflow states and transitions |
| `003_agent_runtime_status.md` | Document agent runtime execution status states and transitions |

## Cross-Collection Relationships

**Dependencies**:
- protocol/ - State machines often implement protocol connection states
- invariant/ - State machines maintain invariants across transitions

**Used By**:
- lifecycle/ - Lifecycle phases may reference state machines
- api/ - APIs expose state machine states

## Adding New State Machines

1. Assign next sequential ID (NNN format)
2. Follow common instance requirements (Scope, Purpose, States, Transitions, Invariants, Cross-References)
3. Update this readme.md Responsibility Table
4. Cross-reference from related collections
