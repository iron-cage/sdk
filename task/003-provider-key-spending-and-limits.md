# Task 003: Add spending and limits per inference provider key

## Dependencies
- Task 002

## Context
Spending enforcement exists globally (`usage_limits`) and per agent (`agent_budgets`), but not per `provider_key_id`. This blocks key-level budget governance and observability.

Critical areas:
- `module/iron_control_api/src/routes/budget/handshake.rs`
- `module/iron_control_api/src/routes/budget/usage.rs`
- `module/iron_control_api/src/routes/budget/refresh.rs`
- `module/iron_control_api/src/routes/providers.rs`
- `module/iron_control_api/src/routes/analytics/*`
- `module/iron_token_manager/migrations/004_create_ai_provider_keys.sql`
- `module/iron_token_manager/migrations/009_create_budget_leases.sql`

## Implementation plan
1. Introduce per-provider-key limit and spending state keyed by `provider_key_id`.
2. Persist `provider_key_id` in lease lifecycle data so reserve, spend, refresh, and return operations are attributable.
3. Enforce per-key limit checks before granting budget in handshake and refresh.
4. Update per-key counters atomically during reserve, spend, and return.
5. Expose provider-key limit and usage through provider management API.
6. Extend analytics queries to filter and aggregate by `provider_key_id`.
7. Add concurrency protections to avoid negative counters and double-accounting.
8. Add async integration tests for spend-limit enforcement under parallel requests.
   - Concurrent handshake and refresh requests against the same `provider_key_id`.
   - Concurrent usage reporting and budget return flows.
   - Verification of atomic counters and deterministic limit rejections.

## Acceptance criteria
- API returns `limit` and `current_spent` for a given `provider_key_id`.
- Handshake and refresh are blocked when key limit is exceeded.
- Successful usage increases key spending; budget return decreases key spending correctly.
- Analytics can return spending grouped or filtered by `provider_key_id`.
- Under concurrent load, key counters remain non-negative and consistent with lease/accounting totals.
- Regression tests confirm existing global and per-agent budget behavior remains intact.
- Async test suite demonstrates spend-limit enforcement remains correct under parallel load.
