# Architecture: Service Integration

### Scope

This document defines the five core runtime services and their communication patterns, with Gateway as the central orchestrator for Safety, Cost, Tool Proxy, and Audit services.

**In scope:**
- Five service architecture (Gateway, Safety, Cost, Tool Proxy, Audit)
- Gateway orchestration pattern (centralized coordination of specialized services)
- Service communication patterns (Gateway→Service request/response flow)
- Port assignments (8080-8084 for each service)
- Service dependencies (database, cache, object storage requirements)
- Failure handling strategies (fail-safe vs fail-open per service)
- Call sequence for typical request flow (7 steps from agent to response)

**Out of scope:**
- Service implementation details → See service-specific documentation
- API endpoint specifications → See protocol documentation
- Database schema details → See [Technology: Infrastructure Choices](../technology/003_infrastructure_choices.md)
- Deployment topology and infrastructure → See [Deployment](../deployment/readme.md)
- Load balancing and service discovery → See deployment documentation

### Purpose

**User Need**: Understand service dependencies and communication patterns for debugging and operations.

**Solution**: Gateway orchestration pattern where a central Gateway service coordinates calls to four specialized services (Safety, Cost, Tool Proxy, Audit):

**Gateway orchestrates calls to specialized services:**

```
                    +-----------------+
                    |    Gateway      |
                    |   (Port 8084)   |
                    +--------+--------+
         +--------------+----+----+--------------+
         v              v         v              v
   +----------+  +----------+ +----------+ +----------+
   |  Safety  |  |   Cost   | |Tool Proxy| |  Audit   |
   |  :8080   |  |  :8081   | |  :8082   | |  :8083   |
   +----------+  +----------+ +----------+ +----------+
```

**The five services:**
- **Safety (8080)**: Input/output validation with database for pattern storage
- **Cost (8081)**: Budget tracking with database + cache for performance
- **Tool Proxy (8082)**: Tool authorization with cache for permissions
- **Audit (8083)**: Compliance logging with database + object storage for audit trail
- **Gateway (8084)**: Orchestration coordinating all above services

**Key Insight**: Gateway acts as the central orchestrator, making synchronous calls to Safety and Cost (blocking for security/budget enforcement), while Audit runs asynchronously (non-blocking for performance). Failure modes differ by service criticality - Safety fails safe (block all), Cost/Audit fail open (allow with degraded tracking). This separation of concerns allows independent scaling and clear service boundaries.

**Status**: Specification
**Version**: 1.0.0
**Last Updated**: 2025-12-13

### The Five Services

| Service | Port | Purpose | Deps |
|---------|------|---------|------|
| Safety | 8080 | Input/output validation | Database |
| Cost | 8081 | Budget tracking | Database, Cache |
| Tool Proxy | 8082 | Tool authorization | Cache |
| Audit | 8083 | Compliance logging | Database, Object Storage |
| Gateway | 8084 | Orchestration | All above |

*Note: Cache = In-memory (pilot) or Redis (production). Database = SQLite/PostgreSQL. Object Storage = S3/compatible. See [technology/003](../technology/003_infrastructure_choices.md).*

### Call Sequence

1. Agent calls SDK -> SDK calls Gateway
2. Gateway -> Safety (validate input)
3. Gateway -> Cost (check budget)
4. Gateway -> Provider (forward request)
5. Gateway -> Safety (validate output)
6. Gateway -> Audit (log event, async)
7. Gateway -> Agent (return response)

### Failure Handling

| Service Down | Behavior |
|--------------|----------|
| Safety | BLOCK all (fail-safe) |
| Cost | ALLOW, track in memory |
| Tool Proxy | BLOCK tool execution |
| Audit | ALLOW, buffer in queue (in-memory or cache) |

---

### Cross-References

#### Related Principles Documents
- [Principles: Design Philosophy](../principles/001_design_philosophy.md) - Fail-Safe principle reflected in Safety service failure mode, Separation of Concerns via service boundaries
- [Principles: Quality Attributes](../principles/002_quality_attributes.md) - Reliability via failure handling strategies, Maintainability via service separation, Scalability via independent service scaling
- [Principles: Development Workflow](../principles/005_development_workflow.md) - Specification-first approach applied to this architecture document

**Related Architecture Documents:**
- [Architecture: Execution Models](001_execution_models.md) - Runtime modes that these services support
- [Architecture: Layer Model](002_layer_model.md) - Processing layers implemented by these services
- [Architecture: Service Boundaries](003_service_boundaries.md) - Data Plane services that these five services represent
- [Architecture: Data Flow](004_data_flow.md) - Request flow through these services (eleven steps)
- [Architecture: Roles and Permissions](006_roles_and_permissions.md) - Authorization enforced by these services
- [Architecture: Entity Model](007_entity_model.md) - Entities managed by these services

#### Used By
- [Architecture: Data Flow](004_data_flow.md) - Eleven-step flow implements service communication patterns
- Implementation documentation - Service deployment and configuration

#### Dependencies
- [Technology: Infrastructure Choices](../technology/003_infrastructure_choices.md) - Cache (In-memory/Redis), Database (SQLite/PostgreSQL), Object Storage (S3/compatible) technology choices

#### Implementation
- Gateway service: Port 8084, orchestrates all service calls, no direct dependencies
- Safety service: Port 8080, database for validation patterns
- Cost service: Port 8081, database + cache for budget tracking
- Tool Proxy service: Port 8082, cache for permission caching
- Audit service: Port 8083, database + object storage for compliance logs
