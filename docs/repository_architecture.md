# Repository Architecture

**Version:** 1.0.0
**Created:** 2025-12-08
**Status:** Active - Two Repository Architecture
**Repository:** iron_runtime (this document exists in both iron_runtime and iron_cage repositories)

---

> **📍 Note:** You are viewing this documentation from the **iron_runtime** repository. This document describes the complete Iron Cage architecture including both iron_runtime (Control Panel, Agent Runtime) and iron_cage (sandboxing, CLI tools, foundation modules). The same document exists in both repositories for reference.

---

## Scope

**Responsibility:**
Documents the Iron Cage repository architecture including the split from monorepo to two-repository model, rationale for the split, module distribution across repositories, and communication patterns between repositories.

**In Scope:**
- Repository split rationale and design decisions
- Module distribution (which modules go to which repo)
- Repository interaction patterns (crates.io, HTTP API)
- Workspace structure for each repository
- CI/CD implications
- Development workflow changes

**Out of Scope:**
- Deployment packages (see `deployment_packages.md`)
- Module-to-package mappings (see `module_package_matrix.md`)
- Package dependencies (see `package_dependencies.md`)
- Migration procedures (see `-migration_plan_by_crate.md`, `-migration_plan_elaborated.md`)
- Business strategy (see `business/` directory)

---

## Overview

Iron Cage transitioned from a **polyglot monorepo** to a **two-repository architecture**:

1. **iron_runtime** - Control Panel, Agent Runtime, and runtime services
2. **iron_cage** - Sandboxing, CLI tools, and supporting infrastructure

This split provides:
- **Clear separation of concerns:** Runtime vs OS-level sandboxing
- **Independent release cycles:** Control Panel and Agent Runtime can evolve separately from sandboxing
- **Improved build times:** Smaller workspaces compile faster
- **Better dependency management:** Shared foundation modules published to crates.io
- **Simplified onboarding:** New contributors focus on specific repository

---

## Architecture Decision

### Previous Architecture: Monorepo

```
iron_cage/dev/
├── module/
│   ├── iron_types          (Foundation)
│   ├── iron_cost           (Domain Logic)
│   ├── iron_telemetry      (Foundation)
│   ├── iron_state          (Infrastructure)
│   ├── iron_safety         (Domain Logic)
│   ├── iron_reliability    (Domain Logic)
│   ├── iron_lang           (Specialized)
│   ├── iron_secrets        (Domain Logic)
│   ├── iron_token_manager  (Feature)
│   ├── iron_runtime        (Integration)
│   ├── iron_api            (Integration)
│   ├── iron_sdk            (Python)
│   ├── iron_sandbox_core   (OS Sandboxing)
│   ├── iron_sandbox        (OS Sandboxing)
│   ├── iron_sandbox_py     (PyO3 + Python)
│   ├── iron_cli            (Application)
│   ├── iron_dashboard      (Vue/TypeScript)
│   └── iron_site           (Static)
└── Cargo.toml (workspace)
```

**Challenges:**
- Large workspace (22 modules) → Slow builds
- Mixed concerns (runtime + sandboxing) → Confusing boundaries
- Hard to publish subsets independently
- All-or-nothing releases

### New Architecture: Two Repositories

**Repository 1: iron_runtime** (New)
```
iron_runtime/
├── module/
│   ├── iron_state          (Infrastructure)
│   ├── iron_safety         (Domain Logic)
│   ├── iron_reliability    (Domain Logic)
│   ├── iron_lang           (Specialized)
│   ├── iron_secrets        (Domain Logic)
│   ├── iron_token_manager  (Feature)
│   ├── iron_runtime        (Integration)
│   └── iron_api            (Integration)
├── frontend/
│   └── dashboard/          (iron_dashboard moved here)
├── packages/
│   ├── iron_sdk/           (Python SDK)
│   └── iron_examples/      (Examples)
└── Cargo.toml (workspace)
```

**Focus:** Control Panel, Agent Runtime, runtime services

**Repository 2: iron_cage** (Existing, refocused)
```
iron_cage/dev/
├── module/
│   ├── iron_sandbox_core   (OS Sandboxing)
│   ├── iron_sandbox        (OS Sandboxing)
│   ├── iron_sandbox_py     (PyO3 + Python)
│   ├── iron_cli            (Application)
│   └── iron_site           (Static)
├── packages/
│   ├── iron_cli_py/        (Python CLI)
│   └── iron_testing/       (Test utilities)
└── Cargo.toml (workspace)
```

**Focus:** OS-level sandboxing, CLI tools, marketing site

**Shared Foundation** (published to crates.io):
- `iron_types` v0.1.0 (Core types)
- `iron_cost` v0.1.0 (Cost calculation)
- `iron_telemetry` v0.1.0 (Logging/tracing)

---

## Module Distribution

### Modules Moving to iron_runtime (9 Rust + 1 Vue + 2 Python)

**Rust Crates:**
1. **iron_state** - State management (SQLite/Redis)
2. **iron_safety** - Safety checks and guardrails
3. **iron_reliability** - Circuit breakers, retries
4. **iron_lang** - Language processing
5. **iron_secrets** - Secrets management
6. **iron_token_manager** - Token lifecycle
7. **iron_runtime** - Core runtime engine
8. **iron_api** - REST/GraphQL API

**Frontend:**
9. **iron_dashboard** - Vue/TypeScript Control Panel UI

**Python Packages:**
10. **iron_sdk** - Pythonic SDK with decorators
11. **iron_examples** - Usage examples (duplicated)

**Total:** 11 modules → iron_runtime

### Modules Staying in iron_cage (4 Rust + 1 Vue + 2 Python)

**Rust Crates:**
1. **iron_sandbox_core** - Landlock/seccomp primitives
2. **iron_sandbox** - High-level sandbox API
3. **iron_sandbox_py** - PyO3 Python bindings
4. **iron_cli** - Command-line tool (HTTP client to iron_api)

**Static Site:**
5. **iron_site** - Marketing website

**Python Packages:**
6. **iron_cli_py** - Python CLI wrapper
7. **iron_testing** - Test utilities

**Total:** 7 modules → iron_cage

### Modules Published to crates.io (3 Rust)

**Foundation Modules** (shared across both repositories):
1. **iron_types** → crates.io v0.1.0
2. **iron_cost** → crates.io v0.1.0
3. **iron_telemetry** → crates.io v0.1.0

**Why Published:**
- Used by both iron_runtime and iron_cage
- Avoids circular dependencies between repositories
- Enables external tools to use core types
- Standard Rust ecosystem pattern

---

## Repository Interaction Patterns

### Pattern 1: Shared Crates via crates.io

```
iron_runtime/Cargo.toml:
[dependencies]
iron_types = "0.1.0"        # From crates.io
iron_cost = "0.1.0"         # From crates.io
iron_telemetry = "0.1.0"    # From crates.io

iron_cage/dev/Cargo.toml:
[dependencies]
iron_types = "0.1.0"        # Same version from crates.io
iron_telemetry = "0.1.0"    # Same version
```

**Benefits:**
- Single source of truth for shared types
- Semantic versioning for compatibility
- No path dependencies between repos
- Easy external consumption

### Pattern 2: HTTP API Communication

```
iron_cage (iron_cli) → HTTPS → iron_runtime (iron_api)

Example:
iron-cli token create \
  --api-url https://localhost:8080 \
  --project my-app
```

**Flow:**
1. iron_cli makes HTTPS POST to `/api/tokens`
2. iron_api (in iron_runtime) handles request
3. Returns JWT token
4. iron_cli saves token locally

**Benefits:**
- Language-agnostic API (not Rust-specific)
- Network boundary enforces clean separation
- Versioned API (v1, v2, etc.)
- Can be called from any language/tool

### Pattern 3: PyPI Package Distribution

```
iron_runtime → PyPI as "iron-cage-runtime"
iron_cage → PyPI as "iron-sandbox"

User installation:
pip install iron-cage-runtime  # Agent runtime features
pip install iron-sandbox        # Optional sandboxing
```

**Benefits:**
- Independent versioning
- Users choose features needed
- Smaller install footprint
- Clear separation of concerns

---

## Workspace Structure

### iron_runtime Workspace

```toml
# iron_runtime/Cargo.toml
[workspace]
resolver = "2"
members = [
  "module/iron_state",
  "module/iron_safety",
  "module/iron_reliability",
  "module/iron_lang",
  "module/iron_secrets",
  "module/iron_token_manager",
  "module/iron_runtime",
  "module/iron_api",
]

[workspace.dependencies]
# From crates.io
iron_types = "0.1.0"
iron_cost = "0.1.0"
iron_telemetry = "0.1.0"

# Local workspace crates
iron_state = { path = "module/iron_state" }
iron_safety = { path = "module/iron_safety" }
# ... etc
```

### iron_cage Workspace

```toml
# iron_cage/dev/Cargo.toml
[workspace]
resolver = "2"
members = [
  "module/iron_sandbox_core",
  "module/iron_sandbox",
  "module/iron_sandbox_py",
  "module/iron_cli",
]

[workspace.dependencies]
# From crates.io
iron_types = "0.1.0"
iron_telemetry = "0.1.0"

# Local workspace crates
iron_sandbox_core = { path = "module/iron_sandbox_core" }
iron_sandbox = { path = "module/iron_sandbox" }
# ... etc
```

---

## CI/CD Implications

### iron_runtime CI

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: cargo nextest run --workspace --all-features
      - name: Build dashboard
        run: cd frontend/dashboard && npm run build
```

**Characteristics:**
- Tests 8 Rust crates + 1 Vue app
- Faster than monorepo (fewer modules)
- Can deploy Control Panel independently

### iron_cage CI

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Redis
        run: sudo systemctl start redis
      - name: Run tests
        run: cargo nextest run --workspace --all-features
```

**Characteristics:**
- Tests 4 Rust crates
- Requires Linux (Landlock/seccomp)
- Independent release cycle from runtime

---

## Development Workflow Changes

### Before: Monorepo Workflow

```bash
# Clone single repo
git clone iron_cage
cd iron_cage/dev

# Run all tests (slow - 22 modules)
cargo nextest run --workspace

# Changes to iron_sandbox trigger runtime tests
# Changes to iron_runtime trigger sandbox tests
```

**Issues:**
- Large test suite (all modules tested together)
- Unclear module boundaries
- All contributors need full context

### After: Two-Repo Workflow

**For Runtime Development:**
```bash
# Clone runtime repo
git clone iron_runtime
cd iron_runtime

# Run runtime tests (fast - 8 modules)
cargo nextest run --workspace

# Changes only affect runtime modules
```

**For Sandbox Development:**
```bash
# Clone cage repo
git clone iron_cage
cd iron_cage/dev

# Run sandbox tests (fast - 4 modules)
cargo nextest run --workspace

# Changes only affect sandbox modules
```

**For Cross-Repo Features:**
```bash
# Example: Adding new shared type
cd iron_types
# Update type
cargo publish  # Publish v0.2.0

# Update iron_runtime
cd iron_runtime
# Update Cargo.toml: iron_types = "0.2.0"
cargo update -p iron_types

# Update iron_cage
cd iron_cage/dev
# Update Cargo.toml: iron_types = "0.2.0"
cargo update -p iron_types
```

**Benefits:**
- Faster iteration (smaller workspaces)
- Clear ownership (runtime team vs sandbox team)
- Independent releases
- Explicit cross-repo dependencies

---

## Dependency Flow

```
┌─────────────────────────────────────────────────────────────┐
│                         crates.io                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ iron_types   │  │ iron_cost    │  │ iron_telemetry│     │
│  │  v0.1.0      │  │  v0.1.0      │  │  v0.1.0      │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
└─────────┼──────────────────┼──────────────────┼─────────────┘
          │                  │                  │
          │                  │                  │
    ┌─────▼──────────────────▼──────────────────▼──────┐
    │              iron_runtime Repository             │
    │                                                   │
    │  ┌─────────────┐         ┌──────────────┐        │
    │  │ iron_state  │─────────│ iron_runtime │        │
    │  └─────────────┘         └───────┬──────┘        │
    │         │                        │               │
    │  ┌──────▼───────┐        ┌───────▼──────┐        │
    │  │ iron_secrets │        │  iron_api    │        │
    │  └──────────────┘        └───────┬──────┘        │
    │                                  │               │
    │                        HTTPS API │               │
    └──────────────────────────────────┼───────────────┘
                                       │
                                       │ HTTP
                                       │
                    ┌──────────────────▼─────────────────┐
                    │     iron_cage Repository           │
                    │                                    │
                    │  ┌────────────────┐                │
                    │  │   iron_cli     │───► HTTP Client│
                    │  └────────────────┘                │
                    │                                    │
                    │  ┌────────────────┐                │
                    │  │ iron_sandbox   │                │
                    │  └────────────────┘                │
                    │         │                          │
                    │  ┌──────▼──────────┐               │
                    │  │iron_sandbox_core│               │
                    │  └─────────────────┘               │
                    └────────────────────────────────────┘
```

**Key Points:**
- Foundation modules (top) published to crates.io
- Both repos depend on same versions
- iron_cage → iron_runtime communication via HTTP only
- No Rust-level dependencies between repos

---

## Versioning Strategy

### Foundation Modules (crates.io)

**Semantic Versioning:**
- MAJOR: Breaking changes to types (0.1.0 → 0.2.0 or 1.0.0)
- MINOR: New fields/types, backward compatible (0.1.0 → 0.1.1)
- PATCH: Bug fixes only (0.1.0 → 0.1.0.1)

**Compatibility Rules:**
- iron_runtime and iron_cage MUST use same MAJOR version
- Can use different MINOR versions (backward compatible)
- Coordinated updates for MAJOR bumps

**Example:**
```
iron_runtime: iron_types = "0.1.0"
iron_cage:    iron_types = "0.1.1"  # OK - backward compatible

iron_runtime: iron_types = "0.2.0"
iron_cage:    iron_types = "0.1.1"  # ERROR - incompatible
```

### Repository Versions

**iron_runtime:** Independent versioning
- v1.0.0 - Initial release
- v1.1.0 - New Control Panel features
- v2.0.0 - Breaking API changes

**iron_cage:** Independent versioning
- v1.0.0 - Initial release
- v1.1.0 - New sandboxing features
- v2.0.0 - Breaking sandbox API

**No coupling required** - repos evolve independently

---

## Migration Path

**Current State:** Monorepo (iron_cage/dev)
**Target State:** Two repos (iron_runtime + iron_cage)

**Migration Steps:**
1. ✅ Update specifications (this document)
2. ⏳ Publish foundation modules to crates.io
3. ⏳ Create iron_runtime repository
4. ⏳ Copy runtime modules to iron_runtime
5. ⏳ Update dependencies (workspace → crates.io)
6. ⏳ Remove moved modules from iron_cage
7. ⏳ Update CI/CD for both repos
8. ⏳ Verify builds and tests

**See:** `-migration_plan_by_crate.md` for detailed migration procedure

---

## Benefits of Two-Repository Architecture

### 1. Faster Build Times

**Before (monorepo):**
```
cargo build --workspace
# Compiles 22 modules: ~10-15 minutes clean build
```

**After (two repos):**
```
cd iron_runtime && cargo build --workspace
# Compiles 8 modules: ~5-7 minutes

cd iron_cage/dev && cargo build --workspace
# Compiles 4 modules: ~2-3 minutes
```

### 2. Clear Ownership

**iron_runtime Team:**
- Control Panel development
- Agent runtime features
- API design
- Business logic

**iron_cage Team:**
- OS-level security
- Kernel integration (Landlock/seccomp)
- Sandboxing features
- CLI tools

### 3. Independent Releases

**Example Timeline:**
- Jan 2026: iron_runtime v1.1.0 - New Control Panel features
- Feb 2026: iron_cage v1.0.1 - Sandbox bug fix
- Mar 2026: iron_runtime v1.2.0 - API enhancements
- Apr 2026: iron_cage v1.1.0 - New Landlock features

**No coordination needed** unless shared types change

### 4. Simplified Onboarding

**New Runtime Developer:**
```bash
git clone iron_runtime
cd iron_runtime
cargo build --workspace
# Only needs to understand 8 modules
```

**New Sandbox Developer:**
```bash
git clone iron_cage
cd iron_cage/dev
cargo build --workspace
# Only needs to understand 4 modules + Linux kernel basics
```

### 5. Better Dependency Management

**Before:** Shared modules via workspace path dependencies
**After:** Shared modules via crates.io with semantic versioning

**Enables:**
- External tools to use iron_types
- Version pinning for stability
- Clear API compatibility contracts

---

## Trade-offs and Challenges

### Benefits ✅

- ✅ Faster builds (smaller workspaces)
- ✅ Clear separation of concerns
- ✅ Independent release cycles
- ✅ Easier onboarding (focused repos)
- ✅ Standard Rust ecosystem patterns
- ✅ Better CI/CD parallelization

### Challenges ⚠️

- ⚠️ Cross-repo changes require coordination
- ⚠️ Foundation module updates affect both repos
- ⚠️ Two sets of CI/CD workflows to maintain
- ⚠️ More complex initial setup (two clones)
- ⚠️ Version compatibility must be managed

### Mitigation Strategies

**For Cross-Repo Changes:**
- Document version compatibility in release notes
- Use feature flags for gradual rollouts
- Maintain changelog for foundation modules

**For Foundation Updates:**
- Deprecation period (2-4 weeks minimum)
- Clear migration guides
- Automated compatibility tests

**For CI/CD:**
- Shared GitHub Actions workflows (reusable)
- Matrix testing for version combinations
- Automated dependency updates (Dependabot)

---

## Related Documentation

**Migration:**
- `-migration_plan_by_crate.md` - Detailed per-module migration steps
- `-migration_plan_elaborated.md` - Phase-by-phase migration plan
- `-migration_missing_modules.md` - Analysis of modules not in original plan

**Architecture:**
- `deployment_packages.md` - Package definitions and deployment
- `module_package_matrix.md` - Module-to-package mappings
- `package_dependencies.md` - Package dependency analysis

**Specifications:**
- `module/*/spec.md` - Individual module specifications
- `business/spec/` - Product specifications

---

## Revision History

| Version | Date       | Changes                                    |
|---------|------------|--------------------------------------------|
| 1.0.0   | 2025-12-08 | Initial repository architecture documentation. Documents transition from polyglot monorepo to two-repository model, module distribution, interaction patterns, and development workflow changes. |

---

**Status:** ✅ Active
**Next Review:** After migration completion or when adding new cross-repo features
