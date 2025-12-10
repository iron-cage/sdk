# Data Flow

**Purpose:** End-to-end journey of a request through the system.
**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-10

---

## User Need

Trace a request from user input to LLM response, understanding each transformation.

## Core Idea

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

## The Eleven Steps

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

## Critical Path

**With Control Panel - Pilot:**
- Simple LLM call: Steps 0, 1, 2, 6a, 7, 7a, 8, 9
- Latency: 0.1ms + 5ms + 10ms + 0.5ms + 3000ms + 5ms + 10ms + 0ms = **~3.031 seconds**

**With Control Panel - Production:**
- Simple LLM call: Steps 0, 1, 2, 6a, 7, 7a, 8, 9
- Latency: 0.1ms + 5ms + 50ms + 0.5ms + 3000ms + 0ms + 50ms + 0ms = **~3.106 seconds**
- With code execution: All steps -> ~5.7 seconds
- **Control Panel overhead: 0.6ms (0.02%)**

---

## Response Processing (Critical for Security)

**The response flows through validation layers BEFORE reaching the agent:**

### Step 8: Output Firewall (30ms)

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

### Step 7a: Cost Reporting (0ms perceived, async)

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

### Step 9: Observability (0ms, async)

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

## Full Flow Summary

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

## Failure Points

| Failure | Behavior | Rationale |
|---------|----------|-----------|
| Input FW down | Block all | Fail-safe |
| Cost Service down | Allow, log warning | Fail-open for availability |
| LLM provider down | Try fallback | Reliability layer handles |

---

*Related: [002_layer_model.md](002_layer_model.md) | [005_service_integration.md](005_service_integration.md)*
