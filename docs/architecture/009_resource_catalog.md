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
- `/api/usage` → Usage statistics across agents/projects
- `/api/spending` → Cost analysis by provider/project

**CLI Mapping:** Maps to CLI reporting commands
- `/api/usage` → `iron usage report`
- `/api/spending` → `iron spending show`

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
| `/api/projects` | Project (1:1) | GET, POST, PUT, DELETE | User Token | ⚠️ Uncertain | `iron projects` |
| `/api/providers` | IP (1:1) | GET, POST, PUT, DELETE | User Token | ⚠️ Uncertain | `iron providers` |

**Notes:**
- IC Token is certain (required for Pilot)
- Project and IP management uncertain (design pending)

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
| `/api/usage` | Usage metrics | Agent, Project, IP | User Token | ⚠️ Uncertain | `iron usage report` |
| `/api/spending` | Cost analysis | Agent, Project, IP Budget | User Token | ⚠️ Uncertain | `iron spending show` |
| `/api/metrics` | Performance metrics | Agent, Request logs | User Token | ⚠️ Uncertain | `iron metrics view` |

**Notes:**
- All analytics resources uncertain (not required for Pilot)
- Design decisions pending: aggregation levels, time ranges, filtering

### Configuration Resources

| Resource | Configuration Managed | Affects | Auth Type | Certainty | CLI Commands |
|----------|----------------------|---------|-----------|-----------|--------------|
| `/api/limits` | Agent Budget limits | Agent Budget | User Token (Admin) | ⚠️ Uncertain | `iron limits set` |
| `/api/settings` | System settings | Control Panel | User Token (Admin) | ⚠️ Uncertain | `iron settings update` |

**Notes:**
- Configuration resources uncertain (admin tooling, not Pilot-critical)
- Agent Budget limits set during agent creation in Pilot

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

### Operation Resources → Commands

| API Resource | HTTP Method | CLI Command | Notes |
|--------------|-------------|-------------|-------|
| `POST /api/auth/login` | POST | `iron login` | Authenticate user |
| `POST /api/auth/logout` | POST | `iron logout` | Invalidate token |
| `POST /api/auth/refresh` | POST | (automatic) | Refresh User Token |

### Analytics Resources → Reporting Commands

| API Resource | HTTP Method | CLI Command | Notes |
|--------------|-------------|-------------|-------|
| `GET /api/usage` | GET | `iron usage report` | Usage statistics |
| `GET /api/spending` | GET | `iron spending show` | Cost analysis |

**Parity Details:** See [../features/004_token_management_cli_api_parity.md](../features/004_token_management_cli_api_parity.md) for complete mapping.

## Certainty Status

### ✅ Certain Resources (Ready to Implement)

**Criteria:**
- Required for Pilot launch
- Implementation details clear
- No major design questions
- Dependencies resolved

**Resources (9 total):**

1. **Entity Resources (1):**
   - `/api/tokens` - IC Token CRUD

2. **Operation Resources (4):**
   - `/api/auth` - User authentication
   - `/api/budget/handshake` - Budget negotiation
   - `/api/budget/report` - Usage reporting
   - `/api/budget/refresh` - Budget refresh

3. **System Resources (2):**
   - `/api/health` - Health check
   - `/api/version` - API version

4. **Analytics Resources (0):**
   - (None certain)

5. **Configuration Resources (0):**
   - (None certain)

### ⚠️ Uncertain Resources (Drafts Only)

**Criteria:**
- Not critical for Pilot
- Implementation unclear
- Design decisions pending
- Marked with `-draft_` prefix

**Resources (11 total):**

1. **Entity Resources (2):**
   - `/api/projects` - Project management
   - `/api/providers` - IP management

2. **Operation Resources (0):**
   - (All operations certain)

3. **Analytics Resources (3):**
   - `/api/usage` - Usage metrics
   - `/api/spending` - Cost analysis
   - `/api/metrics` - Performance metrics

4. **Configuration Resources (2):**
   - `/api/limits` - Budget limits config
   - `/api/settings` - System settings

**Note:** Uncertain resources documented in `-draft_*.md` protocol files for future consideration.

### ❌ Missing Resources (Intentionally Not Exposed)

**Entities Without Direct API:**

1. **User** - Managed via authentication (`/api/auth`), not CRUD
2. **Agent** - Created via agent registration, not direct CRUD
3. **Master Project** - Admin view only, no separate API
4. **User Token** - Managed via authentication lifecycle, not CRUD
5. **IP Token** - Vault-managed, never exposed via API

**Reason:** These entities managed indirectly or through specialized operations.

## Transition Criteria

**When Uncertain → Certain:**

1. Pilot feedback requests feature
2. Implementation approach designed
3. Dependencies identified and resolved
4. CLI-API parity defined
5. Move from `-draft_*.md` to permanent protocol doc

**When to Keep Uncertain:**

1. Feature not requested
2. Implementation complexity high
3. Design alternatives unclear
4. Not blocking other work

---

*Related: [007_entity_model.md](007_entity_model.md) (entities) | [../protocol/002_rest_api_protocol.md](../protocol/002_rest_api_protocol.md) (API protocol) | [../protocol/005_budget_control_protocol.md](../protocol/005_budget_control_protocol.md) (budget protocol)*

**Last Updated:** 2025-12-09
