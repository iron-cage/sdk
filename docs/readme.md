# Documentation Directory

This directory contains comprehensive documentation for the Iron Cage project (Control Panel, Agent Runtime, and runtime services).

---

## Scope

**Responsibility:**
Houses comprehensive design documentation that explains architecture, deployment strategies, module distribution, and package dependencies for the Iron Cage platform. Documents here are reference materials for understanding the system as a whole.

**In Scope:**
- Architecture and design documentation
- Deployment and packaging strategies (6 deployment packages)
- Module-to-package mappings
- System-level design decisions and rationale
- Integration patterns with external systems

**Out of Scope:**
- Module-specific implementation details (see `module/*/spec.md`)
- Module-specific API documentation (see `module/*/readme.md`)
- Product specifications and requirements (see `/spec/` directory)
- Development workflow and contribution guides (see root `readme.md`)

---

## Directory Responsibilities

Root-level reference documents in docs/:

| Entity | Responsibility |
|--------|----------------|
| **vocabulary.md** | Define canonical project terminology (glossary, naming conventions, project-specific terms) |
| **deployment_guide.md** | Document operational deployment procedures (configuration, environment setup, troubleshooting) |
| **post_pilot_abilities.md** | Complete feature catalog with unique codes (F-XYY format) across Pilot and POST-PILOT phases, grouped by actor |
| **protocol_maturity_matrix.md** | Track protocol implementation maturity across all aspects (spec, endpoints, validation, tests, security, docs, corner cases) |

---

## Documentation Collections

All documentation organized as numbered Design Collections (NNN_ format) per documentation.rulebook.md standards.

| Collection | Instances | Description |
|------------|-----------|-------------|
| **[principles/](principles/readme.md)** | 5 (001-005) | Design principles (philosophy, quality, error handling, testing, workflow) |
| **[constraints/](constraints/readme.md)** | 4 (001-004) | System constraints (technical, business, scope, trade-offs) |
| **[capabilities/](capabilities/readme.md)** | 8 (001-008) | Platform capabilities (runtime, LLM control, sandbox, safety, credentials, MCP, observability, data) |
| **[architecture/](architecture/readme.md)** | 8 (001-008) | System architecture concepts (execution models, layers, service boundaries, data flow, integration, roles, entity model, runtime modes) |
| **[deployment/](deployment/readme.md)** | 5 (001-005) | Deployment concepts (package model, actors, distribution, scaling, module mapping) |
| **[security/](security/readme.md)** | 4 (001-004) | Security model concepts (threat model, isolation, credential flow, audit) |
| **[integration/](integration/readme.md)** | 4 (001-004) | External system integration patterns (LLM providers, secrets, identity, observability) |
| **[technology/](technology/readme.md)** | 4 (001-004) | Technology choices (Rust, PyO3, infrastructure, dependencies) |
| **[protocol/](protocol/readme.md)** | 8 (002-008) | Communication protocols (REST API, WebSocket, MCP, budget control, token management, authentication, user management) |
| **[features/](features/readme.md)** | 6 (001-006) | Feature documentation (CLI architecture, token management, user management) |
| **[decisions/](decisions/readme.md)** | 6 (adr_002-007) | Architecture Decision Records (ADR format) |

### Reference Documents

| Document | Description |
|----------|-------------|
| **[vocabulary.md](vocabulary.md)** | Canonical definitions for project terminology |
| **module_package_matrix.md** | Module-to-package mapping for all 20 modules |
| **deployment_guide.md** | Operational deployment procedures |

### Research

| Document | Description |
|----------|-------------|
| **[research/](research/readme.md)** | Time-stamped provider research and analysis |

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
| **user_management.md** | User management feature (lifecycle, RBAC, audit logging, admin operations) |

---

## Key Insights

### Modules (17 total)

**Foundation (3 modules):**
- iron_types - Core types, errors, Result types
- iron_cost - Cost calculation and budget types
- iron_telemetry - Unified logging with tracing

**Runtime & API (5 modules):**
- iron_control_api - REST API + WebSocket server
- iron_runtime - Agent orchestrator + PyO3 bridge
- iron_runtime_state - Multi-backend state management
- iron_token_manager - JWT token management backend
- iron_secrets - Encrypted secrets management

**Safety & Reliability (2 modules):**
- iron_safety - PII detection and redaction
- iron_reliability - Circuit breaker patterns

**CLI & Tools (2 modules):**
- iron_cli - Command-line interface (Rust, authoritative)
- iron_cli_py - Python CLI with wrapper pattern

**Frontend & SDK (2 modules):**
- iron_dashboard - Web dashboard (Vue 3 + TypeScript)
- iron_sdk - Pythonic SDK with decorators (Python, includes examples/)

### Deployment Packages

**Primary deployment packages:**
1. **Control Panel** (Package 1) - Docker container with iron_control_api + iron_dashboard
2. **Agent Runtime** (Package 3) - PyPI package (iron-sdk - user installs); automatically includes iron-cage (PyPI wheel with Rust runtime, internal dependency)
3. **CLI Tools** (Package 5) - Binary + PyPI wheel (iron-cli + iron-cli-py)

---

## Quick Reference

### Where to Find Information

**Understanding the Architecture:**
-> Start with [`architecture/readme.md`](architecture/readme.md) for system concepts (execution models, layers, data flow)

**Security Model:**
-> See [`security/readme.md`](security/readme.md) for threat model, isolation layers, credential flow

**Deployment:**
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
-> See `features/001_cli_architecture.md` for iron_cli vs iron_cli_py wrapper pattern

**Module Organization:**
-> See `module_package_matrix.md` to understand which modules belong where

**Terminology:**
-> See [`vocabulary.md`](vocabulary.md) for canonical definitions of project terms

---

## Documentation Governance

All documentation directories with 3+ files maintain Responsibility Tables per organizational_principles.rulebook.md § Responsibility Table Format § Mandatory Locations.

**Compliance Status:** 22/22 directories (100%)

**Documentation Directories:**

| Directory | Files | Table Status | I/O Validation |
|-----------|-------|--------------|----------------|
| docs/ (root) | 3 | ✅ Compliant | ✅ Unique |
| docs/principles/ | 5 | ✅ Compliant | ✅ Unique |
| docs/constraints/ | 4 | ✅ Compliant | ✅ Unique |
| docs/features/ | 6 | ✅ Compliant | ✅ Unique |
| docs/security/ | 4 | ✅ Compliant | ✅ Unique |
| docs/technology/ | 4 | ✅ Compliant | ✅ Unique |
| docs/integration/ | 4 | ✅ Compliant | ✅ Unique |
| docs/architecture/ | 6 | ✅ Compliant | ✅ Unique |
| docs/protocol/ | 8 | ✅ Compliant | ✅ Unique |
| docs/decisions/ | 7 | ✅ Compliant | ✅ Unique |
| docs/capabilities/ | 8 | ✅ Compliant | ✅ Unique |
| docs/deployment/ | 5 | ✅ Compliant | ✅ Unique |

**Test Directory Compliance:**

| Module | Test Files | Table Status |
|--------|-----------|--------------|
| module/iron_cli/tests/ | 6 | ✅ Compliant |
| module/iron_control_api/tests/ | 10 | ✅ Compliant |
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
- ✅ All Responsibility Tables use correct format (3 columns: ID, Entity, Responsibility)
- ✅ No duplicate file responsibilities within same directory
- ✅ All cross-referenced files verified to exist

---

## Related Documentation

- `/readme.md` - Repository overview, quick start, building instructions
- `/module/*/spec.md` - Module specifications (iron_control_api, iron_runtime, iron_cli, etc.)
- `/module/*/readme.md` - Module API documentation and usage guides
- `/pilot/spec.md` - Pilot platform specification

---

**Last Updated:** 2025-12-09
**Maintained By:** Iron Cage Team
**Status:** Active
