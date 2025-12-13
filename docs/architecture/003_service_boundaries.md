# Architecture: Service Boundaries

### Scope

This document defines the three-plane separation model (Control Plane, Data Plane, Agent Runtime) and their communication patterns.

**In scope:**
- Three plane separation (Control Plane, Data Plane, Agent Runtime)
- Plane responsibilities and service components per plane
- Communication patterns between planes (Control→Data, Data→Runtime, Runtime→Data)
- Role access patterns (Admin, Super User, Developer all use CLI + Dashboard)
- Scaling characteristics per plane (replicas, load-based, HPA)
- State management per plane (database, cache+database, stateless)

**Out of scope:**
- Detailed service implementation → See service-specific documentation
- API Gateway routing logic → See protocol documentation
- Dashboard UI implementation → See deployment documentation
- Database schema details → See [Technology: Infrastructure Choices](../technology/003_infrastructure_choices.md)
- Specific scaling strategies → See deployment documentation
- Network topology and infrastructure → See infrastructure documentation

### Purpose

**User Need**: Know which component to look at for dashboards vs request processing vs agent execution.

**Solution**: Three distinct planes with clear responsibilities and communication patterns:

- **Control Plane**: Admin service (API Gateway, Dashboard, Scheduler) managing policies and configuration - changes weekly, scaled by replicas, uses PostgreSQL database
- **Data Plane**: Request processing services (Safety, Cost, Reliability, Observability) serving agent runtime - changes rarely, scaled by load, uses cache + database
- **Agent Runtime**: Agent execution environment (agent pods, SDK, sandbox) running developer code - changes per-request, scaled by HPA, stateless

**Three distinct planes with clear responsibilities:**

```
+-----------------------------------------------------+
|                   CONTROL PLANE                      |
|  * API Gateway (authentication, routing)            |
|  * Dashboard (Vue SPA, policy management)           |
|  * Scheduler (jobs, lifecycle)                      |
+-----------------------------------------------------+
                         | manages
+-----------------------------------------------------+
|                    DATA PLANE                        |
|  * Safety Service (input/output validation)         |
|  * Cost Service (budget, tracking)                  |
|  * Reliability Service (circuit breakers)           |
|  * Observability (metrics, traces, logs)            |
+-----------------------------------------------------+
                         | serves
+-----------------------------------------------------+
|                  AGENT RUNTIME                       |
|  * Agent pods (LangChain, CrewAI, custom)           |
|  * SDK (intercepts LLM calls)                       |
|  * Sandbox (isolated code execution)                |
+-----------------------------------------------------+
```

**Three Roles (All Use CLI + Dashboard):**
- **Admin:** Full Control Panel access via CLI + Dashboard (allocates budgets, manages IP Tokens)
- **Super User:** Read-only dashboard access via CLI + Dashboard (own budgets only)
- **Developer:** Read-only dashboard access via CLI + Dashboard (own usage only, can select model/IP)

**Key Insight**: Control Plane is ALWAYS deployed as standalone admin service. It is not optional - all deployments require Control Panel for admin to manage developer budgets and access. The three planes separate concerns: Control (admin management), Data (request processing), Runtime (agent execution).

**Status**: Specification
**Version**: 1.0.0
**Last Updated**: 2025-12-13

### Key Boundaries

| Plane | Changes | Scaled By | State |
|-------|---------|-----------|-------|
| Control | Weekly | Replicas | Database (PostgreSQL) |
| Data | Rarely | Load | Cache + Database |
| Runtime | Per-request | HPA (K8s) | Stateless |

*Note: Cache = In-memory (pilot) or Redis (production). Database = SQLite (pilot) or PostgreSQL (production). See [technology/003](../technology/003_infrastructure_choices.md#cache).*

### Communication Pattern

- Control -> Data: Configuration, policies
- Data -> Runtime: Validation results, budgets
- Runtime -> Data: LLM calls, tool requests

### Cross-References

#### Related Principles Documents
- [Principles: Design Philosophy](../principles/001_design_philosophy.md) - Agent-Centric Control principle reflected in Runtime plane separation, Minimal Dependencies via plane independence
- [Principles: Quality Attributes](../principles/002_quality_attributes.md) - Reliability via Data Plane circuit breakers, Security via Control Plane separation, Scalability via independent plane scaling
- [Principles: Development Workflow](../principles/005_development_workflow.md) - Specification-first approach applied to this architecture document

**Related Architecture Documents:**
- [Architecture: Execution Models](001_execution_models.md) - Control Panel context (ALWAYS present) that Control Plane implements
- [Architecture: Layer Model](002_layer_model.md) - Six processing layers that Data Plane services implement
- [Architecture: Service Integration](005_service_integration.md) - How services communicate across these plane boundaries

#### Used By
- [Architecture: Layer Model](002_layer_model.md) - Plane separation architecture that layers operate within
- [Architecture: Data Flow](004_data_flow.md) - Request flow through these three planes
- [Architecture: Service Integration](005_service_integration.md) - Service communication respecting plane boundaries
- [Architecture: Roles and Permissions](006_roles_and_permissions.md) - Role access patterns across Control/Data/Runtime planes

#### Dependencies
- [Architecture: Execution Models](001_execution_models.md) - Control Panel as standalone admin service (Control Plane)
- [Technology: Infrastructure Choices](../technology/003_infrastructure_choices.md) - Cache and database technology choices referenced

#### Implementation
- Control Plane: API Gateway service, Dashboard (Vue SPA), Scheduler service
- Data Plane: Safety service, Cost service, Reliability service, Observability service
- Agent Runtime: Agent pods (Kubernetes), SDK (intercepts LLM calls), Sandbox (isolated execution)
