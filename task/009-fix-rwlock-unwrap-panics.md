# Task 009: Fix RwLock unwrap panics in CLI adapters

## Dependencies
- None

## Context
35 `.unwrap()` calls on RwLock read/write in CLI adapters (32 in `in_memory.rs`, 3 in `http.rs`). A poisoned lock (from any thread panic while holding it) crashes the entire process. This contradicts promise #10 ("memory safety non-optional").

Critical areas:
- `module/iron_cli/src/adapters/implementations/http.rs` (lines 99, 109, 119)
- `module/iron_cli/src/adapters/implementations/in_memory.rs` (32 locations)

## Implementation plan
1. Add `LockPoisoned` variant to `AdapterError` (using thiserror per codebase convention).
2. Replace all `.write().unwrap()` and `.read().unwrap()` with `.map_err(|_| AdapterError::LockPoisoned)?`.
3. Update tests to verify error propagation instead of panics (per ADR-007: use real implementations, no mocking).

## Acceptance criteria
- Zero `.unwrap()` calls on RwLock in adapter implementations.
- `AdapterError::LockPoisoned` variant exists and is documented.
- Existing adapter tests continue to pass.
