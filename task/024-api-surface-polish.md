# Task 024: API surface polish

## Dependencies
- Stage 2 completion (Tasks 020-023)

## Context
Before public release, all public APIs should be ergonomic, well-documented, and forward-compatible.

Critical areas:
- All `pub` items across workspace
- Public error enums
- Crate versions in Cargo.toml files

## Implementation plan
1. Review all `pub` items across workspace for naming consistency.
2. Add `#[non_exhaustive]` on all public error enums.
3. Add comprehensive rustdoc with examples on all public types and functions.
4. Version all crates at 1.0.0.

## Acceptance criteria
- All public types have rustdoc with at least one example.
- All public error types are `#[non_exhaustive]`.
- `cargo doc --workspace --no-deps` produces zero warnings.
- Crate versions are 1.0.0.
