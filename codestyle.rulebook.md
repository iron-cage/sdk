# Codestyle

### Vocabulary
- **Rulebook:** This document, which provides a set of guidelines for formatting Rust code.
- **Rule:** An individual guideline within this rulebook, designed to ensure consistency, readability, and maintainability.
- **Task Markers:** Specially formatted comment tags (e.g., `xxx:`, `qqq:`, `aaa:`) used to track tasks, questions, and resolutions directly within the source code.

### Governing Principles
This rulebook provides a set of guidelines for formatting Rust code to ensure consistency, readability, and maintainability across the project. The foundational principle is that all rules apply universally to all Rust code, regardless of its location.

This project follows **standard Rust formatting** (as enforced by `rustfmt`) with a small number of explicit overrides documented below. Any aspect of formatting not covered in this document defaults to standard `rustfmt` behavior.

### Structure
Most rules in this document follow a consistent structure for clarity:
- **Description:** A detailed explanation of the rule's requirements.
- **Rationale:** An explanation of why the rule exists and the benefits of following it.
- **Examples:** `Good` and `Bad` examples illustrating correct and incorrect application of the rule.

### Quick Reference Summary

**Formatting & Whitespace**
* [Indentation: 2 Spaces](#formatting--whitespace--indentation-2-spaces)

**Imports & Modules**
* [Centralized Workspace Dependency Manifest](#imports--modules--centralized-workspace-dependency-manifest)
* [Forbid Undeclared Workspace Dependencies](#imports--modules--forbid-undeclared-workspace-dependencies)
* [Mandatory `enabled` and `full` Features for Crate Toggling](#imports--modules--mandatory-enabled-and-full-features-for-crate-toggling)
* [The `private` Namespace Must Be an Inline Module](#imports--modules--the-private-namespace-must-be-an-inline-module)

**Lints & Docs**
* [Lint and Warning Compliance](#lints--docs--lint-and-warning-compliance)
* [Strict Workspace Lint Inheritance](#lints--docs--strict-workspace-lint-inheritance)
* [Single Source of Truth for Crate Documentation](#lints--docs--single-source-of-truth-for-crate-documentation)
* [Set `html_root_url` for Public Crates](#lints--docs--set-html_root_url-for-public-crates)
* [Avoid Using Attributes for Documentation, Use Doc Comments](#lints--docs--avoid-using-attributes-for-documentation-use-doc-comments)

**Testing**
* [Centralized Test Directory](#testing--centralized-test-directory)
* [Centralized Benchmarks Directory](#testing--centralized-benchmarks-directory)
* [Benchmark Documentation Automation](#testing--benchmark-documentation-automation)
* [Strategic Benchmarking Focus](#testing--strategic-benchmarking-focus)

**Naming Conventions**
* [File Naming](#naming-conventions--file-naming)
* [Directory Naming Conventions](#naming-conventions--directory-naming-conventions)
* [Entity Naming Order (Noun-Verb)](#naming-conventions--entity-naming-order-noun-verb)

**Comments**
* [Spacing in Comments](#comments--spacing-in-comments)
* [Comment Content and Task Preservation](#comments--comment-content-and-task-preservation)
* [Defining and Using Task Markers](#comments--defining-and-using-task-markers)
* [Annotating Addressed Tasks](#comments--annotating-addressed-tasks)

**Secrets Management**
* [Secret Storage and Naming](#secrets-management--secret-storage-and-naming)
* [Ignoring Secrets with .gitignore](#secrets-management--ignoring-secrets-with-gitignore)

**Project Structure**
* [Canonical Directory Layout](#project-structure--canonical-directory-layout)

---

### Formatting & Whitespace : Indentation: 2 Spaces

**Description:** Use strictly 2 spaces for indentation. All other formatting follows standard `rustfmt` defaults.

The `rustfmt.toml` at the project root enforces this:

```toml
tab_spaces = 2
```

The formatter handles brace placement, spacing, line width, and all other formatting concerns automatically.

**Rationale:** 2-space indentation provides a more compact visual layout while maintaining readability, especially in deeply nested Rust code with generics and trait bounds.

---

### Imports & Modules : Centralized Workspace Dependency Manifest

**Description:** In a Cargo workspace, the root `Cargo.toml` **must** serve as the single source of truth for all dependency definitions. All dependencies — including their versions and sources — **must** be declared in the `[workspace.dependencies]` table. Feature flags **should not** be specified in this central manifest unless the feature is required by all consuming crates, or omitting it would force every crate to repeat the same feature list.

Member crates **must** inherit all dependencies from the workspace using the `workspace = true` syntax. A crate may specify the features it requires for an inherited dependency, but it is **strictly forbidden** from defining a dependency's version or source.

Only the workspace root `Cargo.toml` is allowed to import dependencies. Crate `Cargo.toml` files must reuse what the workspace imported. No exceptions.

**Rationale:** This approach ensures consistent versions across the entire workspace, simplifies dependency management, reduces redundancy, and allows each crate to enable only the specific features it needs. Specifying universally-required features in the workspace manifest avoids repetition across all crates; specifying crate-specific features in the workspace manifest forces unwanted features on crates that do not need them.

> Bad (Dependencies defined directly in crate `Cargo.toml`)

```toml
# my_crate/Cargo.toml
[dependencies]
# FORBIDDEN: This dependency is not inherited from the workspace.
rand = "0.8"
```

> Bad (Crate-specific features specified in the workspace manifest)

```toml
# workspace_root/Cargo.toml
[workspace.dependencies]
# FORBIDDEN: "rc" is only needed by some crates — specify it in the consuming crate.
serde = { version = "1.0", features = ["rc"] }
```

> Good (Universal feature in workspace; crate-specific feature in crate)

```toml
# workspace_root/Cargo.toml
[workspace.dependencies]
# OK: "derive" is required by all consuming crates.
serde = { version = "1.0", features = ["derive"] }
rand = { version = "0.8" }

# my_crate_a/Cargo.toml
[dependencies]
serde = { workspace = true }
rand = { workspace = true }

# my_crate_b/Cargo.toml
[dependencies]
# Adds crate-specific "rc" on top of the workspace-provided "derive".
serde = { workspace = true, features = ["rc"] }
```

### Imports & Modules : Forbid Undeclared Workspace Dependencies

**Description:** In a Cargo workspace, it is **strictly forbidden** for any member crate's `Cargo.toml` to reference a dependency — even using `workspace = true` — that has not first been explicitly declared in the `[workspace.dependencies]` table of the root `Cargo.toml` file.

Every single dependency used by any crate in the workspace **must** originate from the central workspace definition. There are no exceptions.

**Rationale:**
- **Single Source of Truth:** Enforces the root `Cargo.toml` as the absolute single source of truth for all dependencies, their versions, and their sources.
- **Security and Compliance:** Prevents crates from introducing unvetted dependencies.
- **Version Control:** Eliminates version conflicts or resolution ambiguity.

> Bad (Crate references a dependency not declared in the workspace)

```toml
# workspace_root/Cargo.toml
[workspace.dependencies]
serde = { version = "1.0" }

# my_crate/Cargo.toml
[dependencies]
# FORBIDDEN: 'phf_codegen' is not defined in [workspace.dependencies]
phf_codegen = { workspace = true }
serde = { workspace = true }
```

> Good (Dependency is declared in workspace, then inherited by the crate)

```toml
# workspace_root/Cargo.toml
[workspace.dependencies]
phf_codegen = { version = "0.11", default-features = false }
serde = { version = "1.0" }

# my_crate/Cargo.toml
[dependencies]
phf_codegen = { workspace = true }
serde = { workspace = true }
```

### Imports & Modules : Mandatory `enabled` and `full` Features for Crate Toggling

**Description:** This is a rigid rule for managing complex build configurations **for every crate that is a member of the workspace**. It does not apply to external, third-party dependencies. Every workspace crate **must** expose two specific features: `enabled` and `full`.

1. **`enabled` Feature:** Acts as a master switch for the entire crate.
    * It **must** be part of the `default` feature set, ensuring the crate is active by default.
    * It **must** activate all the crate's dependencies (which must be declared as optional).
2. **`full` Feature:** Provides a convenient way to enable all functionality.
    * It **must** be defined to include the `enabled` feature, along with any other optional features the crate provides.
3. **Dependency Gating:** All dependencies of the crate **must** be declared as `optional = true` and activated via the `enabled` feature.
4. **Code Gating:** The entire functional code within the crate's entry points (`lib.rs`, `main.rs`, etc.) **must** be conditionally compiled under the `enabled` feature using `#[cfg(feature = "enabled")]`.

**Rationale:** Cargo's feature system is additive, which makes it difficult to manage complex or mutually exclusive dependency sets. The `enabled` feature pattern provides a robust workaround — it allows a crate to be completely "switched off" or compiled-out, even when it is included as a non-optional dependency by another crate.

> Bad (Dependencies are not optional; code is not gated)

```toml
# my_crate/Cargo.toml
[dependencies]
# FORBIDDEN: Dependencies must be optional and gated by the "enabled" feature.
serde = { workspace = true }
```

```rust
// my_crate/src/lib.rs
// FORBIDDEN: The crate's code is not conditionally compiled.
pub fn my_api() -> bool {
  true
}
```

> Good (Correct implementation of the `enabled` and `full` feature pattern)

```toml
# my_crate/Cargo.toml

[features]
default = ["enabled"]
enabled = ["dep:serde", "dep:log"]
full = ["enabled"]

[dependencies]
serde = { workspace = true, optional = true }
log = { workspace = true, optional = true }
```

```rust
// my_crate/src/lib.rs

#![cfg_attr(not(feature = "enabled"), allow(unused))]

#[cfg(feature = "enabled")]
mod implementation {
  pub fn my_api() -> bool {
    true
  }
}

#[cfg(feature = "enabled")]
pub use implementation::*;
```

### Imports & Modules : The `private` Namespace Must Be an Inline Module

**Description:** If a module uses a `mod private` block to encapsulate implementation details, the `mod private` **must** be defined inline within its parent module file (e.g., `mod.rs`). It is **strictly forbidden** to move it into a separate file (e.g., `my_module/private.rs`) or directory (`my_module/private/`).

**Strictly Prohibited Patterns:**
- `private.rs` files in any location
- `private/` directories in any location
- `mod private;` declarations that reference external files

**Rationale:**
- **Architectural Integrity:** The `private` module is not a standard submodule. Its purpose is to contain implementation details co-located with the public API definition.
- **Encapsulation:** Moving the implementation to a separate file would break the clear "private implementation" pattern.
- **Readability:** Keeping the implementation inline makes the module self-contained.

---

### Lints & Docs : Lint and Warning Compliance

Make sure you have no warnings from clippy with these lints enabled.

**Recommended Lints Configuration:**

```toml
[workspace.lints.rust]
# Warn on unsafe code (not deny - we need FFI for landlock/seccomp)
unsafe_code = "warn"
# Warn if public items lack documentation
missing_docs = "warn"
# Denies non-idiomatic code for Rust 2018 edition
rust_2018_idioms = { level = "warn", priority = -1 }
# Denies using features that may break in future Rust versions
future_incompatible = { level = "warn", priority = -1 }
# Warns for public types not implementing Debug
missing_debug_implementations = "warn"

[workspace.lints.clippy]
# Denies pedantic lints, enforcing strict coding styles and conventions
pedantic = { level = "warn", priority = -1 }
# Denies undocumented unsafe blocks
undocumented_unsafe_blocks = "deny"
# Denies to prefer `core` over `std` when available, for `no_std` compatibility
std_instead_of_core = "warn"
# Denies including files in documentation unconditionally
doc_include_without_cfg = "warn"

# Exceptions
single_call_fn = "allow"
inline_always = "allow"
module_name_repetitions = "allow"
absolute_paths = "allow"
wildcard_imports = "allow"
std_instead_of_alloc = "allow"
items_after_statements = "allow"
cast_precision_loss = "allow"
pub_use = "allow"
question_mark_used = "allow"
implicit_return = "allow"
arbitrary_source_item_ordering = "allow"
mod_module_files = "allow"
missing_docs_in_private_items = "allow"
# Don't require inline in public items (too restrictive for this use case)
missing_inline_in_public_items = "allow"
```

### Lints & Docs : Strict Workspace Lint Inheritance

**Description:** The root `Cargo.toml` serves as the **single, authoritative manifest for all lint configurations**. All lint settings for both `rustc` and `clippy` **must** be defined exclusively in the root `Cargo.toml`.

Member crates **must not** define their own lint configurations. The `[lints]` section in a member crate's `Cargo.toml` must contain **only** the line `workspace = true` and nothing else. It is **strictly forbidden** for a member crate to define its own `[lints.rust]` or `[lints.clippy]` tables, override individual lints, or use `#![...]` attributes in source files for lint configuration.

**Rationale:**
- **Universal Code Quality:** Enforces a single, consistent standard across every crate.
- **Simplified Maintenance:** All lint settings are managed in one place.
- **Clarity and Predictability:** The build and CI process behaves predictably for all crates.

> Bad

```toml
# my_crate/Cargo.toml
[lints.rust]
unsafe_code = "deny"  # FORBIDDEN: Lints must not be defined in a member crate.
```

> Good

```toml
# workspace_root/Cargo.toml
[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "warn"

# my_crate/Cargo.toml
[lints]
workspace = true
```

### Lints & Docs : Single Source of Truth for Crate Documentation

**Description:** The `readme.md` file **must** serve as the single source of truth for crate-level documentation. All library (`lib.rs`) and binary (`main.rs` or `src/bin/*.rs`) entry points **must** include the contents of the `readme.md` file as their inner doc comments.

The **only acceptable method** is a two-part approach at the top of the entry file:
1. A single-line inner doc comment (`//!`) providing a brief crate summary. This satisfies the `missing_docs` lint.
2. The conditional `cfg_attr` attribute immediately following it to include the full `readme.md` content when building documentation (`cargo doc`).

**Rationale:** Prevents documentation from becoming out of sync between the README and the crate's own docs.

> Good

```rust
// In src/lib.rs
//! A brief, one-line summary of the crate.
#![cfg_attr(doc, doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "readme.md")))]
```

### Lints & Docs : Set `html_root_url` for Public Crates

**Description:** For any public-facing crate (intended for publishing to `crates.io`), the `lib.rs` file **must** include the `html_root_url` attribute for correct link generation on `docs.rs`.

> Good

```rust
#![doc(html_root_url = "https://docs.rs/your_crate_name/latest/your_crate_name/")]
```

### Lints & Docs : Avoid Using Attributes for Documentation, Use Doc Comments

For documenting code, prefer using ordinary doc comments `//!` and `///` over attributes like `#![doc = ""]`. Doc comments are more conventional and readable.

> Bad

```rust
#![doc = "Description of file."]
#[doc = "Implements a new type of secure connection."]
mod secure_connection {
  #[doc = "Establishes a secure link."]
  pub fn establish() {}
}
```

> Good

```rust
//! Description of file.

/// Implements a new type of secure connection.
mod secure_connection {
  /// Establishes a secure link.
  pub fn establish() {}
}
```

---

### Testing : Centralized Test Directory

**Description:** All tests, including unit tests and integration tests, **must** be located in the top-level `tests` directory of the crate. It is **strictly forbidden** to have:

1. `#[cfg(test)]` modules or any `#[test]` functions inside the `src` directory
2. Test files in the `examples` directory
3. Files ending with `_test.rs` anywhere except in the `tests` directory

**Common Violations to Avoid:**
- Files named `*_test.rs`, `test_*.rs`, or containing "test" in the filename in the `examples` directory
- Files with `#[test]` functions anywhere except the `tests` directory

**Rationale:**
- **Strict Separation of Concerns:** Clean boundary between production code (`src`), test code (`tests`), and demonstration code (`examples`).
- **Faster Builds:** `cargo build` and `cargo check` will not compile any test code.
- **Unified Test Environment:** All tests can be discovered from a single, predictable location using `cargo test`.

> Bad (Inline test module in `src`)

```rust
// In src/my_module.rs
pub fn add(a: i32, b: i32) -> i32 {
  a + b
}

// FORBIDDEN: Test modules are not allowed in the `src` directory.
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_add() {
    assert_eq!(add(2, 2), 4);
  }
}
```

> Good (Test code is in the `tests` directory)

```rust
// In tests/basic_test.rs
use my_crate::add;

#[test]
fn test_add_from_outside() {
  assert_eq!(add(2, 2), 4);
}
```

### Testing : Centralized Benchmarks Directory

**Description:** All benchmarks and benchmark-related files **must** be located in the top-level `benches` directory of the crate (plural, not singular).

**Mandatory Requirements:**
1. **Directory Name:** Must be `benches/` (plural) at the crate root
2. **Documentation:** `benches/readme.md` file is **mandatory** for every crate with benchmarks
3. **No temporary files:** Only permanent, committed benchmark files allowed in `benches/`

**Strictly Forbidden:**
- Benchmark files in `examples/`, `src/`, or `tests/` directories
- Using `benchmarks/` (singular) instead of `benches/` (plural)
- Missing `readme.md` in the `benches/` directory

**Cargo.toml Integration:** All benchmarks must be declared using `[[bench]]` sections pointing to `benches/` directory.

**Rationale:**
- **Rust Ecosystem Standard:** `benches/` is the official Cargo convention
- **Tool Compatibility:** `cargo bench` expects benchmarks in `benches/`
- **Clear Documentation:** Mandatory `readme.md` ensures benchmarks are documented

> Good

```text
benches/
├── readme.md              # MANDATORY: Documents all benchmarks
├── core_algorithms.rs
└── parsing_performance.rs
```

```toml
[[bench]]
name = "core_algorithms"
path = "benches/core_algorithms.rs"
harness = false
```

### Testing : Benchmark Documentation Automation

**Description:** All benchmark documentation files (`.md`) in the `benches/` directory **must** be updated automatically by the benchmarking utilities during their execution. Manual editing of these files is **strictly forbidden**.

**Requirements:**
1. **Automated `benches/readme.md` Updates:** Benchmark runners must automatically generate and update the readme
2. **Automated `benches/changes.md` Updates:** Major performance changes must be automatically logged
3. **No Manual Editing:** Human editing of benchmark documentation is prohibited to prevent inconsistencies

**Rationale:** Automation ensures that performance documentation is always synchronized with the latest benchmark results.

### Testing : Strategic Benchmarking Focus

**Description:** Benchmarking efforts **must** be strategic, focusing on identifying and measuring critical performance bottlenecks rather than aiming for exhaustive coverage.

**Strategic Philosophy:**
- **Avoid Proliferation:** Do not create an excessive number of benchmarks. Prioritize the most performance-sensitive code paths.
- **Focus on Bottlenecks:** Use profiling tools (e.g., `perf`, `flamegraph`) to identify actual bottlenecks *first*. Then, create targeted benchmarks.
- **Measure What Matters:** Focus on metrics that directly impact user experience or system efficiency.

---

### Naming Conventions : File Naming

Custom file names should use `snake_case` and be in all lowercase letters. **Exception: Standard tooling files** (e.g., `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `Makefile`) must retain their conventional names for proper tool recognition.

**Protected File Names:**
- `Cargo.toml`, `Cargo.lock` (Rust ecosystem)
- `Dockerfile`, `docker-compose.yml` (Docker ecosystem)
- `Makefile` (Make build system)
- `.gitignore`, `.gitattributes` (Git)

**Application of `lowercase_snake_case`:**
- Custom source files (`my_module.rs`)
- Project-specific documentation (`user_guide.md`, `readme.md`)
- Repository files (`license`)
- Custom configuration files (`app_config.toml`)

> Good

```text
my_module.rs
user_guide.md
readme.md            # NOT README.md
license              # NOT LICENSE
Cargo.toml           # Standard tooling file (protected)
Dockerfile           # Standard tooling file (protected)
```

> Bad

```text
MyModule.rs          # Should use snake_case
my-module.rs         # Should use snake_case
README.md            # Should be readme.md
LICENSE              # Should be license
```

### Naming Conventions : Directory Naming Conventions

**Description:** If a crate's name contains a prefix that matches the name of its parent directory, this prefix **must** be removed from the crate's own directory name on the filesystem. The full crate name, including the prefix, **must** be preserved in its `Cargo.toml` file.

**Rationale:** Eliminates redundancy and "stuttering" in file paths (e.g., `api/api_gemini`), leading to cleaner project navigation.

> Bad

```text
└── api/
    └── api_gemini/   # Redundant prefix
        └── Cargo.toml
```

> Good

```text
└── api/
    └── gemini/       # Clean, non-redundant
        └── Cargo.toml
```

```toml
# In api/gemini/Cargo.toml
[package]
name = "api_gemini"  # The full name is preserved here
```

### Naming Conventions : Entity Naming Order (Noun-Verb)

For entities like functions, types, or variables that combine a noun (the subject) and a verb (the action), the noun must precede the verb. This is 'subject-action' ordering.

> Good

```rust
fn files_delete() { /* ... */ }
fn user_create() { /* ... */ }
```

> Bad

```rust
fn delete_files() { /* ... */ }
fn create_user() { /* ... */ }
```

---

### Comments : Spacing in Comments

Inline comments (`//`) should start with a space following the slashes for readability.

### Comments : Comment Content and Task Preservation

**Description:** Comments should primarily explain the "why" or clarify non-obvious aspects of the *current* code. Avoid adding comments that merely state *what* change was just made (e.g., "Removed unused import", "Added derive") or serve purely as a historical log.

**Crucially, do not remove existing task-tracking comments.** These are typically prefixed with labels like `TODO:`, `FIXME:`, `xxx:`, `qqq:`, `ppp:`, `yyy:`, `iii:`, or similar conventions, and are essential for project management and future development.

> Bad (Comment describes the *change*, not the *code*)

```rust
// Removed unused import: use std::collections::HashMap;
use std::fmt;

struct MyData {
  // Added field for caching
  cache_value: Option<i32>,
}
```

> Good (No comment needed for obvious change, or comment explains *why*)

```rust
use std::fmt;

struct MyData {
  /// Stores a cached computation result to avoid re-calculation.
  /// Cleared when relevant inputs change.
  cache_value: Option<i32>,
}
```

### Comments : Defining and Using Task Markers

**Description:** Use structured `Task Markers` in source code comments to track tasks, requests, and their resolutions.

**Schema:**
`// <marker> : [optional context/person] : <description>`

**Marker Types & Meanings:**
- `xxx:`, `todo:`: A general-purpose task or something that needs to be done. Prefer `xxx:` for consistency.
- `qqq:`: A question or a request for a decision, often from a team lead to a developer. The developer should not change the `qqq:` line itself but should respond with an `aaa:` marker.
- `aaa:`: An answer or a report on an action taken in response to another marker (typically a `qqq:` or `xxx:`). It should be placed directly below the marker it addresses.
- `zzz:`: A low-priority task that can be deferred.

> Good

```rust
// xxx: @dev-team : This function is inefficient and needs to be refactored.
// It currently uses a linear search, but a HashMap would be better.
fn find_item_slowly(id: &str) -> Option<Item> { /* ... */ }

// qqq: @lead-dev : Should we support legacy format v1 in this parser?
fn parse_data(data: &[u8]) -> Result<Data, ParseError> { /* ... */ }

// aaa: @dev : Yes, we need to support v1 for now. Please proceed.

// zzz: The logging here is a bit verbose. Could be cleaned up in the future.
log::debug!("Processing item: {:?}", item);
```

### Comments : Annotating Addressed Tasks

**Description:** When addressing or investigating an existing task comment (e.g., `// TODO:`, `// xxx:`, `// FIXME:`), **do not remove the original task comment**. Instead, add a new comment line immediately below it, starting with `// aaa:`, explaining the findings, actions taken, or current status.

> Bad (Removing the original task comment)

```rust
fn calculate_value() -> i32 {
  // Original was: // xxx: This calculation might be wrong for edge cases.
  // aaa: Reviewed calculation, seems correct for expected inputs.
  5
}
```

> Good (Adding `aaa:` annotation directly below the original task)

```rust
fn calculate_value() -> i32 {
  // xxx: This calculation might be wrong for edge cases.
  // aaa: Reviewed calculation, seems correct for expected inputs based on current requirements.
  5
}
```

---

### Secrets Management : Secret Storage and Naming

**Description:** All secrets, credentials, API keys, and sensitive configuration files **must** be stored in the `secret/` directory at the project root. Secret files **must** use the `-` prefix naming convention (e.g., `-api_keys.sh`, `-database.conf`).

> Good

```text
secret/
├── readme.md           # MANDATORY: Secret management documentation
├── -api_keys.sh        # Secret files (- prefix mandatory)
└── -database.conf      # Service configurations
```

### Secrets Management : Ignoring Secrets with .gitignore

**Description:** The `.gitignore` file **must** contain patterns that prevent any secret files from being committed to version control. At minimum, the `secret/` directory contents (except `readme.md`) must be ignored.

---

### Project Structure : Canonical Directory Layout

**Description:** This is the strongly recommended directory structure for all projects. **Structure variations are possible if justified by specific project needs**, but file type separation rules remain absolute. **Any of these directories may be absent** if not needed.

**Strongly Recommended Directory Structure:**
```
project_root/
├── src/                          # Production code ONLY
├── spec/                         # Project specification
│   └── readme.md                 # MANDATORY: Specification overview
├── tests/                        # ALL functional/integration tests
│   ├── readme.md                 # MANDATORY: Test organization & principles
│   └── manual/                   # Manual testing (if needed)
│       └── readme.md             # MANDATORY: Manual testing plan
├── benches/                      # ALL performance/benchmark tests
│   └── readme.md                 # MANDATORY: Benchmark organization
├── examples/                     # Usage demonstrations ONLY
│   └── readme.md                 # MANDATORY: Examples organization
├── secret/                       # Secrets and credentials
│   ├── readme.md                 # MANDATORY: Secret management documentation
│   └── -api_keys.sh             # Secret files (- prefix mandatory)
├── readme.md                     # Primary documentation (lowercase)
├── license                       # License file (lowercase)
└── [tooling files]               # Language-specific tooling (Cargo.toml, etc.)
```

**Mandatory readme.md Requirements:**
Every directory (except `src/`) **must** contain a `readme.md` file (lowercase, never `README.md`) with:
1. **Purpose Statement:** Clear explanation of directory's role
2. **Organization Principles:** How files are categorized
3. **Navigation Guide:** How to find specific items

**ABSOLUTE File Type Separation Rules (No Exceptions):**
- **Performance/Benchmark tests:** MUST be in `benches/` directory
- **Functional tests:** MUST be in `tests/` directory
- **Specifications:** MUST be in `spec/` directory
- **Secrets:** MUST be in `secret/` directory with `-` prefix naming
- **Production code:** Must be separated from tests and benchmarks
- **Examples:** Must be demonstrations only, NO tests, NO benchmarks
