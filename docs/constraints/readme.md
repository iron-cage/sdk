# Constraints

**Purpose:** System constraints and trade-off decisions that shaped Iron Cage design.

---

## Directory Responsibilities

| ID | Entity | Responsibility | Input → Output | Scope | Out of Scope |
|----|--------|----------------|----------------|-------|--------------|
| 001 | **001_technical_constraints.md** | Document technology limitations and requirements | Technical constraint question → Constraint specification | Python 3.8+ requirement, Linux kernel 5.15+, Rust 1.70+, PyO3 0.22, platform limitations | NOT technology rationale (→ docs/technology/), NOT implementation (→ module/*/spec.md), NOT other constraints (→ 002-004) |
| 002 | **002_business_constraints.md** | Explain business and timeline limitations | Business constraint question → Constraint specification | Pilot timeline (22 days), demo scope (5 min), initial budget limits, team size | NOT technical constraints (→ 001), NOT scope decisions (→ 003), NOT other constraints (→ 001, 003-004) |
| 003 | **003_scope_boundaries.md** | Define what's in and out of platform scope | Scope question → Boundary definition | Platform scope (agent governance), explicitly excluded features (agent IDE, LLM training), deferred features (multi-cloud) | NOT technical limits (→ 001), NOT business limits (→ 002), NOT trade-offs (→ 004) |
| 004 | **004_trade_offs.md** | Document key design trade-offs and rationale | Trade-off question → Decision analysis | Latency vs accuracy, simplicity vs features, safety vs performance, memory vs disk, cost vs reliability | NOT principles (→ docs/principles/), NOT ADRs (→ docs/decisions/), NOT other constraints (→ 001-003) |

---

## Constraints Collection

| ID | Name | Purpose |
|----|------|---------|
| 001 | [Technical Constraints](001_technical_constraints.md) | Technology limitations |
| 002 | [Business Constraints](002_business_constraints.md) | Business and timeline limits |
| 003 | [Scope Boundaries](003_scope_boundaries.md) | Platform scope definition |
| 004 | [Trade-offs](004_trade_offs.md) | Design trade-off decisions |

---

*For design principles, see [principles/](../principles/)*
*For architectural decisions, see [decisions/](../decisions/)*
