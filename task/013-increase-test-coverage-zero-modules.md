# Task 013: Increase test coverage for 0% modules

## Dependencies
- None

## Context
`provider_adapter.rs` and `trace_storage.rs` have no tests covering their public functions. `budget_request.rs` and `lease_manager.rs` have partial test coverage with significant untested branches. No coverage configuration exists in the repo (previous coverage percentages are unverifiable).

Critical areas:
- `module/iron_token_manager/src/provider_adapter.rs`
- `module/iron_token_manager/src/trace_storage.rs`
- `module/iron_token_manager/src/budget_request.rs`
- `module/iron_token_manager/src/lease_manager.rs`

## Implementation plan
1. Add tests for all public functions in `provider_adapter.rs` using real implementations (SQLite in-memory per ADR-007).
2. Add tests for all public functions in `trace_storage.rs` using real implementations.
3. Add tests for uncovered branches in `budget_request.rs` and `lease_manager.rs`.
4. Tests go in module's own `tests/` directory per testing strategy.

## Acceptance criteria
- All public functions in `provider_adapter.rs` have at least one test.
- All public functions in `trace_storage.rs` have at least one test.
- Key branches in `budget_request.rs` and `lease_manager.rs` are tested.
- All new tests pass with `cargo test --workspace`.
