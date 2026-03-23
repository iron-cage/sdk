# Task 011: Upgrade to Rust 2024 edition

## Dependencies
- None

## Context
Workspace uses edition 2021 (MSRV 1.85). Rust 2024 edition is available with improved `impl Trait` capture rules and `unsafe extern` semantics.

Critical areas:
- `Cargo.toml` (workspace edition field)
- Any `unsafe extern` blocks
- Any `impl Trait` return types that capture differently under 2024 rules

## Implementation plan
1. Change workspace edition from `"2021"` to `"2024"` in `Cargo.toml`.
2. Run `cargo build --workspace` and fix any breakage.
3. Run `cargo clippy --workspace` and fix new lints.
4. Run `cargo test --workspace` and verify all tests pass.

## Acceptance criteria
- `edition = "2024"` in workspace Cargo.toml.
- All tests pass.
- Zero clippy warnings.
