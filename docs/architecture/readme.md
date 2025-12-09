# Architecture

**Purpose:** Conceptual overview of Iron Cage system architecture - how components are organized and why.

---

## Directory Responsibilities

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| **execution_models.md** | Explain agent execution location options | Execution question → Model comparison | Client-side execution (primary), server-side execution (optional), governance patterns, SDK interception, trade-offs | NOT deployment (→ docs/deployment/), NOT implementation (→ module/iron_runtime/spec.md), NOT service details (→ service_integration.md) |
| **layer_model.md** | Document request processing pipeline layers | Layer question → Processing sequence | Six layers (Safety, Cost, Reliability, Provider, Output Safety, Observability), layer responsibilities, failure modes, ordering requirements | NOT service planes (→ service_boundaries.md), NOT communication (→ service_integration.md), NOT data flow steps (→ data_flow.md) |
| **service_boundaries.md** | Define Control/Data/Runtime plane separation | Plane question → Boundary definition | Control Plane (API, dashboard, scheduler), Data Plane (safety, cost, reliability, observability), Agent Runtime (SDK, sandbox, agents) | NOT layer pipeline (→ layer_model.md), NOT communication (→ service_integration.md), NOT implementation (→ module/*/spec.md) |
| **service_integration.md** | Explain inter-service communication patterns | Integration question → Communication approach | Gateway orchestration, service ports, HTTP communication, dependency graph, call sequences | NOT plane boundaries (→ service_boundaries.md), NOT processing layers (→ layer_model.md), NOT data flow (→ data_flow.md) |
| **data_flow.md** | Trace end-to-end request journey | Data journey question → Flow diagram | Nine steps (API → Input FW → Agent → Tools → LLM → Output FW → Audit → Response), latency budget per step, critical path analysis | NOT layer responsibilities (→ layer_model.md), NOT service boundaries (→ service_boundaries.md), NOT implementation (→ module specifications) |
| **two_repo_model.md** | Justify two-repository architecture decision | Repository structure question → Split rationale | iron_runtime vs iron_cage separation, release cycle differences, crates.io sharing, module distribution | NOT specific modules (→ module/*/), NOT deployment (→ docs/deployment/), NOT ADR details (→ docs/decisions/adr_001) |

---

## The Six Architecture Concepts

| # | Concept | Core Idea |
|---|---------|-----------|
| 1 | [Execution Models](execution_models.md) | Where agents run (client vs server) |
| 2 | [Layer Model](layer_model.md) | Request processing pipeline |
| 3 | [Service Boundaries](service_boundaries.md) | Control Plane / Data Plane / Runtime separation |
| 4 | [Two-Repo Model](two_repo_model.md) | iron_runtime + iron_cage split |
| 5 | [Data Flow](data_flow.md) | End-to-end request journey |
| 6 | [Service Integration](service_integration.md) | How services communicate |

## Relationships

```
                    +---------------------+
                    |  Execution Models   |
                    |  (where agents run) |
                    +----------+----------+
                               |
              +----------------+----------------+
              v                v                v
     +----------------+ +------------+ +----------------+
     |  Layer Model   | | Data Flow  | |   Service      |
     |  (processing)  | | (journey)  | |  Boundaries    |
     +----------------+ +------------+ +----------------+
              |                               |
              +---------------+---------------+
                              v
                    +---------------------+
                    |  Service Integration |
                    |  (communication)     |
                    +---------------------+
```

*For capability concepts, see [capabilities/](../capabilities/)*
*For deployment concepts, see [deployment/](../deployment/)*
