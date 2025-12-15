# Architecture

**Purpose:** Conceptual overview of Iron Cage system architecture - how components are organized and why.

**START HERE:** [000_high_level_overview.md](000_high_level_overview.md) - Comprehensive view of all actors, components, and collaboration patterns.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 000 | **000_high_level_overview.md** | Define ALL actors (human/software/service), ALL major components (Control Panel/Agent Runtime/Data Plane/External Systems/Infrastructure), and ALL collaboration patterns (authentication/request processing/budget control/monitoring/user management/agent lifecycle/failure handling) with precise protocols and system boundaries - THE definitive high-level architecture reference |
| 001 | **001_execution_models.md** | Define WHERE agents execute (local primary 95%, server future 5%) and ALWAYS-present Control Panel architecture managing budgets via IC Token/IP Token protocol across two runtime modes (Router, Library) |
| 002 | **002_layer_model.md** | Document six request processing layers (Safety, Cost, Reliability, Provider, Output Safety, Observability) with failure modes |
| 003 | **003_service_boundaries.md** | Define three plane separation (Control Plane for admin, Data Plane for processing, Agent Runtime for execution) with communication patterns, role access (Admin/Super User/Developer via CLI+Dashboard), and scaling characteristics (replicas, load, HPA) |
| 004 | **004_data_flow.md** | Trace end-to-end request journey through eleven processing steps (IC Token validation → Provider → Response) with bidirectional flow (request/provider/response phases), latency analysis (Pilot ~25ms overhead, Production ~100ms overhead), security validation (input + output firewall), and failure modes (fail-safe input, fail-open cost) |
| 005 | **005_service_integration.md** | Explain how five core services communicate with Gateway (8084) orchestrating Safety (8080), Cost (8081), Tool Proxy (8082), Audit (8083) services through centralized coordination pattern, including port assignments, service dependencies (database, cache, object storage), call sequence (7 steps), and failure handling strategies (fail-safe Safety, fail-open Cost/Audit) |
| 006 | **006_roles_and_permissions.md** | Define three-role RBAC (Admin>User>Viewer) with permission matrix (25 permissions across User Management/Control Panel/Tokens/Agents), user management operations (create/suspend/activate/delete), account lifecycle (Created→Suspended→Activated→Deleted), self-modification prevention, audit trail (user_audit_log append-only), token management scoping (admin regenerates any, user/viewer own only) |
| 007 | **007_entity_model.md** | Define seven core entities (User with roles Admin/User/Viewer, Master Project admin-only containing ALL resources, Project, Agent, IP/Inference Provider, IC Token 1:1 with Agent, Budget Change Request state machine pending→approved/rejected/cancelled) with relationships (1:1 Agent-IC Token/Agent Budget restrictive, 1:N User-Agents, N:M User-Projects), deletion policies (IP ON DELETE CASCADE removes from all agents, User soft delete reassigns agents to Orphaned Agents Project owned by admin), budget types (restrictive: Agent Budget blocks requests when exceeded; informative: Project/IP/Master budgets monitoring only), token lifecycle (IC Token long-lived no auto-expiration, User Token default 30 days, IP Token in encrypted vault) |
| 008 | **008_runtime_modes.md** | Define two runtime deployment modes (Router: HTTP-based separate process ~5ms overhead for existing frameworks LangChain/CrewAI zero code changes or SDK users optional debugging; Library: PyO3-embedded in-process ~0.5ms overhead default for SDK users best performance) with identical developer code across modes (mode is deployment configuration via IRON_RUNTIME_URL environment variable or CLI --mode=router flag, not API), Router use cases (Use Case 1: framework endpoint configuration api.openai.com→localhost:8080 no iron_sdk required; Use Case 2: SDK users HTTP traffic inspection process isolation), Library characteristics (default SDK behavior uv pip install single process PyO3 FFI), trade-offs (deployment complexity, debugging visibility, process isolation, performance overhead), competitive advantage (both modes on-premise no data exposure vs competitors centralized servers) |
| 009 | **009_resource_catalog.md** | Exhaustive catalog of all 23 REST API resources exposed by Iron Cage Control Panel organized by four resource types (Entity Resources: CRUD operations with 1:1 or 1:N entity mapping 6 resources; Operation Resources: RPC-style multi-entity actions 4 resources; Analytics Resources: read-only derived data aggregations 8 resources; Configuration Resources: admin-only system settings 3 resources; System Resources: public monitoring 2 resources), entity mapping patterns (Direct 1:1 resource-to-entity, Multiple Resources 1:N per entity, Resource Spans N:1 multiple entities, No Direct Mapping for derived analytics), three authentication patterns (IC Token: agent auth JWT ic_ prefix long-lived used by iron_runtime; User Token: Control Panel access 30-day refreshable user+project scope used by iron_cli and dashboard; No Authentication: public health/version endpoints), certainty classification (8 Certain Pilot-required, 13 MUST-HAVE production-critical specifications complete, 1 NICE-TO-HAVE UX enhancement, 1 POST-PILOT future), CLI-API parity principle (every user-facing API resource maps to iron CLI command group), references 13 detailed Protocol API specifications (006-017) for request/response schemas |

---

## Architecture Collection

| ID | Name | Purpose |
|----|------|---------|
| 000 | [High-Level Overview](000_high_level_overview.md) | **START HERE** - All actors, components, collaboration patterns |
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
                   +-------------------------+
                   | High-Level Overview     |
                   | (START HERE)            |
                   | All actors, components, |
                   | collaboration patterns  |
                   +-----------+-------------+
                               |
              +----------------+----------------+
              v                                 v
    +---------------------+          +---------------------+
    |  Execution Models   |          | Roles & Permissions |
    |  (where agents run) |          | (who can do what)   |
    +----------+----------+          +---------------------+
               |
+------+-------+-------+-------+
|      |       |       |       |
v      v       v       v       v
Layer  Data   Service Entity  Runtime
Model  Flow   Bound.  Model   Modes
```

*For capability concepts, see [capabilities/](../capabilities/readme.md)*
*For deployment concepts, see [deployment/](../deployment/readme.md)*
