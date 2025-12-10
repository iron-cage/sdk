# Architecture

**Purpose:** Conceptual overview of Iron Cage system architecture - how components are organized and why.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 001 | **001_execution_models.md** | Explain agent execution location options (local primary, server future, Control Panel role, runtime modes) |
| 002 | **002_layer_model.md** | Document six request processing layers (Safety, Cost, Reliability, Provider, Output Safety, Observability) with failure modes |
| 003 | **003_service_boundaries.md** | Define three plane separation (Control, Data, Runtime) and communication patterns |
| 004 | **004_data_flow.md** | Trace end-to-end request journey through eleven processing steps with latency analysis |
| 005 | **005_service_integration.md** | Explain how five core services communicate (Gateway orchestrates Safety, Cost, Tool Proxy, Audit) |
| 006 | **006_roles_and_permissions.md** | Define three access roles (Admin, Super User, Developer) and their permission boundaries |
| 007 | **007_entity_model.md** | Document core entities (Agent, Project, IP, IC Token, budgets) and their 1:1 relationships |
| 008 | **008_runtime_modes.md** | Explain runtime deployment configurations (Router vs Library, same SDK code, different internal machinery) |
| 009 | **009_resource_catalog.md** | Exhaustive inventory of REST API resources, entity mapping, authentication patterns, certainty classification |

---

## Architecture Collection

| ID | Name | Purpose |
|----|------|---------|
| 001 | [Execution Models](001_execution_models.md) | Where agents run (local vs server) |
| 002 | [Layer Model](002_layer_model.md) | Request processing pipeline |
| 003 | [Service Boundaries](003_service_boundaries.md) | Control Plane / Data Plane / Runtime separation |
| 004 | [Data Flow](004_data_flow.md) | End-to-end request journey |
| 005 | [Service Integration](005_service_integration.md) | How services communicate |
| 006 | [Roles and Permissions](006_roles_and_permissions.md) | Access levels and control |
| 007 | [Entity Model](007_entity_model.md) | Core entities and relationships |
| 008 | [Runtime Modes](008_runtime_modes.md) | Router vs Library execution modes |
| 009 | [Resource Catalog](009_resource_catalog.md) | REST API resources and entity mapping |

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
