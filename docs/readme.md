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

| Document | Description |
|----------|-------------|
| **repository_architecture.md** | Two-repository split design, module distribution |
| **deployment_packages.md** | 6 deployment packages across both repositories |
| **module_package_matrix.md** | Module-to-package mapping for all 22 modules |
| **package_dependencies.md** | Runtime dependencies between packages |

### Technical Documentation

| Document | Description |
|----------|-------------|
| **technical_architecture.md** | Complete system architecture and design |
| **architecture.md** | High-level architecture overview |
| **requirements.md** | Technical requirements specification |
| **capabilities.md** | Platform capabilities overview |
| **glossary.md** | Terminology and definitions |

### Developer Guides

| Document | Description |
|----------|-------------|
| **developer_guide.md** | Developer onboarding and workflows |
| **deployment_guide.md** | Deployment procedures and environments |

### Feature Documentation (`features/`)

| Document | Description |
|----------|-------------|
| **token_management.md** | Token management feature overview |
| **token_management_api_reference.md** | Token API reference |
| **token_management_cli_api_parity.md** | CLI/API parity matrix |
| **token_management_implementation_plan.md** | Implementation roadmap |
| **token_management_validation_framework.md** | Validation framework |

### Capability Specifications (`spec/`)

| Document | Description |
|----------|-------------|
| **capability_1_enterprise_data_access.md** | Enterprise data access spec |
| **capability_2_ai_safety_guardrails.md** | AI safety guardrails spec |
| **capability_3_llm_access_control.md** | LLM access control spec |
| **capability_4_safe_execution.md** | Safe execution spec |
| **capability_5_credential_management.md** | Credential management spec |
| **capability_6_observability.md** | Observability spec |
| **capability_7_mcp_integration.md** | MCP integration spec |
| **capability_8_agent_runtime.md** | Agent runtime spec |
| **pilot_platform.md** | Pilot platform specification |

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

**Last Updated:** 2025-12-08
**Maintained By:** Iron Cage Team
**Status:** Active - Two Repository Architecture
