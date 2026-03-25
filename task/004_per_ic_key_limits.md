# Task 004: Add dedicated limits per IC key (per agent)

## Goal

Add an independently configurable spending limit per IC key (agent) that operates as a separate enforcement layer alongside global and provider-key limits. The result is observable through API endpoints exposing per-agent IC-key limit and current usage, and through one agent being blocked at its limit without affecting other agents under the same owner. Scoped to the IC-key limit model, handshake/refresh enforcement, and limit precedence logic. Testable by setting different IC-key limits on two agents and verifying independent enforcement.

## Dependencies
- Task 001

## In Scope

- IC-key limit model bound to agent identity
- API endpoints to read and update IC-key limits per agent with role-based authorization
- IC-key limit enforcement in handshake and refresh flows
- IC-key usage tracking within the lease reserve/spend/return lifecycle
- Clear precedence and error mapping between global, IC-key, and agent budget limits
- Migration and backfill strategy for existing agents

## Out of Scope

- Changes to global usage limits or provider-key spending limits
- Per-model or per-request-type limits within an IC key
- Automatic limit adjustment or alerting when approaching thresholds
- UI or dashboard for IC-key limit management

## Description

Current budget enforcement operates at two levels: a global usage limit and a per-agent budget. There is no dedicated IC-key policy limit as an independently configurable control. This means an agent's spending is governed only by its budget allocation and the global ceiling, with no middle-tier control that an administrator can set and adjust independently.

This task adds a per-IC-key limit model that binds to the agent identity. Administrators can configure limits per agent through new API endpoints with role-based access control. The handshake and refresh flows are updated to check the IC-key limit before granting budget, alongside the existing global and agent budget checks. A clear precedence order is established: global limit, then IC-key limit, then agent remaining budget - with error responses identifying which specific limit blocked the request.

Usage tracking for the IC-key limit follows the same lease lifecycle as existing budget tracking. IC token regeneration does not reset the limit binding or usage continuity for the agent, ensuring spending history is preserved across token rotations.

## Context
Current enforcement is global (`usage_limits`) plus per-agent budget (`agent_budgets`). There is no dedicated IC-key policy limit as an independently configurable runtime control.

Critical areas:
- `module/iron_control_api/src/routes/budget/handshake.rs`
- `module/iron_control_api/src/routes/budget/refresh.rs`
- `module/iron_control_api/src/routes/budget/usage.rs`
- `module/iron_control_api/src/routes/agents.rs`
- `module/iron_control_api/src/routes/limits.rs`
- `module/iron_token_manager/src/agent_budget.rs`
- `module/iron_token_manager/migrations/001_initial_schema.sql`
- `module/iron_token_manager/migrations/010_create_agent_budgets.sql`

## Work Procedure

1. Design the IC-key limit schema - limit value and current usage columns bound to agent identity
2. Create migration for IC-key limit fields and define backfill strategy for existing agents
3. Implement the IC-key limit model in `agent_budget.rs` or a new module
4. Add API endpoints (read, update) for IC-key limits with role-based authorization
5. Add IC-key limit check to handshake flow - enforce before budget reservation
6. Add IC-key limit check to refresh flow - enforce before budget refresh
7. Integrate IC-key usage tracking into lease reserve/spend/return lifecycle
8. Define precedence order (global, IC-key, agent budget) and implement error mapping that identifies the blocking limit
9. Write integration tests for multi-agent behavior under one owner - verify independent limits
10. Write async tests for IC-key spend limits under concurrent traffic

## Implementation plan
1. Add an IC-key limit model bound to agent identity.
2. Add API endpoints to read and update IC-key limits per agent with role-based authorization.
3. Enforce IC-key limit checks before budget reservation in handshake and refresh.
4. Track IC-key current usage in the same lifecycle as lease reserve/spend/return.
5. Define clear precedence and error mapping between:
   - global usage limit
   - IC-key limit
   - agent remaining budget
6. Add migration and backfill strategy for existing agents.
7. Add integration tests for multi-agent behavior under one owner.
8. Add async tests for IC-key spend limits under concurrent traffic.
   - Parallel handshake/refresh per single agent.
   - Parallel usage reporting while near limit threshold.
   - Verification that limit rejections are consistent and no overspend occurs.

## Test Matrix

| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Read IC-key limit for an agent | API returns limit and current usage | Response contains `ic_key_limit` and `current_usage` |
| Update IC-key limit for an agent (admin) | Limit updated | Subsequent read reflects new value |
| Update IC-key limit (non-admin) | Request rejected | Authorization error returned |
| Handshake when IC-key usage is below limit | Handshake succeeds | Budget reservation granted |
| Handshake when IC-key usage exceeds limit | Handshake rejected | Error identifies IC-key limit as blocking |
| Handshake when global limit is exceeded | Handshake rejected | Error identifies global limit as blocking |
| Agent A at limit, agent B below limit | Agent B handshake succeeds | Independent enforcement verified |
| IC token regeneration | Usage continuity preserved | `current_usage` unchanged after regeneration |
| Concurrent handshake near IC-key limit | No overspend | Usage never exceeds configured limit |
| Concurrent usage reporting near limit | Consistent counters | No negative values or double-accounting |

## Validation Checklist

- [ ] IC-key limit model exists and is bound to agent identity
- [ ] API endpoints for reading and updating IC-key limits exist
- [ ] Role-based authorization enforced on limit management endpoints
- [ ] Handshake checks IC-key limit before budget reservation
- [ ] Refresh checks IC-key limit before budget refresh
- [ ] Error response identifies which specific limit blocked the request
- [ ] Exceeding one agent's limit does not affect other agents
- [ ] IC token regeneration preserves usage continuity
- [ ] Migration and backfill for existing agents completed
- [ ] Async tests confirm concurrent correctness

## Validation Procedure

1. Run existing test suite to establish baseline - global and per-agent budget tests pass
2. Set an IC-key limit on agent A via API - verify the limit is returned in subsequent queries
3. Perform handshakes as agent A until the IC-key limit is reached - verify rejection with IC-key limit error
4. Perform handshake as agent B (same owner) - verify success despite agent A being at limit
5. Update agent A's IC-key limit to a higher value - verify handshake succeeds again
6. Regenerate agent A's IC token - verify usage continuity (current usage unchanged)
7. Trigger a global limit exceeded scenario - verify the error identifies the global limit specifically
8. Run concurrent handshake tests near the IC-key limit - verify no overspend
9. Run full test suite and confirm zero regressions in existing budget behavior

## Acceptance Criteria
- Each agent has independently configurable IC-key limit and current usage visibility.
- Exceeding one agent limit does not block other agents of the same owner.
- Global and IC-key checks both apply, and error response identifies the blocking limit.
- IC token regeneration does not reset limit binding or usage continuity for the agent.
- Integration tests cover happy path, edge cases, and concurrent requests.
- Async test suite confirms IC-key spend limits remain correct during parallel requests.
