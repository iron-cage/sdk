# Task 003: Add spending and limits per inference provider key

## Goal

Introduce per-provider-key spending limits and usage tracking so that budget governance operates at the individual key level, not just globally or per agent. The result is observable through API endpoints returning `limit` and `current_spent` for each `provider_key_id`, and through handshake/refresh being blocked when a key's limit is exceeded. Scoped to the lease lifecycle (reserve, spend, refresh, return) and analytics aggregation by key. Testable by setting a key limit, exhausting it, and verifying subsequent requests are rejected while other keys remain functional.

## Dependencies
- Task 002

## In Scope

- Per-provider-key limit and spending state model keyed by `provider_key_id`
- Persisting `provider_key_id` in lease lifecycle data (reserve, spend, refresh, return)
- Per-key limit enforcement in handshake and refresh flows
- Atomic counter updates during reserve, spend, and return operations
- API exposure of per-key limit and usage via provider management endpoints
- Analytics queries supporting filter and aggregation by `provider_key_id`

## Out of Scope

- Changes to global usage limit or per-agent budget enforcement (those remain as-is)
- Per-model spending limits within a single provider key
- Historical spending data migration for existing leases without `provider_key_id`
- UI or dashboard changes for spending visibility

## Description

Spending enforcement currently exists at two levels: globally through `usage_limits` and per agent through `agent_budgets`. However, there is no way to set or enforce spending limits on individual provider keys. This means a single provider key could accumulate unbounded spending as long as global and agent-level limits are not reached, and there is no key-level observability into where spending is occurring.

This task adds a third enforcement layer keyed by `provider_key_id`. Each provider key gains a configurable spending limit and a tracked current spend amount. The lease lifecycle - reserve, spend, refresh, and return - is extended to attribute operations to the specific provider key, enabling accurate per-key accounting. Handshake and refresh flows check the per-key limit before granting budget.

Concurrency protections ensure that atomic counter updates prevent negative counters and double-accounting under parallel load. Analytics queries are extended to support filtering and aggregation by provider key, enabling operators to understand spending distribution across keys.

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

## Work Procedure

1. Design the per-key limit and spending state schema, referencing existing `provider_key_id` in migrations
2. Create or update migration to add limit and spending columns to the provider keys table
3. Implement the per-key limit and spending model in the token manager
4. Extend lease lifecycle operations (reserve, spend, refresh, return) to include `provider_key_id` attribution
5. Add per-key limit checks to handshake flow - reject when key spending exceeds limit
6. Add per-key limit checks to refresh flow - reject when key spending exceeds limit
7. Implement atomic counter updates with concurrency protections (transactions or atomic SQL operations)
8. Expose per-key limit and usage through provider management API endpoints
9. Extend analytics queries to support grouping and filtering by `provider_key_id`
10. Write async integration tests for concurrent handshake/refresh against the same key near limit threshold

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

## Test Matrix

| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Query limit and spending for a provider key | API returns current values | Response contains `limit` and `current_spent` fields |
| Handshake when key spending is below limit | Handshake succeeds | Budget reservation granted |
| Handshake when key spending equals or exceeds limit | Handshake rejected | Per-key limit exceeded error returned |
| Refresh when key spending is below limit | Refresh succeeds | Budget refreshed |
| Refresh when key spending exceeds limit | Refresh rejected | Per-key limit exceeded error returned |
| Successful usage report | Key spending increases | `current_spent` incremented by usage amount |
| Budget return | Key spending decreases | `current_spent` decremented by return amount |
| Concurrent handshake requests near limit | No overspend occurs | Key spending never exceeds limit |
| Concurrent usage reporting | Counters remain consistent | No negative counters or double-accounting |
| Analytics query filtered by `provider_key_id` | Correct per-key data returned | Spending matches sum of attributed operations |
| Global and per-agent limits still enforced | Existing limits unaffected | Regression tests pass |

## Validation List

- [ ] Per-key limit and spending state model exists
- [ ] `provider_key_id` persisted in lease lifecycle data
- [ ] Handshake checks per-key limit before granting budget
- [ ] Refresh checks per-key limit before refreshing budget
- [ ] Counter updates are atomic and concurrency-safe
- [ ] API exposes per-key limit and usage
- [ ] Analytics support filtering by `provider_key_id`
- [ ] No negative counters under concurrent load
- [ ] Global and per-agent budget behavior unchanged
- [ ] Async test suite covers parallel load scenarios

## Validation Procedure

1. Run existing test suite to establish baseline - all global and per-agent budget tests pass
2. Set a spending limit on a provider key via API - verify limit is returned in subsequent queries
3. Perform a handshake under the key limit - verify success and spending increment
4. Continue spending until the key limit is reached - verify subsequent handshake returns limit error
5. Perform a budget return and verify spending decreases
6. Query analytics filtered by the provider key - verify spending totals are accurate
7. Run concurrent handshake tests against a key near its limit - verify no overspend
8. Run concurrent usage reporting tests - verify counters remain consistent and non-negative
9. Verify existing global and per-agent budget tests still pass
10. Run full test suite to confirm zero regressions

## Acceptance criteria
- API returns `limit` and `current_spent` for a given `provider_key_id`.
- Handshake and refresh are blocked when key limit is exceeded.
- Successful usage increases key spending; budget return decreases key spending correctly.
- Analytics can return spending grouped or filtered by `provider_key_id`.
- Under concurrent load, key counters remain non-negative and consistent with lease/accounting totals.
- Regression tests confirm existing global and per-agent budget behavior remains intact.
- Async test suite demonstrates spend-limit enforcement remains correct under parallel load.
