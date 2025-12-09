# Layer Model

**Purpose:** How requests flow through processing layers in the gateway.

---

## User Need

Understand which component handles safety, cost, reliability - and in what order.

## Core Idea

**Six processing layers with bidirectional request-response flow:**

```
REQUEST PATH (Input Processing):
Request → [1. Input Safety] → [2. Cost] → [3. Reliability] → [4. Provider]
            |                   |             |                  |
            ▼                   ▼             ▼                  ▼
        Validate input      Check budget   Circuit           Forward to
        (PII, injection)    (remaining)    breaker           LLM provider

PROVIDER:
[4. Provider] ← Request sent to → OpenAI/Anthropic/etc (3000ms processing)
              ← Response received ←

RESPONSE PATH (Output Processing):
[4. Provider] → [5. Output Safety] → [6. Observability] → Response to Agent
                      |                    |
                      ▼                    ▼
                  Scan for secrets      Log & trace
                  Redact PII            (async)
```

## The Six Layers

| Layer | Phase | Responsibility | Failure Mode |
|-------|-------|---------------|--------------|
| **1. Input Safety** | Request | Validate input (prompt injection, PII) | Block request |
| **2. Cost** | Request | Check budget, track tokens | Block if exceeded |
| **3. Reliability** | Request | Circuit breaker, retry logic | Fallback provider |
| **4. Provider** | Both | Route to LLM, receive response | Error response |
| **5. Output Safety** | Response | Scan response (secrets, PII) | Redact content |
| **6. Observability** | Response | Log, trace, metrics | Async (non-blocking) |

**Request Path:** Layers 1 → 2 → 3 → 4 → LLM Provider
**Response Path:** LLM Provider → 4 → 5 → 6 → Agent

## Design Principles

- **Fail-safe:** Safety layer down = block all (never bypass)
- **Non-blocking:** Observability is async, adds 0ms latency
- **Independent:** Each layer can be deployed/scaled separately
- **Ordered:** Safety MUST run before provider (defense in depth)

## Latency Budget

| Layer | Pilot Target | Production Target | Notes |
|-------|--------------|-------------------|-------|
| Safety (input) | 10ms | 50ms | Regex (pilot) → ML classifier (production) |
| Cost | 5ms | 0.5ms | Per-request (pilot) → Batched (production optimization) |
| Reliability | <5ms | <5ms | Same for both |
| Provider | 1-5s | 1-5s | LLM inference (external) |
| Safety (output) | 10ms | 50ms | Regex (pilot) → ML classifier (production) |
| Observability | 0ms | 0ms | Async, non-blocking |

**Total Added Latency:** ~30ms (pilot) → ~106ms (production)

**See:** [constraints/004: Trade-offs](../constraints/004_trade_offs.md#latency-budget-summary) for authoritative latency reference and rationale.

---

*Related: [004_data_flow.md](004_data_flow.md) | [005_service_integration.md](005_service_integration.md)*
