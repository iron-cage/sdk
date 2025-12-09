# Data Flow

**Purpose:** End-to-end journey of a request through the system.

---

## User Need

Trace a request from user input to LLM response, understanding each transformation.

## Core Idea

**9-step flow with explicit latency budget:**

```
User --> API --> Input FW --> Agent --> Tools --> LLM --> Output FW --> Audit --> Response
 |      5ms     50ms       100ms    2000ms   3000ms    30ms       0ms
 +------------------------------------------------------------------------------+
                              Total: ~5.7 seconds
```

## The Eleven Steps (Model C: Control Panel-Managed)

| Step | Component | Action | Latency | Protocol |
|------|-----------|--------|---------|----------|
| 0 | IC Token Validation | Verify IC Token signature, expiration | <0.1ms | [architecture/006](006_budget_control_protocol.md) |
| 1 | API Gateway | Check permissions, rate limit | 5ms | - |
| 2 | Input Firewall | Prompt injection, PII detect | 50ms | - |
| 3 | Agent Runtime | Route to agent framework | 100ms | - |
| 4 | Data Access | Vector search (if RAG query) | 500ms | - |
| 5 | Tool Proxy | Authorize tool, validate params | 50ms | - |
| 6 | Sandbox | Execute code (if code tool) | 2000ms | - |
| 6a | Token Translation | Replace IC Token with IP Token | <0.5ms | [architecture/006](006_budget_control_protocol.md) |
| 7 | LLM Gateway | Forward with IP Token, track cost | 3000ms | - |
| 7a | Cost Reporting | Report usage to Control Panel (async) | 0ms | [architecture/006](006_budget_control_protocol.md) |
| 8 | Output Firewall | Secret scan, PII redact | 30ms | - |
| 9 | Observability | Log, trace (async) | 0ms | - |

**Total Overhead for Model C:**
- IC Token operations: 0.1ms + 0.5ms = 0.6ms
- Cost reporting: 0ms (async)
- **Total added: <1ms**

## Critical Path

**Model C (Control Panel-Managed):**
- Simple LLM call: Steps 0, 1, 2, 6a, 7, 7a, 8, 9 -> ~3.086 seconds
- With code execution: All steps -> ~5.7 seconds
- **Control Panel overhead: 0.6ms (0.02%)**

**Model A/B (No Control Panel):**
- Simple LLM call: Steps 1, 2, 7, 8, 9 -> ~3.085 seconds (no Step 0, 6a, 7a)

## Failure Points

| Failure | Behavior | Rationale |
|---------|----------|-----------|
| Input FW down | Block all | Fail-safe |
| Cost Service down | Allow, log warning | Fail-open for availability |
| LLM provider down | Try fallback | Reliability layer handles |

---

*Related: [002_layer_model.md](002_layer_model.md) | [005_service_integration.md](005_service_integration.md)*
