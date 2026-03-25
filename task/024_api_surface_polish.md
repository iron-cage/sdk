# Task 024: API surface polish

## Goal
Prepare all public APIs for the 1.0.0 release by ensuring naming consistency, forward compatibility via `#[non_exhaustive]`, comprehensive rustdoc with examples on every public item, and uniform crate versioning. The API surface must be ergonomic and well-documented so that external consumers can adopt iron-cage without reading internal source code.

## Dependencies
- Stage 2 completion (Tasks 020-023)

## In Scope
- Audit and rename all public items for naming consistency across the workspace
- Add `#[non_exhaustive]` to all public error enums
- Comprehensive rustdoc with examples on all public types, functions, and traits
- Version all crates at 1.0.0 in their Cargo.toml files
- Zero warnings from `cargo doc --workspace --no-deps`

## Out of Scope
- Internal (non-public) API cleanup or refactoring
- Semver compatibility tooling or automated API diff checks
- Publishing crates to crates.io (covered by CI/CD task)

## Description
As iron-cage approaches its 1.0.0 public release, the API surface must be polished to meet the expectations of external consumers. This means every public type, function, trait, and error enum needs consistent naming that follows Rust conventions (no abbreviations without precedent, verb-noun patterns for functions, type-safe builders where appropriate).

All public error enums must be annotated with `#[non_exhaustive]` to allow adding new variants in future minor versions without breaking downstream code. Every public item must have rustdoc documentation that includes at least one usage example, giving consumers immediate clarity on how to use each API. Finally, all crate versions in the workspace must be set to 1.0.0 to signal stability and readiness for production use.

## Context
Before public release, all public APIs should be ergonomic, well-documented, and forward-compatible.

Critical areas:
- All `pub` items across workspace
- Public error enums
- Crate versions in Cargo.toml files

## Work Procedure
1. Run `cargo doc --workspace --no-deps 2>&1` and catalog all current warnings.
2. List all `pub` items across the workspace using `grep -rn "pub " --include="*.rs"` and review for naming consistency.
3. Rename any inconsistently named public items, updating all call sites.
4. Add `#[non_exhaustive]` to every public enum that represents errors.
5. Write rustdoc with at least one `# Examples` section for every public type, trait, and function.
6. Verify all doc examples compile by running `cargo test --doc --workspace`.
7. Update the `version` field in every crate's `Cargo.toml` to `1.0.0`.
8. Run `cargo doc --workspace --no-deps` and verify zero warnings.

## Implementation plan
1. Review all `pub` items across workspace for naming consistency.
2. Add `#[non_exhaustive]` on all public error enums.
3. Add comprehensive rustdoc with examples on all public types and functions.
4. Version all crates at 1.0.0.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| `cargo doc --workspace --no-deps` | Build completes | Zero warnings in output |
| `cargo test --doc --workspace` | All doc examples compile and run | Zero failures |
| Public error enum without `#[non_exhaustive]` | None exist | grep finds zero public error enums without the attribute |
| Public function without rustdoc | None exist | All `pub fn` items have `///` doc comments |
| Crate version check | All 1.0.0 | Every Cargo.toml has `version = "1.0.0"` |

## Validation Checklist
- [ ] All public types have rustdoc with at least one example
- [ ] All public error enums have `#[non_exhaustive]`
- [ ] `cargo doc --workspace --no-deps` produces zero warnings
- [ ] `cargo test --doc --workspace` passes
- [ ] All crate versions are set to 1.0.0
- [ ] Naming conventions are consistent across the workspace
- [ ] No public items use unexplained abbreviations

## Validation Procedure
1. Run `cargo doc --workspace --no-deps` and verify the output contains zero warnings.
2. Run `cargo test --doc --workspace` and verify all doc tests pass.
3. Search for public error enums without `#[non_exhaustive]` and confirm none exist.
4. Spot-check 10 public types across different crates to verify rustdoc quality and example presence.
5. Verify every `Cargo.toml` in the workspace has `version = "1.0.0"`.
6. Run `cargo build --workspace` to confirm no compilation regressions from renaming.

## Acceptance Criteria
- All public types have rustdoc with at least one example.
- All public error types are `#[non_exhaustive]`.
- `cargo doc --workspace --no-deps` produces zero warnings.
- Crate versions are 1.0.0.
