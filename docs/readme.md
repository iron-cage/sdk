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

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| **vocabulary.md** | Define canonical project terminology | Term question → Definition | Project-specific terms, glossary, terminology standards, naming conventions | NOT implementation details (→ module/*/spec.md), NOT architecture concepts (→ architecture/), NOT design decisions (→ decisions/) |
| **module_package_matrix.md** | Map modules to deployment packages | Module location question → Package assignment | 20 modules across 5 packages, module distribution analysis, shared modules, reuse patterns | NOT package definitions (→ deployment/001_package_model.md), NOT module specs (→ module/*/spec.md), NOT deployment procedures (→ deployment_guide.md) |
| **deployment_guide.md** | Document operational deployment procedures | Deployment question → Operational guide | Deployment procedures, configuration, environment setup, troubleshooting | NOT package model (→ deployment/001_package_model.md), NOT module mappings (→ module_package_matrix.md), NOT architecture (→ architecture/) |

---

## Documentation Collections

All documentation organized as numbered Design Collections (NNN_ format) per documentation.rulebook.md standards.

| Collection | Instances | Description |
|------------|-----------|-------------|
| **[capabilities/](capabilities/)** | 8 (001-008) | Platform capabilities (runtime, LLM control, sandbox, safety, credentials, MCP, observability, data) |
| **[integration/](integration/)** | 4 (001-004) | External system integration patterns (LLM providers, secrets, identity, observability) |
| **[architecture/](architecture/)** | 6 (001-006) | System architecture concepts (execution models, layers, service boundaries, data flow, integration, budget control) |
| **[deployment/](deployment/)** | 5 (001-005) | Deployment concepts (package model, actors, distribution, scaling, module mapping) |
| **[security/](security/)** | 4 (001-004) | Security model concepts (threat model, isolation, credential flow, audit) |
| **[technology/](technology/)** | 4 (001-004) | Technology choices (Rust, PyO3, infrastructure, dependencies) |
| **[features/](features/)** | 6 (001-006) | Feature documentation (CLI architecture, token management suite) |
| **[decisions/](decisions/)** | 7 (adr_001-007) | Architecture Decision Records (ADR format) |

### Reference Documents

| Document | Description |
|----------|-------------|
| **[vocabulary.md](vocabulary.md)** | Canonical definitions for project terminology |
| **module_package_matrix.md** | Module-to-package mapping for all 20 modules |
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

## Key Insights

### Modules (17 total)

**Foundation (3 modules):**
- iron_types - Core types, errors, Result types
- iron_cost - Cost calculation and budget types
- iron_telemetry - Unified logging with tracing

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

**CLI & Tools (2 modules):**
- iron_cli - Command-line interface (Rust, authoritative)
- iron_cli_py - Python CLI with wrapper pattern

**Frontend & SDK (4 modules):**
- iron_dashboard - Web dashboard (Vue 3 + TypeScript)
- iron_sdk - Pythonic SDK with decorators (Python, includes examples/)
- iron_examples - Moved to iron_sdk/examples/ (archived)
- iron_testing - Testing utilities and fixtures

### Deployment Packages

**Primary deployment packages:**
1. **Control Panel** (Package 1) - Docker container with iron_api + iron_dashboard
2. **Agent Runtime** (Package 3) - PyPI wheel (iron-cage-runtime) with Python SDK
3. **CLI Tool** (Package 5) - Binary distribution (iron-token)
4. **Python CLI** (Package 6) - PyPI wheel (iron-cli-py)

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

**Compliance Status:** 19/19 directories (100%)

**Documentation Directories:**

| Directory | Files | Table Status | I/O Validation |
|-----------|-------|--------------|----------------|
| docs/ (root) | 3 | ✅ Compliant | ✅ Unique |
| docs/features/ | 6 | ✅ Compliant | ✅ Unique |
| docs/security/ | 4 | ✅ Compliant | ✅ Unique |
| docs/technology/ | 4 | ✅ Compliant | ✅ Unique |
| docs/integration/ | 4 | ✅ Compliant | ✅ Unique |
| docs/architecture/ | 5 | ✅ Compliant | ✅ Unique |
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

- `/readme.md` - Repository overview, quick start, building instructions
- `/module/*/spec.md` - Module specifications (iron_api, iron_runtime, iron_cli, etc.)
- `/module/*/readme.md` - Module API documentation and usage guides
- `/pilot/spec.md` - Pilot platform specification

---

**Last Updated:** 2025-12-09
**Maintained By:** Iron Cage Team
**Status:** Active
