# Task 015: Budget lease auto-refresh worker

## Goal
Implement a background tokio worker that monitors budget levels and automatically requests lease refreshes from the server when remaining budget drops below a configurable threshold, fulfilling promise #6 ("auto-refresh"). This is observable when a running agent's budget depletes to the threshold and a new lease is transparently acquired without user intervention. Scoped to the `iron_cost` crate with integration into the router handshake.

## Dependencies
- None

## In Scope
- Creating `BudgetRefreshWorker` as a background tokio task
- Monitoring `CostController::available()` for threshold-based refresh
- Implementing configurable threshold (default 10%) in `CostConfig`
- Exponential backoff on refresh failure
- Returning unused budget to server on graceful shutdown
- Integration with the router handshake

## Out of Scope
- Modifying the server-side budget allocation logic
- Implementing budget pooling across multiple agents
- Adding budget alerts or notification systems beyond the refresh mechanism
- Changing the existing `BudgetClient` API contract

## Description
The `BudgetClient` in `budget_client.rs` already exposes `refresh()` and `return_unused()` methods, but no background process calls them. When an agent's local budget wallet runs dry, it simply fails rather than automatically requesting a new lease from the server. This contradicts the auto-refresh promise and leads to unexpected interruptions during long-running agent tasks.

This task adds a `BudgetRefreshWorker` that runs as a background tokio task, periodically checking `CostController::available()`. When the remaining budget drops below a configurable threshold (default 10% of the original lease), the worker proactively requests a new lease. If the server is unreachable, exponential backoff prevents thundering-herd problems. On graceful shutdown, unused budget is returned to the server to avoid waste.

## Context
`BudgetClient` already has `refresh()` and `return_unused()` methods in `budget_client.rs`, but there is no background worker that calls refresh automatically. When the local budget wallet depletes, there is no automatic refill from the server. This contradicts promise #6 ("auto-refresh").

Critical areas:
- `module/iron_cost/src/budget.rs` (CostController)
- `module/iron_runtime/src/llm_router/router.rs` (handshake integration)

## Work Procedure
1. Review `budget_client.rs` to understand the existing `refresh()` and `return_unused()` methods.
2. Review `CostController` in `budget.rs` to understand the `available()` method and budget tracking.
3. Add a `refresh_threshold` field to `CostConfig` with a default of 0.10 (10%).
4. Create `BudgetRefreshWorker` struct with a tokio `spawn` loop that polls `available()` at a configurable interval.
5. Implement the refresh logic: when `available() / original_lease < threshold`, call `refresh()`.
6. Implement exponential backoff on `refresh()` failure (starting at 1 second, capped at 60 seconds).
7. Add a shutdown hook that calls `return_unused()` on graceful termination.
8. Integrate the worker startup into the router handshake in `router.rs`.
9. Write an integration test with a real localhost budget server demonstrating the full cycle.
10. Run `cargo test --workspace` and confirm all tests pass.

## Implementation plan
1. Add `BudgetRefreshWorker` as background tokio task in `iron_cost`.
2. Monitor `CostController::available()`.
3. When remaining drops below configurable threshold (default 10%), request new lease from server.
4. Implement exponential backoff on refresh failure.
5. Return unused budget to server on shutdown.
6. Add threshold configuration to `CostConfig`.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Budget drops below 10% threshold | Worker calls `refresh()` automatically | New lease acquired, `available()` increases |
| Budget server is unreachable | Exponential backoff activates | Retry intervals increase (1s, 2s, 4s, ...) up to cap |
| Budget server recovers after backoff | Refresh succeeds on next attempt | Backoff resets, new lease acquired |
| Graceful shutdown signal received | `return_unused()` called | Unused budget returned to server |
| Custom threshold configured at 20% | Refresh triggers at 20% remaining | Worker respects configured threshold |
| Budget stays above threshold | No refresh attempted | No unnecessary server calls |

## Validation List
- [ ] `BudgetRefreshWorker` struct exists in `iron_cost`
- [ ] Worker spawns as a background tokio task
- [ ] Refresh triggers when budget drops below configurable threshold
- [ ] Default threshold is 10%
- [ ] Exponential backoff activates on refresh failure
- [ ] Backoff is capped at a maximum interval
- [ ] Unused budget is returned on graceful shutdown
- [ ] `CostConfig` includes the threshold configuration field
- [ ] Integration test demonstrates the full refresh cycle
- [ ] All tests pass with `cargo test --workspace`

## Validation Procedure
1. Run the integration test that starts a localhost budget server, depletes budget below threshold, and verifies auto-refresh.
2. Verify `CostConfig` has a `refresh_threshold` field by inspecting the struct definition.
3. Confirm backoff behavior by checking the retry logic in the worker implementation.
4. Test graceful shutdown by sending a termination signal and verifying `return_unused()` is called.
5. Run `cargo test --workspace` and confirm no regressions.

## Acceptance criteria
- Auto-refresh triggers when budget drops below threshold.
- Exponential backoff on server failure.
- Unused budget returned on graceful shutdown.
- Integration test with real localhost budget server (per ADR-007: no mocking) demonstrates the full cycle.
