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

## The Nine Steps

| Step | Component | Action | Latency |
|------|-----------|--------|---------|
| 1 | API Gateway | Authenticate, rate limit | 5ms |
| 2 | Input Firewall | Prompt injection, PII detect | 50ms |
| 3 | Agent Runtime | Route to agent framework | 100ms |
| 4 | Data Access | Vector search (if RAG query) | 500ms |
| 5 | Tool Proxy | Authorize tool, validate params | 50ms |
| 6 | Sandbox | Execute code (if code tool) | 2000ms |
| 7 | LLM Gateway | Forward to provider, track cost | 3000ms |
| 8 | Output Firewall | Secret scan, PII redact | 30ms |
| 9 | Observability | Log, trace (async) | 0ms |

## Critical Path

For simple LLM call (no tools): Steps 1, 2, 7, 8, 9 -> ~3.1 seconds
For code execution: All steps -> ~5.7 seconds

## Failure Points

| Failure | Behavior | Rationale |
|---------|----------|-----------|
| Input FW down | Block all | Fail-safe |
| Cost Service down | Allow, log warning | Fail-open for availability |
| LLM provider down | Try fallback | Reliability layer handles |

---

*Related: [002_layer_model.md](002_layer_model.md) | [005_service_integration.md](005_service_integration.md)*
