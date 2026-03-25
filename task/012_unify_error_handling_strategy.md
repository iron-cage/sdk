# Task 012: Unify error handling strategy

## Goal
Eliminate three inconsistent error handling strategies in the CLI layer so that all error paths use structured `Result` types with thiserror-derived enums, fully aligning with Principles 003 (Fail-Fast, Loud Failures). Success is observable when no `unwrap_or_else` with silent defaults remains in formatter code and only a single `build()` method exists on config. The scope covers `config.rs` and `tree_formatter.rs` only.

## Dependencies
- None

## In Scope
- Removing the panicking `build()` variant from config and renaming `build_result()` to `build()`
- Replacing all 13 `unwrap_or_else` silent-default sites in `tree_formatter.rs` with `Result` propagation
- Defining appropriate thiserror variants for formatter errors
- Updating all call sites that depend on the changed APIs

## Out of Scope
- Error handling in modules outside `iron_cli`
- Adding new CLI features or commands
- Refactoring error types in non-CLI crates

## Description
The CLI layer currently has three conflicting error strategies: handlers use structured `CliError` enums (correct), formatters silently swallow errors by returning default strings via `unwrap_or_else`, and config offers both a panicking `build()` and a Result-returning `build_result()`. This inconsistency violates the project's Fail-Fast and Loud Failures principles, making debugging harder and hiding real problems.

This task unifies all three approaches under the structured `Result` pattern with thiserror-derived enums. The config builder will have a single `build()` that returns `Result`, and all formatter sites will propagate errors upward instead of substituting silent defaults. This makes failures visible and actionable.

## Context
Three inconsistent error strategies across CLI violate Principles 003 (Fail-Fast, Loud Failures):
- Handlers use structured `CliError` enum via thiserror (correct).
- Formatters use `unwrap_or_else(|| "{}".to_string())` silently returning defaults (13 locations in `tree_formatter.rs`) - violates Loud Failures principle.
- Config has dual `build()` (panics) and `build_result()` (returns Result) - violates Fail-Fast principle.

Critical areas:
- `module/iron_cli/src/config.rs` (lines 167, 207)
- `module/iron_cli/src/formatting/tree_formatter.rs` (lines 78, 80, 111, 113, 185, 243, 247, 255, 263, 267, 274, 317, 322)

## Work Procedure
1. Audit `config.rs` to understand the dual `build()` / `build_result()` API and identify all callers.
2. Remove the panicking `build()` method and rename `build_result()` to `build()`.
3. Update every call site of the old `build()` to handle the `Result` return.
4. Define a `FormatterError` enum using thiserror to cover formatter failure cases.
5. In `tree_formatter.rs`, replace each of the 13 `unwrap_or_else` sites with `?` propagation returning `Result`.
6. Update function signatures in the formatter to return `Result<String, FormatterError>`.
7. Update all call sites of formatter functions to handle the new `Result` types.
8. Run `cargo test --workspace` and fix any breakage.
9. Run `cargo clippy --workspace` and resolve any new warnings.

## Implementation plan
1. Remove `build()` panicking variant from config. Rename `build_result()` to `build()`.
2. Replace all `unwrap_or_else(|| default)` in tree_formatter.rs with `Result` propagation using thiserror.
3. Update all call sites.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Config `build()` with valid inputs | Returns `Ok(Config)` | Config struct is correctly populated |
| Config `build()` with missing required field | Returns `Err(ConfigError)` | Error variant identifies the missing field |
| Formatter receives malformed input data | Returns `Err(FormatterError)` with descriptive message | Error propagates to caller, no silent default |
| Formatter receives valid tree data | Returns `Ok(formatted_string)` | Output matches expected formatted output |
| Calling removed panicking `build()` | Does not compile | No panicking `build()` method exists |

## Validation Checklist
- [ ] No `unwrap_or_else` with silent default values in `tree_formatter.rs`
- [ ] No panicking `build()` method exists in `config.rs`
- [ ] Single `build()` method returns `Result`
- [ ] All formatter public functions return `Result`
- [ ] thiserror-derived error enum exists for formatter errors
- [ ] All existing tests pass
- [ ] No new clippy warnings

## Validation Procedure
1. Search `tree_formatter.rs` for `unwrap_or_else` and confirm zero occurrences with silent defaults.
2. Search `config.rs` for `fn build(` and confirm only one method exists, returning `Result`.
3. Verify that `FormatterError` (or equivalent) is defined with `#[derive(thiserror::Error)]`.
4. Run `cargo test --workspace` and confirm all tests pass.
5. Run `cargo clippy --workspace` and confirm zero warnings.

## Acceptance Criteria
- No `unwrap_or_else` with silent default values in formatter code.
- Single `build()` method on config that returns `Result`.
- All existing tests pass.
