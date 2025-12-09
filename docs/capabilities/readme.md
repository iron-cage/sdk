# Capabilities

**Purpose:** Conceptual overview of Iron Cage platform capabilities - the ideas and approaches behind each functional area.

---

## What is a Capability?

A capability is a high-level functional ability the platform provides, expressed from the user's perspective. Capabilities answer "WHAT can users accomplish?" independent of implementation details.

---

## Directory Responsibilities

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| **agent_runtime.md** | Describe agent lifecycle management capability | Runtime need → Capability description | Agent orchestration (deploy, scale, recover), health monitoring, framework integration (LangChain, CrewAI), PyO3 bridge | NOT SDK details (→ module/iron_sdk/spec.md), NOT runtime implementation (→ module/iron_runtime/spec.md), NOT other capabilities (→ other .md files) |
| **llm_access_control.md** | Explain LLM gateway and budget enforcement | Access control need → Governance model | Unified API across providers, real-time token tracking, budget limits with automatic cutoffs, cost attribution | NOT implementation (→ module/iron_cost/spec.md), NOT observability (→ observability.md), NOT safety (→ ai_safety_guardrails.md) |
| **safe_execution.md** | Define sandboxed code execution capability | Isolation need → Sandbox approach | Container isolation, syscall filtering (seccomp), resource limits (cgroups), network restrictions | NOT implementation (→ module/iron_sandbox/), NOT isolation architecture (→ docs/security/isolation_layers.md), NOT runtime (→ agent_runtime.md) |
| **ai_safety_guardrails.md** | Document AI-specific protection mechanisms | Safety need → Guardrail description | Input validation (prompt injection), output filtering (PII, secrets), action authorization, policy engine | NOT implementation (→ module/iron_safety/spec.md), NOT observability (→ observability.md), NOT threat model (→ docs/security/threat_model.md) |
| **credential_management.md** | Explain unified secrets access capability | Credential need → Management approach | Encrypted storage, on-demand delivery, usage tracking, backend integration (Vault, AWS, Azure, GCP) | NOT implementation (→ module/iron_secrets/spec.md), NOT flow details (→ docs/security/credential_flow.md), NOT access control (→ llm_access_control.md) |
| **mcp_integration.md** | Define Model Context Protocol tool integration | Tool access need → MCP integration | MCP server registry, auto-discovery, policy-based access control, security wrapper for tool invocations | NOT implementation (→ module specifications), NOT runtime (→ agent_runtime.md), NOT safety (→ ai_safety_guardrails.md) |
| **observability.md** | Describe AI-native monitoring capability | Observability need → Monitoring approach | Agent traces (reasoning chains), LLM metrics (tokens, latency, cost), safety event logging, cost attribution | NOT implementation (→ module/iron_telemetry/spec.md), NOT audit model (→ docs/security/audit_model.md), NOT cost tracking (→ llm_access_control.md) |
| **enterprise_data_access.md** | Explain unified data infrastructure for RAG | Data access need → Integration approach | Enterprise connectors (Salesforce, Jira, databases), automated ETL, vector store, real-time sync, access policies | NOT safety (→ ai_safety_guardrails.md), NOT implementation (→ module specifications), NOT execution (→ agent_runtime.md) |

---

## The Eight Capabilities

| # | Capability | Core Concept |
|---|------------|--------------|
| 1 | [Agent Runtime](agent_runtime.md) | Lifecycle management for AI agent workloads |
| 2 | [LLM Access Control](llm_access_control.md) | Centralized gateway with budget enforcement |
| 3 | [Safe Execution](safe_execution.md) | Isolated sandbox environments for code |
| 4 | [AI Safety Guardrails](ai_safety_guardrails.md) | Defense-in-depth input/output protection |
| 5 | [Credential Management](credential_management.md) | Unified secrets access for AI agents |
| 6 | [MCP Integration](mcp_integration.md) | Zero-config tool access via MCP protocol |
| 7 | [Observability](observability.md) | AI-native monitoring and tracing |
| 8 | [Enterprise Data Access](enterprise_data_access.md) | Unified data infrastructure for RAG |

---

## Capability Relationships

```
┌─────────────────────────────────────────────────────┐
│                   AI AGENTS                          │
│            (LangChain, CrewAI, Custom)              │
└─────────────────────┬───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│              IRON CAGE PLATFORM                      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │
│  │ Safety  │ │   LLM   │ │ Sandbox │ │  Data   │   │
│  │Guardrails│ │ Control │ │  Exec   │ │ Access  │   │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘   │
│       └───────────┴───────────┴───────────┘         │
│                       │                              │
│  ┌────────────────────▼─────────────────────────┐   │
│  │  Credentials │ MCP Tools │ Observability     │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

---

**Last Updated:** 2025-12-08
