# Protocol 010: Agents API

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-10

---

## Overview

The Agents API provides HTTP endpoints for managing agents in the Iron Control Panel. Agents are autonomous entities that consume AI provider services within budget constraints. Each agent has a unique IC Token for authentication, an assigned budget, and access to one or more providers.

**Key characteristics:**
- **1:1 Agent-IC Token relationship:** Each agent has exactly one IC Token
- **1:1 Agent-Budget relationship:** Each agent has exactly one Agent Budget (RESTRICTIVE)
- **Many-to-Many Agent-Provider relationship:** Agents can use multiple providers, providers can serve multiple agents
- **Owner-based access control:** Agents are owned by users, with admin override capabilities

---

## Endpoints

### Create Agent

**Endpoint:** `POST /api/v1/agents`

**Description:** Creates a new agent with specified budget and provider assignments. Automatically generates an IC Token for agent authentication.

**Request:**

```json
POST /api/v1/agents
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "name": "Production Agent 1",
  "budget": 100.00,
  "providers": ["ip-openai-001", "ip-anthropic-001"],
  "description": "Main production agent for customer requests",
  "tags": ["production", "customer-facing"]
}
```

**Request Parameters:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `name` | string | Yes | 1-100 chars | Human-readable agent name |
| `budget` | number | Yes | >= 0.01 | Initial agent budget in USD (decimal, 2 places) |
| `providers` | array<string> | No | Max unlimited | Provider IDs agent can use (optional, can be empty) |
| `description` | string | No | Max 500 chars | Optional agent description |
| `tags` | array<string> | No | Max 20 tags, 50 chars each | Optional tags for organization |

**Success Response:**

```json
HTTP 201 Created
Content-Type: application/json

{
  "id": "agent_abc123",
  "name": "Production Agent 1",
  "budget": 100.00,
  "providers": ["ip-openai-001", "ip-anthropic-001"],
  "description": "Main production agent for customer requests",
  "tags": ["production", "customer-facing"],
  "owner_id": "user-xyz789",
  "project_id": "proj-master",
  "ic_token": {
    "id": "ic_def456ghi789",
    "token": "ic_xyz789abc123def456...",
    "created_at": "2025-12-10T10:30:45Z"
  },
  "status": "active",
  "created_at": "2025-12-10T10:30:45Z",
  "updated_at": "2025-12-10T10:30:45Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique agent identifier (agent- prefix) |
| `name` | string | Agent name |
| `budget` | number | Current agent budget in USD |
| `providers` | array<string> | Provider IDs assigned to agent |
| `description` | string | Agent description (omitted if empty) |
| `tags` | array<string> | Agent tags (omitted if empty) |
| `owner_id` | string | User ID who created the agent (inferred from auth token) |
| `project_id` | string | Project ID (defaults to Master Project in Pilot) |
| `ic_token` | object | IC Token details (shown ONLY on creation) |
| `ic_token.id` | string | IC Token identifier |
| `ic_token.token` | string | IC Token value (shown once, never again) |
| `ic_token.created_at` | string | ISO 8601 timestamp |
| `status` | string | Agent status: "active", "exhausted", "inactive" |
| `created_at` | string | ISO 8601 timestamp |
| `updated_at` | string | ISO 8601 timestamp |

**Important:** The IC Token value is returned ONLY on agent creation. It cannot be retrieved later. If lost, the token must be rotated via `POST /api/v1/tokens/{ic_token_id}/rotate`.

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "budget": "Must be >= 0.01",
      "name": "Required field"
    }
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient permissions"
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "PROVIDER_NOT_FOUND",
    "message": "Provider 'ip-invalid-001' does not exist"
  }
}
```

**Authorization:**
- **User (any role):** Can create agents they will own
- **Admin:** Can create agents on behalf of any user

**Audit Log:** Yes (mutation operation)

---

### List Agents

**Endpoint:** `GET /api/v1/agents`

**Description:** Returns paginated list of agents. Users see only their own agents; admins see all agents.

**Request:**

```
GET /api/v1/agents?page=1&per_page=50&name=production&sort=-created_at
Authorization: Bearer <user-token or api-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number (1-indexed) |
| `per_page` | integer | 50 | Results per page (max 100) |
| `name` | string | - | Filter by name (partial match, case-insensitive) |
| `status` | string | - | Filter by status: "active", "exhausted", "inactive" |
| `sort` | string | `-created_at` | Sort field: `name`, `budget`, `created_at` (prefix `-` for desc) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "data": [
    {
      "id": "agent_abc123",
      "name": "Production Agent 1",
      "budget": 100.00,
      "spent": 45.75,
      "remaining": 54.25,
      "providers": ["ip-openai-001", "ip-anthropic-001"],
      "description": "Main production agent",
      "tags": ["production", "customer-facing"],
      "owner_id": "user-xyz789",
      "project_id": "proj-master",
      "status": "active",
      "created_at": "2025-12-10T10:30:45Z",
      "updated_at": "2025-12-10T10:30:45Z"
    },
    {
      "id": "agent_def456",
      "name": "Test Agent",
      "budget": 10.00,
      "spent": 10.00,
      "remaining": 0.00,
      "providers": ["ip-openai-001"],
      "owner_id": "user-xyz789",
      "project_id": "proj-master",
      "status": "exhausted",
      "created_at": "2025-12-09T14:20:30Z",
      "updated_at": "2025-12-09T18:45:12Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 2,
    "total_pages": 1
  }
}
```

**Response Fields:**

- **`data`:** Array of agent objects (includes `spent` and `remaining` fields)
- **`pagination`:** Pagination metadata
  - `page`: Current page number
  - `per_page`: Results per page
  - `total`: Total number of agents matching filters
  - `total_pages`: Total number of pages

**Note:** IC Token is NOT included in list response. Use `GET /api/v1/agents/{id}` to see IC Token ID (but not token value).

**Empty Results:**

```json
HTTP 200 OK
{
  "data": [],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 0,
    "total_pages": 0
  }
}
```

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "page": "Must be >= 1",
      "per_page": "Must be between 1 and 100",
      "sort": "Invalid sort field (allowed: name, budget, created_at)"
    }
  }
}
```

**Authorization:**
- **User:** Can list own agents only
- **Admin:** Can list all agents

**Audit Log:** No (read operation)

---

### Get Agent Details

**Endpoint:** `GET /api/v1/agents/{id}`

**Description:** Returns detailed information about a specific agent, including IC Token ID (but not token value).

**Request:**

```
GET /api/v1/agents/agent-abc123
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "agent_abc123",
  "name": "Production Agent 1",
  "budget": 100.00,
  "spent": 45.75,
  "remaining": 54.25,
  "percent_used": 45.75,
  "providers": [
    {
      "id": "ip-openai-001",
      "name": "openai",
      "endpoint": "https://api.openai.com/v1"
    },
    {
      "id": "ip-anthropic-001",
      "name": "anthropic",
      "endpoint": "https://api.anthropic.com/v1"
    }
  ],
  "description": "Main production agent for customer requests",
  "tags": ["production", "customer-facing"],
  "owner_id": "user-xyz789",
  "project_id": "proj-master",
  "ic_token": {
    "id": "ic_def456ghi789",
    "created_at": "2025-12-10T10:30:45Z",
    "last_used": "2025-12-10T14:22:10Z"
  },
  "status": "active",
  "created_at": "2025-12-10T10:30:45Z",
  "updated_at": "2025-12-10T10:30:45Z"
}
```

**Response Fields (additional vs List):**

| Field | Type | Description |
|-------|------|-------------|
| `spent` | number | Amount spent by agent (USD, 2 decimal places) |
| `remaining` | number | Remaining budget (USD, 2 decimal places) |
| `percent_used` | number | Budget utilization percentage (0-100) |
| `providers` | array<object> | Detailed provider information |
| `providers[].id` | string | Provider ID |
| `providers[].name` | string | Provider name |
| `providers[].endpoint` | string | Provider API endpoint |
| `ic_token` | object | IC Token metadata (ID only, not token value) |
| `ic_token.id` | string | IC Token identifier |
| `ic_token.created_at` | string | Token creation timestamp |
| `ic_token.last_used` | string | Last usage timestamp (omitted if never used) |

**Error Responses:**

```json
HTTP 404 Not Found
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent 'agent-invalid' does not exist"
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient permissions"
  }
}
```

**Authorization:**
- **User:** Can view own agents only
- **Admin:** Can view all agents

**Audit Log:** No (read operation)

---

### Update Agent

**Endpoint:** `PUT /api/v1/agents/{id}`

**Description:** Updates agent metadata (name, description, tags). Budget and provider assignments are modified via separate endpoints.

**Request:**

```json
PUT /api/v1/agents/agent-abc123
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "name": "Production Agent 1 (Updated)",
  "description": "Updated description",
  "tags": ["production", "customer-facing", "high-priority"]
}
```

**Request Parameters:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `name` | string | No | 1-100 chars | Updated agent name |
| `description` | string | No | Max 500 chars | Updated description (empty string to clear) |
| `tags` | array<string> | No | Max 20 tags, 50 chars each | Updated tags (empty array to clear) |

**Important:** At least one field must be provided. To modify budget, admins use `PUT /api/v1/limits/agents/{id}/budget` (see [Protocol 013](013_budget_limits_api.md)); developers create budget change requests (see [Protocol 017](017_budget_requests_api.md)). To modify providers, use `PUT /api/v1/agents/{id}/providers`.

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "agent_abc123",
  "name": "Production Agent 1 (Updated)",
  "budget": 100.00,
  "spent": 45.75,
  "remaining": 54.25,
  "providers": ["ip-openai-001", "ip-anthropic-001"],
  "description": "Updated description",
  "tags": ["production", "customer-facing", "high-priority"],
  "owner_id": "user-xyz789",
  "project_id": "proj-master",
  "status": "active",
  "created_at": "2025-12-10T10:30:45Z",
  "updated_at": "2025-12-10T15:22:10Z"
}
```

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "name": "Must be between 1 and 100 characters",
      "tags": "Maximum 20 tags allowed"
    }
  }
}
```

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "NO_FIELDS_PROVIDED",
    "message": "At least one field must be updated"
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient permissions"
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent 'agent-invalid' does not exist"
  }
}
```

**Authorization:**
- **Owner:** Can update own agents only
- **Admin:** Can update all agents

**Audit Log:** Yes (mutation operation)

---

### Get Agent Status

**Endpoint:** `GET /api/v1/agents/{id}/status`

**Description:** Returns real-time budget status for the agent. Optimized for frequent polling by dashboards and monitoring tools.

**Request:**

```
GET /api/v1/agents/agent-abc123/status
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "agent_id": "agent_abc123",
  "status": "active",
  "budget": {
    "total": 100.00,
    "spent": 45.75,
    "remaining": 54.25,
    "percent_used": 45.75
  },
  "requests": {
    "total": 1247,
    "today": 89,
    "last_hour": 12
  },
  "last_request_at": "2025-12-10T15:22:10Z",
  "checked_at": "2025-12-10T15:30:00Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `agent_id` | string | Agent identifier |
| `status` | string | Current status: "active", "exhausted", "inactive" |
| `budget.total` | number | Total budget (USD) |
| `budget.spent` | number | Amount spent (USD) |
| `budget.remaining` | number | Remaining budget (USD) |
| `budget.percent_used` | number | Budget utilization percentage (0-100) |
| `requests.total` | integer | Total requests since agent creation |
| `requests.today` | integer | Requests today (resets at midnight UTC) |
| `requests.last_hour` | integer | Requests in last 60 minutes |
| `last_request_at` | string | ISO 8601 timestamp of last request (omitted if no requests) |
| `checked_at` | string | ISO 8601 timestamp of status check |

**Status Values:**

- **`active`:** Agent has remaining budget and is operational
- **`exhausted`:** Agent has $0.00 remaining budget (spent >= total)
- **`inactive`:** Agent manually deactivated (future feature)

**Error Responses:**

```json
HTTP 404 Not Found
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent 'agent-invalid' does not exist"
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient permissions"
  }
}
```

**Authorization:**
- **Owner:** Can view own agents only
- **Admin:** Can view all agents

**Rate Limiting:** 20 requests per minute per agent (sufficient for dashboard polling every 3 seconds)

**Audit Log:** No (read operation, high frequency)

---

## Data Models

### Agent Object

```json
{
  "id": "agent_abc123",
  "name": "Production Agent 1",
  "budget": 100.00,
  "spent": 45.75,
  "remaining": 54.25,
  "percent_used": 45.75,
  "providers": ["ip-openai-001", "ip-anthropic-001"],
  "description": "Main production agent",
  "tags": ["production", "customer-facing"],
  "owner_id": "user-xyz789",
  "project_id": "proj-master",
  "ic_token": {
    "id": "ic_def456ghi789",
    "created_at": "2025-12-10T10:30:45Z",
    "last_used": "2025-12-10T14:22:10Z"
  },
  "status": "active",
  "created_at": "2025-12-10T10:30:45Z",
  "updated_at": "2025-12-10T10:30:45Z"
}
```

### Agent Status Object

```json
{
  "agent_id": "agent_abc123",
  "status": "active",
  "budget": {
    "total": 100.00,
    "spent": 45.75,
    "remaining": 54.25,
    "percent_used": 45.75
  },
  "requests": {
    "total": 1247,
    "today": 89,
    "last_hour": 12
  },
  "last_request_at": "2025-12-10T15:22:10Z",
  "checked_at": "2025-12-10T15:30:00Z"
}
```

---

## Relationships

### Agent ↔ IC Token (1:1)

- Each agent has exactly one IC Token
- IC Token is automatically created with agent
- IC Token value shown only on agent creation
- IC Token can be rotated via `POST /api/v1/tokens/{ic_token_id}/rotate`
- Deleting agent (future) invalidates IC Token

### Agent ↔ Agent Budget (1:1)

- Each agent has exactly one Agent Budget (RESTRICTIVE type)
- Budget set at agent creation
- Budget modifiable via admin-only `PUT /api/v1/limits/agents/{id}/budget` (see [Protocol 013](013_budget_limits_api.md))
- Developers request budget changes via request/approval workflow (see [Protocol 017](017_budget_requests_api.md))
- Budget enforcement blocks requests when exhausted

### Agent ↔ Providers (Many-to-Many)

- Agent can have zero or more providers
- No maximum limit on number of providers
- **Provider deletion cascades:** Deleting provider automatically removes it from all agent assignments (ON DELETE CASCADE)
- **Zero providers:** Agents with no providers cannot make inference requests until provider assigned
- **Multiple providers:** Agents with multiple providers use them in order (primary, fallback, etc.)
- Provider assignment managed via `PUT /api/v1/agents/{id}/providers` (see Protocol 010) or `DELETE /api/v1/providers/{id}` (cascade deletion, see Protocol 011)
- Provider usage tracked for analytics

### Agent ↔ User (Many-to-One)

- Each agent has exactly one owner (user)
- Owner inferred from auth token on creation
- Owner can view and modify own agents
- Admin can view and modify all agents

### Agent ↔ Project (Many-to-One)

- Each agent belongs to exactly one project
- Pilot uses Master Project only (project_id defaults to "proj-master")
- Post-Pilot supports multi-project assignment

---

## Security

### Authentication

All endpoints require authentication via:
- **User Token:** Bearer token from `POST /api/v1/auth/login`
- **API Token:** Bearer token from `POST /api/v1/api-tokens`

### Authorization Matrix

| Operation | Owner | Admin | Other User |
|-----------|-------|-------|------------|
| Create agent | ✅ (own) | ✅ (any) | ❌ |
| List agents | ✅ (own) | ✅ (all) | ❌ |
| Get agent details | ✅ (own) | ✅ (all) | ❌ |
| Update agent | ✅ (own) | ✅ (all) | ❌ |
| Get agent status | ✅ (own) | ✅ (all) | ❌ |

### Sensitive Data Handling

**IC Token Value:**
- Returned ONLY on agent creation (`POST /api/v1/agents`)
- Never returned in `GET /api/v1/agents` (list)
- Never returned in `GET /api/v1/agents/{id}` (details)
- Never stored in plain text (hashed in database)
- If lost, must rotate via `POST /api/v1/tokens/{ic_token_id}/rotate`

**Budget Information:**
- Visible to agent owner and admins only
- Not visible to other users (even in same project)
- Returned in all agent endpoints (list, details, status)

---

## Error Handling

### Standard Error Format

All errors use consistent format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "field": "field_name"
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | One or more fields failed validation |
| `NO_FIELDS_PROVIDED` | 400 | Update request with no fields |
| `UNAUTHORIZED` | 401 | Missing or invalid authentication |
| `TOKEN_EXPIRED` | 401 | Authentication token expired |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `AGENT_NOT_FOUND` | 404 | Agent does not exist |
| `PROVIDER_NOT_FOUND` | 404 | One or more providers do not exist |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

### Validation Error Details

Multiple validation errors returned together:

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "budget": "Must be >= 0.01",
      "providers": "At least one provider required",
      "name": "Must be between 1 and 100 characters"
    }
  }
}
```

---

## Rate Limiting

### Limits (per user)

| Endpoint | Limit | Window |
|----------|-------|--------|
| `POST /api/v1/agents` | 10 | 1 minute |
| `GET /api/v1/agents` | 60 | 1 minute |
| `GET /api/v1/agents/{id}` | 60 | 1 minute |
| `PUT /api/v1/agents/{id}` | 30 | 1 minute |
| `GET /api/v1/agents/{id}/status` | 20 | 1 minute |

### Rate Limit Response

```
HTTP 429 Too Many Requests
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1733830860
Retry-After: 60

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests"
  }
}
```

---

## Audit Logging

### Logged Operations

| Endpoint | Method | Logged |
|----------|--------|--------|
| `POST /api/v1/agents` | POST | ✅ Yes |
| `GET /api/v1/agents` | GET | ❌ No |
| `GET /api/v1/agents/{id}` | GET | ❌ No |
| `PUT /api/v1/agents/{id}` | PUT | ✅ Yes |
| `GET /api/v1/agents/{id}/status` | GET | ❌ No |

### Audit Log Entry

```json
{
  "timestamp": "2025-12-10T10:30:45Z",
  "user_id": "user-xyz789",
  "endpoint": "POST /api/v1/agents",
  "method": "POST",
  "resource_type": "agent",
  "resource_id": "agent_abc123",
  "action": "create",
  "parameters": {
    "name": "Production Agent 1",
    "budget": 100.00,
    "providers": ["ip-openai-001", "ip-anthropic-001"]
  },
  "status": "success",
  "ip_address": "203.0.113.42",
  "user_agent": "iron-cli/1.0.0"
}
```

**Retention:** 90 days

**Access:** Admin via `GET /api/v1/audit-logs`

---

## CLI Integration

### iron agents create

```bash
iron agents create \
  --name "Production Agent 1" \
  --budget 100.00 \
  --providers ip-openai-001,ip-anthropic-001 \
  --description "Main production agent" \
  --tags production,customer-facing

# Output:
# Agent created: agent-abc123
# IC Token: ic_xyz789abc123def456...
# ⚠️  Save this token now. You won't be able to see it again.
```

### iron agents list

```bash
iron agents list
iron agents list --name production
iron agents list --status active
iron agents list --sort -budget

# Output:
# ID            NAME                 BUDGET   SPENT   REMAINING  STATUS
# agent-abc123  Production Agent 1   $100.00  $45.75  $54.25     active
# agent-def456  Test Agent           $10.00   $10.00  $0.00      exhausted
```

### iron agents get

```bash
iron agents get agent-abc123

# Output:
# ID:          agent-abc123
# Name:        Production Agent 1
# Owner:       user-xyz789
# Budget:      $100.00
# Spent:       $45.75 (45.75%)
# Remaining:   $54.25
# Providers:   ip-openai-001 (openai), ip-anthropic-001 (anthropic)
# IC Token:    ic_def456ghi789 (last used: 2025-12-10 14:22:10)
# Status:      active
# Created:     2025-12-10 10:30:45
```

### iron agents update

```bash
iron agents update agent-abc123 \
  --name "Production Agent 1 (Updated)" \
  --description "Updated description" \
  --tags production,customer-facing,high-priority

# Output:
# Agent updated: agent-abc123
```

### iron agents status

```bash
iron agents status agent-abc123

# Output:
# Agent:       agent-abc123 (Production Agent 1)
# Status:      active
# Budget:      $54.25 / $100.00 (45.75% used)
# Requests:    1247 total, 89 today, 12 last hour
# Last Request: 2025-12-10 15:22:10 (8 minutes ago)
```

---

## Future Enhancements (Post-Pilot)

### Agent Deletion

**Endpoint:** `DELETE /api/v1/agents/{id}`

**Strategy:** ARCHIVE (not immediate delete)
- Agent marked inactive (status = "inactive")
- IC Token invalidated immediately
- Budget history preserved
- Request logs preserved
- Can be restored via admin tool

**Reasoning:** Preserve audit trail, prevent accidental data loss

---

### Agent Activation/Deactivation

**Endpoints:**
- `POST /api/v1/agents/{id}/activate`
- `POST /api/v1/agents/{id}/deactivate`

**Use case:** Temporarily disable agent without deletion

---

### Multi-Project Support

**Changes:**
- `project_id` becomes user-selectable (not defaulted to Master Project)
- Add `?project_id=proj-abc` filter to `GET /api/v1/agents`
- Project-level budget limits

---

## References

**Related Protocols:**
- [005: Budget Control Protocol](005_budget_control_protocol.md) - Agent Budget enforcement
- [006: Token Management API](006_token_management_api.md) - IC Token rotation
- [011: Providers API](011_providers_api.md) - Provider management
- [012: Analytics API](012_analytics_api.md) - Agent spending analytics
- [013: Budget Limits API](013_budget_limits_api.md) - Budget modification

**Related Documents:**
- [007: Entity Model](../architecture/007_entity_model.md) - Agent entity definition
- [002: REST API Protocol](002_rest_api_protocol.md) - General REST API standards

---

**Protocol 010 Version:** 1.0.0
**Status:** Specification
**Last Updated:** 2025-12-10
