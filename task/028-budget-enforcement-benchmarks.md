# Task 028: Budget enforcement benchmarks

## Dependencies
- None

## Context
Rusticon presentation claims "sub-microsecond atomic budget enforcement" but no benchmarks exist to prove this. The `CostController` uses `AtomicU64` with CAS (compare-and-swap) operations for `reserve()`, `commit()`, and `cancel()` - expected to be sub-microsecond but unverified.

Key files:
- `module/iron_cost/src/budget.rs` - CostController with reserve/commit/cancel pattern
- `module/iron_cost/src/lib.rs` - Public API surface

## Implementation plan
1. Add `criterion` dev-dependency to `iron_cost/Cargo.toml`.
2. Create `module/iron_cost/benches/cost_controller_bench.rs` with benchmarks for:
   - `CostController::reserve()` - single-threaded latency
   - `CostController::commit()` - single-threaded latency
   - `CostController::cancel()` - single-threaded latency
   - Contended reserve/commit cycle - multi-threaded (4, 8, 16 threads)
3. Add CI integration note for benchmark regression detection.

## Acceptance criteria
- `cargo bench -p iron_cost` runs successfully.
- Single-threaded reserve/commit/cancel each complete in < 1 microsecond on CI hardware.
- Contended benchmark demonstrates linear scaling characteristics.
- Results documented in benchmark output (criterion HTML reports).
