# Iron Runtime

Control Panel and Agent Runtime for Iron Cage AI agent management.

## Overview

**iron_runtime** is one of two repositories in the Iron Cage project:
- **iron_runtime** (this repo) - Control Panel, Agent Runtime, runtime services
- **iron_cage** - Sandboxing only (OS-level isolation via Landlock, seccomp, rlimits)

This repository contains the core runtime modules for AI agent management including safety controls, cost tracking, reliability patterns, and the Control Panel web application.

## Repository Architecture

See [`docs/repository_architecture.md`](docs/repository_architecture.md) for complete documentation of the two-repository split.

## Modules

All modules are located in the `module/` directory.

| Module | Responsibility | Language |
|--------|----------------|----------|
| **iron_types** | Shared types, data structures, API contracts | Rust |
| **iron_cost** | Budget tracking, cost calculation, spending limits | Rust |
| **iron_telemetry** | Logging, tracing, metrics collection | Rust |
| **iron_runtime_state** | Agent execution state management | Rust |
| **iron_safety** | PII detection and redaction | Rust |
| **iron_reliability** | Circuit breaker, retry patterns, fault tolerance | Rust |
| **iron_secrets** | Encrypted secrets storage, access control | Rust |
| **iron_token_manager** | User management, API token generation, JWT auth, rate limiting | Rust |
| **iron_control_api** | REST API server, WebSocket endpoints | Rust |
| **iron_runtime** | Agent orchestration, PyO3 Python bridge | Rust |
| **iron_cli** | Command-line interface for API operations (authoritative) | Rust |
| **iron_cli_py** | Python CLI with wrapper to iron_cli for operations | Python |
| **iron_sdk** | Pythonic SDK for Agent Runtime integration (includes examples/) | Python |
| **iron_testing** | Test utilities, fixtures, mock runtime | Python |
| **iron_dashboard** | Control Panel web UI | Vue.js |

### Layer Organization

```
Foundation:     iron_types, iron_cost, iron_telemetry
Infrastructure: iron_runtime_state
Feature:        iron_safety, iron_reliability, iron_secrets, iron_token_manager
Specialized:    iron_control_schema
Integration:    iron_control_api, iron_runtime
Application:    iron_cli, iron_cli_py, iron_sdk, iron_testing
Frontend:       iron_dashboard
```

---

## Documentation

**Governance Compliance:** ✅ 100% (22 directories, 11 Design Collections, 68 numbered files)

**Collections:**
- **[Architecture](docs/architecture/)** (6) - Execution models, layers, boundaries, data flow, integration, **budget control protocol**
- **[Protocol](docs/protocol/)** (8) - REST API, WebSocket, MCP integration, budget control, token management, authentication, user management
- **[Security](docs/security/)** (4) - Threat model, isolation, credential flow, audit
- **[Capabilities](docs/capabilities/)** (8) - Runtime, LLM control, sandbox, safety, credentials, MCP, observability, data
- **[Integration](docs/integration/)** (4) - LLM providers, secrets, identity, observability
- **[Deployment](docs/deployment/)** (5) - Packages, actors, distribution, scaling
- **[Technology](docs/technology/)** (4) - Rust, PyO3, infrastructure, dependencies
- **[Features](docs/features/)** (6) - CLI architecture, token management, user management
- **[Principles](docs/principles/)** (5) - Design, quality, errors, testing, workflow
- **[Constraints](docs/constraints/)** (4) - Technical, business, scope, trade-offs
- **[Decisions](docs/decisions/)** (7) - ADRs (Architecture Decision Records)

**Quick Links:**
- **[Getting Started](docs/getting_started.md)** - Choose your path: Python Developer, Control Panel Admin, or Platform Contributor
- **[contributing.md](contributing.md)** - Contributor workflow guide

**Key Documentation:**
- **[Budget Control Protocol](docs/architecture/006_budget_control_protocol.md)** - Two-token system (IC Token, IP Token), budget borrowing, incremental allocation
- **[Execution Models](docs/architecture/001_execution_models.md)** - Client-Side, Server-Side, Control Panel-Managed
- **[Data Flow](docs/architecture/004_data_flow.md)** - 11-step request flow with token translation
- **[Protocol Collection](docs/protocol/readme.md)** - Communication protocols
- **[Vocabulary](docs/vocabulary.md)** - Canonical terminology

**Complete Index:** [docs/readme.md](docs/readme.md) - All documentation organized by category

---

## Installation & Usage

### For Python Developers (Using Iron SDK)

**99% of users - you just want to protect your AI agents:**

**Prerequisites:**
- Python 3.9+ (`python --version`)

```bash
pip install iron-sdk
```

That's it! No Rust compiler, no cargo, no building required.

**Quick Start:**
```python
from iron_sdk import protect_agent, BudgetConfig

@protect_agent(budget=BudgetConfig(max_usd=50.0))
def my_agent(prompt: str) -> str:
    return llm.chat(prompt)
```

**Next Steps:**
- [Getting Started Guide](docs/getting_started.md) - Complete walkthrough for all users
- [SDK Documentation](module/iron_sdk/readme.md) - Full API reference
- [Examples](module/iron_sdk/examples/) - Runnable code examples

### For Control Panel Admins

**Deploying the Control Panel service for your team:**

```bash
# Clone repository
git clone https://github.com/iron-cage/iron_runtime.git
cd iron_runtime/dev

# Configure secrets
cp .env.example .env
# Generate and add: POSTGRES_PASSWORD, JWT_SECRET, IRON_SECRETS_MASTER_KEY
# (See .env.example for generation commands)

# Deploy with Docker Compose
docker compose up -d

# Access dashboard
# http://localhost:8080
```

**Next Steps:**
- [Getting Started Guide](docs/getting_started.md) § Deploy Control Panel - Complete walkthrough
- [Deployment Guide](docs/deployment_guide.md) - Production deployment and troubleshooting
- [Docker Compose Architecture](docs/deployment/006_docker_compose_deployment.md) - Design details

### For Platform Contributors

<details>
<summary>Building from source (click to expand - only needed for contributing to Iron Cage itself)</summary>

```bash
# Build all Rust modules
cargo build --release

# Run tests
cargo nextest run --all-features

# Build Control Panel UI
cd module/iron_dashboard
npm install
npm run build

# Setup Python modules for development
cd module/iron_sdk
uv sync  # Installs dependencies and creates .venv
```

See [`contributing.md`](contributing.md) for contributor workflow.
</details>

## Status

**Version:** 0.1.0
**Status:** Migration in progress from iron_cage monorepo

## License

Apache-2.0
