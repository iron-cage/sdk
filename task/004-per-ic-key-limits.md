# Task 004: Add dedicated limits per IC key (per agent)

## Dependencies
- Task 001

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

## Acceptance criteria
- Each agent has independently configurable IC-key limit and current usage visibility.
- Exceeding one agent limit does not block other agents of the same owner.
- Global and IC-key checks both apply, and error response identifies the blocking limit.
- IC token regeneration does not reset limit binding or usage continuity for the agent.
- Integration tests cover happy path, edge cases, and concurrent requests.
- Async test suite confirms IC-key spend limits remain correct during parallel requests.
