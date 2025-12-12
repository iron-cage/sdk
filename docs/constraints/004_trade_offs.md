# Trade-offs

**Purpose:** Key design trade-offs and the rationale behind choices made.

---

## User Need

Understand why certain options were chosen over alternatives and what was sacrificed.

## Core Idea

**Every design decision involves trade-offs - document both sides:**

## Latency Budget Summary

**Authoritative reference for all latency numbers across documentation.**

| Operation | Pilot | Full Platform | Reference |
|-----------|-------|---------------|-----------|
| Input Safety | 10ms (Regex) | 50ms (ML classifier) | [002_layer_model.md](../architecture/002_layer_model.md#latency-budget) |
| Cost Layer | 5ms (per-request) | 0.5ms (batched) | [002_layer_model.md](../architecture/002_layer_model.md#latency-budget) |
| Token Translation | <0.5ms | <0.5ms | [004_data_flow.md](../architecture/004_data_flow.md) |
| Cost Reporting | 0ms (async) | 0ms (async) | [004_data_flow.md](../architecture/004_data_flow.md) |
| Reliability Check | <5ms | <5ms | [002_layer_model.md](../architecture/002_layer_model.md#latency-budget) |
| Output Safety | 10ms (Regex) | 50ms (ML classifier) | [002_layer_model.md](../architecture/002_layer_model.md#latency-budget) |
| Observability | 0ms (async) | 0ms (async) | [002_layer_model.md](../architecture/002_layer_model.md#latency-budget) |

**Total Added Latency:** ~30ms (pilot) → ~106ms (full platform)

---

## Latency vs Accuracy

**Trade-off:** Safety validation adds latency but prevents compliance violations

| Choice | Latency | Accuracy | Pilot Choice | Full Platform |
|--------|---------|----------|--------------|---------------|
| Skip validation | 0ms | Low (PII leaks) | ❌ Rejected | ❌ Rejected |
| Regex only | 10ms | Medium (some false negatives) | ✅ **Chosen** | ❌ Insufficient |
| ML classifier | 50ms | High (better detection) | ❌ Too slow for demo | ✅ **Chosen** |

**Rationale:**
- **Pilot:** Regex (10ms) sufficient for 5-minute demo - acceptable latency, adequate accuracy for demonstration.
- **Full Platform:** ML classifier (50ms) required for production - higher accuracy needed for compliance-grade detection.

## Simplicity vs Features

**Trade-off:** Pilot scope vs full platform capabilities

| Dimension | Pilot | Full Platform | Choice |
|-----------|-------|---------------|--------|
| LLM Providers | OpenAI only | Multi-provider | ✅ Simplicity (pilot) |
| Budget Currency | USD only | Multi-currency | ✅ Simplicity (pilot) |
| Execution Model | Client-side | Client + Server | ✅ Simplicity (pilot) |
| Sandboxing | Spec-only | Full isolation | ✅ Defer to production |

**Rationale:** 5-minute demo doesn't need multi-provider or multi-currency. Prove concept first.

## Safety vs Performance

**Trade-off:** Fail-safe blocking vs high availability

| Scenario | Fail-Safe | Fail-Open | Choice |
|----------|-----------|-----------|--------|
| Safety layer down | Block all | Allow all | ✅ **Fail-safe** |
| Cost service down | Block all | Allow, log | ✅ **Fail-open** |
| Reliability down | Block all | Allow | ✅ **Fail-open** |

**Chosen:** Safety always blocks (compliance risk), Cost/Reliability fail-open (availability priority).

**Note:** This decision applies to BOTH pilot and full platform. Safety is non-negotiable in any environment.

## Memory vs Disk

**Trade-off:** IP Token storage location

| Option | Security | Recovery | Choice |
|--------|----------|----------|--------|
| **Encrypted in memory** | High (never on disk) | Lost on crash | ✅ **Chosen** |
| Encrypted on disk | Medium (disk forensics) | Survives restart | ❌ Rejected |
| Plaintext (fetch on demand) | Low (exposed) | Always available | ❌ Rejected |

**Rationale:** Security over convenience. Developer never sees IP Token, runtime fetches from Control Panel on startup.

**Note:** This decision applies to BOTH pilot and full platform. Security architecture is identical in any environment.

## Cost vs Reliability

**Trade-off:** Budget tracking granularity

| Granularity | Overhead | Accuracy | Pilot Choice | Full Platform |
|-------------|----------|----------|--------------|---------------|
| Per-request | 5ms | Exact (real-time) | ✅ **Chosen** | ❌ Will optimize later |
| Batched (every 10 requests) | 0.5ms | Delayed (eventual) | ❌ Too complex for demo | ✅ **Chosen** |
| Per-session | 0ms | Approximate | ❌ Rejected | ❌ Rejected |

**Rationale:**
- **Pilot:** Per-request (5ms) chosen for implementation simplicity. Straightforward logic (report immediately after each call), no buffering complexity. Higher overhead acceptable for 5-minute demo.
- **Full Platform:** Batched (0.5ms) chosen for production optimization. Lower overhead critical at scale, justifies additional buffering logic complexity.

---

*Related: [003_scope_boundaries.md](003_scope_boundaries.md) | [../decisions/](../decisions/readme.md)*
