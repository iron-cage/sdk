# Documentation Directory

This directory contains comprehensive documentation for the Iron Cage project with emphasis on iron_runtime modules (Control Panel, Agent Runtime, and runtime services).

---

## 📍 Repository Context

You are viewing documentation from the **iron_runtime** repository. Iron Cage uses a two-repository architecture:
- **iron_runtime** (this repository) - Control Panel, Agent Runtime, runtime services (11 modules)
- **iron_cage** (separate repository) - OS sandboxing, CLI tools, foundation modules (10 modules)

The design documents in this directory describe the **complete Iron Cage platform** architecture across both repositories.

---

## Scope

**Responsibility:**
Houses comprehensive design documentation that explains architecture, deployment strategies, module distribution, and package dependencies for the Iron Cage platform (both iron_runtime and iron_cage repositories). Documents here are reference materials for understanding the system as a whole.

**In Scope:**
- Architecture and design documentation (two-repository model)
- Deployment and packaging strategies (5 deployment packages)
- Module-to-package mappings (21 modules across both repos)
- Cross-repository dependencies (crates.io + HTTP API)
- System-level design decisions and rationale

**Out of Scope:**
- Module-specific implementation details (see `module/*/spec.md` in respective repos)
- Module-specific API documentation (see `module/*/readme.md`)
- Product specifications and requirements (see `/spec/` directory)
- Business and market strategy (see `business/` directory in iron_cage)
- Development workflow and contribution guides (see root `readme.md`)

---

## Directory Responsibilities

Root-level reference documents in docs/:

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| **vocabulary.md** | Define canonical project terminology | Term question → Definition | Project-specific terms, glossary, terminology standards, naming conventions | NOT implementation details (→ module/*/spec.md), NOT architecture concepts (→ architecture/), NOT design decisions (→ decisions/) |
| **module_package_matrix.md** | Map modules to deployment packages | Module location question → Package assignment | 21 modules across 5 packages, module distribution analysis, shared modules, reuse patterns | NOT package definitions (→ deployment/package_model.md), NOT module specs (→ module/*/spec.md), NOT deployment procedures (→ deployment_guide.md) |
| **deployment_guide.md** | Document operational deployment procedures | Deployment question → Operational guide | Deployment procedures, configuration, environment setup, troubleshooting | NOT package model (→ deployment/package_model.md), NOT module mappings (→ module_package_matrix.md), NOT architecture (→ architecture/) |

---

## Design Collections

Conceptual documentation organized into focused collections (~30-50 lines per file).

| Collection | Files | Description |
|------------|-------|-------------|
| **[architecture/](architecture/)** | 7 | System architecture: execution models, layers, service boundaries |
| **[deployment/](deployment/)** | 5 | Packaging: 5 packages, actors, distribution, scaling |
| **[security/](security/)** | 5 | Security model: threats, isolation, credentials, audit |
| **[integration/](integration/)** | 5 | External systems: LLM providers, secrets, identity, observability |
| **[technology/](technology/)** | 5 | Tech choices: why Rust, PyO3, dependencies, infrastructure |
| **[decisions/](decisions/)** | 6 | Architecture Decision Records (ADRs) |
| **[capabilities/](capabilities/)** | 9 | Platform capabilities: safety, cost, reliability, etc. |

### Reference Documents

| Document | Description |
|----------|-------------|
| **[vocabulary.md](vocabulary.md)** | Canonical definitions for project terminology |
| **module_package_matrix.md** | Module-to-package mapping for all 21 modules |
| **deployment_guide.md** | Operational deployment procedures |

### Research

| Document | Description |
|----------|-------------|
| **[research/](research/)** | Time-stamped provider research and analysis |

### Specifications (see `/spec/`)

| Document | Description |
|----------|-------------|
| **[/spec/requirements.md](../spec/requirements.md)** | Technical requirements specification (FR-x.y identifiers) |

### Feature Documentation (`features/`)

| Document | Description |
|----------|-------------|
| **cli_architecture.md** | CLI tools architecture (wrapper pattern, iron_cli/iron_cli_py) |
| **token_management.md** | Token management feature overview |
| **token_management_api_reference.md** | Token API reference |
| **token_management_cli_api_parity.md** | CLI/API parity matrix |
| **token_management_implementation_plan.md** | Implementation roadmap |
| **token_management_validation_framework.md** | Validation framework |

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
-> Start with [`architecture/readme.md`](architecture/readme.md) for system concepts (execution models, layers, data flow)

**Security Model:**
-> See [`security/readme.md`](security/readme.md) for threat model, isolation layers, credential flow

**Deploying iron_runtime:**
-> See [`deployment/readme.md`](deployment/readme.md) for packages, distribution, scaling patterns

**Technology Decisions:**
-> See [`technology/readme.md`](technology/readme.md) for why Rust, PyO3, infrastructure choices

**Architecture Decision Records:**
-> See [`decisions/readme.md`](decisions/readme.md) for ADRs documenting key decisions

**Integration Patterns:**
-> See [`integration/readme.md`](integration/readme.md) for LLM providers, secrets, identity

**Capability Concepts:**
-> See [`capabilities/readme.md`](capabilities/readme.md) for the 8 platform capabilities

**CLI Tools Architecture:**
-> See `features/cli_architecture.md` for iron_cli vs iron_cli_py wrapper pattern

**Module Organization:**
-> See `module_package_matrix.md` to understand which modules belong where

**Terminology:**
-> See [`vocabulary.md`](vocabulary.md) for canonical definitions of project terms

---

## Document Maintenance

These design documents are synchronized between both repositories:
- **Source of Truth:** Documents exist in both iron_runtime and iron_cage
- **Updates:** When architecture changes, update documents in BOTH repositories
- **Version Control:** Documents include version numbers and update dates
- **Cross-References:** Documents reference each other; keep links valid

---

## Documentation Governance

All documentation directories with 3+ files maintain Responsibility Tables per organizational_principles.rulebook.md § Responsibility Table Format § Mandatory Locations.

**Compliance Status:** 19/19 directories (100%)

**Documentation Directories:**

| Directory | Files | Table Status | I/O Validation |
|-----------|-------|--------------|----------------|
| docs/ (root) | 3 | ✅ Compliant | ✅ Unique |
| docs/features/ | 6 | ✅ Compliant | ✅ Unique |
| docs/security/ | 4 | ✅ Compliant | ✅ Unique |
| docs/technology/ | 4 | ✅ Compliant | ✅ Unique |
| docs/integration/ | 4 | ✅ Compliant | ✅ Unique |
| docs/architecture/ | 6 | ✅ Compliant | ✅ Unique |
| docs/decisions/ | 5 | ✅ Compliant | ✅ Unique |
| docs/capabilities/ | 8 | ✅ Compliant | ✅ Unique |
| docs/deployment/ | 5 | ✅ Compliant | ✅ Unique |

**Test Directory Compliance:**

| Module | Test Files | Table Status |
|--------|-----------|--------------|
| module/iron_cli/tests/ | 6 | ✅ Compliant |
| module/iron_api/tests/ | 10 | ✅ Compliant |
| module/iron_token_manager/tests/ | 8 | ✅ Compliant |

**Module Documentation Compliance:**

| Module | Docs Files | Table Status |
|--------|-----------|--------------|
| module/iron_dashboard/docs/ | 5 | ✅ Compliant |

**Example Directory Compliance:**

| Directory | Example Files | Table Status |
|-----------|--------------|--------------|
| module/iron_sdk/examples/ | 6 subdirs | ✅ Compliant |
| module/iron_sdk/examples/patterns/ | 4 | ✅ Compliant |
| module/iron_sdk/examples/langchain/ | 5 | ✅ Compliant |
| module/iron_sdk/examples/crewai/ | 3 | ✅ Compliant |
| module/iron_sdk/examples/testing/ | 3 | ✅ Compliant |
| module/iron_sdk/examples/raw_api/ | 3 | ✅ Compliant |

**Pilot Directory Compliance:**

| Directory | Files | Table Status |
|-----------|-------|--------------|
| pilot/ | 4 | ✅ Compliant |
| pilot/execution/ | 3 | ✅ Compliant |

**Validation:**
- ✅ One-Second Test passed (no duplicate Input→Output signatures)
- ✅ All Out of Scope columns have ≥3 cross-references
- ✅ All cross-referenced files verified to exist

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

**Last Updated:** 2025-12-08
**Maintained By:** Iron Cage Team
**Status:** Active - Two Repository Architecture
