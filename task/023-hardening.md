# Task 023: Hardening

## Dependencies
- Task 019

## Context
Coverage is 43.76%. Budget enforcement has no concurrency stress tests. PII detection has no fuzz testing. No load testing baseline exists.

Critical areas:
- `module/iron_cost/src/budget.rs`
- `module/iron_safety/src/`
- `module/iron_runtime/src/llm_router/`

## Implementation plan
1. Add property-based tests for budget enforcement race conditions (proptest or quickcheck).
2. Add fuzz testing for PII detection patterns (cargo-fuzz).
3. Run load test: 100+ concurrent agents on single runtime.
4. Security audit: timing attacks on token validation (subtle crate usage verification).
5. Target coverage > 70%.

## Acceptance criteria
- Property tests pass under concurrent budget operations.
- Fuzz testing runs without panics for 10+ minutes.
- Load test establishes baseline: max concurrent agents, p99 latency, CPU overhead.
- No timing-based information leaks in token validation.
- llvm-cov reports > 70%.
