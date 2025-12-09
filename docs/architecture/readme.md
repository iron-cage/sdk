# Architecture

**Purpose:** Conceptual overview of Iron Cage system architecture - how components are organized and why.

---

## Directory Responsibilities

| ID | Entity | Responsibility | Input → Output | Scope | Out of Scope |
|----|--------|----------------|----------------|-------|--------------|
| 001 | **001_execution_models.md** | Explain agent execution location options | Execution question → Model comparison | Client-side execution (primary), server-side execution (optional), governance patterns, SDK interception, trade-offs | NOT deployment (→ docs/deployment/), NOT implementation (→ module/iron_runtime/spec.md), NOT service details (→ 005) |
| 002 | **002_layer_model.md** | Document request processing pipeline layers | Layer question → Processing sequence | Six layers (Safety, Cost, Reliability, Provider, Output Safety, Observability), layer responsibilities, failure modes, ordering requirements | NOT service planes (→ 003), NOT communication (→ 005), NOT data flow steps (→ 004) |
| 003 | **003_service_boundaries.md** | Define Control/Data/Runtime plane separation | Plane question → Boundary definition | Control Plane (API, dashboard, scheduler), Data Plane (safety, cost, reliability, observability), Agent Runtime (SDK, sandbox, agents) | NOT layer pipeline (→ 002), NOT communication (→ 005), NOT implementation (→ module/*/spec.md) |
| 004 | **004_data_flow.md** | Trace end-to-end request journey | Data journey question → Flow diagram | Nine steps (API → Input FW → Agent → Tools → LLM → Output FW → Audit → Response), latency budget per step, critical path analysis | NOT layer responsibilities (→ 002), NOT service boundaries (→ 003), NOT implementation (→ module specifications) |
| 005 | **005_service_integration.md** | Explain inter-service communication patterns | Integration question → Communication approach | Gateway orchestration, service ports, HTTP communication, dependency graph, call sequences | NOT plane boundaries (→ 003), NOT processing layers (→ 002), NOT data flow (→ 004) |

---

## Architecture Collection

| ID | Name | Purpose |
|----|------|---------|
| 001 | [Execution Models](001_execution_models.md) | Where agents run (client vs server) |
| 002 | [Layer Model](002_layer_model.md) | Request processing pipeline |
| 003 | [Service Boundaries](003_service_boundaries.md) | Control Plane / Data Plane / Runtime separation |
| 004 | [Data Flow](004_data_flow.md) | End-to-end request journey |
| 005 | [Service Integration](005_service_integration.md) | How services communicate |

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
