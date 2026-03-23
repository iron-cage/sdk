# Task 013: Increase test coverage for 0% modules

## Goal
Add meaningful test coverage to four modules in `iron_token_manager` that currently have zero or partial test coverage, ensuring every public function has at least one test. This is observable by running `cargo test` and confirming new tests exist for `provider_adapter.rs`, `trace_storage.rs`, `budget_request.rs`, and `lease_manager.rs`. Scoped to the `iron_token_manager` crate only.

## Dependencies
- None

## In Scope
- Writing tests for all public functions in `provider_adapter.rs`
- Writing tests for all public functions in `trace_storage.rs`
- Writing tests for uncovered branches in `budget_request.rs`
- Writing tests for uncovered branches in `lease_manager.rs`
- Using real implementations (SQLite in-memory) per ADR-007

## Out of Scope
- Adding coverage tooling or CI coverage reporting
- Modifying the source code of the modules under test (unless needed for testability)
- Testing private implementation details
- Coverage for modules outside `iron_token_manager`

## Description
Four modules in the `iron_token_manager` crate have inadequate test coverage. `provider_adapter.rs` and `trace_storage.rs` have zero tests covering their public functions, while `budget_request.rs` and `lease_manager.rs` have partial coverage with significant untested branches. No coverage configuration exists in the repo, so previous coverage claims are unverifiable.

This task adds comprehensive tests for all public functions in the zero-coverage modules and fills in branch coverage gaps in the partially-tested modules. All tests use real implementations (SQLite in-memory databases) per ADR-007 rather than mocks. Tests are placed in each module's `tests/` directory following the project's testing strategy.

## Context
`provider_adapter.rs` and `trace_storage.rs` have no tests covering their public functions. `budget_request.rs` and `lease_manager.rs` have partial test coverage with significant untested branches. No coverage configuration exists in the repo (previous coverage percentages are unverifiable).

Critical areas:
- `module/iron_token_manager/src/provider_adapter.rs`
- `module/iron_token_manager/src/trace_storage.rs`
- `module/iron_token_manager/src/budget_request.rs`
- `module/iron_token_manager/src/lease_manager.rs`

## Work Procedure
1. Inventory all public functions in `provider_adapter.rs` and list them.
2. Inventory all public functions in `trace_storage.rs` and list them.
3. Identify untested branches in `budget_request.rs` and `lease_manager.rs` by reading existing tests and source.
4. Write tests for `provider_adapter.rs` using SQLite in-memory backends.
5. Write tests for `trace_storage.rs` using SQLite in-memory backends.
6. Write branch-coverage tests for `budget_request.rs` covering edge cases and error paths.
7. Write branch-coverage tests for `lease_manager.rs` covering edge cases and error paths.
8. Place all tests in the module's `tests/` directory per project conventions.
9. Run `cargo test -p iron_token_manager` and confirm all new tests pass.
10. Run `cargo test --workspace` to confirm no regressions.

## Implementation plan
1. Add tests for all public functions in `provider_adapter.rs` using real implementations (SQLite in-memory per ADR-007).
2. Add tests for all public functions in `trace_storage.rs` using real implementations.
3. Add tests for uncovered branches in `budget_request.rs` and `lease_manager.rs`.
4. Tests go in module's own `tests/` directory per testing strategy.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| `provider_adapter` - register a new provider | Provider is stored and retrievable | Query returns the registered provider |
| `provider_adapter` - query nonexistent provider | Returns `None` or appropriate error | No panic, graceful empty result |
| `trace_storage` - store a trace event | Event persists in SQLite in-memory DB | Query retrieves the stored event with correct fields |
| `trace_storage` - query empty store | Returns empty collection | No error, zero results |
| `budget_request` - request within budget | Request succeeds | Returns `Ok` with allocated amount |
| `budget_request` - request exceeding budget | Request is rejected | Returns `Err` with budget exceeded variant |
| `lease_manager` - acquire lease | Lease is granted with expiry | Lease object has valid expiry timestamp |
| `lease_manager` - lease expiry handling | Expired lease is detected correctly | Expiry check returns true for past timestamps |

## Validation List
- [ ] Every public function in `provider_adapter.rs` has at least one test
- [ ] Every public function in `trace_storage.rs` has at least one test
- [ ] Key branches in `budget_request.rs` are covered (success, failure, edge cases)
- [ ] Key branches in `lease_manager.rs` are covered (acquire, expire, renew)
- [ ] All tests use real SQLite in-memory backends, not mocks
- [ ] Tests are in the module's `tests/` directory
- [ ] `cargo test --workspace` passes with all new tests

## Validation Procedure
1. Run `cargo test -p iron_token_manager` and verify all new tests appear in output and pass.
2. Confirm test files exist under `iron_token_manager/tests/` for each target module.
3. Inspect test code to verify no mock objects are used (per ADR-007).
4. Run `cargo test --workspace` and confirm no regressions in other crates.
5. Review each target module's public API and cross-check against test inventory.

## Acceptance criteria
- All public functions in `provider_adapter.rs` have at least one test.
- All public functions in `trace_storage.rs` have at least one test.
- Key branches in `budget_request.rs` and `lease_manager.rs` are tested.
- All new tests pass with `cargo test --workspace`.
