# Architecture: Actor Model

### Scope

This document defines all actors (human, software, service) that interact with the Iron Cage platform.

**In scope:**
- Complete actor taxonomy (Human, Software, Service actors with precise roles)
- Actor responsibilities and access levels
- Primary interfaces for each actor type
- Actor interaction patterns and communication protocols

**Out of scope:**
- Detailed RBAC permissions → See [Architecture: Roles and Permissions](../architecture/006_roles_and_permissions.md)
- Service implementation details → See service-specific documentation
- Authentication protocols → See [Protocol: Authentication API](../protocol/007_authentication_api.md)

### Purpose

**User Need**: Understand who and what interacts with Iron Cage, their roles, responsibilities, and access levels.

**Solution**: Define three actor types (Human, Software, Service) with 18 total actors - 6 human roles, 7 software systems, and 5 core services - each with clear responsibilities and interfaces.

**Key Insight**: Iron Cage has three actor planes matching the architecture planes - human actors manage via Control Panel (Admin/User/Viewer), software actors integrate via APIs/SDKs, and service actors process requests via Gateway orchestration.

**Status**: Specification
**Version**: 2.0.0
**Last Updated**: 2025-12-14

---

## 1. HUMAN ACTORS

Human actors are people who interact with the Iron Cage platform through CLI and/or Dashboard interfaces.

| Actor | Primary Role | Access Level | Primary Interface | Key Responsibilities |
|-------|-------------|--------------|-------------------|---------------------|
| **Admin** | Platform administrator | Full Control Panel access (all users, all data) | CLI + Dashboard | Create users, allocate budgets, manage IP Tokens (provider credentials), configure safety policies, review audit logs, revoke access |
| **User** | AI agent developer | Standard Control Panel access (own data only) | CLI + Dashboard | Create and run agents locally, view own usage and spending, select models/providers (from allowed list), request budget increases |
| **Viewer** | Read-only stakeholder | Read-only Control Panel access (own data only) | CLI + Dashboard | View own usage, view budgets and spending, monitor spending real-time (graphs/charts) |
| **Operations Engineer** | System monitoring | Full monitoring access | Dashboard (Grafana/similar) | Monitor execution metrics, track system health, investigate performance issues, manage alerts |
| **Security Engineer** | Compliance and isolation | Audit log access | Dashboard + Audit tools | Review audit logs, implement isolation policies, conduct security investigations, manage compliance |
| **Product Visitor** | Pre-customer | Public marketing site | Web browser | View marketing content, learn about platform, sign up for trial |

**Role Hierarchy:**
```
Admin (Full) > User (Standard) > Viewer (Read-Only)
```

**Critical Constraints:**
- All three primary roles (Admin, User, Viewer) use **BOTH CLI and Dashboard** (equivalent interface, not role-specific)
- Admin cannot modify own account (self-modification prevention)
- Admin cannot delete last admin user (system integrity protection)
- Users/Viewers can only regenerate own tokens (Admin can regenerate any tokens)

**See:** [Architecture: Roles and Permissions](../architecture/006_roles_and_permissions.md) for complete RBAC specification

---

## 2. SOFTWARE ACTORS

Software actors are external systems and client applications that interact with Iron Cage.

| Actor | Type | Purpose | Communication Protocol | Authentication |
|-------|------|---------|------------------------|----------------|
| **Python Agent** | User's AI application | Execute LLM-based workflows (LangChain, CrewAI, custom) | SDK intercepts LLM calls → HTTPS to Gateway | IC Token (JWT) |
| **Web Browser** | Client application | Access Control Panel Dashboard | HTTPS REST API + WebSocket | User Token (JWT) |
| **Terminal / CLI Tool** | Client application | Execute iron CLI commands | HTTPS REST API | User Token (JWT) |
| **LLM Provider API** | External service | Process LLM inference requests (OpenAI, Anthropic, etc.) | HTTPS (via Gateway) | IP Token (provider-specific) |
| **Database** | Infrastructure | Persist platform state (users, agents, budgets, audit logs) | PostgreSQL protocol (pilot: SQLite) | Database credentials |
| **Cache** | Infrastructure | Store hot data (budget tracking, permissions) | Redis protocol (pilot: in-memory) | Redis credentials |
| **Object Storage** | Infrastructure | Store audit logs, large payloads | S3 protocol | S3 credentials |

**Critical Flows:**

**Python Agent → Gateway:**
```
Developer code: openai.chat.completions.create(...)
         ↓
SDK intercepts call
         ↓
SDK adds IC Token header: Authorization: Bearer ic_abc123...
         ↓
SDK sends to Gateway: POST http://localhost:8084/v1/chat/completions
         ↓
Gateway validates IC Token, checks budget, forwards to provider
```

**Browser/CLI → Control Panel:**
```
User: iron users create --username alice --role user
         ↓
CLI sends: POST /api/v1/users
           Authorization: Bearer user_xyz789...
           {"username": "alice", "role": "user", ...}
         ↓
Control Panel validates User Token, checks Admin permission, creates user
```

**See:** [Architecture: Data Flow](../architecture/004_data_flow.md) for complete request journey

---

## 3. SERVICE ACTORS

Service actors are internal Iron Cage microservices that process requests and enforce policies.

| Service | Port | Plane | Primary Responsibility | Dependencies | Failure Mode |
|---------|------|-------|------------------------|--------------|--------------|
| **Gateway** | 8084 | Data Plane | Central orchestrator coordinating all services, routing requests | All services below | N/A (single point, must be highly available) |
| **Safety Service** | 8080 | Data Plane | Input/output validation (prompt injection detection, PII scanning, secret redaction) | Database (validation patterns) | **Fail-safe** (block all if down) |
| **Cost Service** | 8081 | Data Plane | Budget tracking and enforcement (check budgets, track spending, report usage) | Database (budgets), Cache (hot data) | **Fail-open** (allow with degraded tracking) |
| **Tool Proxy** | 8082 | Data Plane | Tool authorization and validation (validate tool params, authorize execution) | Cache (permissions) | **Fail-safe** (block tool execution) |
| **Audit Service** | 8083 | Data Plane | Compliance logging (log all requests/responses, track audit trail) | Database (logs), Object Storage (payloads) | **Fail-open** (buffer in queue) |
| **API Gateway** | 443 | Control Plane | Control Panel REST API (user management, agent management, token operations) | Database (platform state) | **Fail-safe** (deny access) |
| **Dashboard** | 443 | Control Plane | Web UI for Control Panel (Vue SPA, policy management, monitoring) | API Gateway | Degrades gracefully (UI unavailable) |
| **Scheduler** | N/A | Control Plane | Background jobs (token expiration, budget rollover, cleanup) | Database | Degrades gracefully (delayed operations) |

**Service Planes:**

```
CONTROL PLANE (Admin Management)
+-----------------------------------------------------------+
|  API Gateway (443) ← handles User Token authentication    |
|  Dashboard (443) ← Vue SPA for admin/user/viewer          |
|  Scheduler ← background jobs                              |
+-----------------------------------------------------------+
                        manages ↓
DATA PLANE (Request Processing)
+-----------------------------------------------------------+
|  Gateway (8084) ← orchestrates all below                  |
|    ├─→ Safety (8080) ← input/output validation           |
|    ├─→ Cost (8081) ← budget enforcement                   |
|    ├─→ Tool Proxy (8082) ← tool authorization            |
|    └─→ Audit (8083) ← compliance logging                 |
+-----------------------------------------------------------+
                        serves ↓
AGENT RUNTIME (Local Execution)
+-----------------------------------------------------------+
|  Python Agent (developer machine) ← uses IC Token        |
|  SDK (intercepts LLM calls) ← sends to Gateway           |
+-----------------------------------------------------------+
```

**See:** [Architecture: Service Boundaries](../architecture/003_service_boundaries.md) for complete plane separation model

---

## Cross-References

### Related Architecture Documents
- [Architecture: High-Level Overview](../architecture/000_high_level_overview.md) - Complete system overview using these actors
- [Architecture: Execution Models](../architecture/001_execution_models.md) - Control Panel deployment context
- [Architecture: Service Boundaries](../architecture/003_service_boundaries.md) - Three-plane separation
- [Architecture: Roles and Permissions](../architecture/006_roles_and_permissions.md) - RBAC for human actors

### Related Protocol Documents
- [Protocol: Authentication API](../protocol/007_authentication_api.md) - User Token authentication for human actors
- [Protocol: Budget Control](../protocol/005_budget_control_protocol.md) - IC Token protocol for agent actors

### Related Deployment Documents
- [Deployment: Package Model](001_package_model.md) - Package structure used by actors
