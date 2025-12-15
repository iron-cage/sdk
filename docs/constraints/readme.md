# Constraints

**Purpose:** System constraints and trade-off decisions that shaped Iron Cage design.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 001 | **001_technical_constraints.md** | Document technology limitations and requirements (Python 3.8+, Linux 5.15+, Rust 1.70+, PyO3 0.22, platform constraints) |
| 002 | **002_business_constraints.md** | Explain business and timeline limitations (pilot timeline, demo scope, budget limits, team size) |
| 003 | **003_scope_boundaries.md** | Define what's in and out of platform scope (agent governance in-scope, excluded features, deferred features) |
| 004 | **004_trade_offs.md** | Document key design trade-offs and rationale (pilot vs production choices for latency, cache, database, storage, cost tracking); authoritative reference for all latency numbers |

---

## Constraints Collection

| ID | Name | Purpose |
|----|------|---------|
| 001 | [Technical Constraints](001_technical_constraints.md) | Technology limitations |
| 002 | [Business Constraints](002_business_constraints.md) | Business and timeline limits |
| 003 | [Scope Boundaries](003_scope_boundaries.md) | Platform scope definition |
| 004 | [Trade-offs](004_trade_offs.md) | Design trade-off decisions |

---

*For design principles, see [principles/](../principles/readme.md)*
*For architectural decisions, see [decisions/](../decisions/readme.md)*
