# Task 012: Unify error handling strategy

## Dependencies
- None

## Context
Three inconsistent error strategies across CLI violate Principles 003 (Fail-Fast, Loud Failures):
- Handlers use structured `CliError` enum via thiserror (correct).
- Formatters use `unwrap_or_else(|| "{}".to_string())` silently returning defaults (13 locations in `tree_formatter.rs`) - violates Loud Failures principle.
- Config has dual `build()` (panics) and `build_result()` (returns Result) - violates Fail-Fast principle.

Critical areas:
- `module/iron_cli/src/config.rs` (lines 167, 207)
- `module/iron_cli/src/formatting/tree_formatter.rs` (lines 78, 80, 111, 113, 185, 243, 247, 255, 263, 267, 274, 317, 322)

## Implementation plan
1. Remove `build()` panicking variant from config. Rename `build_result()` to `build()`.
2. Replace all `unwrap_or_else(|| default)` in tree_formatter.rs with `Result` propagation using thiserror.
3. Update all call sites.

## Acceptance criteria
- No `unwrap_or_else` with silent default values in formatter code.
- Single `build()` method on config that returns `Result`.
- All existing tests pass.
