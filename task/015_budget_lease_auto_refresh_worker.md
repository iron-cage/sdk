# Task 015: Budget lease auto-refresh worker

## Dependencies
- None

## Context
`BudgetClient` already has `refresh()` and `return_unused()` methods in `budget_client.rs`, but there is no background worker that calls refresh automatically. When the local budget wallet depletes, there is no automatic refill from the server. This contradicts promise #6 ("auto-refresh").

Critical areas:
- `module/iron_cost/src/budget.rs` (CostController)
- `module/iron_runtime/src/llm_router/router.rs` (handshake integration)

## Implementation plan
1. Add `BudgetRefreshWorker` as background tokio task in `iron_cost`.
2. Monitor `CostController::available()`.
3. When remaining drops below configurable threshold (default 10%), request new lease from server.
4. Implement exponential backoff on refresh failure.
5. Return unused budget to server on shutdown.
6. Add threshold configuration to `CostConfig`.

## Acceptance criteria
- Auto-refresh triggers when budget drops below threshold.
- Exponential backoff on server failure.
- Unused budget returned on graceful shutdown.
- Integration test with real localhost budget server (per ADR-007: no mocking) demonstrates the full cycle.
