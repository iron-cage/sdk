# Service Boundaries

**Purpose:** Separation between Control Plane, Data Plane, and Agent Runtime.

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

## Key Boundaries

| Plane | Changes | Scaled By | State |
|-------|---------|-----------|-------|
| Control | Weekly | Replicas | PostgreSQL |
| Data | Rarely | Load | Redis + PostgreSQL |
| Runtime | Per-request | HPA (K8s) | Stateless |

## Communication Pattern

- Control -> Data: Configuration, policies
- Data -> Runtime: Validation results, budgets
- Runtime -> Data: LLM calls, tool requests

---

*Related: [two_repo_model.md](two_repo_model.md) | [service_integration.md](service_integration.md)*
