# Iron Cage Vocabulary

**Purpose:** Canonical definitions for terms used across the Iron Cage platform.

**Responsibility:** Single source of truth for project terminology. All documentation should use these terms consistently.

**In Scope:**
- Platform concepts and architectural terms
- Module and package names
- Capability and feature terminology
- Abbreviations and acronyms

**Out of Scope:**
- API-specific terms (see module specs)
- External technology definitions (Rust, Python, K8s)
- Business/market terminology

---

### Platform

| Term | Definition |
|------|------------|
| **Iron Cage** | AI agent governance platform providing safety, cost control, and reliability for enterprise AI agents |
| **iron_runtime** | Repository containing Control Panel, Agent Runtime, runtime services (10 modules) |
| **iron_cage** | Repository containing OS sandboxing, CLI tools, foundation modules (10 modules) |
| **iron_site** | Repository containing marketing website (Vue-based static site) |

### Architecture

| Term | Definition |
|------|------------|
| **Control Plane** | Management layer: API Gateway, Dashboard, Scheduler |
| **Data Plane** | Processing layer: Safety, Cost, Reliability, Observability services |
| **Agent Runtime** | Execution layer: Agent pods, SDK, Sandbox |
| **Model A (Client-Side)** | Primary execution model where agent runs on user's machine (95% of users) |
| **Model B (Server-Side)** | Optional execution model where agent runs on Iron Cage infrastructure (5% of users) |
| **Gateway** | Central orchestrator that routes requests through processing layers |
| **Two-Repo Model** | Architecture split: iron_runtime (frequent changes) + iron_cage (stable foundation) |
| **Layer Model** | Six processing layers: Safety, Cost, Reliability, Provider, Output Safety, Observability |
| **Service Boundaries** | Separation between Control Plane, Data Plane, and Agent Runtime |
| **Data Flow** | End-to-end request journey from user input to LLM response |
| **Execution Models** | Where agents execute: client-side (primary) vs server-side (optional) |

### Capabilities (8 Total)

| Term | Definition |
|------|------------|
| **AI Safety Guardrails** | Input/output validation: prompt injection detection, PII redaction |
| **Cost Management** | Budget tracking, token counting, spending limits |
| **Reliability Engineering** | Circuit breakers, retry logic, fallback chains |
| **Secure Code Execution** | OS-level sandboxing via Landlock, seccomp-bpf |
| **Credential Management** | Just-in-time secret injection with scoped access |
| **Audit & Compliance** | Immutable logging for SOC 2, GDPR, HIPAA |
| **Unified Observability** | OpenTelemetry export to Datadog, Grafana, etc. |
| **Multi-Provider LLM Access** | Unified API across OpenAI, Anthropic, Azure, Google |

### Deployment Packages (5 Total)

| Term | Definition |
|------|------------|
| **Package 1: Control Panel** | Docker image with iron_api + iron_dashboard |
| **Package 2: Marketing Site** | Static website (ironcage.ai) |
| **Package 3: Agent Runtime** | PyPI wheel (iron-cage) with SDK and core services |
| **Package 4: Sandbox** | PyPI wheel (iron-sandbox) with OS isolation |
| **Package 5: CLI Tools** | Binary (iron_cli) + PyPI wrapper (iron-cli-py) |

### Modules (20 Total)

| Term | Definition |
|------|------------|
| **iron_api** | REST API + WebSocket server (Rust/axum) |
| **iron_cli** | Binary CLI for token/usage/limits management (Rust) |
| **iron_cli_py** | Python CLI wrapper delegating to iron_cli (Python) |
| **iron_control_store** | PostgreSQL schema for Control Panel (Rust, spec-only) |
| **iron_cost** | Budget tracking, token counting (Rust, crates.io) |
| **iron_dashboard** | Web control panel (Vue 3 + TypeScript) |
| **iron_lang** | AI agent data protocol (Rust) |
| **iron_reliability** | Circuit breaker patterns, retry logic (Rust) |
| **iron_runtime** | Agent orchestrator + PyO3 bridge (Rust) |
| **iron_safety** | PII detection, prompt injection blocking (Rust) |
| **iron_sandbox** | OS-level isolation legacy bindings (Rust, deprecated) |
| **iron_sandbox_core** | OS sandboxing core with Landlock, seccomp (Rust) |
| **iron_sandbox_py** | Python sandbox API (Python + PyO3) |
| **iron_sdk** | Python SDK with decorators (Python, includes examples/) |
| **iron_secrets** | Encrypted secrets management (Rust) |
| **iron_site** | Marketing website (Vue 3 + TypeScript, static) |
| **iron_state** | Local state management with SQLite (Rust) |
| **iron_telemetry** | Unified logging with tracing (Rust, crates.io) |
| **iron_token_manager** | JWT token management backend (Rust) |
| **iron_types** | Foundation types, errors, Result types (Rust, crates.io) |

### Technology

| Term | Definition |
|------|------------|
| **PyO3 FFI** | Rust-Python bridge enabling <0.1ms in-process calls |
| **Maturin** | Build tool for creating Python wheels from Rust code |
| **Wrapper Pattern** | iron_cli_py delegates to iron_cli binary for single source of truth |
| **OTLP** | OpenTelemetry Protocol for metrics/traces export |
| **SQLite** | Local embedded database for agent state (per-machine) |
| **PostgreSQL** | Centralized database for Control Panel (cloud) |
| **Landlock** | Linux kernel LSM for filesystem access control |
| **seccomp-bpf** | Linux syscall filtering for sandboxing |

### Security

| Term | Definition |
|------|------------|
| **Input Firewall** | Pre-LLM validation layer blocking malicious prompts |
| **Output Firewall** | Post-LLM validation layer redacting sensitive data |
| **Prompt Injection** | Attack where malicious input hijacks agent behavior |
| **PII** | Personally Identifiable Information (names, emails, SSNs) |
| **Fail-Safe** | Default behavior blocks requests when safety service is down |
| **Fail-Open** | Default behavior allows requests when non-critical service is down |
| **Isolation Layers** | Defense in depth: Process, Syscall, Filesystem, Network |

### Process

| Term | Definition |
|------|------------|
| **ADR** | Architecture Decision Record documenting significant decisions |
| **Design Collection** | Directory of focused concept files (~30-50 lines each) |
| **Capability** | High-level platform feature (safety, cost, reliability, etc.) |
| **Spec-only Module** | Module with specification but no implementation (e.g., iron_control_store) |
| **Monorepo** | Current physical state: all modules in single repository |
| **Two-Repo Split** | Target state: iron_runtime and iron_cage as separate repositories |

### Deployment

| Term | Definition |
|------|------------|
| **Pilot Mode** | Single-process deployment for demos and development |
| **Production Mode** | Distributed deployment: Control Panel (cloud) + Agent Runtime (local) |
| **Package** | Deployment unit combining related modules |
| **Foundation Module** | Shared module published to crates.io (iron_types, iron_cost, iron_telemetry) |

### Budget Control

| Term | Definition |
|------|------------|
| **IC Token** | Iron Cage Token - internal token serving as budget ID, visible to developer, used to authenticate with Control Panel |
| **IP Token** | Inference Provider Token - actual LLM provider API key, NOT visible to developer, encrypted in memory, never stored on disk |
| **Budget Allocation** | Total budget assigned to agent by admin via Control Panel (e.g., $100) |
| **Budget Borrowing** | Runtime requests portion of allocated budget from Control Panel (e.g., borrows $10 from $100 total) |
| **Budget Portion** | Chunk of budget borrowed by runtime for local tracking (default $10 per borrow) |
| **Token Translation** | Runtime converts IC Token to IP Token when forwarding requests to LLM provider |
| **Budget Overshoot** | Condition when agent attempts to exceed allocated budget, blocked by runtime |
| **Budget Reporting** | Runtime reports token usage to Control Panel after each LLM request for centralized tracking |
| **Budget Lease** | Borrowed budget portion with runtime tracking, returned/refreshed as needed |
| **Control Panel** | Admin-facing interface for centralized budget control, monitoring, and access management |
| **Agent Runtime** | Developer-facing execution environment for local routing and heavy lifting |

### Token Management

| Term | Definition |
|------|------------|
| **Provider Token Storage** | IP Tokens stored encrypted in memory only, never persisted to disk on developer machine |
| **Token Handshake** | Initial communication where runtime presents IC Token to Control Panel, receives IP Token and budget portion |
| **Budget Refresh** | Runtime requests additional budget portion when current lease depleting |
| **Real-time Tracking** | Runtime reports usage immediately after each request to maintain accurate budget state |
| **Minimal Overhead** | Budget tracking adds <1ms latency to LLM requests |
| **Developer Isolation** | Developer never sees IP Token, only IC Token for authentication |

---

*Last Updated: 2025-12-09*
