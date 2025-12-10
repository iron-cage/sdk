# Resource Catalog

**Purpose:** Exhaustive inventory of all REST API resources, their mapping to entities, and authentication patterns.

> **Note:** This document catalogs REST API resources exposed by the Control Panel. For entity definitions, see [007_entity_model.md](007_entity_model.md).

---

## User Need

Understand all available REST API resources, how they map to domain entities, and which are certain vs uncertain.

## Resource Taxonomy

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

## Complete Resource Inventory

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
| `/api/limits/agents/{agent_id}/budget` | Agent budget modification (increase-only) | Agent Budget | User Token (Owner/Admin) | ✅ MUST-HAVE | `iron limits agent <id> budget` |
| `/api/settings` | System-wide settings | Control Panel | User Token (Admin) | 📋 POST-PILOT | `iron settings` |

**Notes:**
- Budget Limits API is MUST-HAVE (emergency budget modification, specification complete: [013_budget_limits_api.md](../protocol/013_budget_limits_api.md))
- Settings API is POST-PILOT (specification complete for future implementation: [016_settings_api.md](../protocol/016_settings_api.md))
- Budget modification supports increase-only policy (prevents accidental agent shutdowns)

### System Resources

| Resource | Purpose | Auth Type | Certainty |
|----------|---------|-----------|-----------|
| `/api/health` | Health check | None | ✅ Certain |
| `/api/version` | API version | None | ✅ Certain |

**Notes:**
- System resources certain (standard endpoints)
- No authentication required

## Resource-Entity Mapping Patterns

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

## Authentication Patterns

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

## CLI-API Parity

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

## Certainty Status

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

3. **Configuration Resources (1):**
   - `/api/limits/agents/{id}/budget` - Emergency budget modification ([013_budget_limits_api.md](../protocol/013_budget_limits_api.md))

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
- ✅ **MUST-HAVE:** 12 resources - Specifications complete, critical for production
- ✅ **NICE-TO-HAVE:** 1 resource - Specifications complete, enhances user experience
- 📋 **POST-PILOT:** 1 resource - Specifications prepared, implementation deferred

**Total:** 22 resources with complete specifications across all priority levels.

### ❌ Missing Resources (Intentionally Not Exposed)

**Entities Without Direct API:**

1. **User Token** - Managed via authentication lifecycle (`/api/auth`), not direct CRUD
2. **IP Token** - Vault-managed, never exposed via API (obtained via `/api/budget/handshake`)

**Reason:** These entities are managed indirectly through specialized operations for security and lifecycle control.

**Note:** Previously missing entities now have direct APIs:
- **Agent** - Now has direct CRUD API at `/api/agents` (MUST-HAVE, [010_agents_api.md](../protocol/010_agents_api.md))
- **User** - Now has direct CRUD API at `/api/users` (Certain, admin-only access with RBAC, [008_user_management_api.md](../protocol/008_user_management_api.md))
- **Master Project** - Now has read-only API at `/api/projects` (NICE-TO-HAVE, returns single Master Project in Pilot, [015_projects_api.md](../protocol/015_projects_api.md))

## Transition Criteria

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

---

*Related: [007_entity_model.md](007_entity_model.md) (entities) | [../protocol/002_rest_api_protocol.md](../protocol/002_rest_api_protocol.md) (API protocol) | [../protocol/005_budget_control_protocol.md](../protocol/005_budget_control_protocol.md) (budget protocol)*

**Last Updated:** 2025-12-10
