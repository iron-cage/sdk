# Architecture: Layer Model

### Scope

This document defines the six-layer request processing model for Iron Runtime's gateway architecture. It specifies layer responsibilities, failure modes, processing order, and latency budgets for both pilot and production deployments.

**In scope**:
- Six processing layers (Input Safety, Cost, Reliability, Provider, Output Safety, Observability)
- Request path processing (layers 1-4)
- Response path processing (layers 4-6)
- Layer failure modes and fallback behavior
- Latency budget per layer (pilot vs production)
- Design principles (fail-safe, non-blocking, independent, ordered)

**Out of scope**:
- Detailed layer implementation (see service-specific documentation)
- Provider-specific routing logic (see Protocol 004: MCP Integration)
- Budget allocation and IC/IP token system (see Protocol 005: Budget Control)
- Safety layer detection algorithms (see Security 002: Isolation Layers)
- Observability backend configuration (see deployment documentation)

### Purpose

**User Need:** Understand which component handles safety, cost, reliability - and in what order.

**Solution:** Six processing layers with bidirectional request-response flow:

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

**Key Insight:** Each layer has a specific responsibility and failure mode. Safety layers (input/output) are fail-safe (block all if down, never bypass). Observability is async (0ms perceived latency). Processing order ensures defense in depth - safety MUST run before provider access.

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13

### The Six Layers

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

### Design Principles

- **Fail-safe:** Safety layer down = block all (never bypass)
- **Non-blocking:** Observability is async, adds 0ms latency
- **Independent:** Each layer can be deployed/scaled separately
- **Ordered:** Safety MUST run before provider (defense in depth)

### Latency Budget

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

### Cross-References

#### Related Principles Documents
- [Principles: Design Philosophy](../principles/001_design_philosophy.md) - Fail-Safe Defaults principle reflected in fail-safe safety layers (block all if down), Observable Behavior via observability layer
- [Principles: Quality Attributes](../principles/002_quality_attributes.md) - Reliability via circuit breaker/retry logic, Security via input/output safety layers, Performance via latency budgets
- [Principles: Development Workflow](../principles/005_development_workflow.md) - Specification-first approach applied to this architecture document

**Related Architecture Documents:**
- [Architecture: Data Flow](004_data_flow.md) - End-to-end request journey through these processing layers
- [Architecture: Service Integration](005_service_integration.md) - How gateway services communicate using this layer model
- [Architecture: Service Boundaries](003_service_boundaries.md) - Control/Data/Runtime plane separation context for layers

#### Used By
- [Protocol: Budget Control Protocol](../protocol/005_budget_control_protocol.md) - References layer model for Cost layer responsibilities
- [Security: Isolation Layers](../security/002_isolation_layers.md) - Implements Input/Output Safety layers
- [Architecture: Data Flow](004_data_flow.md) - Traces request journey through these six layers
- [Architecture: Service Integration](005_service_integration.md) - Uses layer model to explain service communication

#### Dependencies
- [Constraints: Trade-offs](../constraints/004_trade_offs.md#latency-budget-summary) - Authoritative latency budget reference and rationale
- [Architecture: Service Boundaries](003_service_boundaries.md) - Plane separation architecture that layers operate within

#### Implementation
- Gateway service: Layer orchestration (module paths TBD)
- Safety service: Input/Output safety layers (module paths TBD)
- Cost service: Budget tracking layer (module paths TBD)
