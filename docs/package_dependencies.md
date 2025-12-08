# Package Dependencies

**Version:** 2.0.0
**Date:** 2025-12-08
**Status:** Active - Two Repository Architecture
**Repository:** iron_runtime (this document exists in both iron_runtime and iron_cage repositories)

---

> **📍 Note:** You are viewing from the **iron_runtime** repository. This document describes dependencies between all 6 deployment packages. Key insight: iron_runtime packages (Control Panel, Agent Runtime) depend on iron_cage's foundation modules via crates.io.

---

## ⚠️ IMPORTANT: Two-Repository Architecture

As of December 2025, Iron Cage transitioned to a **two-repository architecture**:

1. **iron_runtime** (this repository) - Control Panel, Agent Runtime, runtime services
2. **iron_cage** (separate repository) - OS sandboxing, CLI tools, foundation modules

**Repository Communication:**
- **Rust dependencies**: Foundation modules (iron_types, iron_cost, iron_telemetry) published to crates.io
- **HTTP API**: iron_runtime exposes REST API for telemetry ingestion (optional for Agent Runtime)
- **No direct linking**: Repositories are completely independent at build time

**See:** `repository_architecture.md` for complete architecture documentation.

---

## Scope

**Responsibility:** Analyzes runtime dependencies between deployment packages and their external system requirements across both repositories

**In Scope:**
- Package-to-package runtime dependencies (6×6 dependency matrix)
- Cross-repository dependencies (iron_runtime ↔ iron_cage via crates.io + HTTP API)
- External runtime dependencies (databases, APIs, libraries, system requirements)
- Network requirements for each package
- Deployment independence analysis
- Integration patterns between packages and repositories
- Optional vs required dependencies

**Out of Scope:**
- Two-repository architecture design (see `repository_architecture.md`)
- Package definitions and module groupings (see `deployment_packages.md`)
- Module-level dependencies within packages (see `module/*/Cargo.toml` in respective repository)
- Build-time dependencies and CI processes (see respective repository CI/CD)
- Development environment dependencies (see `readme.md` § Development in respective repository)
- Business and partnership dependencies (see `business/` directory)

---

## Overview

**Deployment Mode Context:**
This document describes runtime dependencies between deployment packages across **two repositories**. Iron Cage supports two deployment modes:

- **Pilot/Demo Mode:** Single process on localhost, WebSocket communication, shared iron_state (single repository)
- **Production Mode:** Distributed (cloud + local), HTTPS communication, separate repositories and databases

**This document describes Production Mode dependencies** unless otherwise noted. In Pilot Mode, all components run in same process with no cross-repository dependencies.

**Repository Architecture Impact:**
- **iron_runtime** and **iron_cage** are deployed independently
- Foundation modules (iron_types, iron_cost, iron_telemetry) shared via **crates.io**
- No runtime dependencies between repositories except optional HTTP API calls
- Each repository has independent build, test, and deployment pipelines

**See:** `deployment_packages.md` § Deployment Modes and `repository_architecture.md` for complete architecture details.

This document analyzes the runtime dependencies between Iron Cage's 6 deployment packages across 2 repositories. Understanding these dependencies is critical for:

1. **Deployment Planning**: Determine which packages can be deployed independently
2. **System Architecture**: Understand integration points and data flows
3. **Scaling Strategy**: Identify which packages need to scale together
4. **Installation Order**: Plan correct installation sequence for dependent packages
5. **Testing Strategy**: Design integration tests based on package interactions

**Key Finding**: 4 of 6 packages are fully independent; only CLI tools require Control Panel API.

---

## Package Dependency Matrix

This matrix shows runtime dependencies between deployment packages (which packages depend on other packages):

|                    | Control Panel | Marketing Site | Agent Runtime | Sandbox | CLI Tool | Python CLI |
|--------------------|---------------|----------------|---------------|---------|----------|------------|
| **Control Panel**  | -             | -              | -             | -       | -        | -          |
| **Marketing Site** | -             | -              | -             | -       | -        | -          |
| **Agent Runtime**  | ⚠️ Optional   | -              | -             | -       | -        | -          |
| **Sandbox**        | -             | -              | -             | -       | -        | -          |
| **CLI Tool**       | ✅ Required   | -              | -             | -       | -        | -          |
| **Python CLI**     | ✅ Required   | -              | -             | -       | -        | -          |

**Legend:**
- `-` No dependency
- `✅ Required` Package requires another package to function
- `⚠️ Optional` Package can optionally integrate with another package

**Note:** In Pilot/Demo Mode (single process), dependencies are satisfied by module linking within same binary. In Production Mode (distributed), dependencies require network communication (HTTPS).

**Key Dependencies:**
1. **CLI Tool → Control Panel (Required):** Rust CLI calls Control Panel API for token generation
2. **Python CLI → Control Panel (Required):** Python CLI calls Control Panel API for token generation
3. **Agent Runtime → Control Panel (Optional):** Agents can optionally report telemetry to Control Panel API

**Independence Summary:**
- **4 fully independent packages:** Control Panel, Marketing Site, Sandbox, Agent Runtime (without telemetry)
- **2 dependent packages:** CLI Tool and Python CLI both require Control Panel API to be running
- **All packages can be installed separately:** No compile-time or install-time dependencies between packages

---

## Package Runtime Dependencies

### Control Panel Package

**Package Dependencies:**
- None (self-contained web application)

**External Runtime Dependencies:**

**Pilot Mode (localhost):**
- **Database:** SQLite 3.35+ (shared with Agent Runtime via iron_state)
- **System:** Linux/macOS/Windows, glibc 2.31+ (Linux)
- **Network:** None (WebSocket to localhost only)

**Production Mode (cloud):**
- **Database:** PostgreSQL 14+ (for users, tokens, secrets, telemetry)
- **Cache (Optional):** Redis 6.0+ (for session management, rate limiting)
- **System:** Linux (Docker container), glibc 2.31+
- **Network:** HTTPS port 443 (inbound), PostgreSQL port 5432 (database)

**Modules Included:**
- **Pilot Mode:** iron_api, iron_dashboard, iron_state (shared), iron_token_manager, iron_secrets, etc.
- **Production Mode:** iron_api, iron_dashboard, iron_control_store, iron_token_manager, iron_secrets (NO iron_state)

**Depended On By:**
- CLI Tool (required for token generation)
- Python CLI (required for token generation)
- Agent Runtime (optional telemetry reporting in production mode)

**Deployment Note:**
In pilot mode, Control Panel shares iron_state with Agent Runtime in same process. In production mode, Control Panel uses iron_control_store (PostgreSQL) and does NOT have iron_state.

---

### Marketing Site Package

**Package Dependencies:**
- None (static files)

**External Runtime Dependencies:**
- **Web Server:** nginx, Apache, Netlify, Vercel, or any static hosting
- **System:** None (pure HTML/CSS/JS)

**Network Requirements:**
- Static hosting only (no backend needed)
- Outbound: CDN requests for fonts, icons (optional)

**Depended On By:**
- None

**Deployment Notes:**
- Fully independent, can be deployed on any static hosting
- No backend, no database, no API calls
- Can link to Control Panel for signup flow

---

### Agent Runtime Package

**Package Dependencies:**
- Control Panel (optional, for telemetry reporting in production mode only)

**External Runtime Dependencies:**

**Pilot Mode (localhost):**
- **Python:** 3.8, 3.9, 3.10, 3.11, or 3.12
- **LLM API Keys:** OpenAI, Anthropic, or other provider API keys
- **Database:** SQLite (shared with Control Panel via iron_state in same process)
- **System:** Linux/macOS/Windows
- **Network:** None (local execution only, no external connections except LLM APIs)

**Production Mode (distributed):**
- **Python:** 3.8, 3.9, 3.10, 3.11, or 3.12
- **LLM API Keys:** OpenAI, Anthropic, or other provider API keys
- **Database:** SQLite (local agent state, separate from Control Panel's PostgreSQL)
- **System:** Linux (glibc 2.31+), macOS (10.15+), Windows 10+ (PyO3 binary)
- **Network:** Outbound HTTPS to LLM APIs (required), optional HTTPS to Control Panel API (for telemetry)

**Modules Included:**
- **Both Modes:** iron_runtime, iron_sdk, iron_state, iron_safety, iron_cost, iron_reliability, iron_lang, iron_types, iron_telemetry

**Depended On By:**
- None

**Deployment Notes:**
- **Pilot Mode:** Shares iron_state with Control Panel in same Rust process, single SQLite database
- **Production Mode:** Has own iron_state instance (local SQLite per machine), optionally reports telemetry to Control Panel via HTTPS
- Install via PyPI: `pip install iron-cage`
- Requires valid LLM API keys in environment or config

---

### Sandbox Package

**Package Dependencies:**
- None (separate from runtime)

**External Runtime Dependencies:**
- **Linux Kernel:** ≥5.13 (for landlock LSM support)
- **libseccomp:** ≥2.5.0 (for seccomp-bpf filtering)
- **System:** Linux only (uses landlock, seccomp, rlimit)

**Network Requirements:**
- None (isolation feature blocks network by default)

**Depended On By:**
- None

**Deployment Notes:**
- Linux-only package (uses kernel features unavailable on macOS/Windows)
- Install via PyPI: `pip install iron-sandbox`
- Can be used independently of Agent Runtime for general sandboxing

---

### CLI Tool Package

**Package Dependencies:**
- **Control Panel (Required):** Calls Control Panel REST API for token generation

**External Runtime Dependencies:**
- **System:** Linux (glibc 2.31+), macOS (10.15+), Windows 10+ (static binary)
- **Network:** HTTPS connectivity to Control Panel API

**Network Requirements:**
- Outbound HTTPS to Control Panel API (default: https://localhost:3000)

**Depended On By:**
- None

**Deployment Notes:**
- Requires Control Panel to be running and accessible
- Single static binary (no additional dependencies)
- Configure Control Panel URL via environment: `IRON_API_URL=https://api.example.com`

---

### Python CLI Package

**Package Dependencies:**
- **Control Panel (Required):** Calls Control Panel REST API for token generation

**External Runtime Dependencies:**
- **Python:** 3.8, 3.9, 3.10, 3.11, or 3.12
- **PyPI Packages:** click ≥8.0, rich ≥13.0, httpx ≥0.24, pyyaml ≥6.0
- **Network:** HTTPS connectivity to Control Panel API

**Network Requirements:**
- Outbound HTTPS to Control Panel API (default: https://localhost:3000)

**Depended On By:**
- None

**Deployment Notes:**
- Requires Control Panel to be running and accessible
- Install via PyPI: `pip install iron-cli-py`
- Configure Control Panel URL via environment: `IRON_API_URL=https://api.example.com`

---

## Integration Patterns

### Pattern 1: CLI → Control Panel Token Generation

**Flow:**
1. User runs: `iron-cli token generate --budget 100`
2. CLI makes POST request to Control Panel API: `/api/tokens`
3. Control Panel validates request, generates JWT token
4. Control Panel returns token to CLI
5. CLI saves token to `~/.iron/tokens`

**Dependencies:**
- Control Panel must be running and accessible
- User must have valid credentials/API key

**Error Handling:**
- CLI shows clear error if Control Panel unreachable
- CLI retries with exponential backoff (3 attempts)

---

### Pattern 2: Agent Runtime → Control Panel Telemetry (Optional)

**Flow:**
1. Agent runtime configured with telemetry enabled
2. Agent makes LLM call → runtime intercepts
3. Runtime sends telemetry to Control Panel: `/api/telemetry`
4. Control Panel stores metrics in database
5. Dashboard shows real-time agent metrics

**Dependencies:**
- Optional: Agent runtime works without telemetry
- Control Panel must be running if telemetry enabled

**Error Handling:**
- Runtime buffers telemetry if Control Panel unavailable
- Runtime disables telemetry after 3 failed attempts
- No impact on agent execution if telemetry fails

---

## Deployment Independence Analysis

### Fully Independent Packages (4/6)

These packages can be deployed and used without any other Iron Cage packages:

1. **Control Panel**: Standalone web application
2. **Marketing Site**: Static website
3. **Sandbox**: OS-level sandboxing library
4. **Agent Runtime** (with telemetry disabled): Python runtime for agents

### Dependent Packages (2/6)

These packages require another package to be running:

1. **CLI Tool**: Requires Control Panel for token generation
2. **Python CLI**: Requires Control Panel for token generation

### Deployment Strategies

**Strategy 1: Full Platform**
- Deploy all 6 packages for complete platform experience
- Order: Control Panel → Marketing Site → Agent Runtime → CLI tools

**Strategy 2: Agent-Only**
- Deploy only Agent Runtime for standalone agent execution
- No Control Panel needed if telemetry disabled
- No CLI tools needed if tokens managed manually

**Strategy 3: Control Panel + CLIs**
- Deploy Control Panel + CLI tools for token management
- Skip Agent Runtime if not running agents locally
- Marketing Site optional for public presence

**Strategy 4: Sandbox-Only**
- Deploy only Sandbox package for general OS sandboxing use
- Completely independent of agent runtime ecosystem

---

## Cross-References

**Related Documentation:**
- Package definitions and module mappings: `deployment_packages.md`
- Module-level dependencies: See `module/*/Cargo.toml`, `module/*/package.json`, `module/*/pyproject.toml`
- Build and CI dependencies: `.github/workflows/`
- Development environment setup: `readme.md` § Development

**Package Definitions:**
- See `deployment_packages.md` § Package Definitions for detailed package contents

**Module-to-Package Mappings:**
- See `deployment_packages.md` § Module-to-Package Mapping Matrix for which modules belong to which packages

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2025-12-07 | 1.0 | Initial creation. Extracted from deployment_packages.md to separate dependency analysis concerns. Includes 6×6 dependency matrix, runtime dependencies for all 6 packages, integration patterns, and deployment independence analysis. |
