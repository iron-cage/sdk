# Constraints: Trade-offs

### Scope

This document defines key design trade-offs and their rationale across the Iron Runtime platform. It serves as the **authoritative reference for all latency numbers** used throughout the documentation, ensuring consistency across architecture, protocol, and security specifications.

**In scope**:
- Trade-off decisions across six categories (Latency vs Accuracy, Simplicity vs Features, Safety vs Performance, Memory vs Disk, Cost vs Reliability)
- Explicit rationale for pilot vs production choices
- Latency budget summary (authoritative reference table)
- What was gained vs what was sacrificed for each choice
- Bidirectional references to documents depending on these trade-offs

**Out of scope**:
- Implementation details of chosen technologies (see service-specific documentation)
- Final production configuration parameters (see deployment documentation)
- Cost calculation models and pricing formulas (see Protocol 005: Budget Control Protocol)
- Performance benchmarks and measurement methodology (see observability documentation)
- Alternative options not considered during design (see Decisions collection for ADRs)

### Purpose

**User Need:** Understand why certain options were chosen over alternatives and what was sacrificed.

**Solution:** Every design decision involves trade-offs. This document systematically documents both sides of each choice (what was gained vs what was sacrificed), providing explicit rationale for pilot vs production decisions across six key trade-off categories. The Latency Budget Summary serves as the authoritative reference for all latency numbers used throughout the documentation.

**Key Insight:** Trade-offs are not failures - they are conscious decisions with clear rationale. By documenting both sides, we enable future teams to understand context and make informed adjustments when constraints change. The pilot vs production split acknowledges that demo requirements differ from production requirements, and both are valid choices for their respective contexts.

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13

---

### Latency Budget Summary

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

### Trade-off Analysis

#### Latency vs Accuracy

**Trade-off:** Safety validation adds latency but prevents compliance violations

| Choice | Latency | Accuracy | Pilot Choice | Full Platform |
|--------|---------|----------|--------------|---------------|
| Skip validation | 0ms | Low (PII leaks) | ❌ Rejected | ❌ Rejected |
| Regex only | 10ms | Medium (some false negatives) | ✅ **Chosen** | ❌ Insufficient |
| ML classifier | 50ms | High (better detection) | ❌ Too slow for demo | ✅ **Chosen** |

**Rationale:**
- **Pilot:** Regex (10ms) sufficient for 5-minute demo - acceptable latency, adequate accuracy for demonstration.
- **Full Platform:** ML classifier (50ms) required for production - higher accuracy needed for compliance-grade detection.

#### Simplicity vs Features

**Trade-off:** Pilot scope vs full platform capabilities

| Dimension | Pilot | Full Platform | Choice |
|-----------|-------|---------------|--------|
| LLM Providers | OpenAI only | Multi-provider | ✅ Simplicity (pilot) |
| Budget Currency | USD only | Multi-currency | ✅ Simplicity (pilot) |
| Execution Model | Client-side | Client + Server | ✅ Simplicity (pilot) |
| Sandboxing | Spec-only | Full isolation | ✅ Defer to production |

**Rationale:** 5-minute demo doesn't need multi-provider or multi-currency. Prove concept first.

#### Safety vs Performance

**Trade-off:** Fail-safe blocking vs high availability

| Scenario | Fail-Safe | Fail-Open | Choice |
|----------|-----------|-----------|--------|
| Safety layer down | Block all | Allow all | ✅ **Fail-safe** |
| Cost service down | Block all | Allow, log | ✅ **Fail-open** |
| Reliability down | Block all | Allow | ✅ **Fail-open** |

**Chosen:** Safety always blocks (compliance risk), Cost/Reliability fail-open (availability priority).

**Note:** This decision applies to BOTH pilot and full platform. Safety is non-negotiable in any environment.

#### Memory vs Disk

**Trade-off:** IP Token storage location

| Option | Security | Recovery | Choice |
|--------|----------|----------|--------|
| **Encrypted in memory** | High (never on disk) | Lost on crash | ✅ **Chosen** |
| Encrypted on disk | Medium (disk forensics) | Survives restart | ❌ Rejected |
| Plaintext (fetch on demand) | Low (exposed) | Always available | ❌ Rejected |

**Rationale:** Security over convenience. Developer never sees IP Token, runtime fetches from Control Panel on startup.

**Note:** This decision applies to BOTH pilot and full platform. Security architecture is identical in any environment.

#### Cost vs Reliability

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

### Cross-References

#### Authoritative Reference For

This document serves as the **authoritative reference for all latency numbers** across the Iron Runtime documentation. The following documents depend on the latency budget values defined in this document:

- Architecture 002: [Layer Model](../architecture/002_layer_model.md#latency-budget) - References latency budget for each processing layer
- Protocol 005: [Budget Control Protocol](../protocol/005_budget_control_protocol.md) - References latency overhead for budget tracking operations
- Security 002: [Isolation Layers](../security/002_isolation_layers.md) - References performance impact of isolation modes

**Important:** Any changes to latency values in this document must be synchronized with all dependent documents listed above.

#### Related Constraints Documents

- [001_technical_constraints.md](001_technical_constraints.md) - Technology limitations and requirements affecting trade-off decisions
- [002_business_constraints.md](002_business_constraints.md) - Business and timeline limitations driving pilot vs production choices
- [003_scope_boundaries.md](003_scope_boundaries.md) - Platform scope definition informing feature vs simplicity trade-offs

#### Used By

- Architecture: Layer processing model references latency budgets for each layer
- Protocol: Budget control protocol references cost tracking overhead values
- Security: Isolation layer specifications reference performance impact trade-offs

#### Dependencies

- Constraints 001: [Technical Constraints](001_technical_constraints.md) - Platform requirements constraining available options
- Constraints 002: [Business Constraints](002_business_constraints.md) - Timeline and budget constraints forcing pilot simplifications
- Decisions: Architecture Decision Records (ADRs) provide additional context for specific trade-off choices

#### Implementation

- Pilot implementation: Uses pilot-column values from all trade-off tables
- Production implementation: Uses full-platform-column values (deferred pending pilot validation)
- Latency measurements: Defined in observability and performance testing modules (paths TBD)
