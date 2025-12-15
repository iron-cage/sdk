# Architecture: Entity Model

### Scope

This document defines the core entities and their relationships in the Iron Cage platform.

**In Scope:**
- Seven core entities (User, Master Project, Project, Agent, IP/Inference Provider, IC Token, Budget Change Request)
- Entity relationships and cardinality (1:1, 1:N, N:M)
- Entity attributes and unique identifiers
- Deletion cascades and reassignment policies (IP cascades, User reassignment)
- Budget types (Restrictive: Agent Budget; Informative: Project, IP, Master budgets)
- Token lifecycle management (IC Token, User Token, IP Token)
- Special entities (Master Project admin-only, Orphaned Agents Project)
- Entity state machines (Budget Change Request: pending → approved/rejected/cancelled)

**Out of Scope:**
- Database schema implementation (covered in Database Design)
- API endpoint specifications (covered in Protocol documents 008, 017, 005)
- Entity validation rules (covered in API Reference)
- UI representation of entities (covered in Frontend Architecture)
- Entity access control (covered in Architecture: Roles and Permissions)
- Entity lifecycle hooks and business logic (covered in Service Implementation)

### Purpose

**User Need**: Platform developers, architects, and administrators need to understand the main entities (Agent, Project, Inference Provider, User, Tokens, Budgets) and how they relate to design data models, implement APIs, and manage platform resources.

**Solution**: Define seven core entities with clear relationships:

```
User (person)
  ├─ has role (Admin, User, Viewer)
  ├─ belongs to Project(s)
  ├─ owns multiple Agents
  └─ has multiple User Tokens
       |
       └─ Master Project (admin only)
            ├─ contains ALL Projects
            └─ contains ALL Users
                 |
                 └─ Project
                      ├─ contains multiple Users
                      ├─ contains multiple Agents
                      ├─ contains multiple IPs
                      └─ has Project Budget (informative)
                           |
                           └─ Agent
                                ├─ owned by one User
                                ├─ has one IC Token (1:1)
                                ├─ has one Agent Budget (1:1, restrictive)
                                └─ can use multiple IPs (developer selects)
                                     |
                                     └─ IP (Inference Provider)
                                          ├─ has IP Budget (informative)
                                          └─ has IP Token(s)
```

> **Note:** "IP" in this document means **Inference Provider** (e.g., OpenAI, Anthropic), NOT IP address.

**Key Insight**: Budget control is centralized through agents - the Agent Budget is the ONLY restrictive budget that blocks requests when exceeded. All other budgets (Project, IP, Master) are informative for monitoring only. This design keeps control simple and predictable. Critical relationships include 1:1 Agent-IC Token (can't share), deletion cascades (IP deletion removes from all agents), and agent reassignment (deleted user's agents move to Orphaned Agents Project owned by admin).

---

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13

### Entity 1: Agent

**Definition:** AI agent executing on developer's machine

**Relationships:**
- Has exactly one IC Token (1:1, can't share)
- Has exactly one Agent Budget (1:1, restrictive)
- Can use multiple Inference Providers/IPs (developer selects which to use)
- Belongs to exactly one Project

**Attributes:**
- agent_id (unique identifier)
- owner (developer)
- status (running, stopped, etc.)

### Entity 2: Project

**Definition:** Collection of agents, Inference Provider assignments, and related entities

**Relationships:**
- Contains multiple Agents
- Contains multiple Inference Provider (IP) assignments
- Has exactly one Project Budget (1:1, informative)
- Owned by admin or team

**Attributes:**
- project_id (unique identifier)
- name (human-readable)
- owner (admin or team)

### Entity 3: Master Project

**Definition:** Special project containing ALL resources (admin-only)

**Relationships:**
- Contains ALL agents across all projects
- Contains ALL Inference Providers (IPs)
- Has Master Budget (informative, aggregates all)
- Available only to admin

**Pilot Requirement:** Master project MUST be implemented in Pilot for admin oversight

### Entity 4: IP (Inference Provider)

**Definition:** LLM inference provider (OpenAI, Anthropic, etc.)

**Relationships:**
- Has IP Budget (informative, tracks spending per provider)
- Has IP Token(s): Pilot = 1, Future = multiple
- Can be assigned to multiple Agents
- **Deletion cascades:** When provider deleted, automatically removed from all agent assignments (ON DELETE CASCADE)

**Lifecycle:**
- Created: Admin creates provider via `POST /api/v1/providers`
- Assigned: Provider assigned to agents via `POST /api/v1/agents` or `PUT /api/v1/agents/{id}/providers`
- Deleted: Admin deletes provider via `DELETE /api/v1/providers/{id}` - **cascades to remove from all agents**

**Attributes:**
- ip_id (unique identifier)
- provider_name (openai, anthropic, etc.)
- endpoint_url

### Entity 5: User

**Definition:** Person using Iron Cage platform (admin, developer, super user)

**Relationships:**
- Has role (Admin, User, or Viewer)
- Owns/creates multiple Agents (1:N)
- Belongs to Project(s) (N:M)
- Has multiple User Tokens (1:N)
- **Deletion reassigns agents:** When user deleted, all owned agents automatically transferred to admin in "Orphaned Agents" project

**Attributes:**
- user_id (unique identifier)
- email (login)
- role (Admin, User, Viewer)

**Token Permissions:**
- Can regenerate own IC Tokens (for owned agents)
- Can regenerate own User Tokens
- Admin can regenerate ANY tokens (all users, all agents)

**Lifecycle:**
- Created: Admin creates user via `POST /api/v1/users`
- Active: User can login, create agents, use platform
- Suspended: Admin suspends via `PUT /api/v1/users/{id}/suspend` - user cannot login
- Activated: Admin reactivates via `PUT /api/v1/users/{id}/activate` - user can login again
- Deleted: Admin deletes via `DELETE /api/v1/users/{id}` - **soft delete + agent reassignment**

**Deletion Behavior:**
- User soft-deleted (deleted_at timestamp set, cannot login)
- All owned agents reassigned to admin (`owner_id` changed)
- All agents moved to "Orphaned Agents" project (`proj-orphaned`)
- Pending budget requests auto-cancelled
- All API tokens revoked
- Agents continue working (IC Tokens valid, budgets active)
- Audit trail preserved with full reassignment details

**Orphaned Agents Project:**
- Special project containing all deleted users' agents
- Admin can reassign agents to new users or delete
- Agents remain operational (budgets, IC Tokens, providers unchanged)

**See:** [Protocol 008](../protocol/008_user_management_api.md) (User Management API)

### Entity 6: IC Token

**Definition:** Iron Cage Token for agent authentication

**Relationships:**
- Belongs to exactly one Agent (1:1)
- Owned by exactly one User (via agent ownership)
- Agent can't have multiple IC Tokens
- IC Token can't belong to multiple agents

**Lifecycle:** Created with agent, lives until agent deleted or regenerated (long-lived, no auto-expiration)

### Entity 7: Budget Change Request

**Definition:** Request from developer to admin for modifying agent budget

**Relationships:**
- Linked to exactly one Agent (1:1 per request)
- Created by one User (requester, typically developer/agent owner)
- Reviewed by one User (reviewer, must be admin)
- Linked to Budget Modification History entry (on approval)

**Attributes:**
- request_id (unique identifier, breq- prefix)
- agent_id (target agent)
- requester_id (user who created request)
- current_budget (snapshot at request time)
- requested_budget (desired budget amount)
- justification (20-500 chars, required)
- status (pending, approved, rejected, cancelled)
- reviewed_by (admin user_id, null until reviewed)
- review_notes (optional on approval, required on rejection)

**State Machine:**
- Created: `pending`
- Terminal states: `approved`, `rejected`, `cancelled`
- Auto-cancel: If agent deleted while request pending

**Lifecycle:** Created via POST, reviewed by admin via PUT (approve/reject), or cancelled by requester via DELETE (before review only)

**See:** [Protocol 017](../protocol/017_budget_requests_api.md)

### Budget Types

**Restrictive Budget (ONE TYPE ONLY):**
- **Agent Budget:** Blocks requests when exceeded
- Hard limit enforcement
- Real-time tracking

**Informative Budgets (STATISTICS ONLY):**
- **Project Budget:** Shows project spending, no blocking
- **IP Budget:** Shows provider spending, no blocking
- **Master Budget:** Shows all spending, no blocking

**Key Point:** ONLY agent budget blocks requests. All others are for monitoring.

### Token Lifecycle Management

**IC Token (Agent Authentication):**
- Created: When agent is created by admin
- Lifetime: Until agent deleted (long-lived, no auto-expiration)
- Regeneration: User can regenerate own (replaces existing), Admin can regenerate any
- 1:1 with agent: Cannot be shared or transferred
- Invalidation: When agent deleted or IC Token regenerated

**User Token (Control Panel Access):**
- Created: When user account created or when user requests new token
- Lifetime: Configurable (default 30 days)
- Regeneration: User can regenerate own, Admin can regenerate any
- Multiple per user: User can have multiple active User Tokens

**IP Token (Provider Credential):**
- Created: Admin adds to Control Panel vault
- Lifetime: Provider-managed (typically long-lived)
- Regeneration: Admin only (in Control Panel vault)
- Users never see: Stored encrypted in Control Panel

### Budget Control Principle

**Critical Design Decision:** Agents are the ONLY way to control budget.

**Why:**
- Agent budget: Blocks requests when exceeded (restrictive)
- All other budgets: Statistics/monitoring only (informative)

**Want to limit spending?**
- Create agent with specific budget limit
- Agent budget will block requests when exceeded

**Want to monitor spending?**
- View project budget (all agents in project)
- View Inference Provider budget (all usage of provider)
- View master budget (everything)

**Can't control via:**
- Project budget (informative only)
- Inference Provider budget (informative only)
- Master budget (informative only)

This design keeps control simple and predictable.

---

### Cross-References

#### Related Principles Documents
- Design Philosophy - Entity modeling principles, relationship design patterns
- Quality Attributes - Data consistency (entity integrity), Scalability (entity relationships), Security (token lifecycle)

#### Related Architecture Documents
- [Architecture: Execution Models](001_execution_models.md) - Entity contexts across execution modes (local agent uses IC Token, Control Panel uses User Token)
- [Architecture: Data Flow](004_data_flow.md) - Entity data flow through system (Step 1: Agent authentication with IC Token, Step 2: Budget check against Agent Budget)
- [Architecture: Service Integration](005_service_integration.md) - Entity references at service boundaries (Gateway validates IC Token, Budget Service checks Agent Budget, Provider Service uses IP Token)
- [Architecture: Roles and Permissions](006_roles_and_permissions.md) - User entity role definitions (Admin, User, Viewer), token regeneration permissions by role, user management operations affecting User entity
- [Architecture: Resource Catalog](009_resource_catalog.md) - Resource type definitions mapped to entities (agents, projects, users, providers as resources)

#### Used By
- Database Schema Design - Implements entity models as database tables (users, agents, projects, inference_providers, ic_tokens, user_tokens, ip_tokens, budget_change_requests)
- API Gateway - Validates entity relationships on incoming requests (IC Token belongs to Agent, Agent belongs to Project, User has role permissions)
- Budget Service - Enforces budget controls on Agent entity (checks Agent Budget, updates spending, blocks when exceeded)
- Token Management Service - Manages token lifecycle for IC Token, User Token, IP Token entities
- User Management Service - Implements User entity operations (create, suspend, activate, delete with agent reassignment to Orphaned Agents Project)
- Agent Service - Manages Agent entity lifecycle (create with IC Token, assign to Project, configure Agent Budget, select IPs)
- Project Service - Manages Project and Master Project entities (contains Agents, IPs, Users)

#### Dependencies
- User Management Protocol - Defines User entity CRUD operations, role assignment, account lifecycle states
- Budget Control Protocol - Defines Budget entity types (Agent restrictive, Project/IP/Master informative), budget enforcement logic
- Token Management Protocol - Defines token entity lifecycles (IC Token long-lived no expiration, User Token 30 days, IP Token in vault)
- Database Design - Entity persistence layer (tables, relationships, constraints, ON DELETE CASCADE for IP, soft delete for User)

#### Implementation
- Database: `users` table (user_id, role enum [admin, user, viewer], is_active, deleted_at)
- Database: `agents` table (agent_id, owner_id FK users, project_id FK projects, status)
- Database: `projects` table (project_id, name, owner, special flag for Master Project and Orphaned Agents Project)
- Database: `inference_providers` table (ip_id, provider_name, endpoint_url, ON DELETE CASCADE to agent_providers)
- Database: `ic_tokens` table (token_id, agent_id FK agents 1:1, created_at, never expires)
- Database: `user_tokens` table (token_id, user_id FK users, created_at, expires_at default 30 days)
- Database: `ip_tokens` table (token_id, ip_id FK inference_providers, encrypted_value)
- Database: `agent_budgets` table (budget_id, agent_id FK agents 1:1, amount, restrictive flag true)
- Database: `project_budgets` table (budget_id, project_id FK projects 1:1, informative flag true)
- Database: `ip_budgets` table (budget_id, ip_id FK inference_providers 1:1, informative flag true)
- Database: `budget_change_requests` table (request_id, agent_id FK agents, requester_id FK users, current_budget, requested_budget, justification, status enum [pending, approved, rejected, cancelled], reviewed_by FK users, review_notes)
- API: `POST /api/v1/agents` - Create Agent with IC Token and Agent Budget
- API: `DELETE /api/v1/providers/{id}` - Delete IP with ON DELETE CASCADE to all agent assignments
- API: `DELETE /api/v1/users/{id}` - Soft delete User with agent reassignment to Orphaned Agents Project
- API: `POST /api/v1/budget-requests` - Create Budget Change Request (see Protocol 017)
- API: `PUT /api/v1/agents/{id}/providers` - Assign IPs to Agent (N:M relationship)
- See: [Protocol 008](../protocol/008_user_management_api.md) - User Management API
- See: [Protocol 017](../protocol/017_budget_requests_api.md) - Budget Requests API
- See: [Protocol 005](../protocol/005_budget_control_protocol.md) - Budget Control Protocol
