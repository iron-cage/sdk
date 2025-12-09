# Documentation Directory

This directory contains comprehensive documentation for the Iron Cage project with emphasis on iron_runtime modules (Control Panel, Agent Runtime, and runtime services).

---

## 📍 Repository Context

You are viewing documentation from the **iron_runtime** repository. Iron Cage uses a two-repository architecture:
- **iron_runtime** (this repository) - Control Panel, Agent Runtime, runtime services (12 modules)
- **iron_cage** (separate repository) - OS sandboxing, CLI tools, foundation modules (10 modules)

The design documents in this directory describe the **complete Iron Cage platform** architecture across both repositories.

---

## Scope

**Responsibility:**
Houses comprehensive design documentation that explains architecture, deployment strategies, module distribution, and package dependencies for the Iron Cage platform (both iron_runtime and iron_cage repositories). Documents here are reference materials for understanding the system as a whole.

**In Scope:**
- Architecture and design documentation (two-repository model)
- Deployment and packaging strategies (6 deployment packages)
- Module-to-package mappings (22 modules across both repos)
- Cross-repository dependencies (crates.io + HTTP API)
- System-level design decisions and rationale

**Out of Scope:**
- Module-specific implementation details (see `module/*/spec.md` in respective repos)
- Module-specific API documentation (see `module/*/readme.md`)
- Business and market strategy (see `business/` directory in iron_cage)
- Development workflow and contribution guides (see root `readme.md`)

---

## Design Documents

### Core Architecture

| Document | Description | Key Sections |
|----------|-------------|--------------|
| **repository_architecture.md** | Two-repository split design, module distribution, interaction patterns | Previous monorepo structure, new architecture, module distribution (12 to runtime, 10 to cage), repository interaction (crates.io + HTTP), versioning strategy, CI/CD implications, development workflow changes |
| **deployment_packages.md** | 6 independent deployment packages across both repositories | Package definitions (Control Panel, Marketing Site, Agent Runtime, Sandbox, CLI Tool, Python CLI), deployment methods, actor inventory, architecture diagrams, deployment scenarios, build/release process |
| **module_package_matrix.md** | Module-to-package mapping for all 22 modules | 22×6 matrix showing which modules go into which packages, repository distribution, shared module analysis, foundation module distribution, package composition statistics |
| **package_dependencies.md** | Runtime dependencies between packages and across repositories | 6×6 dependency matrix, cross-repository dependencies (crates.io for Rust, HTTP API for telemetry), runtime requirements (databases, APIs), deployment independence analysis |

---

## Key Insights for iron_runtime Developers

### Modules in This Repository (12 total)

**Runtime & API (5 modules):**
- iron_api - REST API + WebSocket server
- iron_runtime - Agent orchestrator + PyO3 bridge
- iron_state - Multi-backend state management
- iron_token_manager - JWT token management backend
- iron_secrets - Encrypted secrets management

**Safety & Reliability (3 modules):**
- iron_safety - PII detection and redaction
- iron_reliability - Circuit breaker patterns
- iron_lang - AI agent data protocol

**Frontend & SDK (4 modules):**
- iron_dashboard - Web dashboard (Vue 3 + TypeScript)
- iron_sdk - Pythonic SDK with decorators (Python)
- iron_examples - Example library for LangChain, CrewAI (Python)
- (iron_control_store - PostgreSQL schema for production, spec-only)

### Dependencies on iron_cage

Iron_runtime depends on **3 foundation modules** published to crates.io from iron_cage:
1. **iron_types** - Core types, errors, Result types
2. **iron_cost** - Cost calculation and budget types
3. **iron_telemetry** - Unified logging with tracing

These are consumed via crates.io (not path dependencies).

### Deployment Packages

Iron_runtime produces **2 primary deployment packages:**
1. **Control Panel** (Package 1) - Docker container with iron_api + iron_dashboard
2. **Agent Runtime** (Package 3) - PyPI wheel (iron-cage-runtime) with Python SDK

---

## Quick Reference

### Where to Find Information

**Understanding the Architecture:**
→ Start with `repository_architecture.md` to understand why two repositories exist

**Deploying iron_runtime:**
→ See `deployment_packages.md` § Package 1 (Control Panel) and § Package 3 (Agent Runtime)

**Module Organization:**
→ See `module_package_matrix.md` to understand which modules belong where

**Cross-Repository Dependencies:**
→ See `package_dependencies.md` to understand how iron_runtime depends on iron_cage

**Module-Specific Details:**
→ See `module/*/spec.md` and `module/*/readme.md` in this repository

---

## Document Maintenance

These design documents are synchronized between both repositories:
- **Source of Truth:** Documents exist in both iron_runtime and iron_cage
- **Updates:** When architecture changes, update documents in BOTH repositories
- **Version Control:** Documents include version numbers and update dates
- **Cross-References:** Documents reference each other; keep links valid

---

## Related Documentation

**In iron_runtime Repository:**
- `/readme.md` - Repository overview, quick start, building instructions
- `/module/*/spec.md` - Module specifications (iron_api, iron_runtime, etc.)
- `/module/*/readme.md` - Module API documentation and usage guides

**In iron_cage Repository:**
- `/readme.md` - Sandboxing, CLI tools, foundation modules overview
- `/docs/repository_architecture.md` - Same content, iron_cage perspective
- `/module/*/spec.md` - Sandbox and foundation module specifications

---

## Responsibility Table

| Entity | Responsibility | Input→Output | Scope | Out of Scope |
|--------|----------------|--------------|-------|--------------|
| `readme.md` | Documentation index and navigation | - → Directory overview | Structure, navigation, repository context | (1) Module-specific details (see module/*/readme.md), (2) Implementation code (see module/*/src/), (3) API usage examples (see api_example.md) |
| `api_example.md` | API endpoint reference | - → Endpoint documentation | All REST endpoints, auth, errors | (1) Implementation details (see iron_api module), (2) Database schema (see iron_state module), (3) Python client API (see python_lib.md) |
| `python_lib.md` | Proposed Python API design | - → Future API spec | SafetyRuntime design (PROPOSED) | (1) Current implementation (see python_lib_current.md), (2) Server-side API (see iron_api module), (3) Database layer (see iron_state module) |
| `python_lib_current.md` | Current Python API reference | - → API documentation | iron_runtime.Runtime methods | (1) Future API design (see python_lib.md), (2) Server implementation (see iron_api module), (3) Safety rules (see iron_safety module) |
| `user_workflow.md` | Architecture and data flow | - → System overview | Client/server interaction, components | (1) API endpoint details (see api_example.md), (2) Python API specifics (see python_lib.md), (3) Module implementation (see module/) |
| `repository_architecture.md` | Two-repository design | - → Architecture docs | Module distribution, versioning | (1) Implementation details (see module/*), (2) Build configuration (see Cargo.toml), (3) Python SDK details (see python_lib.md) |
| `deployment_packages.md` | Package definitions | - → Deployment guide | 6 packages, deployment methods | (1) Build scripts (see package.json, Cargo.toml), (2) Module implementation (see module/), (3) API endpoint details (see api_example.md) |
| `module_package_matrix.md` | Module-to-package mapping | - → 22×6 matrix | Which modules go where | (1) Module internals (see module/*/spec.md), (2) Build process (see Cargo.toml), (3) Deployment procedures (see deployment_packages.md) |
| `package_dependencies.md` | Cross-package dependencies | - → Dependency matrix | Runtime requirements | (1) Code dependencies (see Cargo.toml), (2) Module implementation (see module/), (3) API endpoints (see api_example.md) |

---

**Last Updated:** 2025-12-08
**Maintained By:** Iron Cage Team
**Status:** Active - Two Repository Architecture
