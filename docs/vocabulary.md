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

### Architecture

| Term | Definition |
|------|------------|
| **Control Plane** | Management layer: API Gateway, Dashboard, Scheduler |
| **Data Plane** | Processing layer: Safety, Cost, Reliability, Observability services |
| **Agent Runtime** | Execution layer: Agent pods, SDK, Sandbox |
| **Local Execution** | Agent runs on developer's machine (primary). Control Panel manages budget via IC Token protocol. See [architecture/001](architecture/001_execution_models.md) |
| **Server Execution** | Agent runs on Iron Cage servers (future, post-pilot). Control Panel manages budget identically to local execution |
| **Control Panel** | ALWAYS present standalone admin service. Admin allocates budgets, manages developers, stores IP Tokens, monitors spending. See [architecture/003](architecture/003_service_boundaries.md) |
| **Gateway** | Central orchestrator that routes requests through processing layers |
| **Layer Model** | Six processing layers: Safety, Cost, Reliability, Provider, Output Safety, Observability |
| **Service Boundaries** | Separation between Control Plane, Data Plane, and Agent Runtime |
| **Data Flow** | End-to-end request journey from user input to LLM response |
| **Execution Models** | Where agents execute: Local (primary) vs Server (future, post-pilot) |
| **Library Mode** | Default SDK deployment where runtime embedded in-process via PyO3. Developer code: `from iron_sdk import protect_agent`. Overhead: ~0.5ms (FFI). Single process, no separate runtime. See [architecture/008](architecture/008_runtime_modes.md) |
| **Router Mode** | Optional deployment where runtime runs as separate process exposing HTTP API. Two use cases: (1) Non-SDK frameworks (LangChain, CrewAI) point to localhost:8080, (2) iron_sdk optionally configured for HTTP. Same developer code as Library mode for SDK users. Overhead: ~5ms (HTTP). See [architecture/008](architecture/008_runtime_modes.md) |

### Entities

| Term | Definition |
|------|------------|
| **Agent** | AI agent executing on developer's machine. Has exactly one IC Token (1:1), exactly one Agent Budget (1:1, restrictive), can use multiple Inference Providers. Belongs to one Project |
| **Project** | Collection of agents, Inference Provider assignments, and entities. Has exactly one Project Budget (1:1, informative). Owned by admin or team |
| **Master Project** | Special project containing ALL resources (all agents, all Inference Providers, all budgets). Admin-only. Has Master Budget (informative). MUST be in Pilot |
| **IP** | Inference Provider entity (OpenAI, Anthropic, etc.). Has IP Budget (informative), has IP Token(s). Can be assigned to multiple agents |
| **Agent Budget** | Restrictive budget (ONLY budget that blocks requests). 1:1 with agent. Hard limit enforcement |
| **Project Budget** | Informative budget (statistics only, no blocking). 1:1 with project. Shows aggregate agent spending |
| **IP Budget** | Informative budget (statistics only, no blocking). Per Inference Provider. Shows provider spending |
| **Master Budget** | Informative budget (statistics only, no blocking). Part of master project. Shows all spending across all projects |
| **Budget Control** | Agents are the ONLY way to control budget. Agent budget blocks requests (restrictive). All other budgets (project, Inference Provider, master) are informative only (show spending, can't block) |

### Resources

| Term | Definition |
|------|------------|
| **Resource** | REST API endpoint or endpoint group exposed by Control Panel. Maps to domain entities or operations. See [architecture/009](architecture/009_resource_catalog.md) |
| **Entity Resource** | REST resource with 1:1 or 1:N mapping to domain entity, supporting CRUD operations. Plural names (/api/tokens, /api/projects). Example: /api/tokens → IC Token entity |
| **Operation Resource** | REST resource exposing operations/actions not mapping directly to single entity CRUD. Action-oriented, often POST-only. Example: /api/auth → login/logout operations |
| **Analytics Resource** | Read-only REST resource providing aggregated/derived metrics from multiple entities. GET-only, statistical nature. Example: /api/analytics → usage, spending, and performance metrics |
| **Configuration Resource** | REST resource managing system-level configuration and constraints. Admin-only access, affects multiple entities. Example: /api/limits → Agent Budget limits |
| **Resource Catalog** | Exhaustive inventory of all REST API resources with entity mapping, authentication patterns, and certainty classification. See [architecture/009](architecture/009_resource_catalog.md) |
| **User-Facing Resource** | REST resource accessible via CLI and Control Panel dashboard. Requires User Token authentication. Has CLI-API parity |
| **Agent-Facing Resource** | REST resource used by iron_runtime for agent operations. Requires IC Token authentication. No CLI mapping (e.g., /api/budget/*) |

### Roles

| Term | Definition |
|------|------------|
| **Admin** | Full Control Panel access. Allocates budgets, creates developer accounts, monitors all spending, manages IP Tokens |
| **Super User** | Developer + read-only Control Panel dashboard access (own budgets only). Cannot allocate budgets or see other developers |
| **Developer** | Regular user managed by admin. Runs agents with IC Token, views usage via CLI + Dashboard (read-only own usage). Can select model and Inference Provider among allowed |

### Tokens

| Term | Definition |
|------|------------|
| **IC Token** | Internal Control Token - Developer-visible JWT for agent authentication. 1:1 with agent (one agent = one IC token, can't share). Developer can regenerate their own IC Token (replaces existing). Admin can regenerate any IC Token. Lifetime: Until agent deleted (long-lived, no auto-expiration). See [protocol/005](protocol/005_budget_control_protocol.md) |
| **User Token** | Control Panel CLI/Dashboard authentication token. Different from IC Token (agent auth). Users can have multiple active User Tokens. Developer can regenerate own, admin can regenerate any. Lifetime: 30 days default |
| **IP Token** | Inference Provider Token - LLM provider API key (sk-proj-, sk-ant-). Stored in Control Panel vault, NEVER exposed to developer. Runtime receives encrypted copy from Control Panel. Session-only lifetime. See [protocol/005](protocol/005_budget_control_protocol.md) |
| **Token Translation** | Process where Runtime replaces IC Token with IP Token in LLM requests. Latency: <1ms. IP Token decrypted, used, then zeroed. See [protocol/005](protocol/005_budget_control_protocol.md) |
| **API Token** | Persistent authentication token for Control Panel dashboard and automation scripts. Format: `apitok_` prefix (e.g., `apitok_xyz789abc123...`). SAME-AS-USER scope (inherits user permissions). Primary use: Dashboard access. Secondary use: Admin automation. Token value shown only once at creation (GitHub pattern). Different from IC Token (agent auth) and User Token (session auth). See [protocol/014](protocol/014_api_tokens_api.md) |

### Budget Management

| Term | Definition |
|------|------------|
| **Budget Allocation** | Total budget admin assigns to agent (e.g., $100) in Control Panel. Tracked centrally in database. |
| **Budget Portion** | Incremental amount Runtime borrows from total (e.g., $10). Enables real-time control without upfront full budget transfer. Default: $10 per borrow. |
| **Budget Borrowing** | Protocol where Runtime requests budget portions from Control Panel. Borrows $10 chunks from total allocation. Trigger: remaining < $1. See [protocol/005](protocol/005_budget_control_protocol.md) |
| **Lease ID** | Unique identifier for budget portion allocation. Tracks which $10 portion Runtime currently using. Changes with each borrow (lease-001, lease-002, etc.). |
| **Budget Threshold** | Remaining budget level triggering borrow request. Default: $1.00. When remaining < threshold, Runtime requests more. |
| **Incremental Budget** | Strategy of allocating budget in portions ($10) rather than full amount ($100) upfront. Benefits: Real-time enforcement, admin can stop mid-session, limits exposure if IC Token stolen. |

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
| **Package 1: Control Panel** | Docker image with iron_control_api + iron_dashboard |
| **Package 2: Marketing Site** | Static website (ironcage.ai) |
| **Package 3: Agent Runtime** | PyPI package (iron-sdk - user installs) with Python SDK; automatically includes iron-cage (PyPI wheel with Rust runtime binary, internal dependency) |
| **Package 4: Sandbox** | PyPI wheel (iron-sandbox) with OS isolation |
| **Package 5: CLI Tools** | Binary (iron_cli) + PyPI wrapper (iron-cli-py) |

### Modules (20 Total)

| Term | Definition |
|------|------------|
| **iron_control_api** | REST API + WebSocket server (Rust/axum) |
| **iron_cli** | Binary CLI for token/usage/limits management (Rust) |
| **iron_cli_py** | Python CLI wrapper delegating to iron_cli (Python) |
| **iron_control_schema** | PostgreSQL schema for Control Panel (Rust, spec-only) |
| **iron_cost** | Budget tracking, token counting (Rust, crates.io) |
| **iron_dashboard** | Web control panel (Vue 3 + TypeScript) |
| **iron_reliability** | Circuit breaker patterns, retry logic (Rust) |
| **iron_runtime** | Agent orchestrator + PyO3 bridge (Rust) |
| **iron_safety** | PII detection, prompt injection blocking (Rust) |
| **iron_sandbox** | OS-level isolation legacy bindings (Rust, deprecated) |
| **iron_sandbox_core** | OS sandboxing core with Landlock, seccomp (Rust) |
| **iron_sandbox_py** | Python sandbox API (Python + PyO3) |
| **iron_sdk** | Python SDK with decorators (Python, includes examples/) |
| **iron_secrets** | Encrypted secrets management (Rust) |
| **iron_site** | Marketing website (Vue 3 + TypeScript, static) |
| **iron_runtime_state** | Local state management with SQLite (Rust) |
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
| **Spec-only Module** | Module with specification but no implementation (e.g., iron_control_schema) |

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
| **Budget Reporting** | Runtime reports token usage to Control Panel for centralized tracking. Pilot: per-request (simpler). Production: batched (every 10 requests, optimized). See [constraints/004: Trade-offs](constraints/004_trade_offs.md#cost-vs-reliability) |
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

*Last Updated: 2025-12-10*
