# Capabilities

**Purpose:** Conceptual overview of Iron Cage platform capabilities - the ideas and approaches behind each functional area.

---

## What is a Capability?

A capability is a high-level functional ability the platform provides, expressed from the user's perspective. Capabilities answer "WHAT can users accomplish?" independent of implementation details.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 001 | **001_agent_runtime.md** | Describe agent lifecycle management capability (deployment, scaling, recovery, framework integration) |
| 002 | **002_llm_access_control.md** | Explain LLM gateway and budget enforcement (unified API, real-time tracking, automatic cutoffs, cost attribution) |
| 003 | **003_safe_execution.md** | Define sandboxed code execution capability (container isolation, syscall filtering, resource limits, network restrictions) |
| 004 | **004_ai_safety_guardrails.md** | Document AI-specific protection mechanisms (input validation, output filtering, action authorization, policy engine) |
| 005 | **005_credential_management.md** | Explain unified secrets access capability (encrypted storage, on-demand delivery, backend integration) |
| 006 | **006_mcp_integration.md** | Define Model Context Protocol tool integration (server registry, auto-discovery, policy-based access control) |
| 007 | **007_observability.md** | Describe AI-native monitoring capability (agent traces, LLM metrics, safety event logging, cost attribution) |
| 008 | **008_enterprise_data_access.md** | Explain unified data infrastructure for RAG (enterprise connectors, ETL, vector store, real-time sync) |

---

## Capability Collection

| ID | Name | Purpose |
|----|------|---------|
| 001 | [Agent Runtime](001_agent_runtime.md) | Lifecycle management for AI agent workloads |
| 002 | [LLM Access Control](002_llm_access_control.md) | Centralized gateway with budget enforcement |
| 003 | [Safe Execution](003_safe_execution.md) | Isolated sandbox environments for code |
| 004 | [AI Safety Guardrails](004_ai_safety_guardrails.md) | Defense-in-depth input/output protection |
| 005 | [Credential Management](005_credential_management.md) | Unified secrets access for AI agents |
| 006 | [MCP Integration](006_mcp_integration.md) | Zero-config tool access via MCP protocol |
| 007 | [Observability](007_observability.md) | AI-native monitoring and tracing |
| 008 | [Enterprise Data Access](008_enterprise_data_access.md) | Unified data infrastructure for RAG |

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

**Last Updated:** 2025-12-09

**Note:** This directory follows Design Collections format with NNN_ numbered instances (001-008) per documentation.rulebook.md standards.
