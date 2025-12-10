# Service Boundaries

**Purpose:** Separation between Control Plane, Data Plane, and Agent Runtime.
**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-10

---

## User Need

Know which component to look at for dashboards vs request processing vs agent execution.

## Core Idea

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

**Note:** Control Plane is ALWAYS deployed as standalone admin service. It is not optional - all deployments require Control Panel for admin to manage developer budgets and access.

**Three Roles (All Use CLI + Dashboard):**
- **Admin:** Full Control Panel access via CLI + Dashboard (allocates budgets, manages IP Tokens)
- **Super User:** Read-only dashboard access via CLI + Dashboard (own budgets only)
- **Developer:** Read-only dashboard access via CLI + Dashboard (own usage only, can select model/IP)

## Key Boundaries

| Plane | Changes | Scaled By | State |
|-------|---------|-----------|-------|
| Control | Weekly | Replicas | Database (PostgreSQL) |
| Data | Rarely | Load | Cache + Database |
| Runtime | Per-request | HPA (K8s) | Stateless |

*Note: Cache = In-memory (pilot) or Redis (production). Database = SQLite (pilot) or PostgreSQL (production). See [technology/003](../technology/003_infrastructure_choices.md#cache).*

## Communication Pattern

- Control -> Data: Configuration, policies
- Data -> Runtime: Validation results, budgets
- Runtime -> Data: LLM calls, tool requests

---

*Related: [005_service_integration.md](005_service_integration.md)*
