<div align="center">

# Iron Cage - AI Control Panel

Most AI frameworks assume R&D. We assume production in regulated industries.

<p align="center">
<a href="https://github.com/iron-cage/sdk/actions"><img src="https://img.shields.io/github/actions/workflow/status/iron-cage/sdk/deploy.yaml?branch=master&label=build&color=E5E7EB" alt="build" /></a>
&nbsp;
<a href="https://github.com/iron-cage/sdk/blob/master/license"><img src="https://img.shields.io/badge/license-MIT-E5E7EB.svg" alt="license" /></a>
&nbsp;
<a href="https://ironcage.ai"><img src="https://img.shields.io/badge/website-ironcage.ai-E5E7EB.svg" alt="website" /></a>
&nbsp;
<a href="https://docs.rs/iron_runtime"><img src="https://img.shields.io/badge/docs.rs-iron__runtime-E5E7EB.svg" alt="docs" /></a>
<br>
<a href="https://discord.gg/aR5fujCZhv"><img src="https://img.shields.io/badge/discord-join-E5E7EB.svg" alt="discord" /></a>
&nbsp;
<a href="https://github.com/iron-cage"><img src="https://img.shields.io/badge/github-iron--cage-E5E7EB.svg" alt="github" /></a>
&nbsp;
<a href="https://x.com/ironcageai"><img src="https://img.shields.io/badge/twitter-@ironcageai-E5E7EB.svg" alt="twitter" /></a>
</p>

</div>

---

## Table of Contents

- [Overview](#overview)
- [Architecture at a Glance](#architecture-at-a-glance)
- [Workspace Organization](#workspace-organization)
- [Documentation](#documentation)
- [Installation & Usage](#installation--usage)
  - [For Python Developers (Using Iron SDK)](#for-python-developers-using-iron-sdk)
  - [For Control Panel Admins](#for-control-panel-admins)
  - [For Platform Contributors](#for-platform-contributors)
- [Status](#status)
- [License](#license)


## Overview

Iron Runtime provides the core runtime modules for AI agent management including safety controls, cost tracking, reliability patterns, and the Control Panel web application.


## Architecture at a Glance

This diagram shows Iron Cage's three-boundary architecture at the highest level, designed for architecture reviews and technical stakeholders.

![Iron Cage Architecture - Three-Boundary Model](asset/architecture3.webp)

**Visual Guide:**
- **Left (Developer Zone):** Agent, iron_sdk, Runtime (Safety/Cost/Audit), and Gateway ALL run 100% locally
- **Middle (Your Cloud - Management Plane):** Control Panel for setup and management - **NOT IN DATA PATH**
- **Right (Provider Zone):** LLM provider receives only prompts with IP Token
- **Critical:** Request/response arrows go **AROUND** Control Panel (bypass architecture)

**Key Points:**
- **Left (Developer Machine):** Agent, iron_sdk, Runtime (Safety/Cost/Audit), and Gateway ALL run 100% locally. Nothing leaves your machine except prompts after local validation.
- **Middle (Your Cloud):** Control Panel ONLY for setup and management. NOT in request path. Handles user management, token generation, and analytics.
- **Right (Third Party):** LLM provider receives only prompts with IP Token. Never sees your code, data, or IC Token.

**Business Value:**
1. **Privacy First:** Agent code and data never leave developer machines (100% local execution)
2. **Cost Control:** Centralized budget enforcement prevents runaway AI spending
3. **Security:** Input/output validation protects against prompt injection, PII leaks, and credential exposure
4. **Compliance:** Complete audit trail for regulatory requirements and accountability
5. **Flexibility:** Use any LLM provider (OpenAI, Anthropic, custom) without vendor lock-in


## Workspace Organization

<details>
<summary>Module Responsibilities & Boundaries (click to expand - 19 modules across 7 layers)</summary>

| Entity | Responsibility | Input→Output | Scope | Out of Scope |
|--------|----------------|--------------|-------|--------------|
| iron_types | Shared type definitions | Raw data → Validated typed structures | Core types (AgentId, ProviderId, Budget) | Business logic, persistence |
| iron_cost | Cost calculation and tracking | Provider API responses → Cost metrics | Token counting, pricing, budgets | Actual API calls, rate limiting |
| iron_telemetry | Observability and tracing | Application events → Structured logs/metrics | Tracing, metrics, logging | Log aggregation, alerting |
| iron_runtime_analytics | Usage analytics and reporting | Raw usage data → Analytics reports | Aggregation, trending, forecasting | Data storage, visualization |
| iron_runtime_state | Runtime state management | State transitions → Persisted state | Agent state, session state | Long-term persistence, caching |
| iron_test_db | Test database infrastructure | Test requirements → Isolated DB instances | SQLite test instances, fixtures | Production databases, migrations |
| iron_safety | Content safety and moderation | User input → Safety verdicts | Content filtering, policy enforcement | Policy definition, training |
| iron_reliability | Resilience patterns | Unreliable operations → Reliable execution | Circuit breakers, retries, timeouts | Actual service implementations |
| iron_secrets | Secrets management | Plaintext secrets → Encrypted storage | Encryption, key management, access control | Secret generation, rotation policies |
| iron_token_manager | API token management | Provider keys → Managed tokens | Token lifecycle, usage tracking, limits | Token generation, provider integration |
| iron_control_api | HTTP API for control plane | HTTP requests → JSON responses | REST endpoints, auth, validation | CLI implementations, SDK |
| iron_runtime | Agent runtime and LLM routing | Agent requests → Provider API calls | Request translation, response parsing | Actual LLM provider SDKs |
| iron_cli | Command-line interface | User commands → API operations | CLI parsing, formatting, output | API implementation, business logic |
| iron_control_schema | Database schema definitions | Schema changes → SQL migrations | Table definitions, migrations, indexes | Query logic, application code |

### Layer Organization

```
Foundation:     iron_types, iron_cost, iron_telemetry, iron_runtime_analytics
Infrastructure: iron_runtime_state
Feature:        iron_safety, iron_reliability, iron_secrets, iron_token_manager
Specialized:    iron_control_schema
Integration:    iron_control_api, iron_runtime
Application:    iron_cli, iron_cli_py, iron_sdk, iron_testing
Frontend:       iron_dashboard
```

</details>


## Documentation

**Governance Compliance:** ✅ Complete

**Collections:**
- **[Architecture](docs/architecture/)** - Execution models, layers, boundaries, data flow, integration, **budget control protocol**
- **[Protocol](docs/protocol/)** - REST API, WebSocket, MCP integration, budget control, token management, authentication, user management
- **[Security](docs/security/)** - Threat model, isolation, credential flow, audit
- **[Capabilities](docs/capabilities/)** - Runtime, LLM control, sandbox, safety, credentials, MCP, observability, data
- **[Integration](docs/integration/)** - LLM providers, secrets, identity, observability
- **[Deployment](docs/deployment/)** - Packages, actors, distribution, scaling
- **[Technology](docs/technology/)** - Rust, PyO3, infrastructure, dependencies
- **[Features](docs/features/)** - CLI architecture, token management, user management
- **[Principles](docs/principles/)** - Design, quality, errors, testing, workflow
- **[Constraints](docs/constraints/)** - Technical, business, scope, trade-offs
- **[Decisions](docs/decisions/)** - ADRs (Architecture Decision Records)

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


## Installation & Usage

### For Python Developers (Using Iron SDK)

> [!TIP]
> **99% of users** - you just want to protect your AI agents

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

> [!IMPORTANT]
> **Deploying the Control Panel service for your team**

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
**Status:** Active development


## License

MIT
