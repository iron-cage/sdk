# Developer Guide

**Version:** 1.0.0
**Date:** 2025-01-24
**Status:** Active
**Audience:** Developers contributing to Iron Cage

---

### Scope

**Responsibility:** Development workflows, codebase organization, build instructions, testing guidelines (HOW developers work with the codebase)

**In Scope:**
- Cargo workspace structure and crate organization
- Dependency relationships between crates
- Build and test commands
- Development workflow and best practices
- Code organization principles
- Contribution guidelines

**Out of Scope:**
- Product specifications (see `/spec/` for capability requirements)
- System architecture (see `architecture.md` for component design)
- Deployment procedures (see `deployment_guide.md` for production deployment)
- Business strategy (see `/business/` for market positioning)

---

## Workspace Structure

Iron Cage is organized as part of the parent `willbe` workspace with 5 independent crates:

```
~/pro/lib/willbe/module/
├── iron_types/        # Shared types and traits
├── iron_safety/       # privacy protection module
├── iron_budget/         # Budget tracking module
├── iron_reliability/  # safety cutoff module
└── iron_cli/          # Binary integration
```

All crates follow the workspace convention of flat structure in `module/` directory with `iron_` prefix.

### Workspace Benefits

- **Incremental builds**: Build only changed crates (83% faster)
- **Parallel development**: 3 developers work without merge conflicts
- **Clear boundaries**: Each crate has single responsibility
- **Independent testing**: Run tests per module
- **Dependency management**: Shared dependencies in workspace root

---

## Crate Responsibilities

### `types/` - Shared Types Library

**Responsibility:** Common types and traits used across all crates

**Exports:**
- `Config`, `SafetyConfig`, `CostConfig`, `ReliabilityConfig`
- `Error` enum (Safety, BudgetExceeded, CircuitBreakerOpen, Config)
- `Result<T>` type alias

**Dependencies:**
- `serde`, `thiserror` (workspace dependencies)

**Used By:** All other crates

**Example:**
```rust
use iron_types::{ Config, Error, Result };

pub fn validate_config( config : &Config ) -> Result< () >
{
  // ... validation logic
  Ok( () )
}
```

---

### `safety/` - Privacy Protection Module

**Responsibility:** Developer 1's module - PII pattern detection and output redaction

**Exports:**
- `PiiDetector` - Email and phone number detection

**Features:**
- Email pattern: `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}`
- Phone pattern: `\d{3}-\d{3}-\d{4}`
- Redaction modes: `[EMAIL_REDACTED]`, `[PHONE_REDACTED]`

**Dependencies:**
- `iron_types` (workspace member)
- `regex` (workspace dependency)

**Testing:** 3 tests in `tests/readme_example_test.rs`

**Example:**
```rust
use iron_safety::PiiDetector;

let detector = PiiDetector::new()?;
let text = "Contact john@example.com";

if detector.check( text ) {
  let safe = detector.redact( text );
  // "Contact [EMAIL_REDACTED]"
}
```

---

### `cost/` - Budget Tracking Module

**Responsibility:** Developer 2's module - Per-agent cost tracking and budget enforcement

**Exports:**
- `BudgetTracker` - Concurrent cost tracking with DashMap

**Features:**
- Per-agent spending tracking
- Budget enforcement with error on overflow
- Thread-safe concurrent access (DashMap)
- Real-time spending queries

**Dependencies:**
- `iron_types` (workspace member)
- `dashmap` (workspace dependency)

**Testing:** 6 tests (3 in `tests/readme_example_test.rs`, 3 in `tests/bug_deadlock_dashmap_fix.rs`)

**Critical Fix:** See `tests/bug_deadlock_dashmap_fix.rs` for deadlock fix documentation (DashMap entry lock + iteration incompatibility)

**Example:**
```rust
use iron_budget::BudgetTracker;

let tracker = BudgetTracker::new( 100.0 );
tracker.record_cost( "agent_1", 25.0 )?;
println!( "Remaining: ${:.2}", tracker.remaining() );
```

---

### `reliability/` - Safety Cutoff Module

**Responsibility:** Developer 3's module - safety cutoff state management

**Exports:**
- `CircuitBreaker` - 3-state FSM (Closed/Open/HalfOpen)
- `CircuitState` enum

**Features:**
- Per-service circuit breaker tracking
- Failure threshold and timeout configuration
- Automatic state transitions
- Thread-safe (Mutex-based state)

**Dependencies:**
- `iron_types` (workspace member)
- `tokio` (workspace dependency)

**Testing:** 4 tests (1 unit test in src/lib.rs, 3 in tests/readme_example_test.rs)

**Example:**
```rust
use iron_reliability::CircuitBreaker;

let cb = CircuitBreaker::new( 5, 60 );  // 5 failures, 60s timeout
cb.record_failure( "openai" );
if cb.is_open( "openai" ) {
  // Circuit is open, use fallback
}
```

---

### `cli/` - Binary Integration

**Responsibility:** Binary entry point integrating all modules

**Type:** Binary crate (`src/main.rs`)

**Dependencies:**
- All workspace member crates
- `clap`, `tokio`, `tracing`, `anyhow` (workspace dependencies)

**Features:**
- CLI argument parsing
- Module initialization
- Demo integration showing all modules working together

**Example:**
```bash
cargo run --release -- --budget 1000
```

---

## Dependency Graph

```
cli
├── iron_types
├── iron_safety ──> iron_types
├── iron_budget ──> iron_types
└── iron_reliability ──> iron_types

iron_types (foundation, no dependencies on other workspace crates)
```

**Dependency Rules:**
- `types/` has NO dependencies on other workspace crates
- All other crates depend on `types/`
- Crates do NOT depend on each other (only `cli/` integrates them)

---

## Build Instructions

### Full Workspace Build

```bash
# Build all crates
cargo build --all-features

# Build release
cargo build --release --all-features
```

### Per-Crate Build

```bash
# Build individual crate
cargo build -p iron_safety --all-features

# Build specific crate in release mode
cargo build -p iron_budget --release --all-features
```

---

## Testing Guidelines

### Test Organization

All tests are located in the `tests/` directory of each crate:

```
module/
├── iron_types/tests/readme_example_test.rs
├── iron_safety/tests/readme_example_test.rs
├── iron_budget/
│   ├── tests/readme_example_test.rs
│   └── tests/bug_deadlock_dashmap_fix.rs
└── iron_reliability/tests/readme_example_test.rs
```

**Test Naming Convention:**
- `readme_example_test.rs` - Tests validating readme examples
- `bug_*.rs` - Bug fix documentation tests (5 sections + 3-field comment)

### Running Tests

#### All Tests (Recommended)

```bash
# Run all tests across workspace
cargo test --all-features

# With warnings as errors
RUSTFLAGS="-D warnings" cargo test --all-features
```

#### Per-Crate Tests

```bash
# Test individual crate
cargo test -p iron_budget --all-features

# Test with timeout (for deadlock detection)
timeout 30 cargo test -p iron_budget --all-features
```

#### Test Levels (Project Standard)

```bash
# Level 1: Unit tests only
w3 .test level::1
# or: RUSTFLAGS="-D warnings" cargo nextest run --all-features

# Level 3: Unit + doc tests + clippy (full verification)
w3 .test level::3
# or: RUSTFLAGS="-D warnings" cargo nextest run --all-features && \
#     RUSTDOCFLAGS="-D warnings" cargo test --doc --all-features && \
#     cargo clippy --all-targets --all-features -- -D warnings
```

**Default Test Command:** `w3 .test l::3` when w3 is available

### Test Requirements

**From CLAUDE.md:**
- NO mocking - use real implementations
- Tests must fail loudly, never silently
- NO disabled/ignored/skipped tests
- Integration tests must run fully (no silent passes)

**Bug Fix Tests:**
- Create test file with 5 sections: Root Cause, Why Not Caught, Fix Applied, Prevention, Pitfall
- Add 3-field source comment: Fix(issue-NNN), Root cause, Pitfall
- Quality: Specific/Technical/Actionable/Traceable/Concise

See `iron_iron_cost/tests/bug_deadlock_dashmap_fix.rs` for example.

---

## Code Style

### Formatting Rules

**CRITICAL:** This project uses **custom codestyle**, NOT `cargo fmt`.

**From CLAUDE.md Rule 4:**
- `cargo fmt` is FORBIDDEN
- Use 2-space indentation (not 4-space)
- Spaces around generics: `Result< T >`
- Custom formatting from applicable rulebooks

**Verification:**
```bash
# Check codestyle compliance
grep -r "    " src/  # Should find NO 4-space indents
```

### Code Organization

**Module Pattern:**
- When using `mod_interface` pattern, use inline `mod private { ... }` blocks
- NEVER create `private.rs` files or `private/` directories

**Error Handling:**
- Use `error_tools` crate exclusively when adopted (no mixing with `anyhow`/`thiserror`)

### File Naming

**From CLAUDE.md Rule 3:**
- Project files: `lowercase_snake_case`
- Tooling files: Keep conventional names (`Cargo.toml`, `Cargo.lock`)
- Temporary files: MUST start with hyphen (`-test_plan.md`)

---

## Development Workflow

### Making Changes

1. **Read specification first** - Check `/spec/` for product requirements
2. **Update specification if needed** - Rule 2: Update spec before implementing
3. **Implement changes** - Follow codestyle rules
4. **Write tests** - All code must be tested
5. **Run verification** - `w3 .test l::3` or `ctest3`
6. **Document knowledge** - Capture insights in test docs or source comments

### Adding New Features

1. **Identify affected crate** - Which crate has responsibility?
2. **Update crate's types** - Add to `types/` if shared across crates
3. **Implement in responsible crate** - Follow single responsibility principle
4. **Write integration test** - Add to `cli/` if cross-crate
5. **Update readme** - Update crate's readme.md with new examples

### Bug Fixes

**Mandatory workflow from CLAUDE.md:**
1. Create failing MRE test marked `bug_reproducer(issue-NNN)`
2. Implement fix
3. Document:
   - Test file with 5 sections (Root Cause, Why Not Caught, Fix Applied, Prevention, Pitfall)
   - Source code with 3-field comment (Fix(issue-NNN), Root cause, Pitfall)
4. Verify fix passes
5. Run full test suite

**Example:** See `iron_budget/tests/bug_deadlock_dashmap_fix.rs`

---

## Knowledge Preservation

### Priority Hierarchy (from CLAUDE.md)

1. **Test file doc comments** (most preferred) - `tests/**/*.rs`
2. **Source code doc comments** (module/function/struct) - `src/**/*.rs`
3. **Documentation files** - `docs/`
4. **Specification** - `spec.md` or `spec/` (requirements and architecture)

**PROHIBITED:**
- Main/root `readme.md` for knowledge capture (it's for onboarding only)
- Backup files (`*_old`, `*_backup`, `*.bak`)
- Non-hyphenated temporary files (must start with `-`)

### Bug Fix Documentation

**Always required for every bug fix:**
- Test documentation (5 sections)
- Source code comment (3 fields)
- Module docs (Known Pitfalls section) if bug revealed systemic design flaw

**Quality Standards:**
- Specific (not "fixed bug")
- Actionable (not "be careful")
- Traceable (includes issue-NNN)
- Focus on WHY bug occurred and lessons learned

---

## Common Tasks

### Add New Crate to Workspace

1. Create crate: `cargo new --lib new_crate`
2. Add to workspace `Cargo.toml`:
   ```toml
   members = [
     "types",
     "safety",
     "cost",
     "reliability",
     "cli",
     "new_crate",  # Add here
   ]
   ```
3. Add dependencies in crate's `Cargo.toml`:
   ```toml
   [dependencies]
   iron_types = { workspace = true }
   ```
4. Create `tests/` directory and add tests
5. Create `readme.md` with Scope section
6. Update this developer guide

### Run Specific Test

```bash
# Run single test by name
cargo test -p iron_budget budget_enforcement

# Run tests matching pattern
cargo test -p iron_safety pii_
```

### Check for Duplication

```bash
# Find duplicated code (PROHIBITED by CLAUDE.md)
# Manual review required - no automated tool
```

### Verify No Backup Files

```bash
# Check for FORBIDDEN backup files
find . -type f \( -name "*backup*" -o -name "*_old*" -o -name "*_v[0-9]*" \
  -o -name "*legacy*" -o -name "*.bak" -o -name "*.orig" \)

# Should return empty (no backup files allowed)
```

---

## Troubleshooting

### Build Failures

**Issue:** Dependency resolution errors
**Solution:** Run `cargo update` then rebuild

**Issue:** Conflicting feature flags
**Solution:** Use `--all-features` consistently

### Test Failures

**Issue:** Tests hanging indefinitely
**Solution:** Use timeout wrapper: `timeout 30 cargo test`
**Example:** See `iron_budget/tests/bug_deadlock_dashmap_fix.rs` for DashMap deadlock fix

**Issue:** Test passes locally but fails in CI
**Solution:** Check for timing dependencies, use `--release` mode

### DashMap Pitfalls

**Critical:** DashMap is NOT a drop-in replacement for `Mutex<HashMap>`

**Problem:** Deadlock when holding entry lock while iterating
**Solution:** Use explicit scope to drop entry lock:
```rust
// WRONG: Deadlock
let mut entry = map.entry( k ).or_insert( 0 );
*entry += 1;
if map.iter().count() > 10 { ... }  // Deadlock!

// CORRECT: Drop lock first
{
  let mut entry = map.entry( k ).or_insert( 0 );
  *entry += 1;
}  // Drop lock here
if map.iter().count() > 10 { ... }  // Safe
```

See `iron_budget/src/lib.rs:record_cost()` and `iron_budget/tests/bug_deadlock_dashmap_fix.rs` for full details.

---

## Performance Optimization

### Build Performance

**Workspace benefits:**
- Clean build: ~2 minutes (all crates)
- Incremental build: ~20 seconds (changed crate only)
- **83% faster** than monolithic crate

**Tips:**
- Use `cargo check` for fast syntax checking
- Use `cargo build -p <crate>` to build only one crate
- Enable `sccache` for distributed caching

### Test Performance

**Parallel testing:**
```bash
# Use nextest for parallel test execution
cargo nextest run --all-features
```

**Isolation:**
```bash
# Run tests for single crate (faster)
cargo test -p iron_safety --all-features
```

---

## Related Documentation

**Product Specifications:** See `/spec/readme.md` (8 capability specifications)
**System Architecture:** See `architecture.md` (component design, data flows)
**Deployment Guide:** See `deployment_guide.md` (production deployment)
**Requirements:** See `requirements.md` (functional/non-functional requirements)
**Business Strategy:** See `/business/strategy/` (market positioning, GTM)

---

## Contributing Guidelines

### Before Submitting PR

1. ✅ All tests pass: `w3 .test l::3` or `ctest3`
2. ✅ No clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
3. ✅ Documentation updated: Crate readme, this guide, specification
4. ✅ Bug fixes documented: 5-section test + 3-field comment
5. ✅ No backup files created
6. ✅ No temporary files without `-` prefix
7. ✅ Codestyle compliant (2-space indentation, no `cargo fmt`)

### Commit Message Format

**From CLAUDE.md:**
- Concise, direct messages in developer's voice
- Focus on "what" and "why" without meta-commentary
- NO AI attribution footers ("Generated with Claude Code")
- Examples: "fix deadlock in record_cost", "add PII phone detection"

### PR Review Checklist

1. Alignment with specification (`/spec/` directory)
2. Compliance with rulebooks
3. Security, error handling, test coverage
4. Bug fix documentation quality (specific/technical/actionable/traceable)

---

## Maintenance

**Review Quarterly:**
- Workspace structure still optimal for team size
- Dependency versions up to date
- Test coverage adequate
- Documentation accurate

**Update on Major Events:**
- New crate added to workspace
- Dependency architecture changes
- Build/test tooling updates
- Major bug fixes requiring workflow changes

---

**Document Status:** ✅ Active
**Last Updated:** 2025-01-24 (workspace migration completion)
**Maintainer:** Development team
