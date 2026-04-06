# Task 009: Fix RwLock unwrap panics in CLI adapters

## Goal

Eliminate all `.unwrap()` calls on RwLock operations in CLI adapter implementations to prevent process crashes from poisoned locks. The result is observable through zero unwrap calls in adapter code and a new `LockPoisoned` error variant. Scoped to `in_memory.rs` (32 calls) and `http.rs` (3 calls). Testable by running existing adapter tests and verifying error propagation instead of panics.

## Dependencies
- None

## In Scope

- Adding `LockPoisoned` variant to `AdapterError` using thiserror
- Replacing all `.write().unwrap()` calls with `.write().map_err(|_| AdapterError::LockPoisoned)?`
- Replacing all `.read().unwrap()` calls with `.read().map_err(|_| AdapterError::LockPoisoned)?`
- Updating tests to verify error propagation behavior

## Out of Scope

- Refactoring adapter implementations to avoid RwLock entirely
- Adding lock poisoning recovery or cleanup logic
- Changes to non-adapter modules that may also use RwLock

## Description

The CLI adapter implementations contain 35 `.unwrap()` calls on RwLock read/write operations - 32 in `in_memory.rs` and 3 in `http.rs`. If any thread panics while holding a lock, the lock becomes poisoned and all subsequent `.unwrap()` calls on that lock will cause a cascading panic, crashing the entire process. This violates the project's promise of memory safety being non-optional.

This task replaces every RwLock `.unwrap()` with proper error propagation using a new `LockPoisoned` variant on `AdapterError`. The variant is defined using thiserror consistent with the codebase convention. Each `.read().unwrap()` and `.write().unwrap()` call is converted to use `.map_err()` with the `?` operator, allowing callers to handle lock poisoning gracefully instead of crashing. Tests are updated to verify that errors propagate correctly through the adapter interfaces.

## Context
35 `.unwrap()` calls on RwLock read/write in CLI adapters (32 in `in_memory.rs`, 3 in `http.rs`). A poisoned lock (from any thread panic while holding it) crashes the entire process. This contradicts promise #10 ("memory safety non-optional").

Critical areas:
- `module/iron_cli/src/adapters/implementations/http.rs` (lines 99, 109, 119)
- `module/iron_cli/src/adapters/implementations/in_memory.rs` (32 locations)

## Work Procedure

1. Locate the `AdapterError` enum definition and review existing variants for consistency
2. Add `LockPoisoned` variant to `AdapterError` with a thiserror `#[error("...")]` message
3. Open `in_memory.rs` and identify all 32 `.unwrap()` calls on RwLock operations
4. Replace each `.read().unwrap()` with `.read().map_err(|_| AdapterError::LockPoisoned)?`
5. Replace each `.write().unwrap()` with `.write().map_err(|_| AdapterError::LockPoisoned)?`
6. Open `http.rs` and replace the 3 `.unwrap()` calls on RwLock operations with the same pattern
7. Verify all function signatures return `Result` types compatible with the `?` operator
8. Run existing adapter tests to verify they still pass
9. Verify zero remaining `.unwrap()` calls on RwLock in adapter implementations

## Implementation plan
1. Add `LockPoisoned` variant to `AdapterError` (using thiserror per codebase convention).
2. Replace all `.write().unwrap()` and `.read().unwrap()` with `.map_err(|_| AdapterError::LockPoisoned)?`.
3. Update tests to verify error propagation instead of panics (per ADR-007: use real implementations, no mocking).

## Test Matrix

| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Normal read lock acquisition | Lock acquired, operation succeeds | Function returns Ok with expected data |
| Normal write lock acquisition | Lock acquired, operation succeeds | Function returns Ok |
| Read lock on poisoned RwLock | Error returned instead of panic | `AdapterError::LockPoisoned` propagated |
| Write lock on poisoned RwLock | Error returned instead of panic | `AdapterError::LockPoisoned` propagated |
| Existing in_memory adapter tests | All tests pass | No regressions |
| Existing http adapter tests | All tests pass | No regressions |
| Search for `.unwrap()` on RwLock | Zero matches | No unwrap calls on lock operations in adapter code |

## Validation Checklist

- [ ] `AdapterError::LockPoisoned` variant exists
- [ ] `LockPoisoned` variant has a thiserror `#[error("...")]` annotation
- [ ] Zero `.unwrap()` calls on RwLock in `in_memory.rs`
- [ ] Zero `.unwrap()` calls on RwLock in `http.rs`
- [ ] All replacements use `.map_err(|_| AdapterError::LockPoisoned)?`
- [ ] Existing adapter tests pass without modification
- [ ] Error propagation works correctly through adapter interfaces

## Validation Procedure

1. Verify `AdapterError::LockPoisoned` variant exists in the error enum definition
2. Verify the variant has a descriptive thiserror error message
3. Search `in_memory.rs` for `.unwrap()` - verify zero occurrences on RwLock operations
4. Search `http.rs` for `.unwrap()` - verify zero occurrences on RwLock operations
5. Count the total replacements: 32 in `in_memory.rs` + 3 in `http.rs` = 35
6. Run existing adapter tests and verify all pass
7. Verify that the `LockPoisoned` error variant is documented

## Acceptance Criteria
- Zero `.unwrap()` calls on RwLock in adapter implementations.
- `AdapterError::LockPoisoned` variant exists and is documented.
- Existing adapter tests continue to pass.
