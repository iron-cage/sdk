# codestyle

### Vocabulary
-   **Rulebook:** This document, which provides a set of guidelines for formatting Rust code.
-   **Rule:** An individual guideline within this rulebook, designed to ensure consistency, readability, and maintainability.
-   **Task Markers:** Specially formatted comment tags (e.g., `xxx:`, `qqq:`, `aaa:`) used to track tasks, questions, and resolutions directly within the source code.
-   **wTools Ecosystem:** The collection of all projects, libraries, and tools developed under the wTools umbrella.

### Governing Principles
This rulebook provides a set of guidelines for formatting Rust code to ensure consistency, readability, and maintainability across the project. Adhering to these codestyle rules helps to create a uniform and professional-looking codebase. The foundational principle is that all rules apply universally to all Rust code, regardless of its location.

### Structure
Most rules in this document follow a consistent structure for clarity:
-   **Description:** A detailed explanation of the rule's requirements.
-   **Rationale:** An explanation of why the rule exists and the benefits of following it.
-   **Examples:** `✅ Good` and `❌ Bad` examples illustrating correct and incorrect application of the rule.

### Project Structure : Canonical Directory Layout

**Description:** This is the foundational rule defining the strongly recommended directory structure for all projects. All other rules reference and build upon this structure. **Structure variations are possible if justified by specific project needs**, but file type separation rules remain absolute. **Any of these directories may be absent** if not needed for the specific project.

**Strongly Recommended Directory Structure:**
```
project_root/
├── src/                          # Production code ONLY
├── spec/                        # Project specification (alternative to spec.md)
│   ├── readme.md               # MANDATORY: Specification overview & organization
│   ├── api_spec.md             # API specifications
│   └── architecture.md         # Architecture decisions
├── tests/                       # ALL functional/integration tests
│   ├── readme.md               # MANDATORY: Test organization & principles
│   ├── test_files.*            # Test files
│   └── manual/                 # Manual testing (if needed)
│       └── readme.md           # MANDATORY: Manual testing plan
├── benches/                     # ALL performance/benchmark tests
│   ├── readme.md              # MANDATORY: Benchmark organization & principles
│   ├── benchmark.*            # Benchmark files
│   └── data/                  # Benchmark data
├── examples/                    # Usage demonstrations ONLY
│   ├── readme.md              # MANDATORY: Examples organization & principles
│   ├── basic_usage.*          # Real-world usage examples
│   └── advanced_example.*     # Complex usage scenarios
├── secret/                      # Secrets and credentials
│   ├── readme.md              # MANDATORY: Secret management documentation
│   ├── -api_keys.sh           # Secret files (- prefix mandatory)
│   └── -database.conf         # Service configurations
├── spec.md                      # Project specification (alternative to spec/ dir)
├── readme.md                    # Primary documentation (lowercase)
├── license                      # License file (lowercase)
└── [tooling files]             # Language-specific tooling (Cargo.toml, package.json, etc.)
```

**CRITICAL: Mandatory readme.md Requirements:**
Every directory (except `src/`) **must** contain a `readme.md` file (lowercase, never `README.md`) with:

1. **Purpose Statement:** Clear explanation of directory's role in the project
2. **Organization Principles:** Explanation of categorization system used
3. **Navigation Guide:** How to find specific items (especially critical for directories with hundreds of files)

**Template for required readme.md structure:**
```markdown
# Directory Name

## Purpose
Brief explanation of this directory's role and what types of content it contains.

## Organization Principles
- How files are categorized
- Naming conventions used
- Grouping strategy for large numbers of files

## Navigation Guide
- For X, look in: category_1/
- For Y, look in: category_2/
- Common tasks: see specific_file.ext
```

**Directory Presence Rules:**
- **All directories are OPTIONAL:** Any directory may be absent if not needed for the project
- **Structure variations allowed:** Directory structure may be modified if justified by specific project requirements
- **Conditional readme.md:** Required only when directory exists and contains files
- **Specification Location:** Use either `spec.md` file OR `spec/` directory (not both)

**ABSOLUTE File Type Separation Rules (No Exceptions):**
- **Performance/Benchmark tests:** MUST be in `benches/` directory (or equivalent benchmark directory)
- **Functional tests:** MUST be in `tests/` directory (or equivalent test directory)
- **Specifications:** MUST be in `spec/` directory OR single `spec.md` file (not scattered)
- **Secrets:** MUST be in `secret/` directory with `-` prefix naming
- **Production code:** Must be separated from tests and benchmarks
- **Examples:** Must be demonstrations only, NO tests, NO benchmarks

**ABSOLUTE Prohibitions (Apply Regardless of Directory Structure):**
```
❌ FORBIDDEN EVERYWHERE:
- Performance/benchmark tests outside designated benchmark directory
- Functional tests mixed with performance tests
- Specifications scattered across multiple locations (except single spec.md)
- Secrets outside `secret/` directory
- Tests in examples directories
- Benchmarks in examples directories
- Production code mixed with test code
- Uppercase `README.md` files anywhere (use lowercase `readme.md`)
```

**Language Agnostic:** This structure applies to all programming languages with appropriate file extensions and tooling files.

**Rationale:** This canonical structure provides a strong foundation for project organization across all programming languages. While directory structure can be adapted to project needs, strict file type separation prevents cross-contamination, ensures intuitive navigation even with hundreds of files through mandatory organization documentation, and maintains clear boundaries between different kinds of code regardless of the specific directory layout chosen.

### Quick Reference Summary

**Scope & Applicability**
*   [Universal Applicability of Codestyle](#scope--applicability-universal-applicability-of-codestyle)

**Imports & Modules**
- [codestyle](#codestyle)
    - [Vocabulary](#vocabulary)
    - [Governing Principles](#governing-principles)
    - [Structure](#structure)
    - [Quick Reference Summary](#quick-reference-summary)
    - [Scope \& Applicability : Universal Applicability of Codestyle](#scope--applicability--universal-applicability-of-codestyle)
    - [Imports \& Modules : Structuring: Prefer Specific, Local, and Grouped `use`](#imports--modules--structuring-prefer-specific-local-and-grouped-use)
    - [Imports \& Modules : Structuring: Use of `crate::*`](#imports--modules--structuring-use-of-crate)
    - [Imports \& Modules : Structuring: Local Entities](#imports--modules--structuring-local-entities)
    - [Imports \& Modules : Structuring: Structuring `std` Imports](#imports--modules--structuring-structuring-std-imports)
    - [Imports \& Modules : Structuring: Explicit Exposure Rule](#imports--modules--structuring-explicit-exposure-rule)
    - [Imports \& Modules : Structuring: Factor Common Paths in Imports and Declarations](#imports--modules--structuring-factor-common-paths-in-imports-and-declarations)
    - [Imports \& Modules : Structuring: Use Multi-Line Grouping for Complex Imports](#imports--modules--structuring-use-multi-line-grouping-for-complex-imports)
    - [Imports \& Modules : Structuring: Hierarchical Formatting for Multi-Line `use` Statements](#imports--modules--structuring-hierarchical-formatting-for-multi-line-use-statements)
    - [Imports \& Modules : Structuring: Grouped Formatting for `mod` Declarations](#imports--modules--structuring-grouped-formatting-for-mod-declarations)
    - [Imports \& Modules : Structuring: Structuring Modules with `mod_interface`](#imports--modules--structuring-structuring-modules-with-mod_interface)
    - [Imports \& Modules : Structuring: Integrating Child Layers with `use` in `mod_interface`](#imports--modules--structuring-integrating-child-layers-with-use-in-mod_interface)
    - [Imports \& Modules : Structuring: Keep all Definitions and Details Inside `private` Namespace](#imports--modules--structuring-keep-all-definitions-and-details-inside-private-namespace)
    - [Imports \& Modules : Structuring: The `private` Namespace Must Be an Inline Module](#imports--modules--structuring-the-private-namespace-must-be-an-inline-module)
    - [Imports \& Modules : Structuring: The `private` Namespace Must Not Contain Submodules](#imports--modules--structuring-the-private-namespace-must-not-contain-submodules)
    - [Imports \& Modules : Structuring: Do Not Nest Child Modules Inside `private`](#imports--modules--structuring-do-not-nest-child-modules-inside-private)
    - [Imports \& Modules : Structuring: Preferring `use` or `reuse` Over `layer` in `mod_interface`](#imports--modules--structuring-preferring-use-or-reuse-over-layer-in-mod_interface)
    - [Imports \& Modules : Structuring: Maintain Definition Order in Exports](#imports--modules--structuring-maintain-definition-order-in-exports)
    - [Imports \& Modules : Structuring: Centralized Workspace Dependency Manifest](#imports--modules--structuring-centralized-workspace-dependency-manifest)
    - [Imports \& Modules : Structuring: Forbid Undeclared Workspace Dependencies](#imports--modules--structuring-forbid-undeclared-workspace-dependencies)
    - [Imports \& Modules : Mandatory `enabled` and `full` Features for Crate Toggling](#imports--modules--mandatory-enabled-and-full-features-for-crate-toggling)
    - [Imports \& Modules : Dependencies: Prefer wTools Ecosystem Crates](#imports--modules--dependencies-prefer-wtools-ecosystem-crates)
    - [Lints \& Docs : Lint and Warning Compliance](#lints--docs--lint-and-warning-compliance)
    - [Lints \& Docs : Strict Workspace Lint Inheritance](#lints--docs--strict-workspace-lint-inheritance)
    - [Lints \& Docs : Single Source of Truth for Crate Documentation](#lints--docs--single-source-of-truth-for-crate-documentation)
    - [Lints \& Docs : Set `html_root_url` for Public Crates](#lints--docs--set-html_root_url-for-public-crates)
    - [Lints \& Docs : Avoid Using Attributes for Documentation, Use Doc Comments](#lints--docs--avoid-using-attributes-for-documentation-use-doc-comments)
    - [Testing : Centralized Test Directory](#testing--centralized-test-directory)
    - [Testing : Integration Test Feature Gating](#testing--integration-test-feature-gating)
    - [Testing : Centralized Benchmarks Directory](#testing--centralized-benchmarks-directory)
    - [Testing : Benchmark Documentation Automation](#testing--benchmark-documentation-automation)
    - [Testing : Strategic Benchmarking Focus](#testing--strategic-benchmarking-focus)
    - [Formatting \& Whitespace : New Lines for Blocks](#formatting--whitespace--new-lines-for-blocks)
    - [Formatting \& Whitespace : Indentation](#formatting--whitespace--indentation)
    - [Formatting \& Whitespace : Chained Method Calls](#formatting--whitespace--chained-method-calls)
    - [Formatting \& Whitespace : Line Breaks for Method Chains and Namespace Access](#formatting--whitespace--line-breaks-for-method-chains-and-namespace-access)
    - [Formatting \& Whitespace : Spaces Around Symbols](#formatting--whitespace--spaces-around-symbols)
    - [Formatting \& Whitespace : Spaces for Blocks](#formatting--whitespace--spaces-for-blocks)
    - [Formatting \& Whitespace : Spacing Around Angle Brackets in Generics](#formatting--whitespace--spacing-around-angle-brackets-in-generics)
    - [Formatting \& Whitespace : Attributes: Spaces](#formatting--whitespace--attributes-spaces)
    - [Formatting \& Whitespace : Attributes: Separate Attributes from Items](#formatting--whitespace--attributes-separate-attributes-from-items)
    - [Formatting \& Whitespace : Formatting `where` Clauses](#formatting--whitespace--formatting-where-clauses)
    - [Formatting \& Whitespace : Trait Implementation Formatting](#formatting--whitespace--trait-implementation-formatting)
    - [Formatting \& Whitespace : Function Signature Formatting](#formatting--whitespace--function-signature-formatting)
    - [Formatting \& Whitespace : Match Expression Formatting](#formatting--whitespace--match-expression-formatting)
    - [Formatting \& Whitespace : Lifetime Annotations](#formatting--whitespace--lifetime-annotations)
    - [Formatting \& Whitespace : Nesting](#formatting--whitespace--nesting)
    - [Formatting \& Whitespace : Code Length](#formatting--whitespace--code-length)
    - [Comments : Spacing in Comments](#comments--spacing-in-comments)
    - [Comments : Comment Content and Task Preservation](#comments--comment-content-and-task-preservation)
    - [Comments : Defining and Using Task Markers](#comments--defining-and-using-task-markers)
    - [Comments : Annotating Addressed Tasks](#comments--annotating-addressed-tasks)
    - [Macros : Declarative Macros (macro\_rules)](#macros--declarative-macros-macro_rules)
    - [Macros : The `=>` Token](#macros--the--token)
    - [Macros : Braces in Macro Bodies](#macros--braces-in-macro-bodies)
    - [Macros : Short Macro Matches](#macros--short-macro-matches)
    - [Naming Conventions : File Naming](#naming-conventions--file-naming)
    - [Naming Conventions : Directory Naming Conventions](#naming-conventions--directory-naming-conventions)
    - [Naming Conventions : Entity Naming Order (Noun-Verb)](#naming-conventions--entity-naming-order-noun-verb)
    - [Naming Conventions : Command Naming Conventions (CLI/REPL)](#naming-conventions--command-naming-conventions-clirepl)
    - [Tooling \& Error Handling : Exclusive Use of `error_tools`](#tooling--error-handling--exclusive-use-of-error_tools)
    - [Tooling \& Error Handling : CLI and REPL Tooling: Mandate `unilang` over `clap`](#tooling--error-handling--cli-and-repl-tooling-mandate-unilang-over-clap)
    - [Unilang Framework : Structuring `CommandDefinition`s](#unilang-framework--structuring-commanddefinitions)
    - [Unilang Framework : Structuring `ArgumentDefinition`s](#unilang-framework--structuring-argumentdefinitions)
    - [Unilang Framework : Prefer the Pipeline API for Command Processing](#unilang-framework--prefer-the-pipeline-api-for-command-processing)
    - [Unilang Framework : REPL Implementation Patterns](#unilang-framework--repl-implementation-patterns)
    - [Unilang Framework : Verbosity Control via Environment Variable](#unilang-framework--verbosity-control-via-environment-variable)
    - [Secrets Management : Secret Storage and Naming](#secrets-management--secret-storage-and-naming)
    - [Secrets Management : Ignoring Secrets with .gitignore](#secrets-management--ignoring-secrets-with-gitignore)

**Lints & Docs**
*   [Lint and Warning Compliance](#lints--docs-lint-and-warning-compliance)
*   [Strict Workspace Lint Inheritance](#lints--docs-strict-workspace-lint-inheritance)
*   [Single Source of Truth for Crate Documentation](#lints--docs-single-source-of-truth-for-crate-documentation)
*   [Set `html_root_url` for Public Crates](#lints--docs-set-html_root_url-for-public-crates)
*   [Avoid Using Attributes for Documentation, Use Doc Comments](#lints--docs-avoid-using-attributes-for-documentation-use-doc-comments)

**Testing**
*   [Centralized Test Directory](#testing-centralized-test-directory)
*   [Integration Test Feature Gating](#testing-integration-test-feature-gating)
*   [Centralized Benchmarks Directory](#testing-centralized-benchmarks-directory)
*   [Benchmark Documentation Automation](#testing-benchmark-documentation-automation)
*   [Strategic Benchmarking Focus](#testing-strategic-benchmarking-focus)

**Formatting & Whitespace**
*   [New Lines for Blocks](#formatting--whitespace-new-lines-for-blocks)
*   [Indentation](#formatting--whitespace-indentation)
*   [Chained Method Calls](#formatting--whitespace-chained-method-calls)
*   [Line Breaks for Method Chains and Namespace Access](#formatting--whitespace-line-breaks-for-method-chains-and-namespace-access)
*   [Spaces Around Symbols](#formatting--whitespace-spaces-around-symbols)
*   [Spaces for Blocks](#formatting--whitespace-spaces-for-blocks)
*   [Spacing Around Angle Brackets in Generics](#formatting--whitespace-spacing-around-angle-brackets-in-generics)
*   [Attributes: Spaces](#formatting--whitespace-attributes-spaces)
*   [Attributes: Separate Attributes from Items](#formatting--whitespace-attributes-separate-attributes-from-items)
*   [Formatting `where` Clauses](#formatting--whitespace-formatting-where-clauses)
*   [Trait Implementation Formatting](#formatting--whitespace-trait-implementation-formatting)
*   [Function Signature Formatting](#formatting--whitespace-function-signature-formatting)
*   [Match Expression Formatting](#formatting--whitespace-match-expression-formatting)
*   [Lifetime Annotations](#formatting--whitespace-lifetime-annotations)
*   [Nesting](#formatting--whitespace-nesting)
*   [Code Length](#formatting--whitespace-code-length)

**Comments**
*   [Spacing in Comments](#comments-spacing-in-comments)
*   [Comment Content and Task Preservation](#comments-comment-content-and-task-preservation)
*   [Defining and Using Task Markers](#comments-defining-and-using-task-markers)
*   [Annotating Addressed Tasks](#comments-annotating-addressed-tasks)

**Macros**
*   [Declarative Macros (macro_rules)](#macros-declarative-macros-macrorules)
*   [The `=>` Token](#macros-the--token)
*   [Braces in Macro Bodies](#macros-braces-in-macro-bodies)
*   [Short Macro Matches](#macros-short-macro-matches)

**Naming Conventions**
*   [File Naming](#naming-conventions-file-naming)
*   [Directory Naming Conventions](#naming-conventions-directory-naming-conventions)
*   [Entity Naming Order (Noun-Verb)](#naming-conventions-entity-naming-order-noun-verb)
*   [Command Naming Conventions (CLI/REPL)](#naming-conventions-command-naming-conventions-clirepl)

**Tooling & Error Handling**
*   [Exclusive Use of `error_tools`](#tooling--error-handling-exclusive-use-of-error_tools)
*   [CLI and REPL Tooling: Mandate `unilang` over `clap`](#tooling--error-handling-cli-and-repl-tooling-mandate-unilang-over-clap)

**Unilang Framework**
*   [Structuring `CommandDefinition`s](#unilang-framework-structuring-commanddefinitions)
*   [Structuring `ArgumentDefinition`s](#unilang-framework-structuring-argumentdefinitions)
*   [Prefer the Pipeline API for Command Processing](#unilang-framework-prefer-the-pipeline-api-for-command-processing)
*   [REPL Implementation Patterns](#unilang-framework-repl-implementation-patterns)
*   [Verbosity Control via Environment Variable](#unilang-framework-verbosity-control-via-environment-variable)

**Secrets Management**
*   [Secret Storage and Naming](#secrets-management-secret-storage-and-naming)
*   [Ignoring Secrets with .gitignore](#secrets-management-ignoring-secrets-with-gitignore)

### Scope & Applicability : Universal Applicability of Codestyle

**Description:** This is a foundational, non-negotiable rule. The codestyle standards defined in this document are **strictly mandatory** and apply universally to **all Rust code**, regardless of where it appears. There are no exceptions.

This mandate extends beyond compilable Rust source files (`.rs`) to include:
-   **Markdown Files:** All Rust code snippets within Markdown files (e.g., `readme.md`, design documents) must adhere to these rules.
-   **Documentation Comments:** All Rust code examples inside documentation comments (`///` and `//!`) must be perfectly formatted.
-   **Standard Comments:** Any Rust code pasted into standard comments (`//` or `/* ... */`) for illustrative purposes must also follow the codestyle.

**Rationale:**
-   **Universal Consistency:** Ensures that any code a developer sees, regardless of context, follows the same professional standard. This eliminates confusion and cognitive overhead.
-   **Documentation Quality:** Guarantees that all examples are not only illustrative but also serve as models of correct, high-quality code that can be safely copied and used.
-   **Professionalism:** A single, universally applied standard reflects a disciplined and professional approach to software development.

> ❌ **Bad** (A Markdown file with a non-compliant code snippet)

```text
### Example Usage

Here is how you can use the `run` function.

```rust
// This example violates the codestyle.
fn main() {
  let result:i32=run(5);
  println!("{}", result);
}
```

> ✅ **Good** (The same Markdown file with a compliant code snippet)

```rust
// This example correctly follows the codestyle.
fn main()
{
  let result : i32 = run( 5 );
  println!( "{}", result );
}
```

### Imports & Modules : Structuring: Prefer Specific, Local, and Grouped `use`

-   **Avoid Global `use`**: Do not use `use` statements at the crate root (`lib.rs` or `main.rs`) that are only needed in specific submodules. Place `use` statements within the modules where they are actually used.
-   **Be Specific**: Import only the specific items needed (e.g., `use std::collections::HashMap;`) rather than using wildcards (e.g., `use std::collections::*;`) unless importing a prelude.
-   **Group Imports**: When importing multiple items from the same crate or module, group them within curly braces `{}`.

> ❌ **Bad** (Global `use` only needed in `submodule`)

```rust
// lib.rs
use std::fs::File; // Only used in submodule
mod submodule;

// submodule.rs
fn open_file() -> std::io::Result< File >
{
  File::open( "foo.txt" )
}
```

> ✅ **Good** (Local `use`)

```rust
// lib.rs
mod submodule;

// submodule.rs
use std::fs::File; // Used locally
fn open_file() -> std::io::Result< File >
{
  File::open( "foo.txt" )
}
```

> ❌ **Bad** (Wildcard import)

```rust
use std::fmt::*;
```

> ✅ **Good** (Specific imports, grouped)

```rust
use std::fmt::{ self, Debug, Display };
```

### Imports & Modules : Structuring: Use of `crate::*`

-   **Use `super::*` for Parent Modules**: When accessing items from a direct parent module, prefer `use super::Item;`.
-   **Use `crate::*` Sparingly**: Use `crate::` primarily for accessing items from the crate root or distant modules where `super::` would be unclear or overly verbose. Avoid excessive `crate::` usage when `super::` is sufficient.

> ✅ **Good** (Using `super`)

```rust
// my_crate/mod.rs
pub struct ParentType;
mod child;

// my_crate/child.rs
use super::ParentType; // Accessing direct parent's item

fn use_parent()
{
  let _p = ParentType;
}
```

> ✅ **Good** (Using `crate` for root access)

```rust
// my_crate/lib.rs
pub struct RootType;
mod level1;

// my_crate/level1/mod.rs
mod level2;

// my_crate/level1/level2.rs
use crate::RootType; // Accessing crate root item

fn use_root()
{
  let _r = RootType;
}
```

> ❌ **Bad** (Using `crate` where `super` is clearer)

```rust
// my_crate/mod.rs
pub struct ParentType;
mod child;

// my_crate/child.rs
use crate::ParentType; // Less clear than `super::ParentType`

fn use_parent()
{
  let _p = ParentType;
}
```

### Imports & Modules : Structuring: Local Entities

-   **Prefer High-Level Imports for External Crates**: When using items from external crates, import the top-level module or the specific type directly (e.g., `use anyhow::Result;` or `use serde::Deserialize;`) rather than importing deeply nested items if the higher-level import provides sufficient access.
-   **Use Full Paths for Clarity**: Within function bodies or other code blocks, if an import is not used, refer to items using their full path (e.g., `std::collections::HashMap::new()`) for clarity, especially for less frequently used items or to avoid ambiguity.

> ✅ **Good** (High-level external import)

```rust
use anyhow::Result; // Import the common Result type

fn my_func() -> Result< () >
{
  // ... function body ...
  Ok( () )
}
```

> ❌ **Bad** (Deeply nested external import, less common)

```rust
use anyhow::private::kind::TraitKind; // Avoid importing deep internal items unless necessary
```

> ✅ **Good** (Full path for clarity)

```rust
fn process_data()
{
  let map = std::collections::HashMap::new(); // Clear where HashMap comes from
  // ...
}
```

### Imports & Modules : Structuring: Structuring `std` Imports

-   **Consolidate `std` Imports**: Group all standard library imports together.
-   **Avoid Multi-Level Nesting**: Do not nest imports deeply within curly braces. Prefer separate `use` statements for different top-level `std` modules (like `collections`, `fmt`, `io`).

> ❌ **Bad** (Deeply nested `std` imports)

```rust
use std::
{
  collections::{ HashMap, HashSet },
  fmt::{ self, Debug, Display },
  io::{ self, Read, Write },
};
```

> ✅ **Good** (Separate `use` for top-level `std` modules)

```rust
use std::collections::{ HashMap, HashSet };
use std::fmt::{ self, Debug, Display };
use std::io::{ self, Read, Write };
```

### Imports & Modules : Structuring: Explicit Exposure Rule

**Description:** When using `mod_interface!` to define a module's public API, **always list exported items explicitly**. Avoid using wildcard (`*`) exports like `exposed use private::*;`. Explicitly listing each `struct`, `enum`, `fn`, `trait`, etc., makes the module's public interface clear and prevents accidental exposure of internal details.

> ❌ **Bad** (Using wildcard export in `mod_interface!`)

```rust
// src/my_module/mod.rs
mod private
{
  pub struct PublicThing;
  pub(crate) struct InternalDetail; // Should not be exposed
  pub fn public_func() {}
}

crate::mod_interface!
{
  // Problem: Exposes EVERYTHING public in `private`, including `InternalDetail` if it were `pub`
  exposed use private::*;
}
```

> ✅ **Good** (Explicitly listing exposed items)

```rust
// src/my_module/mod.rs
mod private
{
  pub struct PublicThing;
  pub(crate) struct InternalDetail;
  pub fn public_func() {}
  pub fn another_public_func() {}
}

crate::mod_interface!
{
  // Clear which items are part of the public API
  exposed use private::PublicThing;
  exposed use private::public_func;
  exposed use private::another_public_func;
  // InternalDetail is correctly kept private as it's not listed
}
```

### Imports & Modules : Structuring: Factor Common Paths in Imports and Declarations

**Description:** When importing multiple items from the same module or declaring multiple modules with a common parent, always factor out the common path prefix using curly braces `{}`. This is the primary principle for reducing redundancy.

> ❌ **Bad** (Repetitive paths)

```rust
use std::fmt::Debug;
use std::fmt::Display;
```

> ✅ **Good** (Common path factored out)

```rust
use std::fmt::{ Debug, Display };
```

### Imports & Modules : Structuring: Use Multi-Line Grouping for Complex Imports

**Description:** A `use` or `mod` group **must** be formatted across multiple lines if it meets any of these criteria:
1.  It imports from (or declares) two or more distinct sub-modules of a common root.
2.  The single-line version would exceed the recommended line length.
3.  It contains nested groups of its own.

This rule dictates *when* to use multi-line formatting. The specific formatting is defined in subsequent rules.

> ✅ **Good** (A simple group can be single-line)

```rust
use std::fmt::{ Debug, Display };
```

> ✅ **Good** (This is complex and MUST be multi-line because it imports from `data` and `semantic`)

```rust
// This would be formatted according to the next rule
use unilang::
{
  data::{...},
  semantic::SemanticAnalyzer,
};
```

### Imports & Modules : Structuring: Hierarchical Formatting for Multi-Line `use` Statements

**Description:** When a `use` statement must be multi-line (as per the previous rule), it must be formatted hierarchically. This improves readability for complex imports.
- The `use` keyword and common path prefix are followed by a newline.
- An indented opening brace `{` is placed on its own line.
- Each imported sub-path or item is on its own indented line.
- This nesting is applied recursively for sub-paths.
- A closing brace `}` is placed on its own line, aligned with the `use` keyword.

> ❌ **Bad** (Long single-line group and multiple separate `use` statements)

```rust
use unilang::data::{ CommandDefinition, ArgumentDefinition, ArgumentAttributes, OutputData };
use unilang::semantic::SemanticAnalyzer;
```

> ✅ **Good** (Hierarchical, multi-line grouping)

```rust
use unilang::
{
  data::
  {
    CommandDefinition,
    ArgumentDefinition,
    ArgumentAttributes,
    OutputData,
  },
  semantic::SemanticAnalyzer,
};
```

### Imports & Modules : Structuring: Grouped Formatting for `mod` Declarations

**Description:** When declaring multiple modules, they should be grouped within a `mod { ... }` block if they are numerous or related. This follows the same multi-line formatting principles as `use` statements.

> ❌ **Bad** (Repetitive module declarations)

```rust
mod http_client;
mod http_server;
mod http_types;
```

> ✅ **Good** (Multi-line grouping for `mod`)

```rust
mod
{
  http_client,
  http_server,
  http_types,
}
```

### Imports & Modules : Structuring: Structuring Modules with `mod_interface` (When Used)

**Description:** When using the `mod_interface!` macro to structure modules (especially for layered architectures or fine-grained visibility control), follow these patterns. Define the actual implementation details within a `private` submodule. Use the `mod_interface!` block in the parent `mod.rs` to declare the public API by explicitly exposing (`exposed use`), re-exporting (`pub use`), or integrating (`use` or `layer`) items and submodules from `private` or other modules.

**Key Principles:**
1.  **Encapsulation:** Keep implementation details inside `mod private`.
2.  **Explicit API:** Define the public interface clearly in `mod_interface!`.
3.  **Layering:** Use `mod_interface!` to manage dependencies and visibility between layers.

> ✅ **Good** (Using `mod_interface` for a feature module)

```rust
// src/user_management/mod.rs

mod private // Implementation details here
{
  pub struct User { pub id: u32, pub name: String }
  pub enum UserStatus { Active, Inactive }
  pub fn find_user( id: u32 ) -> Option< User > { /* ... */ }
  fn internal_helper() { /* ... */ } // Not public
}

// Define the public API using mod_interface!
crate::mod_interface!
{
  // Explicitly expose specific items from the private module
  exposed use private::User;
  exposed use private::UserStatus;
  exposed use private::find_user;
  // internal_helper remains private as it's not listed
}
```

> ✅ **Good** (Using `mod_interface` for layering - see "Organize by Feature or Layer" Design Rule for full example)

```rust
// src/api/mod.rs (API Layer)
mod private { /* ... API handlers, DTOs ... */ }
crate::mod_interface!
{
  exposed use private::handlers; // Expose handlers
  use crate::domain; // Integrate the domain layer for use within API logic
}

// src/domain/mod.rs (Domain Layer)
mod private { /* ... Domain entities, traits, logic ... */ }
crate::mod_interface!
{
  exposed use private::User; // Expose domain entities
  exposed use private::UserRepository; // Expose traits for persistence layer
}
```

> ❌ **Bad** (Exposing implementation details directly without `mod_interface` or `private`)

```rust
// src/user_management/mod.rs
// Everything pub here is directly exposed, less control
pub struct User { pub id: u32, pub name: String }
pub enum UserStatus { Active, Inactive }
pub fn find_user( id: u32 ) -> Option< User > { /* ... */ }
fn internal_helper() { /* ... */ } // This is private, but structure is less clear
```

### Imports & Modules : Structuring: Integrating Child Layers with `use` in `mod_interface`

**Description:** When using `mod_interface!` to structure layered modules (e.g., an `api` layer needing access to a `domain` layer), integrate the necessary child/lower layers using a standard `use` statement *inside* the `mod_interface!` block. This clearly declares the dependency and makes the items from the lower layer available within the current module's scope (specifically, within its `private` submodule if following the standard pattern).

> ✅ **Good** (API layer using the Domain layer via `use` inside `mod_interface!`)

```rust
// src/domain/mod.rs
mod private { pub struct User; pub fn validate_user( _u: &User ) -> bool { true } }
crate::mod_interface! { exposed use private::User; exposed use private::validate_user; }

// src/api/mod.rs
mod private
{
  // We need User and validate_user from the domain layer
  use crate::domain; // Make domain items accessible

  pub fn handle_user_request()
  {
    let user = domain::User; // Use domain::User
    if domain::validate_user( &user ) // Use domain::validate_user
    {
      // ...
    }
  }
}

crate::mod_interface!
{
  // Integrate the domain layer
  use crate::domain;
  // Expose the API handler
  exposed use private::handle_user_request;
}

// src/lib.rs
mod domain;
mod api;
pub use api::handle_user_request; // Expose the final API function
```

> ❌ **Bad** (Trying to access `domain` items without `use` inside `mod_interface!`)

```rust
// src/domain/mod.rs - (Same as above)
mod private { pub struct User; pub fn validate_user( _u: &User ) -> bool { true } }
crate::mod_interface! { exposed use private::User; exposed use private::validate_user; }

// src/api/mod.rs
mod private
{
  // Missing `use crate::domain;` here

  pub fn handle_user_request()
  {
    // let user = domain::User; // Compile Error: `domain` not in scope
    // if domain::validate_user( &user ) // Compile Error: `domain` not in scope
    // {
      // ...
    // }
  }
}

crate::mod_interface!
{
  // Missing `use crate::domain;`
  exposed use private::handle_user_request;
}
```

### Imports & Modules : Structuring: Keep all Definitions and Details Inside `private` Namespace (When Using `mod_interface`)

**Description:** When implementing the `mod_interface!` pattern, **all** structs, enums, functions, traits, constants, and implementation details (`impl` blocks) must reside within the inline `mod private { ... }` block. The parent module file (`mod.rs`) should *only* contain the inline `mod private { ... }` block and the `mod_interface! { ... }` block. This enforces strict encapsulation and ensures the `mod_interface!` block serves as the single, authoritative definition of the module's public API.

**CRITICAL WARNING:** Never create `private.rs` files or `private/` directories. The `private` module must always be an inline block within the parent module file.

> ❌ **Bad** (Definitions outside `private`)

```rust
// src/my_module/mod.rs

mod private
{
  // Some details here...
  pub fn helper_func() {}
}

// Problem: Struct defined outside `private`
pub struct PublicThing
{
  pub field: i32,
}

// Problem: Impl block outside `private`
impl PublicThing
{
  pub fn new() -> Self { Self { field: 0 } }
}

crate::mod_interface!
{
  // Interface definition is incomplete because items exist outside `private`
  exposed use private::helper_func;
  // How is PublicThing exposed? It bypasses the interface definition.
}
```

> ✅ **Good** (All definitions inside `private`)

```rust
// src/my_module/mod.rs

mod private
{
  // All definitions reside here
  pub struct PublicThing
  {
    pub field : i32,
  }

  impl PublicThing
  {
    pub fn new() -> Self
    {
      Self { field: 0 }
    }
  }

  pub fn helper_func() {}

  fn internal_detail() {} // Remains private automatically
}

// Interface explicitly defines what is public
crate::mod_interface!
{
  exposed use private::PublicThing; // Expose the struct
  exposed use private::helper_func; // Expose the function
  // `internal_detail` and the `impl` block are not exposed unless specified
  // To expose methods, you typically expose the struct/type.
}
```

### Imports & Modules : Structuring: The `private` Namespace Must Be an Inline Module (When Using `mod_interface`)

**Description:** When using `mod_interface!`, this is a rigid, non-negotiable rule. The `mod private` block, which serves the special purpose of encapsulating all implementation details for a `mod_interface!`, **must** be defined inline within its parent module file (e.g., `mod.rs`). It is **strictly forbidden** to move it into a separate file (e.g., `my_module/private.rs`) and reference it with `mod private;`.

**Rationale:**
-   **Architectural Integrity:** The `private` module is not a standard submodule. Its sole purpose is to contain the implementation details for the public API defined in the *same file* by `mod_interface!`. This co-location enforces a strong architectural pattern where a module's interface and its complete implementation are visible in a single place.
-   **Encapsulation:** Moving the implementation to a separate file would treat it like a regular child module, breaking the clear "private implementation" pattern and weakening the encapsulation boundary.
-   **Readability and Maintenance:** Keeping the implementation inline makes the module self-contained and easier to understand and maintain without navigating to other files.

**CRITICAL WARNING:** Creating `private.rs` files or `private/` directories violates this architectural pattern and is strictly forbidden.

> ✅ **Good** (Inline `private` module)

```rust
// In my_module/mod.rs
mod private
{
  // All definitions for my_module are here.
}

crate::mod_interface! { /* ... */ }
```

> ❌ **Bad** (Separate file for `private` module)

```rust
// In my_module/mod.rs
// Bad: This loads from my_module/private.rs, which is forbidden.
mod private;

crate::mod_interface! { /* ... */ }
```

### Imports & Modules : Structuring: Forbidden `private` File/Directory Patterns (When Using `mod_interface`)

**Description:** When implementing the `mod_interface!` architecture, it is **absolutely forbidden** to create any of the following patterns:

**Strictly Prohibited Patterns:**
- `private.rs` files in any location
- `private/` directories in any location  
- `mod private;` declarations that reference external files
- Any external file or directory structure named `private` for `mod_interface!` implementations

**Enforcement Commands:**
```bash
# Detect forbidden private.rs files
find . -name "private.rs" -type f

# Detect forbidden private/ directories  
find . -name "private" -type d

# Both violations in one command
find . \( -name "private.rs" -type f \) -o \( -name "private" -type d \)
```

**Required Action:** Any detected `private.rs` files or `private/` directories must be immediately restructured as inline `mod private { ... }` blocks within their parent module files.

**Rationale:** The `private` namespace serves a specific architectural purpose as an inline encapsulation mechanism within the `mod_interface!` pattern. Creating external files or directories named `private` breaks this pattern, defeats the architectural purpose, and creates confusion with the intended inline implementation approach.

### Imports & Modules : Structuring: The `private` Namespace Must Not Contain Submodules (When Using `mod_interface`)

**Description:** When using `mod_interface!`, the `mod private` block is intended to be a flat container for all of the module's definitions (structs, functions, impls, etc.). It must not contain any of its own submodules (`mod ...;` or `mod ... { ... }`). All modularity should be handled at the `mod_interface!` level.

> ✅ **Good** (Flat structure inside `private`)

```rust
mod private
{
  pub struct MyStruct;
  pub fn my_function() {}
}
```

> ❌ **Bad** (Nesting submodules inside `private`)

```rust
mod private
{
  // Bad: Submodules are not allowed here.
  mod internal_logic
  {
    pub struct MyStruct;
    pub fn my_function() {}
  }
}
```

### Imports & Modules : Structuring: Do Not Nest Child Modules Inside `private`

**Description:** While all item definitions belong in `mod private`, child module declarations (`mod child;`) do not. Child modules that are part of a layered architecture must be integrated at the interface level using the `layer`, `use`, or `reuse` directives within the `mod_interface!` block, never inside the parent's `private` module.

> ✅ **Good** (Integrating a child layer at the interface level)

```rust
// src/parent/mod.rs
mod child; // The child module file exists alongside the parent's.

mod private
{
  // Parent's own definitions go here.
}

crate::mod_interface!
{
  // The child module is integrated here, not in `private`.
  layer child;
}
```

> ❌ **Bad** (Incorrectly nesting a child module declaration)

```rust
// src/parent/mod.rs

mod private
{
  // Bad: Do not declare child modules inside the private block.
  mod child;
}

crate::mod_interface!
{
  // This structure is incorrect.
}
```

### Imports & Modules : Structuring: Preferring `use` or `reuse` Over `layer` in `mod_interface`

**Description:** When integrating submodules or other layers within a `mod_interface!` block, prefer using explicit `use <path>;` or `reuse <path>;` statements over the `layer <path>;` keyword. While `layer` can achieve a similar result, `use` and `reuse` are standard Rust keywords and more clearly express the intent of bringing items into the current scope or re-exporting them, respectively. This improves the readability and maintainability of the module structure.

-   Use `use my_submodule;` to make the submodule's public items available *within* the current module (typically for use inside the `private` implementation).
-   Use `reuse my_submodule as submodule_alias;` (or `pub use my_submodule as submodule_alias;`) to re-export the submodule itself as part of the current module's public API.

> ✅ **Good** (Using `use` and `reuse`)

```rust
// src/sub_layer/mod.rs
mod private { pub fn helper() {} }
crate::mod_interface!{ exposed use private::helper; }

// src/main_layer/mod.rs
mod private
{
  use crate::sub_layer; // Make sub_layer available internally

  pub fn main_func()
  {
    sub_layer::helper(); // Call helper from sub_layer
  }
}

crate::mod_interface!
{
  exposed use private::main_func; // Expose our function
  use crate::sub_layer; // Make sub_layer available for internal use (redundant if only used in private, but clear)
  reuse crate::sub_layer as helpers; // Re-export sub_layer under a new name 'helpers'
}

// Consumer code
// main_layer::main_func();
// main_layer::helpers::helper();
```

> ❌ **Bad** (Using `layer`, less explicit intent)

```rust
// src/sub_layer/mod.rs - (Same as above)
mod private { pub fn helper() {} }
crate::mod_interface!{ exposed use private::helper; }

// src/main_layer/mod.rs
mod private
{
  // Implicit access to sub_layer items due to `layer` below is less clear
  pub fn main_func()
  {
    // Assuming `layer crate::sub_layer;` makes helper available directly? Or via sub_layer::helper? Ambiguous.
    // sub_layer::helper(); // Requires testing how `layer` actually works here.
  }
}

crate::mod_interface!
{
  exposed use private::main_func;
  layer crate::sub_layer; // Less clear than `use` or `reuse`. Does it import? Re-export? Both?
}
```

### Imports & Modules : Structuring: Maintain Definition Order in Exports

**Description:** When explicitly exporting items using `mod_interface!` (e.g., with `exposed use ...;`), list the items in the `mod_interface!` block in the **same order** as they are defined within the corresponding `private` module. This consistency makes it easier to compare the implementation (`private`) with its public API (`mod_interface!`) and verify that all intended items are exposed correctly.

> ❌ **Bad** (Export order differs from definition order)

```rust
// src/my_module/mod.rs
mod private
{
  // Definition Order: Struct -> Enum -> Function
  pub struct MyStruct { /* ... */ }
  pub enum MyEnum { Variant1, Variant2 }
  pub fn my_function() { /* ... */ }
}

crate::mod_interface!
{
  // Export Order: Function -> Struct -> Enum (Inconsistent)
  exposed use private::my_function;
  exposed use private::MyStruct;
  exposed use private::MyEnum;
}
```

> ✅ **Good** (Export order matches definition order)

```rust
// src/my_module/mod.rs
mod private
{
  // Definition Order: Struct -> Enum -> Function
  pub struct MyStruct { /* ... */ }
  pub enum MyEnum { Variant1, Variant2 }
  pub fn my_function() { /* ... */ }
}

crate::mod_interface!
{
  // Export Order: Struct -> Enum -> Function (Consistent)
  exposed use private::MyStruct;
  exposed use private::MyEnum;
  exposed use private::my_function;
}
```

### Imports & Modules : Structuring: Centralized Workspace Dependency Manifest

**Description:** **CRITICAL WORKSPACE RULE:** In a Cargo workspace, the root `Cargo.toml` **must** serve as the single source of truth for all dependency definitions. All dependencies—including their versions and sources—**must** be declared in the `[workspace.dependencies]` table. Feature flags **must not** be specified in this central manifest.

**ABSOLUTE REQUIREMENT:** Member crates **must** inherit all dependencies from the workspace using the `workspace = true` syntax. A crate may specify the features it requires for an inherited dependency, but it is **strictly forbidden** from defining a dependency's version or source.

**ZERO TOLERANCE:** Only the workspace root `Cargo.toml` is allowed to import dependencies. Crate `Cargo.toml` files must reuse what the workspace imported. No exceptions.

**Rationale:** This approach ensures consistent versions across the entire workspace, simplifies dependency management, reduces redundancy, and allows each crate to enable only the specific features it needs.

> ❌ **Bad** (Dependencies defined directly in crate `Cargo.toml`)

```toml
# my_crate/Cargo.toml
[dependencies]
# FORBIDDEN: This dependency is not inherited from the workspace.
rand = "0.8"
```

> ❌ **Bad** (Features specified in the workspace manifest)

```toml
# workspace_root/Cargo.toml
[workspace.dependencies]
# FORBIDDEN: Features must be specified in the consuming crate, not here.
serde = { version = "1.0", features = ["derive", "rc"] }
```

> ✅ **Good** (Dependencies defined in workspace; features specified in crate)

```toml
# workspace_root/Cargo.toml
[workspace.dependencies]
serde = { version = "1.0" }
rand = { version = "0.8" }

# my_crate_a/Cargo.toml
[dependencies]
# Correct: Inherited from workspace, features enabled locally.
serde = { workspace = true, features = ["derive"] }
rand = { workspace = true }

# my_crate_b/Cargo.toml
[dependencies]
# Correct: Inherited from workspace, different features enabled locally.
serde = { workspace = true, features = ["rc"] }
```

### Imports & Modules : Structuring: Forbid Undeclared Workspace Dependencies

**Description:** **CRITICAL ENFORCEMENT RULE:** This is a rigid and non-negotiable rule. In a Cargo workspace, it is **strictly forbidden** for any member crate's `Cargo.toml` to reference a dependency—even using `workspace = true`—that has not first been explicitly declared in the `[workspace.dependencies]` table of the root `Cargo.toml` file. 

**ABSOLUTE MANDATE:** Every single dependency used by any crate in the workspace **must** originate from the central workspace definition. There are no exceptions. This rule complements the "[Structuring: Centralized Workspace Dependency Manifest](#imports--modules-structuring-centralized-workspace-dependency-manifest)" rule by making it an explicit error to bypass the central manifest.

**ENFORCEMENT:** Any attempt to add dependencies directly to member crates is a critical violation that must be immediately corrected.

**Rationale:**
-   **Single Source of Truth:** Enforces the root `Cargo.toml` as the absolute single source of truth for all dependencies, their versions, and their sources.
-   **Security and Compliance:** Prevents crates from introducing unvetted dependencies. It ensures all dependencies can be audited for security vulnerabilities and license compatibility from a single, central location.
-   **Version Control:** Eliminates the possibility of version conflicts or resolution ambiguity that could arise from dependencies being declared ad-hoc within individual crates.

> ❌ **Bad** (Crate references a dependency not declared in the workspace)

```toml
# workspace_root/Cargo.toml
[workspace.dependencies]
# The 'phf_codegen' dependency is MISSING here.
serde = { version = "1.0" }

# my_crate/Cargo.toml
[dependencies]
# FORBIDDEN: 'phf_codegen' is not defined in [workspace.dependencies]
phf_codegen = { workspace = true }
serde = { workspace = true }
```

> ✅ **Good** (Dependency is declared in workspace, then inherited by the crate)

```toml
# workspace_root/Cargo.toml
[workspace.dependencies]
# Correct: All dependencies are declared here first.
phf_codegen = { version = "0.11", default-features = false }
serde = { version = "1.0" }

# my_crate/Cargo.toml
[dependencies]
# Correct: Both dependencies are inherited from the workspace.
phf_codegen = { workspace = true }
serde = { workspace = true }
```

### Imports & Modules : Mandatory `enabled` and `full` Features for Crate Toggling

**Description:** This is a rigid and non-negotiable rule for managing complex build configurations **for every crate that is a member of the workspace**. It does not apply to external, third-party dependencies. Every workspace crate **must** expose two specific features: `enabled` and `full`.

1.  **`enabled` Feature:** This acts as a master switch for the entire crate.
    *   It **must** be part of the `default` feature set, ensuring the crate is active by default.
    *   It **must** activate all of the crate's dependencies (which must be declared as optional).
2.  **`full` Feature:** This feature provides a convenient way to enable all functionality.
    *   It **must** be defined to include the `enabled` feature, along with any other optional features the crate provides.
3.  **Dependency Gating:** All dependencies of the crate **must** be declared as `optional = true` and activated via the `enabled` feature.
4.  **Code Gating:** The entire functional code within the crate's entry points (`lib.rs`, `main.rs`, etc.) **must** be conditionally compiled under the `enabled` feature using `#[cfg(feature = "enabled")]`.

**Rationale:**
Cargo's feature system is additive, which makes it difficult to manage complex or mutually exclusive dependency sets. For example, if crate `A` depends on `B` with feature `X`, and crate `C` depends on `B` without feature `X`, feature `X` will still be enabled for `B` in the final build. The `enabled` feature pattern provides a robust workaround. It allows a crate to be completely "switched off" or compiled-out, even when it is included as a non-optional dependency by another crate, thus preventing its dependencies and code from affecting the final binary.

> ❌ **Bad** (Dependencies are not optional; code is not gated)

```toml
# my_crate/Cargo.toml
[dependencies]
# FORBIDDEN: Dependencies must be optional and gated by the "enabled" feature.
serde = { workspace = true }
```

```rust
// my_crate/src/lib.rs
// FORBIDDEN: The crate's code is not conditionally compiled.
pub fn my_api() -> bool
{
  true
}
```

> ✅ **Good** (Correct implementation of the `enabled` and `full` feature pattern)

```toml
# my_crate/Cargo.toml

[features]
# The crate is enabled by default.
default = [ "enabled" ]
# The master switch that activates all dependencies.
enabled = [ "dep:serde", "dep:log" ]
# The 'full' feature enables all other features, including 'enabled'.
full = [ "enabled" ]

[dependencies]
# All dependencies are optional.
serde = { workspace = true, optional = true }
log = { workspace = true, optional = true }
```

```rust
// my_crate/src/lib.rs

// This attribute prevents "unused" warnings when the feature is disabled.
#![cfg_attr( not( feature = "enabled" ), allow( unused ) )]

// The entire module is gated by the "enabled" feature.
#[cfg(feature = "enabled")]
mod implementation
{
  // All your crate's code and modules go here.
  pub fn my_api() -> bool
  {
    true
  }
}

#[cfg(feature = "enabled")]
pub use implementation::*;
```

### Imports & Modules : Dependencies: Prefer wTools Ecosystem Crates

**Description:** To foster consistency, reduce dependency conflicts, and leverage shared patterns, projects **must** prefer crates from the official `wTools` ecosystem for common development tasks. An exception can be made for highly specialized requirements, but it must be justified with a comment.

**Ecosystem Crates:**
-   **`error_tools`**: The standard for all error handling.
-   **`macro_tools`**: The primary toolkit for developing procedural macros.
-   **`unilang`**: The designated crate for CLI and REPL development.
-   **`workspace_tools`**: The standard for all workspace-relative file path resolution and configuration loading.
-   **`benchkit`**: The preferred toolkit for performance analysis and documentation-first benchmarking.

**Rationale:** The `wTools` ecosystem provides a suite of cohesive, well-integrated libraries designed to solve common problems in a standardized way. Using them ensures that different projects remain interoperable and that developers can rely on a familiar set of tools.

### Lints & Docs : Lint and Warning Compliance

Make sure you have no warnings from clippy with this lints enabled.

**Recommended Lints Configuration:**

> ✅ **Good**

```toml
[workspace.lints.rust]
# Denies non-idiomatic code for Rust 2018 edition.
rust_2018_idioms = { level = "warn", priority = -1 }
# Denies using features that may break in future Rust versions.
future_incompatible = { level = "warn", priority = -1 }
# Warns if public items lack documentation.
missing_docs = "warn"
# Warns for public types not implementing Debug.
missing_debug_implementations = "warn"
# Denies all unsafe code usage.
unsafe-code = "deny"

[workspace.lints.clippy]
# Denies pedantic lints, enforcing strict coding styles and conventions.
pedantic = { level = "warn", priority = -1 }
# Denies undocumented unsafe blocks.
undocumented_unsafe_blocks = "deny"
# Denies to prefer `core` over `std` when available, for `no_std` compatibility.
std_instead_of_core = "warn"
# Denies including files in documentation unconditionally.
doc_include_without_cfg = "warn"
# Denies missing inline in public items.
missing_inline_in_public_items = "warn"

# exceptions

# Allows functions that are only called once.
single_call_fn = "allow"
# Allows forcing a function to always be inlined.
inline_always = "allow"
# Allows item names that repeat the module name (e.g., `mod user { struct User; }`).
module_name_repetitions = "allow"
# Allows using fully qualified paths instead of `use` statements.
absolute_paths = "allow"
# Allows wildcard imports (e.g., `use std::io::*;`).
wildcard_imports = "allow"
# Allow to prefer `alloc` over `std` when available, for `no_std` compatibility.
std_instead_of_alloc = "allow"
# Allow put definitions of struct at any point in functions.
items_after_statements = "allow"
# Allow precission loss, for example during conversion from i64 to f64
cast_precision_loss = "allow"
# Allows `pub use` statements.
pub_use = "allow"
# Allows the `?` operator.
question_mark_used = "allow"
# Allows implicit returns.
implicit_return = "allow"
# Allow ordering of fields in intuitive way.
arbitrary_source_item_ordering = "allow"
# Allow mod.rs files
mod_module_files = "allow"
# Allow missing docs for private items
missing_docs_in_private_items = "allow"
```

### Lints & Docs : Strict Workspace Lint Inheritance

**Description:** This is a rigid and non-negotiable rule. In a Cargo workspace, the root `Cargo.toml` serves as the **single, authoritative manifest for all lint configurations**. All lint settings for both `rustc` (`[workspace.lints.rust]`) and `clippy` (`[workspace.lints.clippy]`) **must** be defined exclusively in the root `Cargo.toml`.

Member crates **must not** define their own lint configurations. The `[lints]` section in a member crate's `Cargo.toml` must contain **only** the line `workspace = true` and nothing else. It is **strictly forbidden** for a member crate to define its own `[lints.rust]` or `[lints.clippy]` tables, override individual lints, or use `#![...]` attributes in source files for lint configuration.

**Rationale:**
-   **Universal Code Quality:** Enforces a single, consistent standard of code quality and style across every crate in the workspace.
-   **Simplified Maintenance:** Prevents configuration drift and simplifies updates, as all lint settings are managed in one place.
-   **Clarity and Predictability:** Ensures that the build and CI process behaves predictably for all crates, without hidden or crate-specific lint overrides.

> ❌ **Bad** (Defining lints in a crate's `Cargo.toml`)

```toml
# my_crate/Cargo.toml
[lints.rust] # FORBIDDEN: Lints must not be defined in a member crate.
unsafe_code = "deny"
```

> ❌ **Bad** (Overriding lints in a crate's `Cargo.toml`)

```toml
# my_crate/Cargo.toml
[lints]
workspace = true
# FORBIDDEN: Overriding or adding lints is not allowed.
[lints.clippy]
pedantic = "allow"
```

> ❌ **Bad** (Defining lints in a source file)

```rust
// my_crate/src/lib.rs
// FORBIDDEN: Lints must not be defined in source files.
#![deny(unsafe_code)]
```

> ✅ **Good** (Lints are defined centrally in the workspace and inherited by the crate)

```toml
# workspace_root/Cargo.toml
[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "warn"

[workspace.lints.clippy]
pedantic = "warn"

# my_crate/Cargo.toml
# Correct: The [lints] section contains ONLY the workspace inheritance line.
[lints]
workspace = true
```

### Lints & Docs : Single Source of Truth for Crate Documentation

**Description:** To avoid duplication and ensure consistency, the `readme.md` file **must** serve as the single source of truth for crate-level documentation. All library (`lib.rs`) and binary (`main.rs` or `src/bin/*.rs`) entry points **must** include the contents of the `readme.md` file as their inner doc comments.

The **only acceptable method** is to use a two-part approach at the top of the entry file:
1.  A single-line inner doc comment (`//!`) providing a brief crate summary. This satisfies the `missing_docs` lint during normal builds and tests.
2.  The conditional `cfg_attr` attribute immediately following it to include the full `readme.md` content when building documentation (`cargo doc`).

**Rationale:**
-   **DRY (Don't Repeat Yourself):** Prevents documentation from becoming out of sync between the README and the crate's own docs.
-   **Warning-Free Builds:** The summary doc comment satisfies the `missing_docs` lint without needing to suppress it, ensuring a clean build process.
-   **Maintainability:** Simplifies documentation updates by requiring changes in only one location.

> ❌ **Bad** (Manually duplicating documentation in `lib.rs`)

```rust
// In src/lib.rs
//! # My Crate
//!
//! This is a crate that does amazing things. It is the same text
//! that is present in the readme.md file, which leads to duplication.
```

> ❌ **Bad** (Suppressing the `missing_docs` lint)

```rust
// In src/lib.rs
// FORBIDDEN: This method is not acceptable as it suppresses a useful lint.
#![ cfg_attr( not( doc ), allow( missing_docs ) ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
```

> ✅ **Good** (Providing a summary comment and conditionally including the README)

```rust
// In src/lib.rs
// Correct: A one-line summary satisfies the `missing_docs` lint, and the full
// README is included only when building documentation.
//! A brief, one-line summary of the crate.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
```

### Lints & Docs : Set `html_root_url` for Public Crates

**Description:** For any public-facing crate (i.e., intended for publishing to `crates.io`), the `lib.rs` file **must** include the `html_root_url` attribute. This attribute is critical for `docs.rs` to correctly generate links to items from other crates in your documentation. The URL should be formatted as `https://docs.rs/CRATE_NAME/latest/CRATE_NAME/`, replacing `CRATE_NAME` with the actual name of your crate.

> ❌ **Bad** (A public crate missing the attribute)

```rust
// In a public crate's src/lib.rs
// FORBIDDEN: Missing the html_root_url attribute, which will result in broken
// links for external types in the generated documentation on docs.rs.
#![ deny( missing_docs ) ]
```

> ✅ **Good** (The attribute is correctly set)

```rust
// In a public crate's src/lib.rs
// Correct: The html_root_url is set, ensuring correct link generation.
// Replace `your_crate_name` with the actual crate name.
#![ doc( html_root_url = "https://docs.rs/your_crate_name/latest/your_crate_name/" ) ]
#![ deny( missing_docs ) ]
```

### Lints & Docs : Avoid Using Attributes for Documentation, Use Doc Comments

For documenting code, prefer using ordinary doc comments `//!` over attributes like `#![doc = ""]`. Doc comments are more conventional and readable, aligning with Rust's idiomatic documentation practices. This approach ensures consistency in how documentation is written and maintained across the codebase.

> ❌ **Bad**
Using the `doc` attribute for documentation can disrupt the visual flow and consistency of source code documentation.

```rust
#![ doc = "Description of file." ]

#[ doc = "Implements a new type of secure connection." ]
mod secure_connection
{
  #[ doc = "Establishes a secure link." ]
  pub fn establish()
  {
  }
}
```

> ✅ **Good**
Ordinary doc comments `//!` and `///` provide a clearer, more idiomatic way to document modules and functions, enhancing readability.

```rust
//! Description of file.

/// Implements a new type of secure connection.
mod secure_connection
{
  /// Establishes a secure link.
  pub fn establish()
  {
  }
}
```

### Testing : Centralized Test Directory

**Description:** This is a rigid and non-negotiable rule. All tests, including unit tests and integration tests, **must** be located in the top-level `tests` directory of the crate. It is **strictly forbidden** to have:

1. `#[cfg(test)]` modules or any `#[test]` functions inside the `src` directory
2. **Test files in the `examples` directory** - files with `test` in the name, `#[test]` functions, or any testing-related content
3. Files ending with `_test.rs` anywhere except in the `tests` directory
4. Integration tests, comprehensive tests, or any form of testing code outside the designated `tests` directory

There are **absolutely no exceptions** to this rule. The `examples` directory is exclusively for demonstrating library usage, not for testing functionality.

**Common Violations to Avoid:**
- Files named `*_test.rs`, `test_*.rs`, or containing "test" in the filename in the `examples` directory
- Files with `#[test]` functions anywhere except the `tests` directory  
- Integration tests or comprehensive tests in `examples`
- Files that perform testing logic even without `#[test]` attributes

**Enforcement:** Use `find examples -name "*test*" -o -name "*_test.rs"` to detect naming violations. All such files must be moved to the `tests` directory.

**Rationale:**
-   **Strict Separation of Concerns:** Enforces a clean boundary between production code (`src`), test code (`tests`), and demonstration code (`examples`).
-   **Faster Builds:** `cargo build` and `cargo check` will not analyze or compile any test code, leading to faster development cycles.
-   **Examples Purity:** The `examples` directory must contain only real-world usage demonstrations that users can learn from, not testing infrastructure.
-   **Cargo Tool Compatibility:** Avoids confusing `cargo run --example` when examples contain tests instead of demonstrations.
-   **Simplified Configuration:** Eliminates the need for complex `#[cfg(test)]` attributes within the source code, making it cleaner and more focused.
-   **Unified Test Environment:** All tests can be discovered and run from a single, predictable location using `cargo test`.

> ❌ **Bad** (Inline test module in `src`)

```rust
// In src/my_module.rs
pub fn add( a: i32, b: i32 ) -> i32
{
  a + b
}

// FORBIDDEN: Test modules are not allowed in the `src` directory.
#[cfg(test)]
mod tests
{
  use super::*;

  #[test]
  fn test_add()
  {
    assert_eq!( add( 2, 2 ), 4 );
  }
}
```

> ✅ **Good** (Test code is in the `tests` directory)

```text
// Crate directory structure:
// ├── Cargo.toml
// ├── src
// │   └── lib.rs
// └── tests
//     └── basic_test.rs
```

```rust
// In tests/basic_test.rs
// Import the crate being tested.
use my_crate::add;

#[test]
fn test_add_from_outside()
{
  assert_eq!( add( 2, 2 ), 4 );
}
```

> ❌ **Bad** (Tests incorrectly placed in `examples` directory)

```text
// FORBIDDEN: These test files in examples directory violate the rule:
// ├── examples
// │   ├── basic_test.rs                    // FORBIDDEN: Contains #[test] functions
// │   ├── comprehensive_integration_test.rs // FORBIDDEN: Integration test in examples
// │   ├── test_error_handling.rs           // FORBIDDEN: File name indicates testing
// │   └── parser_integration_test.rs       // FORBIDDEN: Integration test, not example
```

```rust
// FORBIDDEN: In examples/basic_test.rs
// This violates the centralized test directory rule.
use my_crate::add;

#[test] // FORBIDDEN: #[test] functions are not allowed in examples
fn test_basic_functionality()
{
  assert_eq!( add( 2, 2 ), 4 );
}

// Even without #[test], files with "test" in the name are forbidden in examples
fn comprehensive_integration_test()
{
  // Testing logic belongs in tests/ directory only
}
```

> ✅ **Good** (Examples directory contains only demonstrations)

```text
// Correct: Examples directory shows only usage demonstrations:
// ├── examples
// │   ├── hello_world.rs          // Shows basic usage
// │   ├── advanced_usage.rs       // Shows complex scenarios  
// │   └── real_world_example.rs   // Shows practical application
// └── tests
//     ├── basic_test.rs           // All tests go here
//     └── integration_test.rs     // Including integration tests
```

```rust
// In examples/hello_world.rs
// Correct: This demonstrates library usage for users
use my_crate::add;

fn main()
{
  let result = add( 2, 2 );
  println!( "The result is: {}", result );
}
```

### Testing : Manual Testing Organization

**Description:** When manual testing is required for a crate, all manual testing related files **must** be organized within a dedicated `tests/manual/` directory structure.

**Mandatory Requirements:**
1. **Manual Testing Plan:** The file `tests/manual/readme.md` **must** contain the comprehensive manual testing plan for the crate.
2. **Manual Testing Files:** All files related to manual testing (scripts, configurations, test data, documentation) **must** be located within the `tests/manual/` directory.
3. **Directory Creation:** The `tests/manual/` directory should only be created if manual testing is actually needed for the crate.

**Rationale:**
- **Centralized Manual Testing:** Keeps all manual testing materials in a predictable, standardized location
- **Clear Documentation:** The `readme.md` provides a single source of truth for manual testing procedures
- **Separation of Concerns:** Manual testing files are clearly separated from automated tests while remaining within the broader testing directory structure

> ✅ **Good** (Proper manual testing organization)

```text
// Crate with manual testing requirements:
// ├── Cargo.toml
// ├── src
// │   └── lib.rs
// └── tests
//     ├── automated_test.rs        // Automated tests
//     ├── integration_test.rs      // Automated integration tests
//     └── manual
//         ├── readme.md             // MANDATORY: Manual testing plan
//         ├── setup_script.sh       // Manual testing setup
//         └── other_files.*         // Other manual testing files
```

> ❌ **Bad** (Manual testing files scattered or missing plan)

```text
// FORBIDDEN: Manual testing files in wrong locations:
// ├── manual_tests/              // FORBIDDEN: Wrong directory name
// ├── tests/
// │   ├── manual_test.md         // FORBIDDEN: Manual testing file not in manual/
// │   └── automated_test.rs
// └── docs/
//     └── testing.md             // FORBIDDEN: Manual testing plan outside tests/
```

### Testing : Integration Test Feature Gating

**Description:** All integration tests **must** be conditionally compiled using a feature named `integration`. This feature **must** be included in the crate's `default` feature set in its `Cargo.toml`. This allows developers to optionally exclude slow or environment-dependent tests from normal build and test cycles.

**Implementation:**
1.  In `Cargo.toml`, define the `integration` feature and add it to the `default` list.
2.  At the top of each integration test file in the `tests` directory, add the attribute `#[ cfg( feature = "integration" ) ]`.

**Rationale:**
-   **Build Flexibility:** Allows for running `cargo test --no-default-features` to execute only the unit tests (if any) and skip integration tests.
-   **CI Optimization:** CI pipelines can have separate, faster jobs that run without default features, and slower, more comprehensive jobs that run with the `integration` feature enabled.

> ❌ **Bad** (Integration test file without a feature gate)

```rust
// In tests/my_integration_test.rs
// FORBIDDEN: This test will always run and cannot be disabled via features.

#[test]
fn test_database_connection()
{
  // ... some slow test ...
}
```

> ✅ **Good** (Correct `Cargo.toml` and feature-gated test file)

```toml
# In Cargo.toml
[features]
default = [ "integration" ]
integration = []
```

```rust
// In tests/my_integration_test.rs
// Correct: This entire test file is gated by the `integration` feature.
#![cfg(feature = "integration")]

#[test]
fn test_database_connection()
{
  // ... some slow test ...
}
```

### Testing : Centralized Benchmarks Directory

**Description:** This is a rigid and non-negotiable rule. All benchmarks and benchmark-related files **must** be located in the top-level `benches` directory of the crate (plural, not singular). This follows the Rust ecosystem standard and Cargo convention.

**Mandatory Requirements:**
1. **Directory Name**: Must be `benches/` (plural) at the crate root
2. **All benchmark source files**: `benches/*.rs` containing benchmark functions
3. **All benchmark utilities and helpers**: Any benchmark-related code must be in `benches/`
4. **Documentation**: `benches/readme.md` file is **mandatory** for every crate with benchmarks
5. **No temporary files**: Only permanent, committed benchmark files allowed in `benches/`

**Strictly Forbidden:**
- Benchmark files in `examples/`, `src/`, or `tests/` directories
- Using `benchmarks/` (singular) instead of `benches/` (plural)
- Missing `readme.md` in the `benches/` directory
- Temporary benchmark files or scratch code in `benches/`

**Cargo.toml Integration:**
All benchmarks must be declared using `[[bench]]` sections pointing to `benches/` directory.

**Rationale:**
- **Rust Ecosystem Standard**: `benches/` is the official Cargo convention for benchmarks
- **Tool Compatibility**: `cargo bench` expects benchmarks in `benches/` directory
- **Clear Documentation**: Mandatory readme.md ensures benchmarks are documented and discoverable
- **Separation of Concerns**: Benchmarks are neither tests nor examples, they measure performance

> ❌ **Bad** (Incorrect benchmark placement and missing documentation)

```text
// FORBIDDEN: Wrong directory names and missing documentation
// ├── benchmarks/           // FORBIDDEN: Should be "benches" (plural)
// │   └── my_benchmark.rs   // FORBIDDEN: Wrong directory
// ├── examples/
// │   └── performance_test.rs  // FORBIDDEN: Benchmarks don't belong in examples
// ├── tests/
// │   └── bench_utils.rs    // FORBIDDEN: Benchmarks don't belong in tests
// └── src/
//     └── bench_helpers.rs  // FORBIDDEN: Benchmark code in src
```

> ✅ **Good** (Correct benchmark structure with mandatory documentation)

```text
// Correct: Standard Rust benchmark structure
// ├── benches/
// │   ├── readme.md              // MANDATORY: Documents all benchmarks
// │   ├── core_algorithms.rs     // Benchmark source files
// │   ├── parsing_performance.rs
// │   └── memory_usage.rs
// ├── Cargo.toml                 // Contains [[bench]] declarations
// └── src/
//     └── lib.rs
```

```toml
# In Cargo.toml - Correct benchmark declarations
[[bench]]
name = "core_algorithms"
path = "benches/core_algorithms.rs"
harness = false

[[bench]] 
name = "parsing_performance"
path = "benches/parsing_performance.rs"
harness = false
```

```rust
// In benches/core_algorithms.rs - Correct benchmark implementation
use criterion::{ black_box, criterion_group, criterion_main, Criterion };
use my_crate::algorithms;

fn benchmark_sort_algorithm( c: &mut Criterion )
{
  c.bench_function( "sort_large_dataset", |b| 
  {
    b.iter( || algorithms::sort( black_box( &large_dataset ) ) );
  });
}

criterion_group!( benches, benchmark_sort_algorithm );
criterion_main!( benches );
```

```markdown
<!-- In benches/readme.md - MANDATORY documentation -->
# Benchmarks

This directory contains performance benchmarks for the crate.

## Running Benchmarks

```bash
cargo bench
```

## Available Benchmarks

- `core_algorithms`: Tests sorting and searching performance
- `parsing_performance`: Measures text parsing throughput  
- `memory_usage`: Evaluates memory allocation patterns

## Requirements

- Rust nightly for benchmark harness
- At least 4GB RAM for large dataset benchmarks
```

**Enforcement Commands:**
- Detect incorrect directory: `find . -name "benchmarks" -type d`
- Check for missing readme: `test -f benches/readme.md || echo "Missing benches/readme.md"`
- Find misplaced benchmarks: `find examples tests src -name "*bench*" -o -name "*performance*"`

### Testing : Strict Separation of Performance Tests from Functional Tests

**Description:** Performance tests, benchmark tests, and any testing focused on measuring execution time, memory usage, or throughput **must** be located exclusively in the `benches/` directory. It is **strictly forbidden** to place performance-oriented testing code in the `tests/` directory.

**Strictly Prohibited in `tests/` Directory:**
- Performance measurement tests (execution time, latency)
- Benchmark tests of any kind
- Memory usage measurement tests
- Throughput measurement tests
- Load testing or stress testing
- Any test primarily focused on performance metrics rather than functional correctness

**Mandatory Requirements:**
- **All performance tests:** Must be in `benches/` directory
- **All benchmark utilities:** Must be in `benches/` directory
- **All performance test data:** Must be in `benches/` directory
- **All benchmark configurations:** Must be in `benches/` directory

**Rationale:**
- **Clear Separation of Concerns:** Functional correctness testing (`tests/`) vs. performance measurement (`benches/`)
- **Different Execution Contexts:** Performance tests require different tooling, execution environments, and interpretation
- **CI/CD Optimization:** Prevents performance tests from affecting fast feedback loops in continuous integration
- **Tool Compatibility:** `cargo test` vs. `cargo bench` have different purposes and execution models

> ❌ **Bad** (Performance tests in wrong location)

```rust
// FORBIDDEN: In tests/performance_test.rs
#[test]
fn test_algorithm_performance() {
    let start = std::time::Instant::now();
    algorithm_under_test();
    let duration = start.elapsed();
    assert!(duration < std::time::Duration::from_millis(100)); // FORBIDDEN: Performance assertion
}

// FORBIDDEN: In tests/benchmark_comparison.rs
#[test]
fn benchmark_different_implementations() {
    // FORBIDDEN: Comparing performance of different implementations
    for implementation in &implementations {
        let start = std::time::Instant::now();
        implementation.run();
        println!("Duration: {:?}", start.elapsed()); // FORBIDDEN: Performance measurement
    }
}
```

> ✅ **Good** (Performance tests in correct location)

```rust
// Correct: In benches/algorithm_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_algorithm(c: &mut Criterion) {
    c.bench_function("algorithm_performance", |b| {
        b.iter(|| algorithm_under_test(black_box(&test_data)));
    });
}

criterion_group!(benches, benchmark_algorithm);
criterion_main!(benches);
```

### Testing : Benchmark Documentation Automation

**Description:** All benchmark documentation files (`.md`) in the `benches/` directory **must** be updated automatically by the benchmarking utilities during their execution. Manual editing of these files is **strictly forbidden**.

**Requirements:**
1. **Automated `benches/readme.md` Updates**: Benchmark runners must automatically generate and update the readme
2. **Automated `benches/changes.md` Updates**: Major performance changes must be automatically logged
3. **No Manual Editing**: Human editing of benchmark documentation is prohibited to prevent inconsistencies

**Rationale:** Automation ensures that performance documentation is always synchronized with the latest benchmark results, eliminating human error and the risk of outdated information.

> ✅ **Good** (Automated benchmark documentation)

```rust
fn main()
{
  // 1. Run the benchmark and collect metrics.
  let results = run_parsing_benchmark();

  // 2. Format the key results for the README.
  let readme_content = format!( "# Parsing Performance\n\nThroughput: {} MB/s", results.throughput );
  std::fs::write( "benches/readme.md", readme_content ).unwrap();

  // 3. (If a major change is detected) Append to the changes log.
  if results.is_major_change
  {
    let change_entry = format!( "## {} - Performance Improvement\nThroughput improved by {}%", 
                               chrono::Utc::now().format( "%Y-%m-%d" ), results.improvement_percent );
    std::fs::write( "benches/changes.md", change_entry ).unwrap();
  }
}
```

> ❌ **Bad** (Manual benchmark documentation editing)

```text
// FORBIDDEN: Developer runs a benchmark, sees "12,345 ns/iter" in the console,
// then manually opens `benches/readme.md` and types in the new result.
// This is error-prone and not reproducible.
```

### Testing : Strategic Benchmarking Focus

**Description:** Benchmarking efforts **must** be strategic, focusing on identifying and measuring critical performance bottlenecks rather than aiming for exhaustive coverage.

**Strategic Philosophy:**
- **Avoid Proliferation**: Do not create an excessive number of benchmarks. Prioritize creating benchmarks for the most performance-sensitive or frequently used code paths that have been identified as potential bottlenecks.
- **Focus on Bottlenecks**: Whenever possible, use profiling tools (e.g., `perf`, `flamegraph`) to identify actual bottlenecks *first*. Then, create targeted benchmarks to measure, track, and validate improvements to that specific code.
- **Measure What Matters**: Focus on metrics that directly impact the user experience or system efficiency (e.g., throughput, latency, memory usage) rather than chasing micro-optimizations with negligible real-world impact.

**Rationale:** A focused approach ensures that engineering effort is spent on performance improvements that provide the most value. It prevents the maintenance burden of a large, unfocused suite of benchmarks that can become noisy and difficult to interpret.

### Testing : Real API Integration Testing

**Description:** Integration tests that target external APIs **must** use real API endpoints with authentic tokens and credentials. Tests **must** run by default and **must** fail explicitly with clear error messages when required tokens are missing. Silent passes or skipped tests due to missing credentials are **strictly forbidden**.

**Mandatory Requirements:**
1. **Default Execution**: API integration tests must run by default (via `integration` feature in default features)
2. **Explicit Token Validation**: Tests must fail loudly when tokens are missing with helpful error messages
3. **Real API Endpoints**: Integration tests must connect to actual API services, not mock servers
4. **Authentic Credentials**: Use real API tokens, keys, and authentication credentials from workspace secrets
5. **No Silent Failures**: Tests must never silently pass or skip when credentials are unavailable

**Strictly Forbidden:**
- Silent passing when API tokens are missing
- Skipping tests without explicit failure when credentials unavailable
- Mock HTTP clients or HTTP responses for API integration tests
- Fake API responses or stubbed network calls
- Tests that "succeed" without actually testing the API

**Rationale:**
- **Fail Fast**: Missing credentials should be caught immediately, not discovered later
- **Clear Feedback**: Developers must know exactly what credentials are needed and where to get them
- **Authentic Integration**: Real API calls reveal actual integration issues that mocks cannot detect
- **Reliable CI/CD**: Integration tests provide meaningful signal, not false confidence

> ❌ **Bad** (Silent skip when token missing)

```rust
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_anthropic_api_integration()
{
  // FORBIDDEN: Silently skipping test when token is missing
  let api_key = std::env::var( "ANTHROPIC_API_KEY" );
  if api_key.is_err()
  {
    println!( "Skipping test - no API key" );
    return; // FORBIDDEN: Silent success
  }
  
  // Test never actually validates API integration when key is missing
}
```

> ❌ **Bad** (Using unwrap_or with fake fallback)

```rust
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_anthropic_api_integration()
{
  // FORBIDDEN: Using fake token as fallback
  let api_key = workspace_tools::secret::load_secret( "anthropic_api_key" )
    .unwrap_or_else( |_| "fake-token".to_string() ); // FORBIDDEN
  
  // Test with fake token provides no real integration validation
}
```

> ✅ **Good** (Explicit failure with helpful message)

```rust
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_anthropic_api_integration()
{
  // Correct: Explicit failure with setup instructions when token missing
  let api_key = workspace_tools::secret::load_secret( "anthropic_api_key" )
    .expect( "ANTHROPIC_API_KEY secret not found. 
    
To set up integration test credentials:
1. Get your API key from: https://console.anthropic.com
2. Save it to workspace secrets: echo 'your-key-here' > secret/-anthropic_api_key
3. Re-run tests: cargo test

Integration tests MUST use real credentials to validate actual API behavior." );
  
  // Correct: Create client with real API endpoint
  let api_client = AnthropicClient::new( &api_key );
  
  // Correct: Make real network request to actual API
  let response = api_client.send_message( "Hello, this is an integration test" )
    .expect( "Real API integration test failed - check network connectivity and API key validity" );
  
  // This validates actual API behavior
  assert!( !response.content.is_empty(), "API response should contain content" );
  assert!( response.model.contains( "claude" ), "Response should indicate Claude model" );
}
```

> ✅ **Good** (Cargo.toml ensuring integration tests run by default)

```toml
[ features ]
default = [ "integration" ]  # CRITICAL: Integration tests run by default
integration = [ ]

[ [ test ] ]
name = "anthropic_api_integration"
path = "tests/anthropic_api_integration.rs" 
required-features = [ "integration" ]
```

**Setup Instructions for Teams:**

```bash
# Required: Set up workspace secrets for all team members
echo "sk-ant-api03-..." > secret/-anthropic_api_key
echo "sk-..." > secret/-openai_api_key

# Integration tests now run by default and validate real API access
cargo test

# To temporarily skip integration tests (discouraged):
cargo test --no-default-features
```

**CI/CD Configuration:**
Integration tests should run in CI with real credentials from secure environment variables or secret management systems. Tests that cannot access real APIs should fail the CI build until credentials are properly configured.

### Formatting & Whitespace : New Lines for Blocks

-   Open `{`, `(`, `<` on new lines, except when the block is concise enough to fit on a single line (e.g., `vec![ 1 ]`, `Some( x )`, `if condition { return; }`).
-   Do not place the opening brace `{` on the same line as a function signature, control structure signature (like `if`, `match`, `loop`), or type definition (`struct`, `enum`, `union`).
-   **Struct, Enum, and Union Definitions**: Place each field or variant on a new line, indented by 2 spaces, after the opening brace `{`. The closing brace `}` should be on its own line, aligned with the start of the definition keyword (`struct`, `enum`, `union`).
-   Macro is not exception. When using macros like `quote!` and arguments for macro is longer to be on the same line with, place the opening `{` on a new line following the macro call.
-   For lambda expressions ensure that the opening brace `{` of the closure's body is placed on a new line, unless whole body of closure is on the same line.
-   For lambda expressions, ensure that the `|` symbols and parameters inside are separated by spaces from the parameters and the body block.

> ❌ **Bad (Function/Control Structure)**

```rust
fn f1() {
  if condition {
    // Code block
  }
}
```

> ✅ **Good (Function/Control Structure)**

```rust
fn f1()
{
  if condition
  {
    // Code block
  }
}
```

> ❌ **Bad (Macro)**

```rust
let result = quote! {
  #( #from_impls )*
};
```

> ✅ **Good (Macro)**

```rust
let result = quote!
{
  #( #from_impls )*
};
```

> ❌ **Bad (Lambda)**

```rust
fields
.iter()
.map( | field | {
  f1( field );
});
```

> ✅ **Good (Lambda)**

```rust
fields
.iter()
.map( | field |
{
  f1( field );
});
```

> ❌ **Bad (Struct Initialization)**

```rust
let test_object = TestObject {
  created_at : 1627845583,
  tools : Some( vec![ {
    let mut map = HashMap::new();
    map.insert( "tool1".to_string(), "value1".to_string() );
    map
  } ] ),
};
```

> ✅ **Good** (Struct Initialization)

```rust
let test_object = TestObject
{
  created_at : 1627845583,
  tools : Some
  (
    vec!
    [
      {
        let mut map = HashMap::new();
        map.insert( "tool1".to_string(), "value1".to_string() );
        map
      }
    ]
  ),
};
```

> ❌ **Bad (Struct Definition)**

```rust
struct Point { x: i32, y: i32 }
```

> ❌ **Bad (Enum Definition)**

```rust
enum Color { Red, Green, Blue }
```

> ✅ **Good (Struct Definition)**

```rust
struct Point
{
  x : i32,
  y : i32,
}
```

> ✅ **Good (Enum Definition)**

```rust
enum Color
{
  Red,
  Green,
  Blue,
}
```

> ✅ **Good (Concise Single Line Block)**

```rust
let v = vec![ 1 ]; // Okay for very short blocks
if condition { return; } // Okay for very short blocks
```

### Formatting & Whitespace : Indentation

-   Use strictly 2 spaces over tabs for consistent indentation across environments. Avoid using 4 spaces or tabs.
-   When chaining method calls, start each method on a new line if including everything on the same line exceeds an admissible line length. Each method call in the chain should start directly below the first character of the chain start, without additional indentation. This ensures clarity and consistency without unnecessary spaces or alignment efforts.
-   When chaining method calls that exceed a single line's admissible length, each method call in the chain should start on a new line directly below the object or variable initiating the chain. These subsequent method calls should align with the first character of the initiating call, without additional indentation beyond what is used for the start of the chain. Avoid adding extra indentation before each method in the chain, as this can obscure the structure and flow of chained operations.
-   Method calls starting with a dot (`.method`) should not be indented further than the first call in the chain.

> ✅ **Good**

```rust
struct Struct1
{
  a : i32,
  b : i32,
}
```

> ✅ **Good**

```rust
fields
.iter()
.map( | field |
{
  f1( field );
});
```

### Formatting & Whitespace : Chained Method Calls

When chaining method calls that exceed a single line's admissible length, each method call in the chain should start on a new line directly below the object or variable initiating the chain. Avoid additional indentation for these subsequent method calls beyond what is used for the start of the chain. This ensures clarity and consistency without unnecessary spaces or alignment efforts.

> ❌ **Bad**

```rust
Request::builder()
  .method( Method::GET )
  .header( "X-MBX-APIKEY", api_key )
  .build()?;
```

> ✅ **Good**

```rust
Request::builder()
.method( Method::GET )
.header( "X-MBX-APIKEY", api_key )
.build()?;
```

### Formatting & Whitespace : Line Breaks for Method Chains and Namespace Access

When breaking a line due to a method chain (using `.`) or namespace access (using `::`), maintain the same indentation as the first line. This rule applies to both method chaining and accessing nested namespaces or modules. This approach maintains visual consistency and makes it easier to follow the flow of method calls or namespace access, especially in longer chains or nested structures.

> ❌ **Bad**

```rust
chrome.tabs.query( {} )
  .then( function( tabs )
  {
    const tabList = document.getElementById( 'tabList' );
  });

std::collections::HashMap
  ::new()
  .insert( key, value );
```

> ✅ **Good**

```rust
chrome.tabs.query( {} )
.then( function( tabs )
{
  const tabList = document.getElementById( 'tabList' );
});

std::collections::HashMap
::new()
.insert( key, value );
```

### Formatting & Whitespace : Spaces Around Symbols

-   Include a space before and after `:`, `=`, and operators, excluding the namespace operator `::`.
-   Don't include a space before and after namespace operator `::`.
-   Place a space after `,` to separate list items, such as function arguments or array elements.

> ✅ **Good**

```rust
fn f1( a : f32, b : f32 )
{
  2 * ( a + b )
}
```

### Formatting & Whitespace : Spaces for Blocks

-   **Space After Opening Symbols** : After opening `{`, `(`, `<`, `[`, and `|`, insert a space if they are followed by content on the same line. This includes not just braces and parentheses, but also less than symbols `<` when used in generic type parameters or comparisons, to enhance readability.
-   **Space Before Closing Symbols** : Before closing `|`, `]`, `}`, `)`, and `>`, insert a space if they are preceded by content on the same line. This rule is particularly important for greater than symbols `>` in generic type parameters or comparisons to avoid confusion with other operators or punctuation.

> ❌ **Bad**

```rust
use std::fmt::{Debug,Display};
#[derive(Debug,Display)]
struct MyInt(i32);
struct Struct1<T,U>{a:T,b:U};
let lambda = |x:i32, y:i32|{f1(x + y)};
fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result;
let slice: &[u32]
&
[u32; 3]
mem::size_of::<u32>()
```

> ✅ **Good**

```rust
use std::fmt::{ Debug, Display };
#[ derive( Debug, Display ) ]
struct MyInt( i32 );
struct Struct1< T, U > { a : T, b : U };
let lambda = | x : i32, y : i32 | { f1( x + y ) };
fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result;
let slice : &[ u32 ];
&[ 10, 20, 30 ];
[ u32; 3 ];
mem::size_of::< u32 >();
```

### Formatting & Whitespace : Spacing Around Angle Brackets in Generics

-   **Use Spaces Inside Angle Brackets**: When using angle brackets `<` and `>` for generic type parameters, always include a space after the opening bracket `<` and before the closing bracket `>`. This aligns with the general `Spaces for Blocks` rule and improves readability.

> ❌ **Bad**

```rust
Option<u64>
Result<T, E>
Vec<String>
Vec<Message>
```

> ✅ **Good**

```rust
Option< u64 >
Result< T, E >
Vec< String >
Vec< Message >
```

### Formatting & Whitespace : Attributes: Spaces

**Description:** Attributes provide metadata about code items. For clarity and consistency:
*   Place each attribute on its own line before the item it annotates.
*   **Ensure there are spaces immediately inside *both* the brackets `[]` and the parentheses `()` if present.** This means a space after `[` and before `]`, and a space after `(` and before `)` if the attribute takes arguments.
*   Ensure there is a space between the attribute name and the opening parenthesis `(` if arguments are present.

> ❌ **Bad** (Missing internal spaces in `[]` and `()`)

```rust
#![feature(async_fn_in_trait)]
#[repr(transparent)]
#[derive(Debug)]
#[test]
#[inline]
```

> ✅ **Good** (Correct internal spacing in `[]` and `()`)

```rust
#![ feature( async_fn_in_trait ) ]
#[ repr( transparent ) ]
#[ derive( Debug ) ]
#[ test ]
#[ inline ]
```

### Formatting & Whitespace : Attributes: Separate Attributes from Items

**Description:** Each attribute (`#[...]` or `#![...]`) must be placed on its own line. Furthermore, the entire block of attributes annotating an item (like a struct, enum, function, field, etc.) must be separated from the item itself by a newline. This ensures clear visual separation between metadata (attributes) and the code element they modify.

> ❌ **Bad** (Attribute on same line as item, incorrect brace placement, incorrect spacing)

```rust
#[derive(Debug)] struct MyStruct { field: i32 } // Bad: Attribute, brace, missing space around :, missing space inside ()

struct AnotherStruct {
    #[serde(skip)] pub data: String, // Bad: Attribute on same line as field, missing space around :, missing space inside ()
}

#[inline] fn my_function() {} // Bad: Attribute on same line as function
```

> ❌ **Bad** (Multiple attributes, last one on same line as item, incorrect brace placement, incorrect spacing)

```rust
#[derive(Debug)] #[serde(rename = "my_struct")] struct MyStruct { field: i32 } // Bad: Attributes, brace, missing space around :, missing space inside ()

struct YetAnotherStruct {
    /// Some doc comment
    #[serde(skip)] #[repr(transparent)] pub value: i32, // Bad: Last attribute on same line, missing space around :, missing space inside ()
}
```

> ✅ **Good** (Attributes on separate lines above the item, correct brace placement, correct spacing)

```rust
#[ derive( Debug ) ] // Good: Spaces inside ()
struct MyStruct // Good: Struct keyword starts declaration
{ // Good: Opening brace on new line
  field : i32, // Good: Space around :
}

struct AnotherStruct
{ // Good: Opening brace on new line
  #[ serde( skip ) ] // Good: Attribute on its own line, spaces inside ()
  pub data : String, // Good: Field definition follows attribute, space around :
}

#[ inline ] // Good: Attribute on its own line, no content inside () so no extra space needed
fn my_function() // Good: Function signature follows attribute
{ // Good: Opening brace on new line
  // ...
}

#[ derive( Debug ) ] // Good: Attribute on its own line, spaces inside ()
#[ serde( rename = "my_struct" ) ] // Good: Attribute on its own line, spaces inside ()
struct MyStruct2 // Good: Struct keyword starts declaration
{ // Good: Opening brace on new line
  field : i32, // Good: Space around :
}

struct YetAnotherStruct
{ // Good: Opening brace on new line
  /// Some doc comment
  #[ serde( skip ) ] // Good: Attribute on its own line, spaces inside ()
  #[ repr( transparent ) ] // Good: Attribute on its own line, spaces inside ()
  pub value : i32, // Good: Field definition follows attributes, space around :
}
```

### Formatting & Whitespace : Formatting `where` Clauses

-   New Line for Where Clause : The `where` keyword should start on a new line when the preceding function, struct, or impl declaration line is too long, or when it contributes to better readability.
-   One Parameter Per Line : Each parameter in the `where` clause should start on a new line. This enhances readability, especially when there are multiple constraints or when constraints are lengthy.

> ✅ **Good**

```rust
impl< K, Definition, > CommandFormer< K, Definition, >
where
  K : core::hash::Hash + std::cmp::Eq,
  Definition : former::FormerDefinition,
  Definition::Types : former::FormerDefinitionTypes< Storage = CommandFormerStorage< K, > >
{
  // Implementation goes here
}
```

> ❌ **Bad**

```rust
impl< K, Definition, > CommandFormer< K, Definition, > where K : core::hash::Hash + std::cmp::Eq, Definition : former::FormerDefinition, Definition::Types : former::FormerDefinitionTypes< Storage = CommandFormerStorage< K, > > {
  // Implementation goes here
}
```

### Formatting & Whitespace : Trait Implementation Formatting

-   **Trait on New Line**: When defining a trait implementation (`impl`) for a type, if the trait and the type it is being implemented for do not fit on the same line, the trait should start on a new line.
-   **Consistent Where Clause**: The `where` clause should also start on a new line to maintain readability, especially when there are constraints or multiple bounds.

> ✅ **Good**

```rust
impl< K, __Context, __Formed, > ::Trait1
for CommandFormerDefinitionTypes< K, __Context, __Formed, >
where
  K : core::hash::Hash + std::cmp::Eq,
{
}
```

> ❌ **Bad**

```rust
impl< K, __Context, __Formed, > ::Trait1 for CommandFormerDefinitionTypes< K, __Context, __Formed, > where K : core::hash::Hash + std::cmp::Eq,
{
}
```

### Formatting & Whitespace : Function Signature Formatting

-   **Parameter Alignment**: Function parameters should be listed with one per line, each starting on a new line after the opening parenthesis. This enhances readability and version control diff clarity.
-   **Return Type on New Line**: The return type should start on a new line when the parameters or function signature is too long or for consistency with the rest of the codebase.
-   **Where Clause Alignment**: The `where` clause should start on a new line, aligning it consistently beneath the function signature, not inline with the last parameter or return type.

> ✅ **Good**

```rust
#[ inline( always ) ]
pub fn begin< IntoEnd >
(
  mut storage : core::option::Option< < Definition::Types as former::FormerDefinitionTypes >::Storage >,
  context : core::option::Option< < Definition::Types as former::FormerDefinitionTypes >::Context >,
  on_end : IntoEnd,
)
-> Self
where
  IntoEnd : ::core::convert::Into< < Definition as former::FormerDefinition >::End >
{
}
```

> ❌ **Bad**

```rust
#[ inline( always ) ]
pub fn begin< IntoEnd >( mut storage : core::option::Option< < Definition::Types as former::FormerDefinitionTypes >::Storage >, context : core::option::Option< < Definition::Types as former::FormerDefinitionTypes >::Context >, on_end : IntoEnd, ) -> Self
where IntoEnd : ::core::convert::Into< < Definition as former::FormerDefinition >::End >
{
}
```

### Formatting & Whitespace : Match Expression Formatting

When using `match` expressions, place the opening brace `{` for multi-line blocks on a new line after the match arm. This enhances readability and consistency, especially for complex match patterns.

> ❌ **Bad**

```rust
match self.index
{
  0 => {
    self.index += 1;
  },
}
```

> ✅ **Good**

```rust
match self.index
{
  0 =>
  {
    self.index += 1;
  },
}
```
This formatting rule applies to all `match` expressions where the arm's body spans multiple lines, ensuring consistent and readable code structure.

### Formatting & Whitespace : Lifetime Annotations

-   **No Spaces Around Lifetime Specifier**: When using lifetime annotations (e.g., `'a`), do not include spaces between the ampersand `&` and the lifetime specifier.

> ✅ **Good**

```rust
fn info< 'a >( src : &'a str ) -> &'a str
{
  src
}
```

> ❌ **Bad**

```rust
fn info< 'a >( src : & 'a str ) -> & 'a str
{
  src
}
```

### Formatting & Whitespace : Nesting

-   Avoid complex, multi-level inline nesting. Prefer splitting content across multiple lines.
-   Opt for shorter, clearer lines over long, deeply nested ones to enhance code maintainability.

### Formatting & Whitespace : Code Length

-   Aim for concise, focused functions to improve both readability and ease of maintenance.
-   Keep lines under 110 characters to accommodate various editor and IDE setups without horizontal scrolling.

### Comments : Spacing in Comments

-   Inline comments (`//`) should start with a space following the slashes for readability.

### Comments : Comment Content and Task Preservation

**Description:** Comments should primarily explain the "why" or clarify non-obvious aspects of the *current* code, adhering to the principles in the main "[Comments and Documentation](#development-process-comments-and-documentation)" rule. Avoid adding comments that merely state *what* change was just made (e.g., "Removed unused import", "Added derive") or serve purely as a historical log. Such transitory comments clutter the code without providing lasting value.

**Crucially, do not remove existing task-tracking comments.** These are typically prefixed with labels like `TODO:`, `FIXME:`, `xxx:`, `qqq:`, `ppp:`, `yyy:`, `iii:`, or similar conventions, and are essential for project management and future development. See the "[Comments: Defining and Using Task Markers](#comments-defining-and-using-task-markers)" rule for guidance on adding *new* tasks and the "[Comments: Annotating Addressed Tasks](#comments-annotating-addressed-tasks)" for annotating existing ones.

> ❌ **Bad** (Comment describes the *change*, not the *code*)

```rust
// Removed unused import: use std::collections::HashMap;
use std::fmt;

struct MyData
{
  // Added field for caching
  cache_value : Option< i32 >,
}
```

> ✅ **Good** (No comment needed for obvious change, or comment explains *why*)

```rust
use std::fmt; // No comment needed for simple removal

struct MyData
{
  /// Stores a cached computation result to avoid re-calculation.
  /// Cleared when relevant inputs change.
  cache_value : Option< i32 >,
}
```

> ✅ **Good** (Preserving existing task comments)

```rust
use std::fmt;

struct MyData
{
  // TODO: Implement proper caching logic here. // Keep existing TODO
  // xxx: Consider using a different Option type for performance. // Keep existing xxx
  cache_value : Option< i32 >,
}
```

### Comments : Defining and Using Task Markers

**Description:** Use structured `Task Markers` in source code comments to track tasks, requests, and their resolutions. This practice connects the codebase directly to the task management process.

**Schema:**
`// <marker> : [optional context/person] : <description>`
-   The description can be multi-line, with subsequent lines also commented.

**Marker Types & Meanings:**
-   `xxx:`, `todo:`: A general-purpose task or something that needs to be done. Prefer `xxx:` for consistency.
-   `qqq:`: A question or a request for a decision, often from a team lead to a developer. The developer should not change the `qqq:` line itself but should respond with an `aaa:` marker.
-   `aaa:`: An answer or a report on an action taken in response to another marker (typically a `qqq:` or `xxx:`). It should be placed directly below the marker it addresses.
-   `zzz:`: A low-priority task that can be deferred.

> ✅ **Good** (Using various task markers)

```rust
// xxx: @dev-team : This function is inefficient and needs to be refactored.
// It currently uses a linear search, but a HashMap would be better.
fn find_item_slowly( id: &str ) -> Option< Item > { /* ... */ }

// qqq: @lead-dev : Should we support legacy format v1 in this parser?
// Supporting it adds complexity but maintains backward compatibility.
fn parse_data( data: &[u8] ) -> Result< Data, ParseError > { /* ... */ }

// aaa: @dev : Yes, we need to support v1 for now. Please proceed.
// (This would be the response to the qqq above)

// zzz: The logging here is a bit verbose. Could be cleaned up in the future.
log::debug!( "Processing item: {:?}", item );
```

### Comments : Annotating Addressed Tasks

**Description:** When addressing or investigating an existing task comment (e.g., `// TODO:`, `// xxx:`, `// FIXME:`), **do not remove the original task comment**. Instead, add a new comment line immediately below it, starting with `// aaa:` (for "addressed" or "analyzed"), explaining the findings, actions taken, or current status regarding that specific task. This preserves the original context while providing an update.

> ❌ **Bad** (Removing the original task comment)

```rust
fn calculate_value() -> i32
{
  // Original comment was: // xxx: This calculation might be wrong for edge cases.
  // aaa: Reviewed calculation, seems correct for expected inputs.
  5 // Calculation logic
}
```

> ❌ **Bad** (Adding `aaa:` comment far away from the original task)

```rust
fn calculate_value() -> i32
{
  // xxx: This calculation might be wrong for edge cases.
  let result = 5; // Calculation logic
  // ... other code ...
  // aaa: Reviewed calculation, seems correct for expected inputs. // Annotation is disconnected
  result
}
```

> ✅ **Good** (Adding `aaa:` annotation directly below the original task)

```rust
fn calculate_value() -> i32
{
  // xxx: This calculation might be wrong for edge cases.
  // aaa: Reviewed calculation, seems correct for expected inputs based on current requirements.
  5 // Calculation logic
}

fn another_function()
{
  // TODO: Refactor this section for clarity.
  // aaa: Refactored loop structure and added comments.
  for i in 0..10
  {
    // ... complex logic ...
  }
}
```

### Macros : Declarative Macros (macro_rules)

Overall, code style for macros is the same as for the simple code, but there are some caveats you should know.

### Macros : The `=>` Token

Generally, `=>` token should reside on a separate line from macro pattern

> ❌ **Bad**

```rust
macro_rules! count
{
  ( @count $( $rest : expr ),* ) =>
  (
    /* body */
  );
}
```

> ❌ **Bad**

```rust
macro_rules! count
{
  (
    @count $( $rest : expr ),*
  ) => (
    /* body */
  );
}
```

> ✅ **Good**

```rust
macro_rules! count
{
  (
    @count $( $rest : expr ),*
  )
  =>
  (
    /* body */
  );
}
```

### Macros : Braces in Macro Bodies

You are allowed to place the starting `{{` and the ending `}}` on the same line to improve readability

> ❌ **Bad**

```rust
macro_rules! hmap
{
  (
    /* pattern */
  )
  =>
  {
    {
      let _cap = hmap!( @count $( $key ),* );
      let mut _map = std::collections::HashMap::with_capacity( _cap );
      $(
        let _ = _map.insert( $key.into(), $value.into() );
      )*
      _map
    }
  };
}
```

> ✅ **Good**

```rust
macro_rules! hmap
{
  (
    /* pattern */
  )
  =>
  {{
    let _cap = hmap!( @count $( $key ),* );
    let mut _map = std::collections::HashMap::with_capacity( _cap );
    $(
      let _ = _map.insert( $key.into(), $value.into() );
    )*
    _map
  }};
}
```

### Macros : Short Macro Matches

You can place the macro pattern and its body on the same line if they are short enough.

> ❌ **Bad**

```rust
macro_rules! empty
{
  (
    @single $( $x : tt )*
  )
  =>
  (
    ()
  );
}
```

> ✅ **Good**

```rust
macro_rules! empty
{
  ( @single $( $x : tt )* ) => ( () );
}
```

### Naming Conventions : File Naming

Custom file names should use `snake_case` and be in all lowercase letters. **Exception: Standard tooling files** (e.g., `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `Makefile`) must retain their conventional names for proper tool recognition. This rule applies to project-specific files, custom modules, and user-created documentation.

> ✅ **Good**

```text
my_module.rs          # Custom source files
user_guide.md         # Custom documentation  
app_config.toml       # Custom configuration
build_helper.sh       # Custom scripts
readme.md            # Documentation (follows naming rules)
license              # Repository file (follows naming rules)
Cargo.toml           # Standard tooling file (protected)
Dockerfile           # Standard tooling file (protected)
```

> ❌ **Bad**

```text
MyModule.rs          # Custom file should use snake_case
my-module.rs         # Custom file should use snake_case
UserGuide.md         # Custom documentation should use snake_case
cargo.toml           # WRONG: Standard tooling file renamed
dockerfile           # WRONG: Standard tooling file renamed
```

### Naming Conventions : Standard Tooling File Exceptions

**Description:** Certain files must retain their conventional names to ensure proper recognition by standard development tools and ecosystem conventions.

**Protected File Names:**
- `Cargo.toml`, `Cargo.lock` (Rust ecosystem)
- `Dockerfile`, `docker-compose.yml` (Docker ecosystem) 
- `Makefile` (Make build system)
- `.gitignore`, `.gitattributes` (Git)
- Package manager files (`package.json`, `requirements.txt`, etc.)

**Rationale:** These files have standardized names expected by tooling. Renaming them breaks tool discovery and ecosystem integration.

**Application:** The `lowercase_snake_case` rule applies to:
- Custom source files (`my_module.rs`)
- Project-specific documentation (`user_guide.md`, `readme.md`)  
- Repository files (`license`)
- Custom configuration files (`app_config.toml`)
- Utility scripts (`build_helper.sh`)

> ✅ **Good** (Protected standard files)

```text
Cargo.toml           # Rust package manifest
Cargo.lock           # Rust dependency lockfile  
Dockerfile           # Docker container definition
Makefile             # Make build configuration
.gitignore           # Git ignore patterns
package.json         # Node.js package manifest
```

> ❌ **Bad** (Incorrectly renamed standard files)

```text
cargo.toml           # WRONG: Tools expect "Cargo.toml"
dockerfile           # WRONG: Docker expects "Dockerfile" 
makefile             # WRONG: Make expects "Makefile"
README.md            # WRONG: Should be "readme.md" per naming rules
LICENSE              # WRONG: Should be "license" per naming rules
```

### Naming Conventions : Directory Naming Conventions

**Description:** If a crate's name contains a prefix that matches the name of its parent directory, this prefix **must** be removed from the crate's own directory name on the filesystem. The full crate name, including the prefix, **must** be preserved in its `Cargo.toml` file.

**Rationale:** This convention eliminates redundancy and "stuttering" in file paths (e.g., `api/api_gemini`), leading to cleaner, more intuitive project navigation. It keeps the filesystem organized by logical grouping (the parent directory) while ensuring the crate's canonical name remains explicit for dependency management and publishing purposes.

> ❌ **Bad** (Redundant `api` prefix in the directory name)

```text
// Filesystem structure
└── api/
    └── api_gemini/  <-- Redundant prefix
        └── Cargo.toml
```

```toml
# In api/api_gemini/Cargo.toml
[package]
name = "api_gemini"
```

> ✅ **Good** (Prefix is removed from the directory name, but kept in `Cargo.toml`)

```text
// Filesystem structure
└── api/
    └── gemini/      <-- Clean, non-redundant
        └── Cargo.toml
```

```toml
# In api/gemini/Cargo.toml
[package]
name = "api_gemini" # The full name is preserved here
```

### Naming Conventions : Entity Naming Order (Noun-Verb)

For entities like functions, types, or variables that combine a noun (the subject) and a verb (the action), the noun must precede the verb. This is often referred to as 'subject-action' ordering.

> ✅ **Good**

```rust
fn files_delete() { /* ... */ }
fn user_create() { /* ... */ }
```

> ❌ **Bad**

```rust
fn delete_files() { /* ... */ }
fn create_user() { /* ... */ }
```

### Naming Conventions : Command Naming Conventions (CLI/REPL)

Names for commands, such as those used in a CLI or REPL interface, must adhere to three standards:
1. They must use `snake_case` and be lowercase.
2. They must follow the Noun-Verb entity naming order.
3. They must be prefixed with a dot (`.`), with dots also used as separators for sub-commands.

> ✅ **Good**

```
.files.delete
.users.create_new
```

> ❌ **Bad**

```
files.delete       // Missing dot prefix
.delete_files      // Incorrect verb-noun order
.files.DELETE      // Not lowercase
.files-delete      // Not snake_case
```

### Tooling & Error Handling : Exclusive Use of `error_tools` (When Used)

When using the `error_tools` crate for error handling, it must be used exclusively. The use of other error handling libraries, specifically `anyhow` and `thiserror`, is forbidden within the same project to ensure a single, consistent approach to error management across the codebase.

> ✅ **Good** (Using `error_tools`)

```rust
use error_tools::{ err, Result };

#[derive(Debug, error_tools::Error)]
pub enum MyError
{
  #[error("Failed to read data")]
  ReadError,
}

fn do_something() -> Result< () >
{
  // ...
  Err( err!( MyError::ReadError ) )
}
```

> ❌ **Bad** (Using `anyhow` or `thiserror`)

```rust
// Using anyhow is forbidden
use anyhow::Result;

fn do_anyhow_thing() -> Result< () >
{
  // ...
  Ok( () )
}

// Using thiserror is forbidden
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnotherError
{
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),
}
```

### Tooling & Error Handling : CLI and REPL Tooling: Mandate `unilang` over `clap`

**Description:** For building Command Line Interfaces (CLIs) or REPLs, the use of the `unilang` crate is **mandatory**. The use of the `clap` crate is **strictly forbidden**.

**Rationale:** Standardizing on a single tool for interface creation ensures consistency in command structure, parsing logic, and user experience across all tools in the workspace. `unilang` is chosen for its specific features that align with the project's architectural goals.

### Unilang Framework : Structuring `CommandDefinition`s

**Description:** When defining commands for `unilang`, the `CommandDefinition` struct must be initialized with explicit field names. Key fields like `name`, `namespace`, `description`, and `arguments` must be clearly defined. The formatting must follow the standard multi-line struct initialization rules.

**Rationale:** Enforces clarity and consistency, making command definitions easy to read, maintain, and audit.

> ✅ **Good**

```rust
use unilang::prelude::*;

let greet_cmd = CommandDefinition
{
  name : "greet".to_string(),
  namespace : String::new(),
  description : "A friendly greeting command".to_string(),
  hint : "Says hello to someone".to_string(),
  arguments : vec![ /* ... */ ],
  aliases : vec![ "hello".to_string() ],
  ..Default::default()
};
```

### Unilang Framework : Structuring `ArgumentDefinition`s

**Description:** Similar to `CommandDefinition`, `ArgumentDefinition` structs must be initialized with explicit field names. All relevant fields like `name`, `kind`, `description`, `attributes`, and `validation_rules` should be specified clearly.

**Rationale:** Ensures that command arguments are well-documented, their properties are explicit, and their validation rules are transparent.

> ✅ **Good**

```rust
use unilang::prelude::*;

let username_arg = ArgumentDefinition
{
  name : "username".to_string(),
  kind : Kind::String,
  description : "Username for the operation".to_string(),
  hint : "User identifier".to_string(),
  attributes : ArgumentAttributes::default(),
  validation_rules : vec!
  [
    ValidationRule::MinLength( 3 ),
    ValidationRule::Pattern( "^[a-zA-Z0-9_]+$".to_string() ),
  ],
  ..Default::default()
};
```

### Unilang Framework : Prefer the Pipeline API for Command Processing

**Description:** The high-level `Pipeline` API is the mandatory approach for processing commands in `unilang`. It encapsulates parsing, analysis, and execution, ensuring a consistent and robust workflow. The lower-level Component API or direct routine execution should only be used in exceptional cases where fine-grained control is explicitly required and justified.

**Rationale:** Promotes a simpler, more maintainable, and less error-prone implementation by using the intended high-level abstraction provided by the framework.

> ✅ **Good**

```rust
use unilang::prelude::*;

let registry = CommandRegistry::new();
let pipeline = Pipeline::new( registry );
let result = pipeline.process_command_simple( ".greet name::Alice" );
```

> ❌ **Bad**

```rust
// FORBIDDEN: Bypassing the Pipeline API for standard use cases.
use unilang::prelude::*;

let registry = CommandRegistry::new();
let instruction = Parser::new( Default::default() ).parse_single_instruction( ".greet name::Alice" )?;
let commands = SemanticAnalyzer::new( &[ instruction ], &registry ).analyze()?;
let interpreter = Interpreter::new( &commands, &registry );
interpreter.run( &mut ExecutionContext::default() )?;
```

### Unilang Framework : REPL Implementation Patterns

**Description:** When building a REPL (Read-Eval-Print Loop) with `unilang`, it must be designed to be **stateless**, with each command execution being independent. The implementation should leverage `unilang`'s features for interactive arguments (for sensitive data) and provide robust error recovery with contextual help.

**Rationale:** Ensures that REPL applications are secure, user-friendly, and stable, following the best practices demonstrated by the `unilang` framework. A stateless design prevents errors from one command from affecting subsequent commands.

> ✅ **Good** (Conceptual REPL loop)

```rust
use unilang::prelude::*;
use std::io;

fn repl_loop( pipeline : Pipeline ) -> !
{
  loop
  {
    // 1. Read input
    let mut input = String::new();
    io::stdin().read_line( &mut input ).unwrap();

    // 2. Process command statelessly
    let result = pipeline.process_command_simple( input.trim() );

    // 3. Handle result and provide contextual help on error
    if !result.success
    {
      if let Some( error ) = result.error
      {
        eprintln!( "Error: {}", error );
        if error.to_string().contains( "Command not found" )
        {
          // Provide suggestions or help
        }
      }
    }
  }
}
```

### Unilang Framework : Verbosity Control via Environment Variable

**Description:** Any CLI application built with `unilang` must respect the `UNILANG_VERBOSITY` environment variable to control output levels (0 for quiet, 1 for normal, 2 for debug). This functionality is built into the `Pipeline` API and should not be overridden.

**Rationale:** Provides a standard, user-friendly mechanism for debugging and controlling the application's output without requiring code changes or command-line flags.

> ✅ **Good** (Usage from the command line)

```sh
# Quiet mode - suppress all debug output
UNILANG_VERBOSITY=0 my_cli_app .command

# Normal mode (default) - standard output only
UNILANG_VERBOSITY=1 my_cli_app .command

# Debug mode - include parser traces
UNILANG_VERBOSITY=2 my_cli_app .command
```

### Secrets Management : Secret Storage and Naming

**Description:** All secret values (API keys, tokens, etc.) **must** be stored within a single dedicated directory named `secret`, located in the workspace root.
-   **Directory Structure:** All secrets must reside within the `secret/` directory.
-   **File Naming:** Each file containing secrets within this directory **must** have a name that starts with a hyphen (`-`), for example, `-openai.sh` or `-database.conf`.
-   **File Format:** For easy integration with shell scripts, it is **highly recommended** that secret files use a `key=value` format, making them sourceable by Bash.
-   The use of a single root-level secrets file (e.g., `.secrets.sh`) or other extensions like `.env` is forbidden.

**Rationale:** Grouping secrets within a dedicated `secret` directory allows for better organization (e.g., one file per service) while maintaining a single, centralized location. The hyphen-prefixed file naming convention is an additional safeguard, making the files less likely to be accidentally processed by standard tools. The `key=value` format provides a consistent and simple loading mechanism for all crates and tools in the workspace.

> ❌ **Bad** (Storing secrets in a single root file)

```text
.secrets.sh
```

> ❌ **Bad** (Using a different directory name or incorrect file naming)

```text
.secrets/
├── secrets.env
└── openai.conf
```

> ✅ **Good** (Correct directory and file naming convention)

```text
secret/
├── -openai.sh
└── -database.conf
```
**Example content for `secret/-openai.sh`:**
```sh
OPENAI_API_KEY="sk-..."
OPENAI_ORG_ID="org-..."
```

### Secrets Management : Ignoring Secrets with .gitignore

**Description:** The workspace root `.gitignore` file **must** contain patterns to ignore the `secret/` directory and prevent accidental commits of sensitive information. Specific patterns should be used to protect the secret directory.

**Rationale:** Explicitly ignoring the `secret/` directory in `.gitignore` provides a robust, secondary layer of protection against leaking secrets into the version control history.

> ✅ **Good** (in `.gitignore`)

```text
# Ignore secret directory and all its contents
secret/
```
