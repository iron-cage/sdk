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

## Vocabulary

### Platform

| Term | Definition |
|------|------------|
| **Iron Cage** | AI agent governance platform providing safety, cost control, and reliability for enterprise AI agents |
| **iron_runtime** | Repository containing Control Panel, Agent Runtime, and runtime services (12 modules) |
| **iron_cage** | Repository containing OS sandboxing, CLI tools, and foundation modules (10 modules) |

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

### Deployment Packages (6 Total)

| Term | Definition |
|------|------------|
| **Package 1: Control Panel** | Docker image with iron_api + iron_dashboard |
| **Package 2: Marketing Site** | Static website (ironcage.ai) |
| **Package 3: Agent Runtime** | PyPI wheel (iron-cage) with SDK and core services |
| **Package 4: Sandbox** | PyPI wheel (iron-sandbox) with OS isolation |
| **Package 5: CLI Tool** | Binary (iron_cli) for token management |
| **Package 6: Python CLI** | PyPI wheel (iron-cli-py) developer experience wrapper |

### Modules

| Term | Definition |
|------|------------|
| **iron_api** | REST API + WebSocket server (Rust/axum) |
| **iron_dashboard** | Web control panel (Vue 3 + TypeScript) |
| **iron_sdk** | Python SDK with decorators for agent protection |
| **iron_safety** | PII detection, prompt injection blocking |
| **iron_cost** | Budget tracking, token counting |
| **iron_reliability** | Circuit breaker patterns, retry logic |
| **iron_sandbox** | OS-level isolation (Landlock, seccomp) |
| **iron_cli** | Binary CLI for token/usage/limits management |
| **iron_cli_py** | Python CLI wrapper delegating to iron_cli |
| **iron_types** | Foundation types, errors, Result types (crates.io) |
| **iron_telemetry** | Unified logging with tracing (crates.io) |

### Technology

| Term | Definition |
|------|------------|
| **PyO3 FFI** | Rust-Python bridge enabling <0.1ms in-process calls |
| **Wrapper Pattern** | iron_cli_py delegates to iron_cli binary for single source of truth |
| **OTLP** | OpenTelemetry Protocol for metrics/traces export |

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

---

*Last Updated: 2025-12-08*
