# Trade-offs

**Purpose:** Key design trade-offs and the rationale behind choices made.

---

## User Need

Understand why certain options were chosen over alternatives and what was sacrificed.

## Core Idea

**Every design decision involves trade-offs - document both sides:**

## Latency vs Accuracy

**Trade-off:** Safety validation adds latency but prevents compliance violations

| Choice | Latency | Accuracy | Decision |
|--------|---------|----------|----------|
| Skip validation | 0ms | Low (PII leaks) | ❌ Rejected |
| Regex only | 10ms | Medium (some false negatives) | ✅ **Pilot choice** |
| ML classifier | 50ms | High (better detection) | 🔮 Full platform |

**Chosen:** Regex (10ms) for pilot - acceptable latency, sufficient accuracy for demo.

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

## Memory vs Disk

**Trade-off:** IP Token storage location

| Option | Security | Recovery | Choice |
|--------|----------|----------|--------|
| **Encrypted in memory** | High (never on disk) | Lost on crash | ✅ **Chosen** |
| Encrypted on disk | Medium (disk forensics) | Survives restart | ❌ Rejected |
| Plaintext (fetch on demand) | Low (exposed) | Always available | ❌ Rejected |

**Rationale:** Security over convenience. Developer never sees IP Token, runtime fetches from Control Panel on startup.

## Cost vs Reliability

**Trade-off:** Budget tracking granularity

| Granularity | Overhead | Accuracy | Choice |
|-------------|----------|----------|--------|
| Per-request | 5ms | Exact (real-time) | ✅ **Chosen** |
| Batched (every 10 requests) | 0.5ms | Delayed (eventual) | ❌ Rejected |
| Per-session | 0ms | Approximate | ❌ Rejected |

**Rationale:** Budget enforcement requires real-time accuracy. 5ms overhead acceptable for compliance.

## Monorepo vs Two-Repo

**Trade-off:** Code organization

| Aspect | Monorepo | Two-Repo | Choice |
|--------|----------|----------|--------|
| **Simplicity** | All in one place | Coordination needed | Monorepo (pilot) |
| **Release Cadence** | All together | Independent | Two-repo (future) |
| **CI/CD** | Single pipeline | Per-repo | Monorepo (pilot) |

**Decision:** Pilot uses monorepo (simpler), production migrates to two-repo (ADR-001) for independent releases.

---

*Related: [003_scope_boundaries.md](003_scope_boundaries.md) | [../decisions/](../decisions/)*
