# Deployment Packages

**Version:** 2.0.0
**Created:** 2025-12-06
**Updated:** 2025-12-08
**Status:** Active
**Repository:** iron_runtime (this document exists in both iron_runtime and iron_cage repositories)

---

> **📍 Note:** You are viewing from the **iron_runtime** repository. This document describes all 6 deployment packages across both repositories. Key packages for iron_runtime: **Package 1 (Control Panel)** and **Package 3 (Agent Runtime)**.

---

### Scope

**Responsibility:**
Documents how Iron Cage modules are grouped into independent deployment packages for shipping and distribution. Defines 6 deployment packages (Control Panel, Marketing Site, Agent Runtime, Sandbox, CLI Tool, Python CLI), explains deployment methods and characteristics, provides actor inventory, and shows typical usage patterns.

**In Scope:**
- Package definitions (6 packages) and purposes
- Deployment characteristics (Docker, PyPI, static hosting, binary)
- Actor inventory (18 actors: human, software, package)
- Architecture diagrams (deployment, composition, user journeys)
- Typical usage patterns for each package
- Python SDK integration layer
- Build and release process
- Version compatibility matrix
- Deployment scenarios

**Out of Scope:**
- Module-to-package mappings (see `module_package_matrix.md`)
- Package runtime dependencies (see `package_dependencies.md`)
- Module implementation details (see individual module specs in `module/*/spec.md`)
- Build processes and CI/CD pipelines (see `.github/workflows/` when created)
- Pricing and business strategy (see `business/` directory)
- Installation instructions (see `readme.md` and module readmes)
- API documentation (see `module/iron_api/spec.md`)
- Development workflow (see `readme.md` § Contributing)

---

## Overview

Iron Cage is organized as a **two-repository architecture** with 22 modules (15 Rust crates + 2 TypeScript/Vue applications + 5 Python packages) distributed across two repositories:
- **iron_runtime** - Control Panel, Agent Runtime, and runtime services (9 Rust crates + 1 Vue app + 2 Python packages)
- **iron_cage** - Sandboxing, CLI tools, and supporting infrastructure (4 Rust crates + 1 Vue app + 2 Python packages)
- **Shared via crates.io** - 3 foundation modules (iron_types, iron_cost, iron_telemetry)

These modules are grouped into **6 independent deployment packages** based on how they're shipped and used together.

**Key Principle:** Modules are grouped by deployment unit, not by technology. A package can contain both Rust and TypeScript modules if they're always deployed together.

**Repository Architecture:** See `repository_architecture.md` for detailed documentation of the two-repository split, module distribution, repository interaction patterns, and migration path.

---

## Actors

### Human Actors
- **Developer** - Python developer building AI agents with LangChain/CrewAI
- **Operations Engineer** - Team member monitoring agent execution and costs
- **Security Engineer** - Team member implementing OS-level sandboxing
- **Administrator** - Team member managing tokens and secrets
- **Website Visitor** - Potential customer viewing marketing site

### Software Actors
- **Python Agent** - User's AI agent process (LangChain/CrewAI)
- **Web Browser** - Browser accessing Control Panel dashboard
- **Command Line** - Terminal running CLI commands
- **LLM API** - External services (OpenAI, Anthropic, etc.)
- **Database** - SQLite or PostgreSQL storing state
- **Static Host** - Netlify/Vercel/CloudFront serving marketing site
- **PyPI Registry** - Python Package Index hosting packages
- **Docker Registry** - Container registry hosting Control Panel image

### Package Actors
- **Control Panel Package** - Web application for monitoring
- **Marketing Site Package** - Static website for marketing
- **Agent Runtime Package** - Python library for agent protection
- **Sandbox Package** - Python library for OS isolation
- **CLI Tool Package** - Binary for token management

---

## Architecture Diagrams

### Diagram 1: Package Deployment & Communication Architecture

This diagram shows how the 5 packages are deployed and how they communicate at runtime.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          DEPLOYMENT ENVIRONMENT                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────┐         ┌────────────────────────────────┐ │
│  │  CLOUD (AWS/GCP/Azure) │         │  STATIC HOSTING (Netlify/CDN)  │ │
│  │                        │         │                                │ │
│  │  ┌──────────────────┐  │         │  ┌──────────────────────────┐  │ │
│  │  │ Control Panel    │  │         │  │  Marketing Site          │  │ │
│  │  │  (Docker)        │  │         │  │  (iron_site)             │  │ │
│  │  │                  │  │         │  │                          │  │ │
│  │  │ • iron_api       │  │         │  │ • Static HTML/CSS/JS     │  │ │
│  │  │ • iron_dashboard │  │         │  │ • Terminal theme         │  │ │
│  │  │ • iron_secrets   │  │         │  │ • No backend             │  │ │
│  │  │                  │  │         │  └──────────────────────────┘  │ │
│  │  │ Port: 3000       │  │         │           ▲                    │ │
│  │  └────────┬─────────┘  │         │           │ HTTPS              │ │
│  │           │            │         │           │                    │ │
│  │           │ HTTPS API  │         └───────────┼────────────────────┘ │
│  │           │ WebSocket  │                     │                      │
│  └───────────┼────────────┘                     │                      │
│              │                                  │                      │
│              │                                  │                      │
└──────────────┼──────────────────────────────────┼──────────────────────┘
               │                                  │
               │                                  │
┌──────────────┼──────────────────────────────────┼──────────────────────┐
│              ▼                                  ▼                      │
│         DEVELOPER WORKSTATION                                         │
├───────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  ┌──────────────────┐      ┌──────────────────┐    ┌──────────────┐  │
│  │ Web Browser      │      │ Python Process   │    │ CLI Terminal │  │
│  │ (Dashboard UI)   │      │ (Agent Runtime)  │    │              │  │
│  │                  │      │                  │    │              │  │
│  │ Connects to:     │      │ • iron_runtime   │    │ iron-cli     │  │
│  │ Control Panel    │      │ • iron_safety    │    │              │  │
│  │ via HTTPS        │      │ • iron_cost      │    │ Manages:     │  │
│  │                  │      │ • iron_lang      │    │ • Tokens     │  │
│  └──────────────────┘      │                  │    │ • Config     │  │
│                            │ pip install:     │    │              │  │
│  ┌──────────────────┐      │ iron-cage        │    └──────┬───────┘  │
│  │ Web Browser      │      │                  │           │          │
│  │ (Marketing Site) │      └────────┬─────────┘           │          │
│  │                  │               │                     │          │
│  │ Views static     │               │ Calls LLM API       │ Calls    │
│  │ content from     │               ▼                     │ API      │
│  │ iron_site        │      ┌─────────────────┐            ▼          │
│  └──────────────────┘      │  LLM APIs       │   ┌──────────────┐   │
│                            │  (External)     │   │ Control Panel│   │
│  ┌──────────────────┐      │                 │   │ API          │   │
│  │ Python Process   │      │ • OpenAI        │   │ (Token Gen)  │   │
│  │ (Sandbox)        │      │ • Anthropic     │   └──────────────┘   │
│  │                  │      │ • Gemini        │                      │
│  │ • iron_sandbox   │      └─────────────────┘                      │
│  │ • Landlock       │                                               │
│  │ • Seccomp        │      ┌─────────────────┐                      │
│  │                  │      │  SQLite/Redis   │                      │
│  │ pip install:     │      │  (Local State)  │                      │
│  │ iron-sandbox     │      │                 │                      │
│  └──────────────────┘      │ Used by Runtime │                      │
│     (Optional)             └─────────────────┘                      │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘

Legend:
────  Data flow / Communication
┌──┐  Package / Component
HTTPS API calls, WebSocket for real-time updates
```

**Key Interactions:**
1. **Control Panel ↔ Browser:** Web dashboard accessed via HTTPS, real-time updates via WebSocket
2. **Agent Runtime → LLM APIs:** Python agents make HTTPS calls to OpenAI/Anthropic
3. **CLI Tool → Control Panel API:** CLI generates tokens by calling Control Panel REST API
4. **Marketing Site ← Browser:** Static content served over HTTPS (no backend)
5. **Sandbox:** Standalone, no network communication (isolation by design)

---

### Diagram 2: Package Composition - Modules Distribution

This diagram shows which modules are compiled into each package (visual representation of the matrix).

```
┌────────────────────────────────────────────────────────────────────────┐
│                      IRON CAGE MODULE WORKSPACE                        │
│                (21 modules: 14 Rust + 2 TypeScript + 5 Python)         │
└────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
         ┌──────────▼──────┐  ┌─────▼─────┐  ┌────▼────────┐
         │ Foundation (3)  │  │ Domain (3) │  │ Infra (2)   │
         │ • iron_types    │  │ • iron_safety│ │ • iron_state│
         │ • iron_telemetry│  │ • iron_cost  │ │ • iron_api  │
         │ • iron_lang     │  │ • iron_reliability│         │
         └────────┬────────┘  └─────┬──────┘  └────┬────────┘
                  │                 │               │
                  │                 │               │
       ┌──────────┼─────────────────┼───────────────┼──────────┐
       │          │                 │               │          │
   ┌───▼──┐  ┌───▼──┐  ┌──────▼────────┐  ┌────▼─────┐  ┌───▼──────┐
   │ PKG1 │  │ PKG2 │  │     PKG3      │  │  PKG4    │  │   PKG5   │
   │      │  │      │  │               │  │          │  │          │
   │ CTRL │  │ SITE │  │    RUNTIME    │  │ SANDBOX  │  │   CLI    │
   │PANEL │  │      │  │    (PyPI)     │  │  (PyPI)  │  │   TOOL   │
   └──────┘  └──────┘  └───────────────┘  └──────────┘  └──────────┘

PACKAGE 1: Control Panel (8 modules, Docker)
┌────────────────────────────────────────────┐
│ ✅ iron_api (Rust)                         │
│ ✅ iron_dashboard (Vue/TypeScript)         │
│ ✅ iron_token_manager (Rust)               │
│ ✅ iron_secrets (Rust)                     │
│ ✅ iron_cost (Rust)                        │
│ ✅ iron_state (Rust)                       │
│ ✅ iron_telemetry (Rust)                   │
│ ✅ iron_types (Rust)                       │
└────────────────────────────────────────────┘
       Backend API + Frontend Dashboard

PACKAGE 2: Marketing Site (1 module, Static)
┌────────────────────────────────────────────┐
│ ✅ iron_site (Vue/TypeScript)              │
└────────────────────────────────────────────┘
       Static HTML/CSS/JS files

PACKAGE 3: Agent Runtime (11 modules, PyPI Wheel)
┌────────────────────────────────────────────┐
│ ✅ iron_runtime (Rust + Python)            │
│ ✅ iron_sdk (Python)                       │
│ ✅ iron_examples (Python)                  │
│ ✅ iron_testing (Python)                   │
│ ✅ iron_safety (Rust)                      │
│ ✅ iron_cost (Rust)                        │
│ ✅ iron_reliability (Rust)                 │
│ ✅ iron_lang (Rust)                        │
│ ✅ iron_types (Rust)                       │
│ ✅ iron_state (Rust)                       │
│ ✅ iron_telemetry (Rust)                   │
└────────────────────────────────────────────┘
       Rust compiled to .so, Python as pure modules

PACKAGE 4: Sandbox (3 modules, PyPI Wheel)
┌────────────────────────────────────────────┐
│ ✅ iron_sandbox (Rust + Python)            │
│ ✅ iron_sandbox_py (Rust + Python)         │
│ ✅ iron_sandbox_core (Rust)                │
└────────────────────────────────────────────┘
       Compiled into single .so extension

PACKAGE 5: CLI Tool (1 module, Binary)
┌────────────────────────────────────────────┐
│ ✅ iron_cli (Rust)                         │
└────────────────────────────────────────────┘
       Standalone static binary

PACKAGE 6: Python CLI (1 module, PyPI Wheel)
┌────────────────────────────────────────────┐
│ ✅ iron_cli_py (Python)                    │
└────────────────────────────────────────────┘
       Pure Python CLI alternative

Note: Foundation modules (iron_types, iron_state, iron_telemetry, iron_cost)
      are compiled separately for each package that needs them.
      They are NOT distributed as independent packages.
```

---

### Diagram 3: User Journey - Actor Interactions

This diagram shows which human actors interact with which packages and how.

```
┌────────────────────────────────────────────────────────────────────────┐
│                            USER JOURNEYS                               │
└────────────────────────────────────────────────────────────────────────┘

JOURNEY 1: Python Developer Building AI Agent
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  👤 Developer                                                        │
│   │                                                                  │
│   │ Step 1: Install runtime                                         │
│   ├──────────► pip install iron-cage                                │
│   │               │                                                  │
│   │               ▼                                                  │
│   │            ┌─────────────────┐                                  │
│   │            │ Agent Runtime   │  (PKG3 - PyPI)                   │
│   │            │ Package         │                                  │
│   │            └─────────────────┘                                  │
│   │                                                                  │
│   │ Step 2: Write agent code                                        │
│   ├──────────► from iron_cage import protect_agent                  │
│   │            @protect_agent(budget=50.0)                          │
│   │            def my_agent(): ...                                  │
│   │                                                                  │
│   │ Step 3: Run agent                                               │
│   ├──────────► python my_agent.py                                   │
│   │               │                                                  │
│   │               │ Agent runs with:                                │
│   │               │ • Budget tracking                               │
│   │               │ • PII detection                                 │
│   │               │ • Circuit breakers                              │
│   │               ▼                                                  │
│   │            Calls LLM APIs (OpenAI, Anthropic)                   │
│   │                                                                  │
└──────────────────────────────────────────────────────────────────────┘

JOURNEY 2: Operations Engineer Monitoring Agents
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  👤 Operations Engineer                                             │
│   │                                                                  │
│   │ Step 1: Access dashboard                                        │
│   ├──────────► https://control-panel.company.com                    │
│   │               │                                                  │
│   │               ▼                                                  │
│   │            ┌─────────────────┐                                  │
│   │            │ Control Panel   │  (PKG1 - Docker)                 │
│   │            │ Package         │                                  │
│   │            └─────────────────┘                                  │
│   │                                                                  │
│   │ Step 2: View real-time metrics                                 │
│   │            • Agent status                                       │
│   │            • Token usage                                        │
│   │            • Costs                                              │
│   │            • PII violations                                     │
│   │                                                                  │
│   │ Step 3: Manage secrets                                          │
│   ├──────────► Add/rotate API keys in UI                            │
│   │                                                                  │
└──────────────────────────────────────────────────────────────────────┘

JOURNEY 3: Administrator Managing Tokens
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  👤 Administrator                                                    │
│   │                                                                  │
│   │ Step 1: Install CLI                                             │
│   ├──────────► cargo install iron-cli                               │
│   │               │                                                  │
│   │               ▼                                                  │
│   │            ┌─────────────────┐                                  │
│   │            │ CLI Tool        │  (PKG5 - Binary)                 │
│   │            │ Package         │                                  │
│   │            └─────────────────┘                                  │
│   │                                                                  │
│   │ Step 2: Generate tokens                                         │
│   ├──────────► iron-cli token generate --project my-app             │
│   │               │                                                  │
│   │               │ Calls Control Panel API                         │
│   │               ▼                                                  │
│   │            Returns JWT token                                    │
│   │                                                                  │
│   │ Step 3: Distribute to developers                                │
│   ├──────────► Share token.json with team                           │
│   │                                                                  │
└──────────────────────────────────────────────────────────────────────┘

JOURNEY 4: Security Engineer Sandboxing Agent
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  👤 Security Engineer                                                │
│   │                                                                  │
│   │ Step 1: Install sandbox                                         │
│   ├──────────► pip install iron-sandbox                             │
│   │               │                                                  │
│   │               ▼                                                  │
│   │            ┌─────────────────┐                                  │
│   │            │ Sandbox         │  (PKG4 - PyPI)                   │
│   │            │ Package         │                                  │
│   │            └─────────────────┘                                  │
│   │                                                                  │
│   │ Step 2: Wrap agent in sandbox                                   │
│   ├──────────► from iron_sandbox import Sandbox                     │
│   │            with Sandbox(max_memory=512): ...                    │
│   │                                                                  │
│   │ Step 3: Agent runs with OS-level isolation                      │
│   │            • Landlock filesystem restrictions                   │
│   │            • Seccomp syscall filtering                          │
│   │            • Memory/CPU limits                                  │
│   │                                                                  │
└──────────────────────────────────────────────────────────────────────┘

JOURNEY 5: Website Visitor Learning About Product
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  👤 Website Visitor                                                  │
│   │                                                                  │
│   │ Step 1: Visit marketing site                                    │
│   ├──────────► https://ironcage.dev                                 │
│   │               │                                                  │
│   │               ▼                                                  │
│   │            ┌─────────────────┐                                  │
│   │            │ Marketing Site  │  (PKG2 - Static)                 │
│   │            │ Package         │                                  │
│   │            └─────────────────┘                                  │
│   │                                                                  │
│   │ Step 2: Read documentation                                      │
│   │            • Features                                           │
│   │            • Pricing                                            │
│   │            • Getting started                                    │
│   │                                                                  │
│   │ Step 3: Become developer → Journey 1                            │
│   │                                                                  │
└──────────────────────────────────────────────────────────────────────┘

CROSS-PACKAGE INTERACTIONS:
1. Developer (PKG3) → LLM APIs → External services
2. Operations (PKG1) ← WebSocket ← Control Panel backend
3. Administrator (PKG5) → API calls → Control Panel (PKG1)
4. All actors can use multiple packages simultaneously
5. Packages are independent - can be used alone or combined
```

---

## Package Definitions

## Deployment Modes

Iron Cage supports two deployment modes with different package compositions and communication patterns:

---

### Mode 1: Pilot/Demo (Single Process, Localhost)

**Use Case:** Conference demonstrations, local development, single-user testing, proof-of-concept

**Architecture:**
```
Developer Laptop (localhost)
├── Terminal 1: Rust Process (port 8080)
│   ├── iron_runtime (agent orchestration)
│   ├── iron_api (REST + WebSocket server)
│   ├── iron_state (shared in-memory + SQLite)
│   ├── iron_safety, iron_cost, iron_reliability
│   └── iron_telemetry, iron_types
│
└── Terminal 2: Node.js Dev Server (port 5173)
    └── iron_dashboard (Vue application)
```

**Communication:**
- Dashboard → WebSocket (ws://localhost:8080/ws) → iron_api
- iron_api reads from iron_state (shared instance)
- iron_runtime writes to iron_state (shared instance)
- **Single SQLite database** (./iron_state.db)

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
         Dashboard UI (real-time)
```

**Deployment:**
```bash
# Terminal 1: Start runtime + agent
cargo run --release -- lead_gen_agent.py --budget 50

# Terminal 2: Start dashboard
cd module/iron_dashboard
npm run dev  # http://localhost:5173
```

**Characteristics:**
- ✅ Zero network dependencies (works offline)
- ✅ Simple setup (2 terminal commands)
- ✅ Real-time WebSocket updates
- ✅ Single SQLite database
- ❌ Single user only
- ❌ Single machine only
- ❌ Not suitable for teams

**Status:** Current pilot implementation (Dec 2025)

---

### Mode 2: Production (Distributed, Cloud + Local)

**Use Case:** Multi-user SaaS, enterprise deployments, team collaboration, remote agents

**Architecture:**
```
Cloud Server (AWS/GCP/Azure)          Developer Workstations
├── Docker Container                  ├── Machine 1: Engineer Alice
│   ├── iron_api (REST API)          │   ├── pip install iron-cage
│   ├── iron_dashboard (static UI)    │   ├── iron_runtime (local agent)
│   ├── iron_token_manager            │   ├── iron_state (local SQLite)
│   ├── iron_secrets                  │   └── Sends telemetry via HTTPS
│   ├── iron_control_store            │
│   └── PostgreSQL (users/tokens)     ├── Machine 2: Engineer Bob
│                                     │   ├── pip install iron-cage
└── HTTPS API (port 443)              │   └── iron_runtime (different agent)
        ▲                              │
        │                              └── Machine 3: Data Scientist Carol
        └─── Telemetry Reports              └── iron_runtime (ML pipeline)
           (optional, HTTPS POST)
```

**Communication:**
- Dashboard → HTTPS API → Control Panel backend
- Agent Runtime → HTTPS POST /api/telemetry → Control Panel (optional)
- **No shared state** (separate databases, separate machines)

**Data Separation:**

**Control Panel Database (PostgreSQL):**
```sql
-- Multi-user SaaS data
users (id, email, password_hash, created_at)
api_tokens (id, user_id, token_hash, project_id)
secrets (id, user_id, key_name, encrypted_value)
telemetry_events (id, user_id, agent_id, event_type, payload)
```

**Agent Runtime Database (SQLite, local per machine):**
```sql
-- Local agent execution tracking
audit_events (id, agent_id, event_type, timestamp, details)
-- In-memory only: agent_state (agent_id, status, budget_spent, pii_detections)
```

**Deployment:**

**Control Panel (Cloud):**
```bash
docker build -t iron-control-panel .
docker run -p 443:443 \
  -e DATABASE_URL=postgres://... \
  -e JWT_SECRET=... \
  iron-control-panel
```

**Agent Runtime (Developer Machines):**
```bash
pip install iron-cage
export IRON_CONTROL_PANEL_URL=https://control.company.com
export IRON_API_TOKEN=<from-control-panel>

python my_agent.py  # Runs locally, optionally reports telemetry
```

**Characteristics:**
- ✅ Multi-user support (teams)
- ✅ Agents run on any machine
- ✅ Centralized token/secrets management
- ✅ Aggregated telemetry dashboard
- ✅ Scalable (cloud auto-scaling)
- ❌ Requires network connectivity
- ❌ More complex setup (cloud deployment)
- ❌ PostgreSQL required

**Status:** Planned post-pilot (Q1 2026)

---

### Key Differences Summary

| Aspect               | Pilot Mode                    | Production Mode                 |
|----------------------|-------------------------------|---------------------------------|
| **Deployment**       | Single machine, localhost     | Cloud + distributed machines    |
| **Processes**        | 2 (Rust + Node.js)            | N (1 cloud + M developer machines) |
| **Communication**    | WebSocket (localhost)         | HTTPS (internet)                |
| **Database**         | SQLite (shared)               | PostgreSQL (cloud) + SQLite (local per machine) |
| **iron_state**       | Shared instance               | Agent Runtime only (not in Control Panel) |
| **Control Panel DB** | iron_state (SQLite)           | iron_control_store (PostgreSQL) |
| **Users**            | Single user                   | Multi-user                      |
| **Telemetry**        | Real-time (WebSocket)         | Optional reporting (HTTPS POST) |
| **Network**          | Not required (offline OK)     | Required (HTTPS)                |
| **Use Case**         | Demos, development, testing   | Production SaaS, teams          |

---

### Package Composition by Mode

**Pilot Mode Package:**
- Name: "Iron Cage Pilot"
- Format: Single binary + static UI
- Modules: ALL (iron_runtime, iron_api, iron_state, iron_dashboard, etc.)
- Database: SQLite (shared)

**Production Mode Packages:**
- **Control Panel:** iron_api, iron_dashboard, iron_token_manager, iron_secrets, iron_control_store
- **Agent Runtime:** iron_runtime, iron_sdk, iron_state, iron_safety, iron_cost, iron_reliability
- **Database:** Control Panel uses PostgreSQL, Agent Runtime uses SQLite (local)

**Module Mappings in This Document:**
Unless otherwise noted, package definitions and module lists in this document describe **Production Mode**. For Pilot Mode, see the architecture diagram above.

---

### Package 1: Control Panel (Web Application)

**What it is:** Web-based management dashboard for monitoring and controlling AI agents

**Deployment Method:** Docker container, cloud hosting (AWS/GCP/Azure)

**Deployment Modes:**
- **Pilot Mode:** Runs on localhost, shares iron_state with iron_runtime
- **Production Mode:** Runs on cloud, uses iron_control_store (PostgreSQL), no iron_state

**Target Users:** Operations teams, administrators, developers monitoring agent execution

**Modules Included (Production Mode):**
- `iron_api` (Rust) - REST API + WebSocket server, telemetry ingestion
- `iron_dashboard` (TypeScript/Vue) - Dashboard UI with real-time metrics
- `iron_token_manager` (Rust) - JWT authentication and token management
- `iron_secrets` (Rust) - Secrets management with encrypted storage
- `iron_control_store` (Rust) - PostgreSQL schema for users, tokens, secrets, telemetry
- `iron_cost` (Rust) - Shared types only (no agent data)
- `iron_telemetry` (Rust) - Logging and tracing
- `iron_types` (Rust) - Shared types

**NOT Included (Production Mode):**
- `iron_state` (Agent Runtime only - see note below)
- `iron_runtime` (Agent Runtime only)
- `iron_safety` (Agent Runtime only)

**Why Together:**
These modules provide the complete web UI + API stack for multi-user token/secrets management. The frontend depends on the API, and they're always deployed together as a single web service.

**Deployment Note:**
In Production Mode, Control Panel does NOT include iron_state because it doesn't track local agent execution. It receives aggregated telemetry from distributed Agent Runtime instances via HTTPS POST and stores it in PostgreSQL (iron_control_store).

In Pilot Mode, Control Panel runs in same process as Agent Runtime and shares iron_state instance.

**Deployment Example:**
```bash
docker build -t iron-control-panel .
docker run -p 3000:3000 \
  -e DATABASE_URL=sqlite:///data/iron.db \
  -e JWT_SECRET=<secret> \
  iron-control-panel
```

**Typical Use:**
```
User opens browser → https://control-panel.company.com
Dashboard shows:
- Active agents and their status
- Token usage and costs
- Security events (PII detection)
- Secrets management UI
```

---

### Package 2: Marketing Site (Static Web)

**What it is:** Public-facing marketing website for Iron Cage product

**Deployment Method:** Static hosting (Netlify, Vercel, CloudFront, GitHub Pages)

**Target Users:** Website visitors, potential customers, documentation readers

**Modules Included:**
- `iron_site` (TypeScript/Vue) - Static marketing site with terminal theme

**Why Independent:**
Completely separate from product functionality. No backend dependencies. Pure static HTML/CSS/JS. Different deployment lifecycle than product (marketing updates vs feature releases).

**Deployment Example:**
```bash
cd module/iron_site
npm run build
# Deploy dist/ to static hosting
netlify deploy --prod --dir=dist
```

**Typical Use:**
```
Visitor opens → https://ironcage.dev
Static site shows:
- Product features
- Pricing information
- Documentation links
- Contact information
```

---

### Package 3: Agent Runtime (Python Package)

**What it is:** Python library that developers `pip install` to add Iron Cage safety/cost controls to their AI agents

**Deployment Method:** Published to PyPI (Python Package Index)

**Target Users:** Python developers building AI agents with LangChain/CrewAI

**Modules Included:**
- `iron_runtime` (Rust + Python bindings) - Agent orchestration and PyO3 bridge
- `iron_safety` (Rust) - PII detection and redaction
- `iron_cost` (Rust) - Budget tracking and enforcement
- `iron_reliability` (Rust) - Circuit breaker patterns
- `iron_types` (Rust) - Shared types
- `iron_state` (Rust) - Local state management
- `iron_telemetry` (Rust) - Logging
- `iron_lang` (Rust) - LLM protocol integration

**Why Together:**
These are the core agent protection features. Compiled into a single `.so` Python extension module that developers import. All runtime safety/cost/reliability features needed for local agent execution.

**Deployment Example:**
```bash
# Build wheel with maturin
maturin build --release
# Publish to PyPI
twine upload target/wheels/*.whl
```

**Typical Use:**
```python
# Developer installs
pip install iron-cage

# Developer uses in their agent
from iron_cage import protect_agent

@protect_agent(budget_usd=50.0, pii_detection=True)
def my_langchain_agent():
    # Agent code with LangChain/CrewAI
    return agent.run(task)
```

---

### Package 4: Sandbox (Python Package)

**What it is:** Separate Python library for sandboxing agent execution using Linux kernel isolation

**Deployment Method:** Published to PyPI (optional install)

**Target Users:** Security-focused developers requiring OS-level isolation

**Modules Included:**
- `iron_sandbox` (Rust + Python bindings) - PyO3 sandbox API
- `iron_sandbox_core` (Rust) - Linux kernel isolation (Landlock, Seccomp, rlimit)

**Why Separate from Runtime:**
- **Optional feature:** Not all users need kernel-level sandboxing
- **Platform-specific:** Linux-only (kernel ≥5.13), won't work on macOS/Windows
- **Heavy dependencies:** Requires libseccomp, kernel features
- **Security focus:** Separate security boundary from runtime features
- **Independent releases:** Sandbox can update without affecting runtime

**Deployment Example:**
```bash
# Build and publish separately from iron-cage
cd module/iron_sandbox
maturin build --release
twine upload target/wheels/*.whl
```

**Typical Use:**
```python
# Separate optional install
pip install iron-sandbox

# Use for OS-level isolation
from iron_sandbox import Sandbox

with Sandbox(allowed_paths=["/tmp"], max_memory_mb=512):
    run_untrusted_agent()  # Kernel-enforced restrictions
```

---

### Package 5: CLI Tool (Local Binary)

**What it is:** Command-line tool for generating API tokens and managing configuration

**Deployment Method:** Installed locally via `cargo install` or downloaded binary

**Target Users:** Developers setting up Iron Cage, operations teams managing tokens

**Modules Included:**
- `iron_cli` (Rust) - Standalone binary with minimal dependencies

**Why Separate:**
- **Developer tool:** Used for setup/configuration, not production runtime
- **Minimal dependencies:** Self-contained binary, doesn't need full runtime
- **Different lifecycle:** Utility tool vs runtime library
- **Installation method:** Binary download/install vs library import

**Deployment Example:**
```bash
# Install from crates.io
cargo install iron-cli

# Or download binary
wget https://github.com/iron-cage/releases/iron-cli-linux-amd64
chmod +x iron-cli-linux-amd64
mv iron-cli-linux-amd64 /usr/local/bin/iron-cli
```

**Typical Use:**
```bash
# Generate API tokens
iron-cli token generate --project my-app --output token.json

# Validate configuration
iron-cli config validate --file iron.toml

# Initialize project
iron-cli init --template langchain
```

---

### Package 6: Python CLI (Python Package)

**What it is:** Python-based command-line tool for token management and agent control

**Deployment Method:** Published to PyPI, installed via `pip install`

**Target Users:** Python developers who prefer Python tools, teams without Rust toolchain

**Modules Included:**
- `iron_cli_py` (Python) - Pure Python CLI using Click framework

**Why Separate from Rust CLI:**
- **Different ecosystem:** Python developers may not have Rust toolchain installed
- **Easier contribution:** Python developers can contribute without learning Rust
- **Framework integration:** Can be imported as library for programmatic use
- **Alternative choice:** Users can choose based on preference/environment

**Deployment Example:**
```bash
# Install from PyPI
pip install iron-cli-py

# Or install in development mode
cd module/iron_cli_py
pip install -e .
```

**Typical Use:**
```bash
# Generate API tokens
iron-py token generate --project my-app --output token.json

# Validate configuration
iron-py config validate --file iron.toml

# Initialize project
iron-py init --template langchain

# Import as library (unique to Python CLI)
from iron_cli_py import TokenGenerator
generator = TokenGenerator(api_url="https://control-panel.company.com")
token = generator.create_token(project="my-app")
```

---

## Module-to-Package Mappings

**See:** `module_package_matrix.md` for complete module-to-package mapping analysis including:
- Module-to-package mapping matrix (21 modules × 6 packages)
- Shared module identification (4 foundation modules appearing in multiple packages)
- Module reuse pattern analysis
- Foundation module distribution across packages
- Package composition statistics
- Quick reference lookup (which package contains module X)

**Quick Summary:**
- **21 total modules** mapped to **6 deployment packages**
- **5 shared modules:** iron_cost, iron_state, iron_telemetry, iron_types appear in multiple packages
- **16 exclusive modules** appear in only one package each
- **Package sizes:** 1-11 modules per package

---

## Package Dependencies

**See:** `package_dependencies.md` for complete dependency analysis including:
- Package-to-package dependency matrix (6×6)
- Detailed runtime dependencies for each package
- Integration patterns between packages
- Deployment independence analysis
- Network and system requirements

**Quick Summary:**
- **4 fully independent packages:** Control Panel, Marketing Site, Sandbox, Agent Runtime (without telemetry)
- **2 dependent packages:** CLI Tool and Python CLI both require Control Panel API to be running

---

## Deployment Scenarios

### Scenario 1: Enterprise Deployment
**Packages Used:** Control Panel + Agent Runtime + CLI Tool

```
1. Install Control Panel on cloud server (Docker)
2. Developers install Agent Runtime via pip
3. Ops team uses CLI to generate tokens
4. Agents connect to Control Panel API for monitoring
```

### Scenario 2: Local Development
**Packages Used:** Agent Runtime only

```
1. Developer: pip install iron-cage
2. Use runtime features locally (budget tracking, PII detection)
3. No Control Panel needed for local testing
```

### Scenario 3: Maximum Security
**Packages Used:** Agent Runtime + Sandbox

```
1. pip install iron-cage iron-sandbox
2. Use Sandbox for kernel isolation
3. Use Runtime for safety/cost controls
4. Layered security (OS isolation + application controls)
```

---

## Build and Release Process

### Control Panel
```bash
# Build Docker image
cd module/iron_api && cargo build --release
cd module/iron_dashboard && npm run build
docker build -t iron-control-panel:latest .

# Push to registry
docker push ghcr.io/iron-cage/control-panel:latest
```

### Marketing Site
```bash
cd module/iron_site
npm run build
# Deploy dist/ to static hosting
```

### Agent Runtime
```bash
cd module/iron_runtime
maturin build --release --features pyo3
twine upload target/wheels/*
```

### Sandbox
```bash
cd module/iron_sandbox
maturin build --release
twine upload target/wheels/*
```

### CLI Tool
```bash
cd module/iron_cli
cargo build --release
# Upload binary to GitHub releases
```

### Python CLI
```bash
cd module/iron_cli_py
python -m build
twine upload dist/*
```

---

## Version Compatibility

**Independent Versioning:** Each package has its own version number.

**Compatibility Matrix:**

| Control Panel | Agent Runtime | Sandbox | CLI Tool | Python CLI | Compatible? |
|---------------|---------------|---------|----------|------------|-------------|
| 1.0.x         | 1.0.x         | 1.0.x   | 1.0.x    | 1.0.x      | ✅ Fully    |
| 1.0.x         | 1.1.x         | 1.0.x   | 1.0.x    | 1.0.x      | ✅ Yes      |
| 1.0.x         | 2.0.x         | 1.0.x   | 1.0.x    | 1.0.x      | ⚠️ API changes |
| 2.0.x         | 1.0.x         | 1.0.x   | 1.0.x    | 1.0.x      | ❌ No       |

**Rules:**
- Control Panel, CLI Tool, and Python CLI must have matching major versions
- Agent Runtime can be 1 minor version ahead of Control Panel
- Sandbox is independent (no version coupling)

---

## Package Size Estimates

| Package        | Download Size | Installed Size | Modules Count |
|----------------|---------------|----------------|---------------|
| Control Panel  | ~50 MB (Docker) | ~150 MB      | 8 modules     |
| Marketing Site | ~2 MB (static)  | ~5 MB        | 1 module      |
| Agent Runtime  | ~18 MB (wheel)  | ~55 MB       | 11 modules    |
| Sandbox        | ~6 MB (wheel)   | ~18 MB       | 3 modules     |
| CLI Tool       | ~8 MB (binary)  | ~8 MB        | 1 module      |
| Python CLI     | ~1 MB (wheel)   | ~3 MB        | 1 module      |

---

## Cross-References

### Related Documentation
- **Module-to-Package Mappings:** See `module_package_matrix.md` for which modules belong to which packages (21×6 matrix)
- **Package Dependencies:** See `package_dependencies.md` for runtime dependencies, integration patterns, and deployment independence analysis
- **Module Architecture:** See `pilot/spec.md` § Module Architecture for detailed module breakdown
- **Workspace Overview:** See `readme.md` § Architecture for layer organization
- **Individual Module Specs:** See `module/*/spec.md` for module-specific details
- **Business Strategy:** See `business/` directory for pricing and market positioning

### Related Rulebooks
- **Organizational Principles:** See `$PRO/genai/code/rules/organizational_principles.rulebook.md` for module definition
- **Crate Distribution:** See `$PRO/genai/code/rules/crate_distribution.rulebook.md` for Rust packaging patterns

---

## Revision History

| Version | Date       | Author | Changes                                    |
|---------|------------|--------|--------------------------------------------|
| 2.0.0   | 2025-12-08 | [Team] | Updated to reflect two-repository architecture (iron_runtime + iron_cage split). Changed from "polyglot monorepo" to distributed repository model. Added cross-reference to repository_architecture.md. Updated module count from 21 to 22. |
| 1.3.0   | 2025-12-07 | [Team] | Added Deployment Modes section (Pilot vs Production). Clarified iron_state removal from Control Panel in production mode. Added iron_control_store for production database. Updated package definitions with deployment mode notes. |
| 1.2.0   | 2025-12-07 | [Team] | Extracted module-to-package mappings to separate document (module_package_matrix.md). Removed redundant mapping matrix section. Updated scope to reflect focused responsibility on package definitions and deployment. Added cross-reference. |
| 1.1.0   | 2025-12-07 | [Team] | Extracted package dependencies to separate document (package_dependencies.md). Removed redundant dependency matrix and runtime dependencies sections. Added cross-reference. |
| 1.0.0   | 2025-12-06 | [Team] | Initial deployment packages documentation  |

---

**Status:** ✅ Active
**Next Review:** When adding new modules or changing deployment strategy
