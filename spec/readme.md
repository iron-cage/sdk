# Specifications Directory

**Purpose:** Product specifications and requirements for Iron Cage platform, defining WHAT to build.

---

## Directory Responsibilities

| Directory/File | Responsibility | In Scope | Out of Scope |
|----------------|----------------|----------|--------------|
| **capabilities/** | Product specifications for all 8 capabilities | Features, pricing, market positioning, build priorities | Technical implementation (→ docs/architecture.md), Business strategy (→ business/) |
| **requirements.md** | Technical requirements specification | Functional requirements (FR-x.y), non-functional requirements | Product specs (→ capabilities/), Architecture (→ docs/) |

---

## Contents

### Capability Specifications (`capabilities/`)

Product specifications for all 8 Iron Cage capabilities with build priorities:

| File | Capability | Score | Build Priority |
|------|------------|-------|----------------|
| capability_1_enterprise_data_access.md | Enterprise Data Access for AI | 92/100 | 🥇 BUILD FIRST |
| capability_2_ai_safety_guardrails.md | AI Safety Guardrails | 85/100 | 🥈 BUILD SECOND |
| capability_3_llm_access_control.md | Unified LLM Access Control | 58/100 | Platform Component |
| capability_4_safe_execution.md | Safe Execution Environment | 55/100 | Platform Component |
| capability_5_credential_management.md | Credential Management | 42/100 | Thin Wrapper |
| capability_6_observability.md | Comprehensive Observability | 35/100 | Platform Component |
| capability_7_mcp_integration.md | Zero-Config MCP | 30/100 | Thin Wrapper |
| capability_8_agent_runtime.md | Production Agent Runtime | 18/100 | DO NOT BUILD |

See [`capabilities/readme.md`](capabilities/readme.md) for detailed guidance.

### Requirements (`requirements.md`)

Technical requirements specification with:
- Functional requirements (FR-1.x through FR-1.7)
- Non-functional requirements (performance, security, compliance)
- Requirements-to-capability mapping

---

## Relationship to Other Directories

| Directory | Relationship | Content Type |
|-----------|--------------|--------------|
| **business/** | WHY to build | Strategic rationale, market analysis |
| **spec/** (this) | WHAT to build | Product features, requirements |
| **docs/** | HOW to build | Architecture, deployment, implementation |
| **research/** | CONTEXT | Competitive analysis, market research |

---

**Last Updated:** 2025-12-08
