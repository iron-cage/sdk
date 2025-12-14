# Architecture: Data Flow

### Scope

This document traces the complete end-to-end journey of a request through the Iron Cage system, from user input to LLM response, with detailed latency analysis and security validation.

**In scope:**
- Request journey through eleven processing steps (IC Token validation → Provider → Response)
- Bidirectional flow architecture (request path, provider processing, response path)
- Latency analysis for Pilot vs Production environments (overhead quantification)
- Security validation layers (input firewall, output firewall, observability)
- Critical path analysis for simple LLM calls vs code execution
- Failure modes and fail-safe/fail-open decisions
- Response processing security guarantees (both input AND output validated)

**Out of scope:**
- Service implementation details → See service-specific documentation
- Specific ML models for PII detection → See Safety service documentation
- Infrastructure deployment topology → See [Deployment](../deployment/readme.md)
- Database schema for cost tracking → See [Technology: Infrastructure Choices](../technology/003_infrastructure_choices.md)
- Detailed protocol specifications → See [Protocol: Budget Control](../protocol/005_budget_control_protocol.md)

### Purpose

**User Need**: Trace a request from user input to LLM response, understanding each transformation and latency contribution.

**Solution**: Bidirectional flow architecture with eleven processing steps spanning three phases (request processing, provider processing, response processing):

**Bidirectional flow with request processing and response processing:**

```
REQUEST PATH (Steps 0-7):
User/Agent → IC Token Val → API → Input Safety → Cost → Tools → Token Translate → Provider
             0.1ms          5ms    50ms          100ms  2000ms    0.5ms          → LLM

PROVIDER PROCESSING:
LLM Provider processes request (3000ms)

RESPONSE PATH (Steps 7a-9):
LLM Provider → Cost Report → Output Safety → Observability → Agent
               0ms (async)   30ms            0ms (async)     ← Response

TOTAL LATENCY: ~5.7 seconds (provider dominates, overhead <1ms)
```

**The eleven processing steps:**
- **Step 0**: IC Token Validation (0.1ms) - Verify token signature and validity
- **Steps 1-2**: API Gateway + Input Firewall (5ms + 50ms) - Permissions, rate limit, prompt injection detection
- **Steps 3-6**: Agent Runtime + Data Access + Tool Proxy + Sandbox (2650ms) - Agent framework routing, RAG, tool authorization, code execution
- **Step 6a**: Token Translation (0.5ms) - Replace IC Token with IP Token for security
- **Step 7**: LLM Gateway (3000ms) - Forward to provider with IP Token
- **Step 7a**: Cost Reporting (0ms async) - Report token usage to Control Panel
- **Step 8**: Output Firewall (50ms) - Secret scanning, PII redaction
- **Step 9**: Observability (0ms async) - Complete request-response logging

**Key Insight**: Security validation occurs on BOTH input and output. Agent never receives unvalidated LLM responses. Response processing (Steps 7a-9) is critical for security - Output Firewall scans for secrets/PII, Cost Reporting tracks budgets, Observability logs complete interaction. Control Panel overhead is minimal (0.6ms total, 0.02% of request time).

**Status**: Specification
**Version**: 1.0.0
**Last Updated**: 2025-12-13

### The Eleven Steps

| Step | Component | Action | Pilot Latency | Production Latency | Protocol |
|------|-----------|--------|--------------|--------------------|----------|
| 0 | IC Token Validation | Verify IC Token signature, validity (not revoked) | <0.1ms | <0.1ms | [protocol/005](../protocol/005_budget_control_protocol.md) |
| 1 | API Gateway | Check permissions, rate limit | 5ms | 5ms | - |
| 2 | Input Firewall | Prompt injection, PII detect | 10ms (Regex) | 50ms (ML) | - |
| 3 | Agent Runtime | Route to agent framework | 100ms | 100ms | - |
| 4 | Data Access | Vector search (if RAG query) | 500ms | 500ms | - |
| 5 | Tool Proxy | Authorize tool, validate params | 50ms | 50ms | - |
| 6 | Sandbox | Execute code (if code tool) | 2000ms | 2000ms | - |
| 6a | Token Translation | Replace IC Token with IP Token | <0.5ms | <0.5ms | [protocol/005](../protocol/005_budget_control_protocol.md) |
| 7 | LLM Gateway | Forward with IP Token, track cost | 3000ms | 3000ms | - |
| 7a | Cost Reporting | Report usage to Control Panel | 5ms (per-request) | 0ms (async batched) | [protocol/005](../protocol/005_budget_control_protocol.md) |
| 8 | Output Firewall | Secret scan, PII redact | 10ms (Regex) | 50ms (ML) | - |
| 9 | Observability | Log, trace (async) | 0ms | 0ms | - |

**Total Overhead:**
- IC Token operations: 0.1ms + 0.5ms = 0.6ms (same for both)
- **Pilot:** Input Safety (10ms) + Cost Reporting (5ms) + Output Safety (10ms) = **~25ms added**
- **Production:** Input Safety (50ms) + Cost Reporting (0ms async) + Output Safety (50ms) = **~100ms added**

**See:** [constraints/004: Trade-offs](../constraints/004_trade_offs.md#latency-budget-summary) for complete latency budget.

### Critical Path

**With Control Panel - Pilot:**
- Simple LLM call: Steps 0, 1, 2, 6a, 7, 7a, 8, 9
- Latency: 0.1ms + 5ms + 10ms + 0.5ms + 3000ms + 5ms + 10ms + 0ms = **~3.031 seconds**

**With Control Panel - Production:**
- Simple LLM call: Steps 0, 1, 2, 6a, 7, 7a, 8, 9
- Latency: 0.1ms + 5ms + 50ms + 0.5ms + 3000ms + 0ms + 50ms + 0ms = **~3.106 seconds**
- With code execution: All steps -> ~5.7 seconds
- **Control Panel overhead: 0.6ms (0.02%)**

---

### Response Processing (Critical for Security)

**The response flows through validation layers BEFORE reaching the agent:**

#### Step 8: Output Firewall (30ms)

**Purpose:** Scan LLM response for sensitive data leakage

**Checks Performed:**
- **Secret Scanning:** Detect API keys, passwords, tokens in response
- **PII Detection:** Find email addresses, phone numbers, SSNs, credit cards
- **Redaction:** Replace detected sensitive data with `[REDACTED]`
- **Logging:** Record what was redacted for audit

**Why Critical:**
- LLMs can accidentally leak secrets from training data
- Agents may include PII in prompts, LLM echoes it back
- Without output scanning, sensitive data reaches agent/logs/UI

**Example:**
```
LLM Response: "Your API key is sk-proj-abc123..."
After Output Firewall: "Your API key is [REDACTED:SECRET]"
```

#### Step 7a: Cost Reporting (0ms perceived, async)

**Purpose:** Report token usage to Control Panel after receiving response

**Process:**
1. Provider response includes: `{"usage": {"total_tokens": 1523}}`
2. Runtime calculates cost: 1523 tokens × $0.03/1K = $0.0457
3. Runtime sends BUDGET_USAGE_REPORT to Control Panel (async, non-blocking)
4. Control Panel updates central tracking
5. Admin sees real-time spending in dashboard

**Why After Response:**
- Token count only known after provider responds
- Reporting doesn't delay agent (async)
- Real-time tracking for budget enforcement

#### Step 9: Observability (0ms, async)

**Purpose:** Log complete request-response for monitoring and debugging

**Data Logged:**
- Request details (model, input tokens, prompt hash)
- Response details (output tokens, response hash, latency)
- Safety events (PII detected, secrets redacted)
- Cost data (tokens, USD amount)
- Timing (per-step latencies)

**Why Last Step:**
- Has complete context (request + response + all processing)
- Async doesn't delay agent
- Comprehensive audit trail

---

### Full Flow Summary

**Request Processing (Secure Input):**
1. Validate IC Token (auth)
2. Check permissions (authz)
3. Scan for prompt injection (safety)
4. Check budget (cost control)
5. Translate IC → IP Token (security)
6. Forward to provider

**Provider Processing:**
7. LLM generates response (3000ms)

**Response Processing (Secure Output):**
8. Report cost to Control Panel (budget tracking)
9. Scan for secrets/PII (output safety)
10. Log full interaction (observability)
11. Return to agent

**Security Guarantee:** Both input AND output are validated. Agent never receives unvalidated LLM responses.

---

### Failure Points

| Failure | Behavior | Rationale |
|---------|----------|-----------|
| Input FW down | Block all | Fail-safe |
| Cost Service down | Allow, log warning | Fail-open for availability |
| LLM provider down | Try fallback | Reliability layer handles |

---

### Cross-References

#### Related Principles Documents
- [Principles: Design Philosophy](../principles/001_design_philosophy.md) - Fail-Safe principle reflected in Input Firewall failure mode, Minimal Dependencies via token translation
- [Principles: Quality Attributes](../principles/002_quality_attributes.md) - Security via bidirectional validation (input + output), Reliability via failure modes, Performance via latency optimization
- [Principles: Development Workflow](../principles/005_development_workflow.md) - Specification-first approach applied to this architecture document

**Related Architecture Documents:**
- [Architecture: Execution Models](001_execution_models.md) - Control Panel context receiving cost reports in Step 7a
- [Architecture: Layer Model](002_layer_model.md) - Six processing layers that these eleven steps implement
- [Architecture: Service Boundaries](003_service_boundaries.md) - Three-plane separation (Control/Data/Runtime) that request flows through
- [Architecture: Service Integration](005_service_integration.md) - How services communicate during these flow steps
- [Architecture: Roles and Permissions](006_roles_and_permissions.md) - Permission checks in Step 1 (API Gateway)
- [Architecture: Entity Model](007_entity_model.md) - IC Token and IP Token entities in Steps 0 and 6a

#### Used By
- [Architecture: Service Integration](005_service_integration.md) - Service communication implementing this flow
- [Constraints: Trade-offs](../constraints/004_trade_offs.md) - Latency budget analysis references this flow
- Implementation documentation - Request processing implementation

#### Dependencies
- [Protocol: Budget Control Protocol](../protocol/005_budget_control_protocol.md) - IC Token validation (Step 0), Token translation (Step 6a), Cost reporting (Step 7a)
- [Constraints: Trade-offs](../constraints/004_trade_offs.md) - Complete latency budget analysis
- [Technology: Infrastructure Choices](../technology/003_infrastructure_choices.md) - Cache and database technology for cost tracking

#### Implementation
- Step 0: IC Token validation service
- Steps 1-2: API Gateway, Input Firewall (Safety service)
- Steps 3-6: Agent Runtime (LangChain/CrewAI pods), Data Access service, Tool Proxy service, Sandbox service
- Step 6a: Token translation middleware
- Step 7: LLM Gateway service
- Step 7a: Cost reporting service → Control Panel
- Step 8: Output Firewall (Safety service)
- Step 9: Observability service (metrics, traces, logs)
