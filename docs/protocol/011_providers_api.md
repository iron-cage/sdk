# Protocol: Providers API

### Scope

This protocol defines the HTTP API for managing Inference Providers (IPs) - external AI services that agents use for LLM inference requests.

**In scope**:
- Provider CRUD operations (create, list, get details, update, delete)
- Provider credential management (secure storage, encryption at rest, HTTPS transmission)
- Provider-agent assignment operations (assign, list, remove)
- Many-to-many agent-provider relationship management
- Admin-only provider management with authorization enforcement
- Cascade deletion behavior (automatic removal from all agents)
- Credential security model (AES-256-GCM encryption, never in responses)
- Audit logging for mutation operations (credentials excluded)

**Out of scope**:
- Provider deletion dry-run preview (Future Enhancement - see lines 1103-1148)
- Provider usage analytics (see Protocol 012: Analytics API)
- Agent management (see Protocol 010: Agents API)
- User authentication (see Protocol 007: Authentication API)
- IC Token management (see Protocol 006: Token Management API)
- Database schema implementation (reference only - see lines 1042-1098)
- Actual credential encryption implementation (reference only - see lines 816-835)

### Purpose

**User Need**: Admins need to configure and manage external AI provider integrations (OpenAI, Anthropic, etc.) with secure credential storage and role-based access control. Developers need to assign providers to agents for LLM access. Both need transparent provider usage visibility and safe credential rotation without service disruption.

**Solution**: This API provides RESTful endpoints for complete provider lifecycle management with admin-controlled CRUD operations and developer-controlled agent assignments. Provider credentials (IP Tokens) are encrypted at rest using AES-256-GCM and never exposed in API responses. The many-to-many agent-provider relationship enables flexible provider selection while cascade deletion automatically removes provider assignments when providers are deleted. Owner-based access control separates system-level provider management (admin-only) from agent-level provider assignment (developer + admin).

**Key Insight**: The provider is the system integration point for external AI services. By separating provider management (admin CRUD) from provider assignment (developer agent configuration), we enable centralized credential security while allowing distributed provider selection. Cascade deletion with detailed impact reporting (agents_affected list) ensures safe provider removal without orphaned assignments while maintaining operational transparency.

---

**Status**: Specification
**Version**: 1.0.0
**Last Updated**: 2025-12-14
**Priority**: MUST-HAVE

### Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- Providers API uses short alphanumeric IDs for provider (IP Token) and agent identifiers to optimize performance, readability, and operational clarity
- `provider_id`: `ip_<name>_<numeric>` for IP Token identifiers with regex `^ip_[a-z0-9-]+_[0-9]{3}$` (e.g., `ip_openai_001`, `ip_anthropic_001`)
- `agent_id`: `agent_<alphanumeric>` with regex `^agent_[a-z0-9]{6,32}$` (e.g., `agent_abc123`)
- `user_id`: `user_<uuid>` for cross-system compatibility

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Booleans: JSON boolean `true`/`false` (not strings)
- Arrays: Empty array `[]` when no items (not `null`)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Consistent error response structure across all endpoints
- Machine-readable error codes: `VALIDATION_ERROR`, `UNAUTHORIZED`, `NOT_FOUND`, `DUPLICATE_NAME`, `IN_USE`
- HTTP status codes: 200, 201, 400, 401, 403, 404, 409

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- Pagination: Offset-based with `?page=N&per_page=M` (default 50 items/page)
- Filtering: Query parameters for `name`, `status`, `type`
- Sorting: Optional `?sort=-created_at` (newest first, default)
- URL structure: `/api/v1/providers`, `/api/v1/providers/{id}`


### Endpoints

#### Create Provider

**Endpoint:** `POST /api/v1/providers`

**Description:** Creates a new provider with credentials and model configuration. Credentials are encrypted at rest and never returned in API responses.

**Request:**

```json
POST /api/v1/providers
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "name": "openai",
  "endpoint": "https://api.openai.com/v1",
  "credentials": {
    "api_key": "sk-proj_xyz789..."
  },
  "models": ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"]
}
```

**Request Parameters:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `name` | string | Yes | 1-50 chars, lowercase, alphanumeric + hyphen | Provider identifier (e.g., "openai", "anthropic") |
| `endpoint` | string | Yes | Valid HTTPS URL | Provider API endpoint |
| `credentials` | object | Yes | - | Provider authentication credentials |
| `credentials.api_key` | string | Yes | 1-500 chars | Provider API key/token |
| `models` | array<string> | Yes | Min 1, Max 100 | Available model identifiers |

**Security Requirements:**
- Endpoint MUST use HTTPS (reject http://)
- API key encrypted at rest using AES-256
- API key never logged (excluded from audit logs and application logs)
- API key transmitted only via HTTPS

**Success Response:**

```json
HTTP 201 Created
Content-Type: application/json

{
  "id": "ip_openai-001",
  "name": "openai",
  "endpoint": "https://api.openai.com/v1",
  "models": ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"],
  "credentials_configured": true,
  "status": "active",
  "created_at": "2025-12-10T10:30:45Z",
  "updated_at": "2025-12-10T10:30:45Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique provider identifier (ip_ prefix) |
| `name` | string | Provider name |
| `endpoint` | string | Provider API endpoint |
| `models` | array<string> | Available models |
| `credentials_configured` | boolean | True if credentials exist (never shows actual credentials) |
| `status` | string | Provider status: "active", "inactive", "error" |
| `created_at` | string | ISO 8601 timestamp |
| `updated_at` | string | ISO 8601 timestamp |

**Important:** Credentials are NEVER returned in responses. Only `credentials_configured: true` indicates credentials exist.

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "name": "Must be lowercase alphanumeric with hyphens only",
      "endpoint": "Must be HTTPS URL",
      "models": "At least one model required"
    }
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Admin role required"
  }
}
```

```json
HTTP 409 Conflict
{
  "error": {
    "code": "PROVIDER_EXISTS",
    "message": "Provider 'openai' already exists"
  }
}
```

**Authorization:**
- **Admin only:** Only admins can create providers

**Audit Log:** Yes (mutation operation, credentials EXCLUDED from log)


#### List Providers

**Endpoint:** `GET /api/v1/providers`

**Description:** Returns paginated list of providers with metadata. Credentials never included.

**Request:**

```
GET /api/v1/providers?page=1&per_page=50&name=openai&sort=name
Authorization: Bearer <user-token or api-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number (1-indexed) |
| `per_page` | integer | 50 | Results per page (max 100) |
| `name` | string | - | Filter by name (partial match, case-insensitive) |
| `status` | string | - | Filter by status: "active", "inactive", "error" |
| `sort` | string | `name` | Sort field: `name`, `created_at` (prefix `-` for desc) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "data": [
    {
      "id": "ip_openai-001",
      "name": "openai",
      "endpoint": "https://api.openai.com/v1",
      "models": ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"],
      "credentials_configured": true,
      "status": "active",
      "agent_count": 12,
      "created_at": "2025-12-10T10:30:45Z",
      "updated_at": "2025-12-10T10:30:45Z"
    },
    {
      "id": "ip_anthropic-001",
      "name": "anthropic",
      "endpoint": "https://api.anthropic.com/v1",
      "models": ["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"],
      "credentials_configured": true,
      "status": "active",
      "agent_count": 8,
      "created_at": "2025-12-09T14:20:30Z",
      "updated_at": "2025-12-09T14:20:30Z"
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

**Response Fields (additional vs Create):**

| Field | Type | Description |
|-------|------|-------------|
| `agent_count` | integer | Number of agents using this provider |

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
      "per_page": "Must be between 1 and 100"
    }
  }
}
```

**Authorization:**
- **Any authenticated user:** Can list providers (needed to assign to agents)

**Audit Log:** No (read operation)


#### Get Provider Details

**Endpoint:** `GET /api/v1/providers/{id}`

**Description:** Returns detailed information about a specific provider, including usage statistics.

**Request:**

```
GET /api/v1/providers/ip_openai-001
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "ip_openai-001",
  "name": "openai",
  "endpoint": "https://api.openai.com/v1",
  "models": ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"],
  "credentials_configured": true,
  "status": "active",
  "usage": {
    "agent_count": 12,
    "total_requests": 45789,
    "total_spend": 1245.67,
    "requests_today": 892,
    "spend_today": 34.21
  },
  "created_at": "2025-12-10T10:30:45Z",
  "updated_at": "2025-12-10T10:30:45Z"
}
```

**Response Fields (additional vs List):**

| Field | Type | Description |
|-------|------|-------------|
| `usage` | object | Provider usage statistics |
| `usage.agent_count` | integer | Number of agents using provider |
| `usage.total_requests` | integer | Total requests since creation |
| `usage.total_spend` | number | Total spend (USD, 2 decimal places) |
| `usage.requests_today` | integer | Requests today |
| `usage.spend_today` | number | Spend today (USD, 2 decimal places) |

**Error Responses:**

```json
HTTP 404 Not Found
{
  "error": {
    "code": "PROVIDER_NOT_FOUND",
    "message": "Provider 'ip_invalid' does not exist"
  }
}
```

**Authorization:**
- **Any authenticated user:** Can view provider details

**Audit Log:** No (read operation)


#### Update Provider

**Endpoint:** `PUT /api/v1/providers/{id}`

**Description:** Updates provider configuration. At least one field must be provided. Credentials can be updated securely.

**Request:**

```json
PUT /api/v1/providers/ip_openai-001
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "endpoint": "https://api.openai.com/v2",
  "credentials": {
    "api_key": "sk-proj_new-key-abc123..."
  },
  "models": ["gpt-4", "gpt-4-turbo", "gpt-4o", "gpt-3.5-turbo"]
}
```

**Request Parameters:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `name` | string | No | 1-50 chars | Updated provider name |
| `endpoint` | string | No | Valid HTTPS URL | Updated endpoint |
| `credentials` | object | No | - | Updated credentials |
| `credentials.api_key` | string | Yes (if credentials provided) | 1-500 chars | Updated API key |
| `models` | array<string> | No | Min 1, Max 100 | Updated model list |

**Important:** At least one field must be provided. Credential updates replace existing credentials entirely (no partial updates).

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "ip_openai-001",
  "name": "openai",
  "endpoint": "https://api.openai.com/v2",
  "models": ["gpt-4", "gpt-4-turbo", "gpt-4o", "gpt-3.5-turbo"],
  "credentials_configured": true,
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
    "code": "NO_FIELDS_PROVIDED",
    "message": "At least one field must be updated"
  }
}
```

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "endpoint": "Must be HTTPS URL",
      "models": "At least one model required"
    }
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Admin role required"
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "PROVIDER_NOT_FOUND",
    "message": "Provider 'ip_invalid' does not exist"
  }
}
```

**Authorization:**
- **Admin only:** Only admins can update providers

**Audit Log:** Yes (mutation operation, credentials EXCLUDED from log)


#### Delete Provider

**Endpoint:** `DELETE /api/v1/providers/{id}`

**Description:** Deletes a provider and automatically removes it from all agents. Provider-agent assignments are cascade-deleted. Agents with no remaining providers after deletion cannot make inference requests until a provider is assigned.

**Request:**

```
DELETE /api/v1/providers/ip_openai-001
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "ip_openai-001",
  "name": "openai",
  "deleted": true,
  "agents_affected": ["agent_abc123", "agent_def456", "agent_ghi789"],
  "agents_count": 3
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Provider identifier that was deleted |
| `name` | string | Provider name |
| `deleted` | boolean | Always true for successful deletion |
| `agents_affected` | array<string> | List of agent IDs that had this provider assigned |
| `agents_count` | integer | Number of agents affected by deletion |

**Note:** `agents_affected` shows all agents that had this provider before deletion. Agents with other providers will continue to work. Agents left with zero providers cannot make inference requests until a provider is assigned.

**Edge Cases:**

1. **Agent with Single Provider:**
   - If agent has only this provider, agent will have zero providers after deletion
   - Agent status remains `active` (if has budget)
   - Agent **cannot make inference requests** until provider assigned
   - Error on request attempt: `PROVIDER_NOT_ASSIGNED` or `NO_PROVIDERS_AVAILABLE`

2. **Agent with Multiple Providers:**
   - If agent has multiple providers, agent continues working with remaining providers
   - No service disruption

3. **In-Flight Requests:**
   - Requests using deleted provider mid-execution will fail
   - Acceptable failure mode (admin explicitly deleted provider)

4. **Concurrent Agent Creation:**
   - If agent creation uses deleted provider, creation fails with 404 PROVIDER_NOT_FOUND
   - Client should retry with different provider

**Error Responses:**

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Admin role required"
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "PROVIDER_NOT_FOUND",
    "message": "Provider 'ip_invalid' does not exist"
  }
}
```

**Authorization:**
- **Admin only:** Only admins can delete providers

**Audit Log:** Yes (deletion operation with cascade details)

**Audit Log Entry Example:**
```json
{
  "action": "DELETE_PROVIDER",
  "provider_id": "ip_openai-001",
  "provider_name": "openai",
  "user_id": "user_admin",
  "agents_affected": ["agent_abc123", "agent_def456", "agent_ghi789"],
  "agents_count": 3,
  "cascade": true,
  "timestamp": "2025-12-10T15:30:00Z"
}
```

**Fields:**
- `cascade: true` indicates cascade deletion occurred
- `agents_affected` lists all agents that had provider (for audit compliance)
- `agents_count` shows scale of impact


### Provider-Agent Assignment

#### Assign Providers to Agent

**Endpoint:** `PUT /api/v1/agents/{agent_id}/providers`

**Description:** Replaces the complete provider list for an agent. At least one provider required.

**Request:**

```json
PUT /api/v1/agents/agent_abc123/providers
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "providers": ["ip_openai-001", "ip_anthropic-001"]
}
```

**Request Parameters:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `providers` | array<string> | Yes | Min 0, Max unlimited | Provider IDs to assign (can be empty to remove all providers) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "agent_id": "agent_abc123",
  "providers": [
    {
      "id": "ip_openai-001",
      "name": "openai",
      "endpoint": "https://api.openai.com/v1"
    },
    {
      "id": "ip_anthropic-001",
      "name": "anthropic",
      "endpoint": "https://api.anthropic.com/v1"
    }
  ],
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
      "providers": "At least one provider required"
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
    "message": "Provider 'ip_invalid' does not exist"
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent 'agent_invalid' does not exist"
  }
}
```

**Authorization:**
- **Agent Owner:** Can assign providers to own agents
- **Admin:** Can assign providers to any agent

**Audit Log:** Yes (mutation operation)


#### Get Agent Providers

**Endpoint:** `GET /api/v1/agents/{agent_id}/providers`

**Description:** Returns current provider assignments for an agent.

**Request:**

```
GET /api/v1/agents/agent_abc123/providers
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "agent_id": "agent_abc123",
  "providers": [
    {
      "id": "ip_openai-001",
      "name": "openai",
      "endpoint": "https://api.openai.com/v1",
      "models": ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"]
    },
    {
      "id": "ip_anthropic-001",
      "name": "anthropic",
      "endpoint": "https://api.anthropic.com/v1",
      "models": ["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"]
    }
  ]
}
```

**Error Responses:**

```json
HTTP 404 Not Found
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent 'agent_invalid' does not exist"
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
- **Agent Owner:** Can view own agents
- **Admin:** Can view any agent

**Audit Log:** No (read operation)


#### Remove Provider from Agent

**Endpoint:** `DELETE /api/v1/agents/{agent_id}/providers/{provider_id}`

**Description:** Removes a single provider from an agent. Blocked if it's the agent's last provider.

**Request:**

```
DELETE /api/v1/agents/agent_abc123/providers/ip_openai-001
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "agent_id": "agent_abc123",
  "removed_provider": "ip_openai-001",
  "remaining_providers": ["ip_anthropic-001"]
}
```

**Error Responses:**

```json
HTTP 404 Not Found
{
  "error": {
    "code": "PROVIDER_NOT_ASSIGNED",
    "message": "Provider 'ip_openai-001' is not assigned to agent 'agent_abc123'"
  }
}
```

**Authorization:**
- **Agent Owner:** Can modify own agents
- **Admin:** Can modify any agent

**Audit Log:** Yes (mutation operation)


### Data Models

#### Provider Object

```json
{
  "id": "ip_openai-001",
  "name": "openai",
  "endpoint": "https://api.openai.com/v1",
  "models": ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"],
  "credentials_configured": true,
  "status": "active",
  "usage": {
    "agent_count": 12,
    "total_requests": 45789,
    "total_spend": 1245.67,
    "requests_today": 892,
    "spend_today": 34.21
  },
  "created_at": "2025-12-10T10:30:45Z",
  "updated_at": "2025-12-10T10:30:45Z"
}
```

#### Provider Assignment Object

```json
{
  "agent_id": "agent_abc123",
  "providers": [
    {
      "id": "ip_openai-001",
      "name": "openai",
      "endpoint": "https://api.openai.com/v1",
      "models": ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"]
    }
  ],
  "updated_at": "2025-12-10T15:22:10Z"
}
```


### Security

#### Credential Storage

**At Rest:**
- Encrypted using AES-256-GCM
- Encryption key stored in secure key management system (not in database)
- Each credential has unique initialization vector (IV)
- Credentials never stored in plain text

**In Transit:**
- HTTPS required for all API requests (TLS 1.2+)
- API endpoints reject HTTP requests (http:// rejected in validation)
- Credentials transmitted only during Create/Update operations
- Credentials never included in responses

**In Logs:**
- Credentials excluded from audit logs
- Credentials excluded from application logs
- Credential validation errors logged without credential values
- Example: "Provider credentials validation failed" (not "API key 'sk-xyz' invalid")

#### Authorization Matrix

| Operation | User | Admin |
|-----------|------|-------|
| Create provider | ❌ | ✅ |
| List providers | ✅ | ✅ |
| Get provider details | ✅ | ✅ |
| Update provider | ❌ | ✅ |
| Delete provider | ❌ | ✅ |
| Assign providers to agent | ✅ (own agents) | ✅ (all agents) |
| Get agent providers | ✅ (own agents) | ✅ (all agents) |
| Remove provider from agent | ✅ (own agents) | ✅ (all agents) |

**Reasoning:**
- **Provider CRUD:** Admin-only (system-wide configuration)
- **Provider assignment:** Agent owners + admins (agent_level configuration)


### Error Handling

#### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Field validation failed |
| `NO_FIELDS_PROVIDED` | 400 | Update with no fields |
| `UNAUTHORIZED` | 401 | Missing/invalid authentication |
| `TOKEN_EXPIRED` | 401 | Authentication token expired |
| `FORBIDDEN` | 403 | Insufficient permissions (not admin) |
| `AGENT_NOT_FOUND` | 404 | Agent does not exist |
| `PROVIDER_NOT_FOUND` | 404 | Provider does not exist |
| `PROVIDER_NOT_ASSIGNED` | 404 | Provider not assigned to agent |
| `PROVIDER_EXISTS` | 409 | Provider name already exists |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Unexpected server error |


### Rate Limiting

#### Limits (per user)

| Endpoint | Limit | Window |
|----------|-------|--------|
| `POST /api/v1/providers` | 10 | 1 minute |
| `GET /api/v1/providers` | 60 | 1 minute |
| `GET /api/v1/providers/{id}` | 60 | 1 minute |
| `PUT /api/v1/providers/{id}` | 30 | 1 minute |
| `DELETE /api/v1/providers/{id}` | 10 | 1 minute |
| `PUT /api/v1/agents/{id}/providers` | 30 | 1 minute |
| `GET /api/v1/agents/{id}/providers` | 60 | 1 minute |
| `DELETE /api/v1/agents/{id}/providers/{pid}` | 30 | 1 minute |


### Audit Logging

#### Logged Operations

| Endpoint | Method | Logged | Credentials in Log |
|----------|--------|--------|--------------------|
| `POST /api/v1/providers` | POST | ✅ Yes | ❌ No (excluded) |
| `GET /api/v1/providers` | GET | ❌ No | N/A |
| `GET /api/v1/providers/{id}` | GET | ❌ No | N/A |
| `PUT /api/v1/providers/{id}` | PUT | ✅ Yes | ❌ No (excluded) |
| `DELETE /api/v1/providers/{id}` | DELETE | ✅ Yes | N/A |
| `PUT /api/v1/agents/{id}/providers` | PUT | ✅ Yes | N/A |
| `DELETE /api/v1/agents/{id}/providers/{pid}` | DELETE | ✅ Yes | N/A |

#### Audit Log Entry (Provider Creation)

```json
{
  "timestamp": "2025-12-10T10:30:45Z",
  "user_id": "user_admin_001",
  "endpoint": "POST /api/v1/providers",
  "method": "POST",
  "resource_type": "provider",
  "resource_id": "ip_openai-001",
  "action": "create",
  "parameters": {
    "name": "openai",
    "endpoint": "https://api.openai.com/v1",
    "models": ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"]
    // Note: credentials EXCLUDED
  },
  "status": "success",
  "ip_address": "203.0.113.42",
  "user_agent": "iron-cli/1.0.0"
}
```


### CLI Integration

#### iron providers create

```bash
iron providers create \
  --name openai \
  --endpoint https://api.openai.com/v1 \
  --api-key sk-proj_xyz789... \
  --models gpt-4,gpt-4-turbo,gpt-3.5-turbo

# Output:
# Provider created: ip_openai-001
# Name: openai
# Endpoint: https://api.openai.com/v1
# Models: gpt-4, gpt-4-turbo, gpt-3.5-turbo
# Status: active
```

#### iron providers list

```bash
iron providers list
iron providers list --name openai
iron providers list --status active

# Output:
# ID               NAME        AGENTS  STATUS
# ip_openai-001    openai      12      active
# ip_anthropic-001 anthropic   8       active
```

#### iron providers get

```bash
iron providers get ip_openai-001

# Output:
# ID:       ip_openai-001
# Name:     openai
# Endpoint: https://api.openai.com/v1
# Models:   gpt-4, gpt-4-turbo, gpt-3.5-turbo
# Status:   active
# Usage:
#   Agents:         12
#   Total Requests: 45,789
#   Total Spend:    $1,245.67
#   Requests Today: 892
#   Spend Today:    $34.21
# Created:  2025-12-10 10:30:45
```

#### iron providers update

```bash
iron providers update ip_openai-001 \
  --endpoint https://api.openai.com/v2 \
  --models gpt-4,gpt-4-turbo,gpt-4o,gpt-3.5-turbo

# Rotate credentials
iron providers update ip_openai-001 \
  --api-key sk-proj_new-key-abc123...

# Output:
# Provider updated: ip_openai-001
```

#### iron providers delete

```bash
iron providers delete ip_openai-001

# Confirmation prompt (if provider has agents):
# Delete provider 'openai' (ip_openai-001)?
# This will affect 3 agents:
#   - agent_abc123 (Production Agent 1)
#   - agent_def456 (Test Agent)
#   - agent_ghi789 (Dev Agent)
# These agents will have this provider removed automatically.
# Continue? [y/N]

# Output (after confirmation):
# Provider deleted: ip_openai-001
# Affected agents: 3
#   - agent_abc123 (has 2 remaining providers)
#   - agent_def456 (has 1 remaining provider)
#   - agent_ghi789 (has 0 providers - cannot make requests until provider assigned)
```

#### iron agents assign-providers

```bash
# Replace provider list
iron agents assign-providers agent_abc123 \
  --providers ip_openai-001,ip_anthropic-001

# Add provider (read current, append new, update)
iron agents assign-providers agent_abc123 --add ip_anthropic-001

# Remove provider
iron agents assign-providers agent_abc123 --remove ip_openai-001

# Output:
# Providers updated for agent_abc123
# Current providers:
#   - ip_openai-001 (openai)
#   - ip_anthropic-001 (anthropic)
```


### Database Schema (Reference)

#### providers Table

```sql
CREATE TABLE providers (
  id VARCHAR(50) PRIMARY KEY,           -- 'ip_openai-001'
  name VARCHAR(255) NOT NULL UNIQUE,    -- 'openai'
  endpoint VARCHAR(500) NOT NULL,       -- 'https://api.openai.com/v1'
  credentials TEXT NOT NULL,            -- Encrypted JSON (AES-256-GCM)
  models TEXT NOT NULL,                 -- JSON array ['gpt-4', 'gpt-3.5-turbo']
  status VARCHAR(20) NOT NULL,          -- 'active', 'inactive'
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,

  CONSTRAINT chk_status CHECK (status IN ('active', 'inactive'))
);
```

#### agent_providers Table (Many-to-Many Relationship)

```sql
CREATE TABLE agent_providers (
  agent_id VARCHAR(50) NOT NULL,
  provider_id VARCHAR(50) NOT NULL,
  assigned_at TIMESTAMP NOT NULL DEFAULT NOW(),

  PRIMARY KEY (agent_id, provider_id),

  -- CASCADE deletion: deleting agent OR provider removes assignment
  CONSTRAINT fk_agent FOREIGN KEY (agent_id)
    REFERENCES agents(id) ON DELETE CASCADE,
  CONSTRAINT fk_provider FOREIGN KEY (provider_id)
    REFERENCES providers(id) ON DELETE CASCADE
);

-- Indexes for query performance
CREATE INDEX idx_agent_providers_agent ON agent_providers(agent_id);
CREATE INDEX idx_agent_providers_provider ON agent_providers(provider_id);
```

**Cascade Behavior:**
- **Deleting agent:** Automatically removes all agent-provider assignments for that agent
- **Deleting provider:** Automatically removes all agent-provider assignments for that provider (CASCADE delete behavior)
- Both operations are automatic at database level (no application logic required)

**Migration Notes:**
- If upgrading from BLOCK delete to CASCADE delete, run migration:
  ```sql
  ALTER TABLE agent_providers
  DROP CONSTRAINT fk_provider;

  ALTER TABLE agent_providers
  ADD CONSTRAINT fk_provider
  FOREIGN KEY (provider_id) REFERENCES providers(id) ON DELETE CASCADE;
  ```


### Future Enhancements (Post-Pilot)

#### Provider Deletion Impact Preview (Dry-Run)

**Endpoint:** `GET /api/v1/providers/{id}/deletion-impact`

**Description:** Preview the impact of deleting a provider without actually deleting it. Returns list of agents that would be affected.

**Use Case:** Admin can preview impact before deleting, especially useful for large deployments.

**Request:**
```
GET /api/v1/providers/ip_openai-001/deletion-impact
Authorization: Bearer <user-token or api-token>
```

**Success Response:**
```json
HTTP 200 OK
Content-Type: application/json

{
  "provider_id": "ip_openai-001",
  "provider_name": "openai",
  "agents_affected": ["agent_abc123", "agent_def456", "agent_ghi789"],
  "agents_count": 3,
  "warnings": [
    {
      "agent_id": "agent_abc123",
      "agent_name": "Production Agent 1",
      "warning": "Agent has only this provider - will have zero providers after deletion",
      "severity": "high"
    },
    {
      "agent_id": "agent_def456",
      "agent_name": "Test Agent",
      "warning": "Agent has 2 providers - will continue with 1 remaining",
      "severity": "low"
    }
  ]
}
```

**Authorization:** Admin only

**Note:** This is a read-only operation. No changes are made to the database.

**Status:** POST-PILOT enhancement (not in initial release)


### Cross-References

#### Related Principles Documents

None.

#### Related Architecture Documents

- [007: Entity Model](../architecture/007_entity_model.md) - Provider entity definition and relationships

#### Used By

- Protocol 010: Agents API - Agent-provider assignments via /agents/{id}/providers endpoints
- Protocol 012: Analytics API - Provider usage and spending analytics

#### Dependencies

- Protocol 002: REST API Protocol - General REST API standards and conventions
- Protocol 010: Agents API - Agent entity for provider assignments

#### Implementation

- `/home/user1/pro/lib/wip_iron/iron_runtime/dev/module/iron_control_api/src/routes/providers.rs` - Providers API endpoint handlers
- `/home/user1/pro/lib/wip_iron/iron_runtime/dev/module/iron_control_api/src/routes/agent_provider_key.rs` - Agent-provider assignment handlers
- `/home/user1/pro/lib/wip_iron/iron_runtime/dev/module/iron_types/src/provider.rs` - Provider data structures

