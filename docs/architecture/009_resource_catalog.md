# Architecture: Resource Catalog

### Scope

This document provides an exhaustive catalog of all REST API resources exposed by the Iron Cage Control Panel.

**In Scope:**
- Complete inventory of 23 API resources across four resource types (Entity, Operation, Analytics, Configuration)
- Four resource type taxonomy (Entity Resources: CRUD 1:1 or 1:N entity mapping; Operation Resources: RPC-style multi-entity actions; Analytics Resources: read-only derived data aggregations; Configuration Resources: admin-only system settings)
- Entity mapping patterns (Direct 1:1 resource-to-entity, Multiple Resources per Entity 1:N, Resource Spans Multiple Entities N:1, No Direct Mapping for derived data)
- Three authentication patterns (IC Token: agent auth JWT with ic_ prefix long-lived; User Token: Control Panel access 30-day refreshable; No Authentication: public health/version endpoints)
- Certainty classification (8 Certain Pilot-required resources, 13 MUST-HAVE production-critical, 1 NICE-TO-HAVE UX enhancement, 1 POST-PILOT future implementation)
- CLI-API parity principle (every user-facing API resource maps to iron CLI command group)
- HTTP method conventions (GET read, POST create/action, PUT update, DELETE remove)
- Resource naming conventions (plural for entities /api/tokens, singular/verb for operations /api/auth)
- Protocol document references (13 detailed API specifications in Protocol collection)
- Inventory tables (Entity: 6 resources, Operation: 4 resources, Analytics: 8 resources, Configuration: 3 resources, System: 2 resources)

**Out of Scope:**
- Detailed request/response schemas (covered in individual Protocol documents 006-017)
- Authentication implementation details (covered in Protocol 007: Authentication API)
- Budget control protocol specifics (covered in Protocol 005: Budget Control Protocol)
- User management RBAC rules (covered in Architecture 006: Roles and Permissions)
- Entity relationships and lifecycle (covered in Architecture 007: Entity Model)
- REST API design principles (covered in Protocol 002: REST API Protocol)
- HTTP status codes and error handling (covered in Protocol 002)
- Rate limiting and throttling (covered in Protocol 002)
- Pagination and filtering implementation (covered in Protocol 002)
- Specific endpoint request/response examples (covered in Protocol 006-017)

### Purpose

**User Need**: Architects, backend developers, SDK developers, and CLI developers need a single authoritative catalog of all 23 REST API resources exposed by the Iron Cage Control Panel to understand the complete API surface (four resource types: Entity CRUD 1:1 mapping, Operation RPC-style actions, Analytics derived data, Configuration admin settings), entity mappings (1:1, 1:N, N:1, derived), authentication requirements (IC Token agent auth, User Token dashboard access, public endpoints), certainty levels (8 Certain Pilot-required, 13 MUST-HAVE production-critical, 1 NICE-TO-HAVE, 1 POST-PILOT), and CLI-API parity mappings for consistent implementation across API endpoints, Python SDK methods, and iron CLI command groups.

**Solution**: Exhaustive inventory organized by four resource types with detailed tables cataloging:

**Complete Resource Inventory (23 Total Resources):**

| Resource Type | Count | Examples | Purpose |
|---------------|-------|----------|---------|
| **Entity Resources** | 6 | `/api/tokens`, `/api/users`, `/api/agents` | CRUD operations, 1:1 or 1:N entity mapping |
| **Operation Resources** | 4 | `/api/auth`, `/api/budget/handshake` | RPC-style actions, multi-entity operations |
| **Analytics Resources** | 8 | `/api/usage`, `/api/metrics` | Read-only derived data, aggregations |
| **Configuration Resources** | 3 | `/api/limits`, `/api/settings` | Admin-only system configuration |
| **System Resources** | 2 | `/api/health`, `/api/version` | Public operational monitoring |

**Three Authentication Patterns:**
- **IC Token**: Agent authentication (JWT with `ic_` prefix, long-lived until agent deleted, used by iron_runtime)
- **User Token**: Control Panel access (JWT, 30-day refreshable, user + project scope, used by iron_cli and web dashboard)
- **No Authentication**: Public endpoints (`/api/health`, `/api/version`)

**Certainty Classification:**
- ✅ **Certain (8)**: Required for Pilot launch
- ✅ **MUST-HAVE (13)**: Critical for production, specifications complete
- ✅ **NICE-TO-HAVE (1)**: Enhances UX, specification complete
- 📋 **POST-PILOT (1)**: Future implementation, specification prepared

**CLI-API Parity**: Every user-facing API resource maps to `iron` CLI command group (e.g., `/api/tokens` → `iron tokens`, `/api/agents` → `iron agents`).

**Key Insight**: Resource taxonomy uses four distinct patterns (Entity Resources for CRUD with direct entity mapping, Operation Resources for RPC-style multi-entity actions, Analytics Resources for read-only derived aggregations, Configuration Resources for admin-only system settings) with three-tier authentication model (IC Token for agent runtime, User Token for human users, public for monitoring) and four-level certainty classification (Certain Pilot-required, MUST-HAVE production-critical with complete specifications, NICE-TO-HAVE UX enhancements, POST-PILOT future work) ensuring comprehensive API surface coverage across 23 resources with consistent CLI parity (every user-facing API resource has corresponding iron CLI command group) and clear entity mapping patterns (1:1 direct, 1:N multiple resources per entity, N:1 resource spans entities, derived for analytics with no direct mapping).

---

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13

**Note:** This document references the entity model from [007_entity_model.md](007_entity_model.md). For entity definitions, relationships, and lifecycle details, see that document.

### Resource Taxonomy

**Four Resource Types:**

### 1. Entity Resources (CRUD)

**Definition:** REST resources with 1:1 or 1:N mapping to domain entities, supporting standard CRUD operations.

**Characteristics:**
- Direct entity manipulation (Create, Read, Update, Delete)
- Plural resource names (`/api/tokens`, `/api/projects`)
- Standard HTTP methods (GET, POST, PUT, DELETE)
- Maps to entity in [007_entity_model.md](007_entity_model.md)

**Examples:**
- `/api/tokens` → IC Token entity (GET, POST, DELETE, PUT/rotate)
- `/api/projects` → Project entity (GET, POST, PUT, DELETE)

**CLI Mapping:** Entity name matches CLI command group
- `/api/tokens` → `iron tokens`
- `/api/projects` → `iron projects`

### 2. Operation Resources (RPC-style)

**Definition:** Resources exposing operations or actions that don't map directly to single entity CRUD.

**Characteristics:**
- Action-oriented (login, refresh, handshake)
- Often POST-only or POST+GET
- May involve multiple entities
- Singular or verb-based names (`/api/auth`, `/api/budget/handshake`)

**Examples:**
- `/api/auth` → Login/logout operations (User + User Token)
- `/api/budget/handshake` → Budget negotiation (Agent + Project)

**CLI Mapping:** Maps to CLI commands (not command groups)
- `/api/auth` → `iron login`, `iron logout`
- `/api/budget/refresh` → `iron budget refresh`

### 3. Analytics Resources (Derived Data)

**Definition:** Read-only resources providing aggregated or derived metrics from multiple entities.

**Characteristics:**
- Read-only (GET only)
- Derived from multiple entities
- Statistical/analytical nature
- Names reflect data type (`/api/usage`, `/api/metrics`)

**Examples:**
- `/api/analytics` → Usage, cost, and performance metrics
- `/api/analytics/usage` → Usage statistics across agents/projects
- `/api/analytics/spending` → Cost analysis by provider/project
- `/api/analytics/metrics` → Performance and latency metrics

**CLI Mapping:** Maps to CLI reporting commands
- `/api/analytics/usage` → `iron usage report` or `iron analytics usage`
- `/api/analytics/spending` → `iron spending show` or `iron analytics spending`

### 4. Configuration Resources (System Config)

**Definition:** Resources managing system-level configuration and constraints.

**Characteristics:**
- System administration scope
- Often admin-only access
- Affects multiple entities
- Configuration-focused names (`/api/limits`, `/api/settings`)

**Examples:**
- `/api/limits` → Agent Budget limits configuration
- `/api/settings` → System-wide settings

**CLI Mapping:** Maps to CLI admin commands
- `/api/limits` → `iron limits set`
- `/api/settings` → `iron settings update`

### Complete Resource Inventory

### Entity Resources

| Resource | Entity Mapping | HTTP Methods | Auth Type | Certainty | CLI Command Group |
|----------|----------------|--------------|-----------|-----------|-------------------|
| `/api/tokens` | IC Token (1:1) | GET, POST, DELETE, PUT | User Token | ✅ Certain | `iron tokens` |
| `/api/users` | User (1:1) | GET, POST, PUT, DELETE | User Token (Admin) | ✅ Certain | `iron users` |
| `/api/agents` | Agent (1:1) | GET, POST, PUT | User Token | ✅ MUST-HAVE | `iron agents` |
| `/api/providers` | IP (1:1) | GET, POST, PUT, DELETE | User Token | ✅ MUST-HAVE | `iron providers` |
| `/api/projects` | Project (1:1) | GET | User Token | ✅ NICE-TO-HAVE | `iron projects` |
| `/api/api-tokens` | API Token (1:N per user) | GET, POST, DELETE | User Token | ✅ MUST-HAVE | `iron api-tokens` |

**Notes:**
- IC Token is certain (required for Pilot)
- User management is certain (admin functionality, RBAC enforcement with audit logging)
- Agents are MUST-HAVE (core entity for agent lifecycle management, specification complete: [010_agents_api.md](../protocol/010_agents_api.md))
- Providers are MUST-HAVE (AI provider integration, specification complete: [011_providers_api.md](../protocol/011_providers_api.md))
- Projects are NICE-TO-HAVE (read-only in Pilot, full CRUD post-Pilot, specification complete: [015_projects_api.md](../protocol/015_projects_api.md))
- API Tokens are MUST-HAVE (dashboard authentication, specification complete: [014_api_tokens_api.md](../protocol/014_api_tokens_api.md))

### Operation Resources

| Resource | Operations | Entities Involved | Auth Type | Certainty | CLI Commands |
|----------|-----------|-------------------|-----------|-----------|--------------|
| `/api/auth` | login, logout, refresh | User, User Token | User Token | ✅ Certain | `iron login`, `iron logout` |
| `/api/budget/handshake` | negotiate | Agent, Project, IP | IC Token | ✅ Certain | (agent-facing, no CLI) |
| `/api/budget/report` | report_usage | Agent, Project, IP | IC Token | ✅ Certain | (agent-facing, no CLI) |
| `/api/budget/refresh` | refresh_budget | Agent, Project | IC Token | ✅ Certain | (agent-facing, no CLI) |

**Notes:**
- Budget protocol resources certain (already implemented in [005_budget_control_protocol.md](../protocol/005_budget_control_protocol.md))
- Authentication certain (User Token lifecycle required for Pilot)
- Agent-facing resources (budget/*) not exposed via CLI (used by iron_runtime directly)

### Analytics Resources

| Resource | Data Provided | Source Entities | Auth Type | Certainty | CLI Commands |
|----------|---------------|-----------------|-----------|-----------|--------------|
| `/api/analytics/spending/total` | Total spending across agents/providers | Agent, Budget, Provider | User Token | ✅ MUST-HAVE | `iron analytics spending total` |
| `/api/analytics/spending/by-agent` | Spending breakdown by agent | Agent, Budget | User Token | ✅ MUST-HAVE | `iron analytics spending by-agent` |
| `/api/analytics/budget/status` | Budget status with risk levels | Agent, Budget | User Token | ✅ MUST-HAVE | `iron analytics budget status` |
| `/api/analytics/spending/by-provider` | Spending breakdown by provider | Provider, Budget | User Token | ✅ MUST-HAVE | `iron analytics spending by-provider` |
| `/api/analytics/usage/requests` | Request count statistics | Request logs | User Token | ✅ MUST-HAVE | `iron analytics usage requests` |
| `/api/analytics/usage/tokens/by-agent` | Token usage by agent | Agent, Request logs | User Token | ✅ MUST-HAVE | `iron analytics usage tokens` |
| `/api/analytics/usage/models` | Model usage statistics | Request logs | User Token | ✅ MUST-HAVE | `iron analytics usage models` |
| `/api/analytics/spending/avg-per-request` | Average cost per request | Budget, Request logs | User Token | ✅ MUST-HAVE | `iron analytics spending avg` |

**Notes:**
- Analytics resources consolidated under single `/api/analytics` namespace
- 8 critical use cases specified for budget visibility and cost analysis
- All MUST-HAVE (specification complete: [012_analytics_api.md](../protocol/012_analytics_api.md))
- Support filtering by agent_id, provider_id, and time period (today, yesterday, last-7-days, last-30-days, all-time)
- CLI commands use consistent `iron analytics {category} {subcategory}` pattern

### Configuration Resources

| Resource | Configuration Managed | Affects | Auth Type | Certainty | CLI Commands |
|----------|----------------------|---------|-----------|-----------|--------------|
| `/api/limits/agents/{agent_id}/budget` | Agent budget modification (direct, admin-only) | Agent Budget | User Token (Admin) | ✅ MUST-HAVE | `iron limits agent <id> budget` |
| `/api/budget-requests` | Budget change requests (developer workflow) | Agent Budget | User Token (Owner + Admin) | ✅ MUST-HAVE | `iron budget-requests` |
| `/api/settings` | System-wide settings | Control Panel | User Token (Admin) | 📋 POST-PILOT | `iron settings` |

**Notes:**
- Budget Limits API is MUST-HAVE (admin-only direct modification, specification complete: [013_budget_limits_api.md](../protocol/013_budget_limits_api.md))
- Budget Requests API is MUST-HAVE (request/approval workflow, specification complete: [017_budget_requests_api.md](../protocol/017_budget_requests_api.md))
- Settings API is POST-PILOT (specification complete for future implementation: [016_settings_api.md](../protocol/016_settings_api.md))
- Budget modification supports force flag for safe decreases (prevents accidental agent shutdowns)
- Two paths for budget modification: Admin uses direct PUT, developers use request workflow

### System Resources

| Resource | Purpose | Auth Type | Certainty |
|----------|---------|-----------|-----------|
| `/api/health` | Health check | None | ✅ Certain |
| `/api/version` | API version | None | ✅ Certain |

**Notes:**
- System resources certain (standard endpoints)
- No authentication required

### Resource-Entity Mapping Patterns

**Four Mapping Patterns:**

### 1. Direct 1:1 Mapping (Entity Resources)

**Pattern:** One resource provides full CRUD for one entity.

**Example:**
- Resource: `/api/tokens`
- Entity: IC Token
- Operations: GET (list/get), POST (create), DELETE (delete), PUT (rotate)

**Characteristics:**
- Most straightforward pattern
- Resource name = plural entity name
- All CRUD ops available
- Maps directly to `iron {entity}` CLI group

### 2. Multiple Resources per Entity (1:N)

**Pattern:** One entity managed through multiple specialized resources.

**Example:**
- Entity: Agent Budget
- Resources: `/api/budget/handshake`, `/api/budget/report`, `/api/budget/refresh`
- Reason: Different operations with different auth/purpose

**Characteristics:**
- Complex entity with distinct operations
- Each resource focused on specific operation
- May have different auth requirements
- CLI may consolidate (`iron budget` group)

### 3. Resource Spans Multiple Entities (N:1)

**Pattern:** One resource operates on multiple entities.

**Example:**
- Resource: `/api/auth`
- Entities: User, User Token
- Operations: login (creates User Token), logout (invalidates User Token)

**Characteristics:**
- Operation-oriented resource
- Coordinates multiple entities
- RPC-style rather than CRUD
- CLI often splits to separate commands

### 4. No Direct Mapping (Derived)

**Pattern:** Resource provides derived/aggregated data from multiple entities.

**Example:**
- Resource: `/api/usage`
- Source Entities: Agent, Project, IP, Request logs
- Output: Aggregated usage statistics

**Characteristics:**
- Analytics/reporting resources
- Read-only (GET)
- Computation/aggregation involved
- CLI provides reporting commands

### Authentication Patterns

**Three Authentication Types:**

### 1. IC Token (Agent Authentication)

**Definition:** JWT token authenticating agents to Control Panel.

**Characteristics:**
- Format: JWT with `ic_` prefix
- Lifetime: Until agent deleted (long-lived)
- Scope: Single agent
- Used By: iron_runtime (agent execution)

**Resources Using IC Token:**
- `/api/budget/handshake` - Negotiate budget
- `/api/budget/report` - Report usage
- `/api/budget/refresh` - Refresh budget

**CLI Access:** None (agent-facing only)

### 2. User Token (Control Panel Access)

**Definition:** JWT token authenticating users to Control Panel.

**Characteristics:**
- Format: JWT
- Lifetime: 30 days (configurable, refreshable)
- Scope: User + accessible projects
- Used By: iron_cli, web dashboard

**Resources Using User Token:**
- `/api/tokens` - Manage IC Tokens
- `/api/projects` - Manage projects (uncertain)
- `/api/providers` - Manage IPs (uncertain)
- `/api/usage` - View usage (uncertain)
- All user-facing resources

**CLI Access:** All `iron` commands (after `iron login`)

### 3. No Authentication

**Definition:** Public endpoints requiring no authentication.

**Resources:**
- `/api/health` - Health check
- `/api/version` - API version

**Purpose:** Operational monitoring, discovery

### CLI-API Parity

**Principle:** Every user-facing API resource maps to CLI command group.

**Exceptions:**
- Agent-facing resources (budget protocol) - No CLI mapping
- System resources (health, version) - No CLI mapping

### Entity Resources → Command Groups

| API Resource | HTTP Method | CLI Command | Notes |
|--------------|-------------|-------------|-------|
| `GET /api/tokens` | GET (list) | `iron tokens list` | List all IC Tokens |
| `POST /api/tokens` | POST | `iron tokens create` | Create IC Token |
| `DELETE /api/tokens/{id}` | DELETE | `iron tokens delete <id>` | Delete IC Token |
| `PUT /api/tokens/{id}/rotate` | PUT | `iron tokens rotate <id>` | Rotate IC Token |
| `GET /api/users` | GET (list) | `iron users list` | List all users |
| `GET /api/users/{id}` | GET | `iron users get <id>` | Get user details |
| `POST /api/users` | POST | `iron users create` | Create user |
| `PUT /api/users/{id}/suspend` | PUT | `iron users suspend <id>` | Suspend user |
| `PUT /api/users/{id}/activate` | PUT | `iron users activate <id>` | Activate user |
| `DELETE /api/users/{id}` | DELETE | `iron users delete <id>` | Soft delete user |
| `PUT /api/users/{id}/role` | PUT | `iron users change-role <id>` | Change user role |
| `PUT /api/users/{id}/password` | PUT | `iron users reset-password <id>` | Reset user password |

### Operation Resources → Commands

| API Resource | HTTP Method | CLI Command | Notes |
|--------------|-------------|-------------|-------|
| `POST /api/auth/login` | POST | `iron login` | Authenticate user |
| `POST /api/auth/logout` | POST | `iron logout` | Invalidate token |
| `POST /api/auth/refresh` | POST | (automatic) | Refresh User Token |

### Analytics Resources → Reporting Commands

| API Resource | HTTP Method | CLI Command | Notes |
|--------------|-------------|-------------|-------|
| `GET /api/analytics/usage` | GET | `iron usage report` or `iron analytics usage` | Usage statistics |
| `GET /api/analytics/spending` | GET | `iron spending show` or `iron analytics spending` | Cost analysis |
| `GET /api/analytics/metrics` | GET | `iron metrics view` or `iron analytics metrics` | Performance metrics |

**Parity Details:** See [../features/004_token_management_cli_api_parity.md](../features/004_token_management_cli_api_parity.md) for complete mapping.

### Certainty Status

### ✅ Certain Resources (Pilot Required)

**Criteria:**
- Required for Pilot launch
- Implementation details clear
- No major design questions
- Dependencies resolved

**Resources (8 total):**

1. **Entity Resources (2):**
   - `/api/tokens` - IC Token CRUD ([006_token_management_api.md](../protocol/006_token_management_api.md))
   - `/api/users` - User account management CRUD ([008_user_management_api.md](../protocol/008_user_management_api.md))

2. **Operation Resources (4):**
   - `/api/auth` - User authentication ([007_authentication_api.md](../protocol/007_authentication_api.md))
   - `/api/budget/handshake` - Budget negotiation ([005_budget_control_protocol.md](../protocol/005_budget_control_protocol.md))
   - `/api/budget/report` - Usage reporting ([005_budget_control_protocol.md](../protocol/005_budget_control_protocol.md))
   - `/api/budget/refresh` - Budget refresh ([005_budget_control_protocol.md](../protocol/005_budget_control_protocol.md))

3. **System Resources (2):**
   - `/api/health` - Health check ([002_rest_api_protocol.md](../protocol/002_rest_api_protocol.md))
   - `/api/version` - API version ([002_rest_api_protocol.md](../protocol/002_rest_api_protocol.md))

### ✅ MUST-HAVE Resources (Specifications Complete)

**Criteria:**
- Critical for production operation
- Full specifications written
- Ready for implementation
- All design questions resolved

**Resources (13 total):**

1. **Entity Resources (3):**
   - `/api/agents` - Agent lifecycle management ([010_agents_api.md](../protocol/010_agents_api.md))
   - `/api/providers` - Provider integration ([011_providers_api.md](../protocol/011_providers_api.md))
   - `/api/api-tokens` - API token management ([014_api_tokens_api.md](../protocol/014_api_tokens_api.md))

2. **Analytics Resources (8):**
   - `/api/analytics/spending/total` - Total spending ([012_analytics_api.md](../protocol/012_analytics_api.md))
   - `/api/analytics/spending/by-agent` - Agent spending breakdown ([012_analytics_api.md](../protocol/012_analytics_api.md))
   - `/api/analytics/budget/status` - Budget risk monitoring ([012_analytics_api.md](../protocol/012_analytics_api.md))
   - `/api/analytics/spending/by-provider` - Provider spending breakdown ([012_analytics_api.md](../protocol/012_analytics_api.md))
   - `/api/analytics/usage/requests` - Request statistics ([012_analytics_api.md](../protocol/012_analytics_api.md))
   - `/api/analytics/usage/tokens/by-agent` - Token usage by agent ([012_analytics_api.md](../protocol/012_analytics_api.md))
   - `/api/analytics/usage/models` - Model usage statistics ([012_analytics_api.md](../protocol/012_analytics_api.md))
   - `/api/analytics/spending/avg-per-request` - Average cost per request ([012_analytics_api.md](../protocol/012_analytics_api.md))

3. **Configuration Resources (2):**
   - `/api/limits/agents/{id}/budget` - Emergency budget modification, admin-only ([013_budget_limits_api.md](../protocol/013_budget_limits_api.md))
   - `/api/budget-requests` - Budget change request/approval workflow ([017_budget_requests_api.md](../protocol/017_budget_requests_api.md))

### ✅ NICE-TO-HAVE Resources (Specifications Complete)

**Criteria:**
- Enhances user experience
- Full specifications written
- Can be prioritized for implementation
- All design questions resolved

**Resources (1 total):**

1. **Entity Resources (1):**
   - `/api/projects` - Project access (read-only in Pilot) ([015_projects_api.md](../protocol/015_projects_api.md))

### 📋 POST-PILOT Resources (Specifications Prepared)

**Criteria:**
- Future implementation
- Full specifications written for completeness
- Design documented for reference
- Implementation deferred (cost-benefit analysis: 48:1 ratio)

**Resources (1 total):**

1. **Configuration Resources (1):**
   - `/api/settings` - System-wide settings management ([016_settings_api.md](../protocol/016_settings_api.md))

**Implementation Status Summary:**
- ✅ **Certain (Pilot):** 8 resources - Ready for Pilot launch
- ✅ **MUST-HAVE:** 13 resources - Specifications complete, critical for production
- ✅ **NICE-TO-HAVE:** 1 resource - Specifications complete, enhances user experience
- 📋 **POST-PILOT:** 1 resource - Specifications prepared, implementation deferred

**Total:** 23 resources with complete specifications across all priority levels.

### ❌ Missing Resources (Intentionally Not Exposed)

**Entities Without Direct API:**

1. **User Token** - Managed via authentication lifecycle (`/api/auth`), not direct CRUD
2. **IP Token** - Vault-managed, never exposed via API (obtained via `/api/budget/handshake`)

**Reason:** These entities are managed indirectly through specialized operations for security and lifecycle control.

**Note:** Previously missing entities now have direct APIs:
- **Agent** - Now has direct CRUD API at `/api/agents` (MUST-HAVE, [010_agents_api.md](../protocol/010_agents_api.md))
- **User** - Now has direct CRUD API at `/api/users` (Certain, admin-only access with RBAC, [008_user_management_api.md](../protocol/008_user_management_api.md))
- **Master Project** - Now has read-only API at `/api/projects` (NICE-TO-HAVE, returns single Master Project in Pilot, [015_projects_api.md](../protocol/015_projects_api.md))

### Transition Criteria

**When Uncertain → Certain:**

1. Pilot feedback requests feature
2. Implementation approach designed
3. Dependencies identified and resolved
4. CLI-API parity defined
5. Create permanent protocol specification document

**When to Keep Uncertain:**

1. Feature not requested
2. Implementation complexity high
3. Design alternatives unclear
4. Not blocking other work

### Cross-References

#### Related Principles Documents
- Design Philosophy - API design consistency principle, resource taxonomy clarity, CLI-API parity
- Quality Attributes - Completeness (exhaustive 23-resource catalog), Consistency (four resource type patterns), Developer Experience (CLI parity for all user-facing resources)

#### Related Architecture Documents
- [Architecture: Entity Model](007_entity_model.md) - Seven core entities (User, Master Project, Project, Agent, IP, IC Token, Budget Change Request) with relationships, Entity Resources map 1:1 or 1:N to these entities
- [Architecture: Roles and Permissions](006_roles_and_permissions.md) - Three roles (Admin, User, Viewer) determine User Token access scope for API resources, admin-only resources restricted
- [Architecture: Data Flow](004_data_flow.md) - API requests follow eleven-step flow (IC Token validation → Provider → Response), applies to all resource types
- [Architecture: Service Integration](005_service_integration.md) - Control Panel Gateway service (port 8080) exposes all cataloged API resources via HTTP endpoints

#### Used By
- [Protocol: REST API Protocol](../protocol/002_rest_api_protocol.md) - Defines HTTP conventions, status codes, error handling for all 23 cataloged resources
- [Protocol: Budget Control Protocol](../protocol/005_budget_control_protocol.md) - Implements Operation Resources `/api/budget/handshake`, `/api/budget/report`, `/api/budget/refresh`
- [Protocol: Token Management API](../protocol/006_token_management_api.md) - Implements Entity Resource `/api/tokens` (IC Token CRUD)
- [Protocol: Authentication API](../protocol/007_authentication_api.md) - Implements Operation Resources `/api/auth` (login, refresh, logout)
- [Protocol: User Management API](../protocol/008_user_management_api.md) - Implements Entity Resource `/api/users` (admin-only CRUD)
- [Protocol: Agents API](../protocol/010_agents_api.md) - Implements Entity Resource `/api/agents` (agent lifecycle CRUD)
- [Protocol: Providers API](../protocol/011_providers_api.md) - Implements Entity Resource `/api/providers` (IP integration CRUD)
- [Protocol: Analytics API](../protocol/012_analytics_api.md) - Implements Analytics Resources `/api/usage`, `/api/metrics`, `/api/analytics/*`
- [Protocol: Budget Limits API](../protocol/013_budget_limits_api.md) - Implements Configuration Resource `/api/limits` (budget threshold configuration)
- [Protocol: API Tokens API](../protocol/014_api_tokens_api.md) - Implements Entity Resource `/api/api-tokens` (dashboard authentication tokens)
- [Protocol: Projects API](../protocol/015_projects_api.md) - Implements Entity Resource `/api/projects` (read-only Pilot, full CRUD post-Pilot)
- [Protocol: Settings API](../protocol/016_settings_api.md) - Implements Configuration Resource `/api/settings` (system-wide configuration)
- [Protocol: Budget Requests API](../protocol/017_budget_requests_api.md) - Implements Entity Resource `/api/budget/requests` (budget modification requests)
- Python SDK Implementation - Maps API resources to SDK methods (e.g., `sdk.tokens.create()` → `POST /api/tokens`)
- iron CLI Implementation - Maps API resources to command groups (e.g., `iron tokens list` → `GET /api/tokens`)
- Web Dashboard - Consumes User Token authenticated resources for visual management interface
- iron_runtime - Consumes IC Token authenticated Operation Resources for budget control

#### Dependencies
- [Architecture: Entity Model](007_entity_model.md) - Seven entities define Entity Resources mapping (1:1 direct, 1:N multiple resources per entity)
- [Architecture: Roles and Permissions](006_roles_and_permissions.md) - Three roles (Admin, User, Viewer) determine resource access control
- [Protocol: REST API Protocol](../protocol/002_rest_api_protocol.md) - HTTP conventions, status codes, error handling framework for all resources
- HTTP Standard - RESTful resource design principles (GET read, POST create, PUT update, DELETE remove)
- JWT Standard - Authentication token format for IC Token and User Token patterns

#### Implementation
- Control Panel Gateway: HTTP server (port 8080) exposing all 23 API resources
- Entity Resources: CRUD endpoints with 1:1 or 1:N entity mapping (e.g., `/api/tokens`, `/api/agents`)
- Operation Resources: RPC-style POST endpoints for multi-entity actions (e.g., `/api/auth`, `/api/budget/handshake`)
- Analytics Resources: GET-only endpoints returning derived aggregations (e.g., `/api/usage`, `/api/metrics`)
- Configuration Resources: Admin-only endpoints for system settings (e.g., `/api/limits`, `/api/settings`)
- System Resources: Public endpoints for operational monitoring (e.g., `/api/health`, `/api/version`)
- IC Token Authentication: `Authorization: Bearer ic_...` header for agent runtime resources
- User Token Authentication: `Authorization: Bearer ...` header for Control Panel resources
- CLI Commands: `iron <resource-group> <action>` mapping to API endpoints (e.g., `iron tokens list` → `GET /api/tokens`)
- SDK Methods: `sdk.<resource_group>.<action>()` mapping to API endpoints (e.g., `sdk.tokens.create()` → `POST /api/tokens`)

---

*Related: [007_entity_model.md](007_entity_model.md) (entities) | [../protocol/002_rest_api_protocol.md](../protocol/002_rest_api_protocol.md) (API protocol) | [../protocol/005_budget_control_protocol.md](../protocol/005_budget_control_protocol.md) (budget protocol)*

**Last Updated:** 2025-12-13
