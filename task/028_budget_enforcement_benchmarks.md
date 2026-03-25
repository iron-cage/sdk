# Task 028: Budget enforcement benchmarks

## Goal
Create criterion benchmarks for the `CostController` reserve/commit/cancel operations to verify the "sub-microsecond atomic budget enforcement" claim from the Rusticon presentation, covering both single-threaded latency and multi-threaded contention scenarios. Results must be reproducible and integrated into CI for regression detection.

## Dependencies
- None

## In Scope
- Criterion benchmarks for `CostController::reserve()`, `commit()`, and `cancel()` in single-threaded mode
- Contended benchmarks with 4, 8, and 16 threads performing concurrent reserve/commit cycles
- CI integration notes for benchmark regression detection
- Criterion HTML report generation

## Out of Scope
- Benchmarks for other modules beyond `iron_cost`
- Performance optimization (this task measures, optimization is separate)
- Flame graph generation or profiling

## Description
The Rusticon presentation makes a specific performance claim: "sub-microsecond atomic budget enforcement." The `CostController` in `iron_cost` uses `AtomicU64` with compare-and-swap operations for its reserve/commit/cancel pattern, which should indeed be sub-microsecond on modern hardware, but no benchmarks exist to verify this claim or detect regressions.

This task adds criterion benchmarks that measure the latency of each `CostController` operation in isolation (single-threaded) and under contention (multi-threaded with 4, 8, and 16 threads). The single-threaded benchmarks establish baseline latency, while the multi-threaded benchmarks verify that the atomic operations scale linearly rather than degrading under contention. Criterion's HTML reports provide visual results, and a CI integration note ensures future pipeline work can include benchmark regression detection.

## Context
Rusticon presentation claims "sub-microsecond atomic budget enforcement" but no benchmarks exist to prove this. The `CostController` uses `AtomicU64` with CAS (compare-and-swap) operations for `reserve()`, `commit()`, and `cancel()` - expected to be sub-microsecond but unverified.

Key files:
- `module/iron_cost/src/budget.rs` - CostController with reserve/commit/cancel pattern
- `module/iron_cost/src/lib.rs` - Public API surface

## Work Procedure
1. Add `criterion` as a dev-dependency in `module/iron_cost/Cargo.toml` with the `html_reports` feature.
2. Add a `[[bench]]` section to `Cargo.toml` with `name = "cost_controller_bench"` and `harness = false`.
3. Create `module/iron_cost/benches/cost_controller_bench.rs`.
4. Implement single-threaded benchmark for `CostController::reserve()` using `criterion::black_box`.
5. Implement single-threaded benchmarks for `CostController::commit()` and `CostController::cancel()`.
6. Implement multi-threaded contended benchmark using `std::thread::scope` with 4, 8, and 16 threads doing reserve/commit cycles.
7. Run `cargo bench -p iron_cost` and verify all benchmarks complete.
8. Review criterion HTML reports in `target/criterion/` for sub-microsecond results.
9. Add a CI integration note documenting how to use `critcmp` or `criterion`'s baseline comparison for regression detection.

## Implementation plan
1. Add `criterion` dev-dependency to `iron_cost/Cargo.toml`.
2. Create `module/iron_cost/benches/cost_controller_bench.rs` with benchmarks for:
   - `CostController::reserve()` - single-threaded latency
   - `CostController::commit()` - single-threaded latency
   - `CostController::cancel()` - single-threaded latency
   - Contended reserve/commit cycle - multi-threaded (4, 8, 16 threads)
3. Add CI integration note for benchmark regression detection.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Single-threaded `reserve()` | Completes in sub-microsecond | Criterion reports mean < 1us |
| Single-threaded `commit()` | Completes in sub-microsecond | Criterion reports mean < 1us |
| Single-threaded `cancel()` | Completes in sub-microsecond | Criterion reports mean < 1us |
| Contended reserve/commit with 4 threads | Scales linearly | Throughput within 2x of single-threaded baseline |
| Contended reserve/commit with 8 threads | Scales linearly | Throughput within 3x of single-threaded baseline |
| Contended reserve/commit with 16 threads | Scales linearly | Throughput within 5x of single-threaded baseline |

## Validation Checklist
- [ ] `criterion` dev-dependency added to `iron_cost/Cargo.toml`
- [ ] Bench file exists at `module/iron_cost/benches/cost_controller_bench.rs`
- [ ] `cargo bench -p iron_cost` runs without errors
- [ ] Single-threaded reserve benchmark shows sub-microsecond mean
- [ ] Single-threaded commit benchmark shows sub-microsecond mean
- [ ] Single-threaded cancel benchmark shows sub-microsecond mean
- [ ] Multi-threaded benchmarks complete without panics or deadlocks
- [ ] Criterion HTML reports generated in `target/criterion/`
- [ ] CI integration note documents regression detection approach

## Validation Procedure
1. Run `cargo bench -p iron_cost` and verify all benchmarks complete without errors.
2. Open the criterion HTML report at `target/criterion/report/index.html` and verify results are present.
3. Check that single-threaded reserve, commit, and cancel each show mean latency below 1 microsecond.
4. Check that contended benchmarks show reasonable scaling (not exponential degradation).
5. Run benchmarks twice and compare with `critcmp` to verify reproducibility.
6. Verify that `cargo test -p iron_cost` still passes (benchmarks should not interfere with tests).

## Acceptance Criteria
- `cargo bench -p iron_cost` runs successfully.
- Single-threaded reserve/commit/cancel each complete in < 1 microsecond on CI hardware.
- Contended benchmark demonstrates linear scaling characteristics.
- Results documented in benchmark output (criterion HTML reports).
