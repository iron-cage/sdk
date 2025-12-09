# Layer Model

**Purpose:** How requests flow through processing layers in the gateway.

---

## User Need

Understand which component handles safety, cost, reliability - and in what order.

## Core Idea

**Six processing layers, each with single responsibility:**

```
Request --> [Safety] --> [Cost] --> [Reliability] --> [Provider] --> Response
              |           |            |              |
              v           v            v              v
          Validate    Check        Circuit       Forward to
          input       budget       breaker       OpenAI/etc
```

## The Six Layers

| Layer | Responsibility | Failure Mode |
|-------|---------------|--------------|
| **1. Safety** | Validate input (prompt injection, PII) | Block request |
| **2. Cost** | Check budget, track tokens | Block if exceeded |
| **3. Reliability** | Circuit breaker, retry logic | Fallback provider |
| **4. Provider** | Route to LLM (OpenAI, Anthropic) | Error response |
| **5. Output Safety** | Scan response (secrets, PII) | Redact content |
| **6. Observability** | Log, trace, metrics | Async (non-blocking) |

## Design Principles

- **Fail-safe:** Safety layer down = block all (never bypass)
- **Non-blocking:** Observability is async, adds 0ms latency
- **Independent:** Each layer can be deployed/scaled separately
- **Ordered:** Safety MUST run before provider (defense in depth)

## Latency Budget

| Layer | Target | Notes |
|-------|--------|-------|
| Safety (input) | <10ms | Regex + ML classifier |
| Cost | <5ms | Redis lookup |
| Reliability | <5ms | State machine check |
| Provider | 1-5s | LLM inference (external) |
| Safety (output) | <10ms | Pattern matching |
| Observability | 0ms | Async, non-blocking |

---

*Related: [data_flow.md](data_flow.md) | [service_integration.md](service_integration.md)*
