# Module-to-Package Mapping Matrix

**Version:** 3.0.0
**Date:** 2025-12-09
**Status:** Active

---

## Scope

**Responsibility:** Maps all Iron Cage modules to their deployment packages showing which modules are included in each package

**In Scope:**
- Module-to-package mapping matrix (modules × packages)
- Shared module identification (modules appearing in multiple packages)
- Package composition analysis
- Module reuse pattern analysis
- Foundation module distribution across packages
- Package composition statistics
- Quick reference lookup (which package contains module X)

**Out of Scope:**
- Two-repository architecture design (see `repository_architecture.md`)
- Package definitions and characteristics (see `deployment_packages.md`)
- Package-to-package dependencies (see `package_dependencies.md`)
- Module implementation details (see `module/*/spec.md` in respective repository)
- Build and compilation processes (see respective repository CI/CD)
- Module architecture and layering (see `readme.md` § Architecture in respective repository)

---

## Deployment Modes

Iron Cage supports two deployment modes with different module-to-package mappings:

### Pilot/Demo Mode (Single Process, Localhost)

**Use Case:** Conference demos, local development, single-user testing

**Deployment:** All modules run in single Rust process on localhost. Dashboard connects via WebSocket.

**Module Sharing:** iron_state is shared instance accessed by both iron_runtime and iron_api in same process.

**Package Structure:**
- Combined "Iron Cage Pilot" package
- Includes: iron_runtime, iron_api, iron_dashboard, iron_state (all modules)
- Deployment: Single binary + static UI files
- Communication: WebSocket (localhost:8080)

**Data Flow:**
```
Agent Event → iron_runtime
                ↓
         iron_state.save_agent_state()
                ↓
         broadcast StateUpdate
                ↓
           iron_api/ws
                ↓
         Dashboard UI
```

**Pilot Mode Package Composition:**
- All 20 modules compiled into single binary
- iron_state shared between iron_runtime and iron_api
- Single SQLite database (./iron_state.db)

---

### Production Mode (Distributed, Cloud + Local)

**Use Case:** Multi-user SaaS, enterprise deployments, agents on developer machines

**Deployment:** Control Panel runs on cloud (AWS/GCP), Agent Runtime runs on developer machines.

**Module Separation:** Control Panel and Agent Runtime have completely separate data models and databases.

**Package Structure:**

**Control Panel Package (Docker, cloud):**
- Modules: iron_api, iron_dashboard, iron_token_manager, iron_secrets, **iron_control_store**
- Database: PostgreSQL (users, tokens, secrets, telemetry)
- **NO iron_state** (doesn't track agent execution)
- Receives telemetry from distributed agents via HTTPS POST

**Agent Runtime Package (PyPI, local install):**
- Modules: iron_runtime, iron_sdk, iron_state, iron_safety, iron_cost
- Database: SQLite (local agent state, audit events per machine)
- Sends telemetry to Control Panel via HTTPS (optional)

**Data Flow:**
```
Cloud: Control Panel                Developer Machines
├── iron_api                         ├── Machine 1: Alice
│   └── iron_control_store           │   ├── iron_runtime
│       └── PostgreSQL               │   └── iron_state → SQLite
│           (users, tokens,          │
│            telemetry)               ├── Machine 2: Bob
│                                     │   ├── iron_runtime
│   ▲                                 │   └── iron_state → SQLite
│   │ HTTPS POST /api/telemetry      │
│   └─────────────────────────────────┘
```

**Key Difference:** Control Panel does NOT have iron_state in production mode. It uses iron_control_store for user/token/secrets data.

---

### Current Status

**Pilot Implementation (Dec 2025):** Uses Pilot/Demo Mode only
**Production Implementation:** Planned post-pilot (Q1 2026)

**This document shows Production Mode mappings unless otherwise noted.**

---

## Overview

This document provides the definitive mapping between Iron Cage's 20 modules (across **two repositories**) and 5 deployment packages. Use this as a quick reference to answer questions like:

- "Which repository contains `iron_api`?" (Answer: iron_runtime)
- "Which package contains `iron_sdk`?" (Answer: Agent Runtime)
- "What modules are in the Agent Runtime package?"
- "Which modules are shared across multiple packages?"
- "How many modules does each package contain?"

**Quick Stats:**
- **20 total modules** across **2 repositories** and **5 deployment packages**
- **iron_runtime repository**: 10 modules (9 Rust + 1 Vue + 0 Python)
- **iron_cage repository**: 10 modules (6 Rust + 1 Vue + 3 Python)
- **3 foundation modules** published to crates.io (shared between repositories)
- **Package sizes:** 1-10 modules per package

---

## Module-to-Package Mapping Matrix

**Note:** This matrix shows **Production Mode** mappings. For Pilot/Demo Mode (single process, localhost), see "Deployment Modes" section above.

**Production Mode:** Control Panel (cloud) + Agent Runtime (local) + 4 other packages

This matrix shows which modules are included in each deployment package and which repository hosts each module:

| Module                | Repository     | Control Panel | Site | Runtime (PyPI) | Sandbox (PyPI) | CLI Tools |
|-----------------------|----------------|---------------|------|----------------|----------------|-----------|
| iron_api              | iron_runtime   | ✅            |      |                |                |           |
| iron_cli              | iron_cage      |               |      |                |                | ✅        |
| iron_cli_py           | iron_cage      |               |      |                |                | ✅        |
| iron_control_store    | iron_runtime   | ✅            |      |                |                |           |
| iron_cost             | iron_cage*     | ✅            |      | ✅             |                |           |
| iron_dashboard        | iron_runtime   | ✅            |      |                |                |           |
| iron_lang             | iron_runtime   |               |      | ✅             |                |           |
| iron_reliability      | iron_runtime   |               |      | ✅             |                |           |
| iron_runtime          | iron_runtime   |               |      | ✅             |                |           |
| iron_safety           | iron_runtime   |               |      | ✅             |                |           |
| iron_sandbox          | iron_cage      |               |      |                | ✅             |           |
| iron_sandbox_core     | iron_cage      |               |      |                | ✅             |           |
| iron_sandbox_py       | iron_cage      |               |      |                | ✅             |           |
| iron_sdk              | iron_runtime   |               |      | ✅             |                |           |
| iron_secrets          | iron_runtime   | ✅            |      |                |                |           |
| iron_site             | iron_cage      |               | ✅   |                |                |           |
| iron_state            | iron_runtime   | ❌            |      | ✅             |                |           |
| iron_telemetry        | iron_cage*     | ✅            |      | ✅             |                |           |
| iron_token_manager    | iron_runtime   | ✅            |      |                |                |           |
| iron_types            | iron_cage*     | ✅            |      | ✅             |                |           |

**Matrix Dimensions:** 21 modules × 5 packages = 105 cells (24 ✅ marks)

**Repository Legend:**
- **iron_runtime**: Module source code hosted in iron_runtime repository
- **iron_cage**: Module source code hosted in iron_cage repository
- **iron_cage***: Foundation module hosted in iron_cage, published to crates.io, consumed by iron_runtime

---

## Package Composition

### Package 1: Control Panel (8 modules - Production Mode)

**Purpose:** Self-contained web application for token management and admin dashboard

**Deployment Modes:**
- **Pilot Mode:** Runs on localhost, shares iron_state with iron_runtime (same process)
- **Production Mode:** Runs on cloud (AWS/GCP), uses iron_control_store (PostgreSQL), NO iron_state

**Included Modules (Production Mode):**
1. iron_api (Rust) - REST API server, telemetry ingestion
2. iron_dashboard (Vue/TypeScript) - Web UI
3. iron_token_manager (Rust) - JWT token management
4. iron_secrets (Rust) - Secrets storage
5. iron_control_store (Rust) - PostgreSQL schema for users/tokens/secrets/telemetry (production only)
6. iron_cost (Rust) - Shared types only
7. iron_types (Rust) - Foundation types
8. iron_telemetry (Rust) - Logging

**NOT Included (Production Mode):**
- iron_state (Agent Runtime only)
- iron_runtime (Agent Runtime only)
- iron_safety (Agent Runtime only)

**Technology Mix:** 7 Rust + 1 TypeScript

---

### Package 2: Marketing Site (1 module)

**Purpose:** Static marketing website

**Included Modules:**
1. iron_site (Vue/TypeScript) - Static marketing site

**Technology Mix:** 1 TypeScript

---

### Package 3: Agent Runtime (9 modules)

**Purpose:** PyPI package for running protected AI agents

**Deployment Mode Notes:**
- **Pilot Mode:** Runs on localhost sharing iron_state with iron_api
- **Production Mode:** Runs on developer machine, reports telemetry to Control Panel API (optional)

**Included Modules:**
1. iron_sdk (Python) - Pythonic SDK layer (includes examples/)
2. iron_runtime (Rust) - Agent orchestrator
3. iron_safety (Rust) - PII detection
4. iron_cost (Rust) - Budget tracking
5. iron_reliability (Rust) - Circuit breakers
6. iron_lang (Rust) - AI data protocol
7. iron_types (Rust) - Foundation types
8. iron_state (Rust) - Local state management, audit logs
9. iron_telemetry (Rust) - Logging

**Technology Mix:** 1 Python + 8 Rust

---

### Package 4: Sandbox (3 modules)

**Purpose:** PyPI package for OS-level isolation

**Included Modules:**
1. iron_sandbox_py (Python + PyO3) - Pythonic sandbox API
2. iron_sandbox_core (Rust) - OS sandboxing core
3. iron_sandbox (Rust, deprecated) - Legacy PyO3 bindings

**Technology Mix:** 1 Python+Rust hybrid + 2 Rust

---

### Package 5: CLI Tools (2 modules)

**Purpose:** Token management CLI with Rust binary + Python wrapper

**Architecture:** See [ADR-005](../docs/decisions/adr_005_cli_architecture.md) for wrapper pattern decision.

**Included Modules:**
1. iron_cli (Rust) - CLI tool
2. iron_cli_py (Python) - CLI with wrapper pattern (native: init/config/agent/secrets; wrapper: token/usage/limits/traces/auth)

**Technology Mix:** 1 Rust + 1 Python (wrapper delegates to binary)

---

## Shared Module Analysis

### Deployment Mode Impact

**Pilot Mode (Single Process):**
In pilot mode, these modules are **shared within same process**:

| Module         | Shared By                     | Mechanism           |
|----------------|-------------------------------|---------------------|
| iron_state     | iron_runtime + iron_api       | Arc<StateManager>   |
| iron_types     | All modules                   | Compile-time types  |
| iron_telemetry | All modules                   | Global logger       |

**Production Mode (Distributed):**
In production mode, these modules are **compiled separately** (no sharing):

| Module             | Packages                     | Count | Sharing |
|--------------------|------------------------------|-------|---------|
| iron_cost          | Control Panel, Agent Runtime | 2     | NO      |
| iron_telemetry     | Control Panel, Agent Runtime | 2     | NO      |
| iron_types         | Control Panel, Agent Runtime | 2     | NO      |
| iron_control_store | Control Panel only           | 1     | N/A     |
| iron_state         | Agent Runtime only           | 1     | N/A     |

**Key Point:** In production, Control Panel and Agent Runtime run on DIFFERENT MACHINES with DIFFERENT DATABASES. No module sharing occurs.

---

### Modules Appearing in Multiple Packages (Production Mode)

These foundation modules are compiled/bundled separately for each package:

| Module           | Packages                     | Count | Usage Pattern              |
|------------------|------------------------------|-------|----------------------------|
| iron_cost        | Control Panel, Agent Runtime | 2     | Budget tracking foundation |
| iron_telemetry   | Control Panel, Agent Runtime | 2     | Logging foundation         |
| iron_types       | Control Panel, Agent Runtime | 2     | Foundation types           |

**Key Observations:**
- **3 shared modules** appear in both Control Panel and Agent Runtime (production mode)
- **No module appears in all 5 packages**
- **Shared modules are foundation only** (types, telemetry, cost)
- **Application modules are exclusive** (CLI, SDK, Dashboard, etc.)
- **iron_state is NOT shared** in production mode (Agent Runtime only)

---

## Module Reuse Patterns

### Pattern 1: Foundation Module Sharing

**Modules:** iron_types, iron_state, iron_telemetry, iron_cost

**Shared Between:** Control Panel ↔ Agent Runtime

**Rationale:**
- Both packages need logging (iron_telemetry)
- Both packages need state management (iron_state)
- Both packages share common types (iron_types)
- Both packages track costs (iron_cost)

**Implementation:** Compiled separately for each package (no shared library)

---

### Pattern 2: Exclusive Application Modules

**Modules:** iron_api, iron_dashboard, iron_cli, iron_cli_py, iron_sdk, iron_testing, iron_site

**Characteristics:**
- Each appears in exactly one package
- Application-level functionality
- Not reused across packages

---

### Pattern 3: Technology-Specific Modules

**Python Modules:** iron_sdk, iron_cli_py, iron_sandbox_py
- Distributed via PyPI
- Grouped in Runtime/Sandbox/CLI Tools packages

**Rust Modules:** iron_*, iron_*_core (majority)
- Compiled into binaries or .so files
- Foundation and core logic

**TypeScript Modules:** iron_dashboard, iron_site
- Bundled with Vite
- Web applications

---

## Package Statistics

| Package        | Modules | Rust | Python | TypeScript | Shared Modules |
|----------------|---------|------|--------|------------|----------------|
| Control Panel  | 8       | 7    | 0      | 1          | 4              |
| Marketing Site | 1       | 0    | 0      | 1          | 0              |
| Agent Runtime  | 9       | 8    | 1      | 0          | 4              |
| Sandbox        | 3       | 2    | 1      | 0          | 0              |
| CLI Tools      | 2       | 1    | 1      | 0          | 0              |
| **Total**      | **23*** | **18**| **3**  | **2**      | **4**          |

\* Total is 23 (not 20) because 3 modules appear in 2 packages each: 20 + 3 = 23

---

## Quick Reference Lookup

### Where is module X?

Use this section for quick lookups:

**Python Modules:**
- `iron_sdk` → Agent Runtime (includes examples/)
- `iron_cli_py` → CLI Tools
- `iron_sandbox_py` → Sandbox

**Rust Core Modules:**
- `iron_api` → Control Panel
- `iron_cli` → CLI Tools
- `iron_runtime` → Agent Runtime
- `iron_sandbox_core` → Sandbox
- `iron_token_manager` → Control Panel

**TypeScript Modules:**
- `iron_dashboard` → Control Panel
- `iron_site` → Marketing Site

**Foundation Modules (appear in 2 packages - Production Mode):**
- `iron_cost` → Control Panel + Agent Runtime
- `iron_telemetry` → Control Panel + Agent Runtime
- `iron_types` → Control Panel + Agent Runtime

**State Management:**
- `iron_state` → Agent Runtime only (production mode) | Shared in pilot mode
- `iron_control_store` → Control Panel only (production mode)

**Feature Modules:**
- `iron_safety` → Agent Runtime
- `iron_reliability` → Agent Runtime
- `iron_lang` → Agent Runtime
- `iron_secrets` → Control Panel

---

## Module Distribution by Layer

Based on the 7-layer architecture (see `readme.md` § Architecture):

**Layer 1 (OS Sandboxing):**
- Package: Sandbox
- Modules: iron_sandbox_core, iron_sandbox, iron_sandbox_py

**Layer 2 (Foundation):**
- Packages: Control Panel, Agent Runtime
- Modules: iron_types, iron_state, iron_telemetry

**Layer 3 (Features):**
- Packages: Control Panel, Agent Runtime
- Modules: iron_safety, iron_cost, iron_reliability, iron_token_manager, iron_secrets

**Layer 4 (Infrastructure):**
- Package: Agent Runtime
- Modules: iron_lang, iron_testing

**Layer 5 (Integration):**
- Packages: Control Panel, Agent Runtime
- Modules: iron_api, iron_runtime, iron_sdk

**Layer 6 (Application):**
- Packages: Control Panel, Marketing Site, CLI Tools, Agent Runtime
- Modules: iron_cli, iron_cli_py, iron_dashboard, iron_site

---

## Cross-References

**Related Documentation:**
- **Package Definitions:** See `deployment_packages.md` for package definitions, actors, and architecture diagrams
- **Package Dependencies:** See `package_dependencies.md` for runtime dependencies between packages
- **Workspace Architecture:** See `readme.md` § Architecture for 7-layer module organization
- **Module Specifications:** See `module/*/spec.md` for individual module details

**Usage:**
- Use this document to find which package contains a specific module
- Use `deployment_packages.md` to understand what each package does
- Use `package_dependencies.md` to understand package relationships

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2025-12-07 | 1.1.0 | Added comprehensive deployment modes section (Pilot vs Production). Removed iron_state from Control Panel in production mode. Added iron_control_store for Control Panel database. Clarified shared vs compiled separately distinction. Updated all package compositions with deployment mode notes. |
| 2025-12-07 | 1.0 | Initial creation. Extracted from deployment_packages.md to separate module mapping concerns. Includes 21×6 mapping matrix, shared module analysis, package composition statistics, and quick reference lookup. |
