# Task 011: Upgrade to Rust 2024 edition

## Goal
Upgrade the entire workspace from Rust edition 2021 to 2024, ensuring all crates compile, pass tests, and produce zero clippy warnings under the new edition semantics. This is observable by verifying the workspace Cargo.toml declares `edition = "2024"` and the full CI pipeline remains green. The scope is limited to edition migration and any resulting fix-ups - no feature work.

## Dependencies
- None

## In Scope
- Changing the workspace-level `edition` field in the root `Cargo.toml`
- Fixing any `impl Trait` capture rule changes introduced in edition 2024
- Updating any `unsafe extern` blocks to comply with 2024 semantics
- Resolving new clippy lints introduced by the edition change
- Updating MSRV if required by the 2024 edition

## Out of Scope
- Adding new features or refactoring unrelated code
- Upgrading dependency crate versions beyond what the edition requires
- Changing the project's CI pipeline configuration

## Description
The iron-cage workspace currently targets Rust edition 2021 with an MSRV of 1.85. Rust 2024 edition introduces meaningful language changes including revised `impl Trait` lifetime capture rules and stricter `unsafe extern` block semantics. Migrating now ensures the project benefits from the latest language improvements and avoids accumulating technical debt.

This task is a mechanical upgrade: change the edition field, build, fix any breakage, clear clippy warnings, and verify all tests pass. The scope is deliberately narrow to keep the change reviewable and low-risk.

## Context
Workspace uses edition 2021 (MSRV 1.85). Rust 2024 edition is available with improved `impl Trait` capture rules and `unsafe extern` semantics.

Critical areas:
- `Cargo.toml` (workspace edition field)
- Any `unsafe extern` blocks
- Any `impl Trait` return types that capture differently under 2024 rules

## Work Procedure
1. Read the current workspace `Cargo.toml` and note the edition and MSRV fields.
2. Review the Rust 2024 edition migration guide for breaking changes relevant to this codebase.
3. Change `edition = "2021"` to `edition = "2024"` in the workspace `Cargo.toml`.
4. Run `cargo build --workspace` and catalogue all compilation errors.
5. Fix each compilation error, focusing on `impl Trait` capture changes and `unsafe extern` blocks.
6. Run `cargo clippy --workspace -- -D warnings` and resolve all new lints.
7. Run `cargo test --workspace` and verify the full test suite passes.
8. Search for any per-crate `edition` overrides and update them if present.

## Implementation plan
1. Change workspace edition from `"2021"` to `"2024"` in `Cargo.toml`.
2. Run `cargo build --workspace` and fix any breakage.
3. Run `cargo clippy --workspace` and fix new lints.
4. Run `cargo test --workspace` and verify all tests pass.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| `cargo build --workspace` after edition change | All crates compile without errors | Exit code 0, no compilation errors |
| `cargo clippy --workspace -- -D warnings` | No new warnings or errors | Exit code 0, zero warnings |
| `cargo test --workspace` | All existing tests pass | Exit code 0, no test failures |
| Code with `impl Trait` return types | Captures behave correctly under 2024 rules | Affected functions compile and tests pass |
| Any `unsafe extern` blocks | Comply with 2024 stricter semantics | Compilation succeeds without unsafe warnings |

## Validation List
- [ ] Workspace `Cargo.toml` contains `edition = "2024"`
- [ ] No per-crate `Cargo.toml` overrides edition to 2021
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo clippy --workspace -- -D warnings` produces zero warnings
- [ ] `cargo test --workspace` passes all tests
- [ ] MSRV field updated if the 2024 edition requires a newer minimum version

## Validation Procedure
1. Open the root `Cargo.toml` and confirm `edition = "2024"` is set in the workspace section.
2. Run `cargo build --workspace` and verify clean compilation.
3. Run `cargo clippy --workspace -- -D warnings` and confirm zero diagnostics.
4. Run `cargo test --workspace` and confirm all tests pass with no failures.
5. Check `rust-version` (MSRV) field is consistent with the minimum toolchain that supports edition 2024.

## Acceptance criteria
- `edition = "2024"` in workspace Cargo.toml.
- All tests pass.
- Zero clippy warnings.
