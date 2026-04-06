# Task 023: Hardening

## Goal
Increase test coverage from 43.76% to above 70%, add property-based concurrency tests for budget enforcement, fuzz-test PII detection patterns, establish load testing baselines, and audit for timing-based side channels in token validation. This ensures the codebase is production-hardened before public release.

## Dependencies
- Task 019

## In Scope
- Property-based concurrency tests for budget enforcement (proptest or quickcheck)
- Fuzz testing for PII detection regex patterns (cargo-fuzz)
- Load testing with 100+ concurrent agents on a single runtime
- Timing attack audit on token validation paths
- Raising llvm-cov coverage above 70%

## Out of Scope
- Performance optimization work (this task measures, does not optimize)
- Penetration testing by external security auditors
- Coverage for generated code or proc-macro outputs

## Description
The current test coverage of 43.76% leaves significant portions of the codebase unverified, particularly in the areas of concurrent budget enforcement and PII detection. Budget enforcement uses atomic operations (CAS on AtomicU64) that are correct in theory but have never been stress-tested under real concurrency. PII detection uses regex patterns that have never been fuzz-tested for edge cases, false positives, or panics on malformed input.

This task adds three categories of hardening tests. First, property-based tests using proptest or quickcheck that exercise budget reserve/commit/cancel under concurrent access, verifying that invariants hold regardless of interleaving. Second, fuzz testing via cargo-fuzz that feeds random and adversarial input to PII detection patterns. Third, a load test baseline that measures maximum concurrent agents, p99 latency, and CPU overhead under sustained load. Additionally, a security audit of token validation code checks for timing-based information leaks by verifying that the `subtle` crate is used for constant-time comparisons.

## Context
Coverage is 43.76%. Budget enforcement has no concurrency stress tests. PII detection has no fuzz testing. No load testing baseline exists.

Critical areas:
- `module/iron_cost/src/budget.rs`
- `module/iron_safety/src/`
- `module/iron_runtime/src/llm_router/`

## Work Procedure
1. Add `proptest` as a dev-dependency to `iron_cost/Cargo.toml`.
2. Write property-based tests in `iron_cost` that spawn multiple threads performing concurrent reserve/commit/cancel and assert budget invariants (total reserved + committed never exceeds budget).
3. Add `cargo-fuzz` targets for PII detection patterns in `iron_safety`.
4. Create fuzz corpus with edge cases: partial matches, Unicode variants, mixed encodings.
5. Run fuzz tests for a minimum of 10 minutes and fix any panics discovered.
6. Write a load test harness that spawns 100+ concurrent mock agents on a single runtime instance.
7. Collect and record p99 latency, max concurrent agents, and CPU overhead.
8. Audit `iron_token_manager` and `iron_control_api` for token comparison paths, verify `subtle::ConstantTimeEq` usage.
9. Add unit tests to fill coverage gaps identified by llvm-cov report.
10. Run `cargo llvm-cov` and verify total coverage exceeds 70%.

## Implementation plan
1. Add property-based tests for budget enforcement race conditions (proptest or quickcheck).
2. Add fuzz testing for PII detection patterns (cargo-fuzz).
3. Run load test: 100+ concurrent agents on single runtime.
4. Security audit: timing attacks on token validation (subtle crate usage verification).
5. Target coverage > 70%.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| 8 threads doing concurrent reserve/commit | Budget invariants hold | Total reserved + committed never exceeds budget |
| 8 threads doing concurrent reserve/cancel | No negative budget values | Budget remains non-negative after all operations |
| Fuzz PII detector with random bytes | No panics | 10+ minutes of fuzzing without crashes |
| Fuzz PII detector with Unicode edge cases | Graceful handling | No panics, false positives documented |
| 100 concurrent agents on single runtime | Runtime remains responsive | p99 latency under acceptable threshold |
| Token comparison with near-miss input | Constant-time rejection | No measurable timing difference vs random input |

## Validation Checklist
- [ ] Property-based tests for budget concurrency exist and pass
- [ ] Fuzz targets for PII detection are configured and run without panics
- [ ] Load test harness executes with 100+ concurrent agents
- [ ] Load test results are recorded (p99 latency, max agents, CPU overhead)
- [ ] Token validation uses constant-time comparison (subtle crate)
- [ ] llvm-cov reports total coverage above 70%
- [ ] No new unsafe code introduced
- [ ] All pre-existing tests continue to pass

## Validation Procedure
1. Run `cargo test --workspace` and verify all tests pass including new property-based tests.
2. Run `cargo fuzz run pii_detector -- -max_total_time=600` and verify no crashes.
3. Execute the load test harness and record p99 latency, max concurrent agents, and CPU usage.
4. Inspect token validation code paths and confirm `subtle::ConstantTimeEq` is used for all comparisons.
5. Run `cargo llvm-cov --workspace` and verify the coverage report shows above 70%.
6. Review the coverage report for remaining uncovered critical paths and document them.

## Acceptance Criteria
- Property tests pass under concurrent budget operations.
- Fuzz testing runs without panics for 10+ minutes.
- Load test establishes baseline: max concurrent agents, p99 latency, CPU overhead.
- No timing-based information leaks in token validation.
- llvm-cov reports > 70%.
