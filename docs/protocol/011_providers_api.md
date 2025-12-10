# Protocol 011: Providers API

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-10

---

## Overview

The Providers API manages Inference Providers (IPs) - external AI services (OpenAI, Anthropic, etc.) that agents use to make inference requests. Each provider has configuration (name, endpoint, credentials), available models, and is assigned to one or more agents.

**Key characteristics:**
- **Full CRUD:** Create, Read, Update, Delete operations supported
- **Credential management:** IP Tokens (API keys) stored securely, transmitted via HTTPS
- **Many-to-Many Agent-Provider relationship:** Providers can serve multiple agents
- **Admin-controlled:** Provider management requires admin permissions

---

## Endpoints

### Create Provider

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
    "api_key": "sk-proj-xyz789..."
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
  "id": "ip-openai-001",
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
| `id` | string | Unique provider identifier (ip- prefix) |
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

---

### List Providers

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
      "id": "ip-openai-001",
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
      "id": "ip-anthropic-001",
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

---

### Get Provider Details

**Endpoint:** `GET /api/v1/providers/{id}`

**Description:** Returns detailed information about a specific provider, including usage statistics.

**Request:**

```
GET /api/v1/providers/ip-openai-001
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "ip-openai-001",
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
    "message": "Provider 'ip-invalid' does not exist"
  }
}
```

**Authorization:**
- **Any authenticated user:** Can view provider details

**Audit Log:** No (read operation)

---

### Update Provider

**Endpoint:** `PUT /api/v1/providers/{id}`

**Description:** Updates provider configuration. At least one field must be provided. Credentials can be updated securely.

**Request:**

```json
PUT /api/v1/providers/ip-openai-001
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "endpoint": "https://api.openai.com/v2",
  "credentials": {
    "api_key": "sk-proj-new-key-abc123..."
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
  "id": "ip-openai-001",
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
    "message": "Provider 'ip-invalid' does not exist"
  }
}
```

**Authorization:**
- **Admin only:** Only admins can update providers

**Audit Log:** Yes (mutation operation, credentials EXCLUDED from log)

---

### Delete Provider

**Endpoint:** `DELETE /api/v1/providers/{id}`

**Description:** Deletes a provider. Blocked if provider is assigned to any agents (safety mechanism).

**Request:**

```
DELETE /api/v1/providers/ip-openai-001
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "ip-openai-001",
  "deleted": true
}
```

**Error Responses:**

```json
HTTP 409 Conflict
{
  "error": {
    "code": "PROVIDER_IN_USE",
    "message": "Cannot delete provider: 3 agents are using this provider",
    "agents": ["agent-abc123", "agent-def456", "agent-ghi789"]
  }
}
```

**Workflow to delete provider in use:**
1. Remove provider from all agents: `PUT /api/v1/agents/{agent_id}/providers`
2. Verify no agents use provider: `GET /api/v1/providers/{id}` (check `agent_count: 0`)
3. Delete provider: `DELETE /api/v1/providers/{id}`

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
    "message": "Provider 'ip-invalid' does not exist"
  }
}
```

**Authorization:**
- **Admin only:** Only admins can delete providers

**Audit Log:** Yes (mutation operation)

---

## Provider-Agent Assignment

### Assign Providers to Agent

**Endpoint:** `PUT /api/v1/agents/{agent_id}/providers`

**Description:** Replaces the complete provider list for an agent. At least one provider required.

**Request:**

```json
PUT /api/v1/agents/agent-abc123/providers
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "providers": ["ip-openai-001", "ip-anthropic-001"]
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
  "agent_id": "agent-abc123",
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
    "message": "Provider 'ip-invalid' does not exist"
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
- **Agent Owner:** Can assign providers to own agents
- **Admin:** Can assign providers to any agent

**Audit Log:** Yes (mutation operation)

---

### Get Agent Providers

**Endpoint:** `GET /api/v1/agents/{agent_id}/providers`

**Description:** Returns current provider assignments for an agent.

**Request:**

```
GET /api/v1/agents/agent-abc123/providers
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "agent_id": "agent-abc123",
  "providers": [
    {
      "id": "ip-openai-001",
      "name": "openai",
      "endpoint": "https://api.openai.com/v1",
      "models": ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"]
    },
    {
      "id": "ip-anthropic-001",
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
- **Agent Owner:** Can view own agents
- **Admin:** Can view any agent

**Audit Log:** No (read operation)

---

### Remove Provider from Agent

**Endpoint:** `DELETE /api/v1/agents/{agent_id}/providers/{provider_id}`

**Description:** Removes a single provider from an agent. Blocked if it's the agent's last provider.

**Request:**

```
DELETE /api/v1/agents/agent-abc123/providers/ip-openai-001
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "agent_id": "agent-abc123",
  "removed_provider": "ip-openai-001",
  "remaining_providers": ["ip-anthropic-001"]
}
```

**Error Responses:**

```json
HTTP 404 Not Found
{
  "error": {
    "code": "PROVIDER_NOT_ASSIGNED",
    "message": "Provider 'ip-openai-001' is not assigned to agent 'agent-abc123'"
  }
}
```

**Authorization:**
- **Agent Owner:** Can modify own agents
- **Admin:** Can modify any agent

**Audit Log:** Yes (mutation operation)

---

## Data Models

### Provider Object

```json
{
  "id": "ip-openai-001",
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

### Provider Assignment Object

```json
{
  "agent_id": "agent-abc123",
  "providers": [
    {
      "id": "ip-openai-001",
      "name": "openai",
      "endpoint": "https://api.openai.com/v1",
      "models": ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"]
    }
  ],
  "updated_at": "2025-12-10T15:22:10Z"
}
```

---

## Security

### Credential Storage

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

### Authorization Matrix

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
- **Provider assignment:** Agent owners + admins (agent-level configuration)

---

## Error Handling

### Error Codes

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
| `PROVIDER_IN_USE` | 409 | Cannot delete (agents using provider) |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

---

## Rate Limiting

### Limits (per user)

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

---

## Audit Logging

### Logged Operations

| Endpoint | Method | Logged | Credentials in Log |
|----------|--------|--------|--------------------|
| `POST /api/v1/providers` | POST | ✅ Yes | ❌ No (excluded) |
| `GET /api/v1/providers` | GET | ❌ No | N/A |
| `GET /api/v1/providers/{id}` | GET | ❌ No | N/A |
| `PUT /api/v1/providers/{id}` | PUT | ✅ Yes | ❌ No (excluded) |
| `DELETE /api/v1/providers/{id}` | DELETE | ✅ Yes | N/A |
| `PUT /api/v1/agents/{id}/providers` | PUT | ✅ Yes | N/A |
| `DELETE /api/v1/agents/{id}/providers/{pid}` | DELETE | ✅ Yes | N/A |

### Audit Log Entry (Provider Creation)

```json
{
  "timestamp": "2025-12-10T10:30:45Z",
  "user_id": "user-admin-001",
  "endpoint": "POST /api/v1/providers",
  "method": "POST",
  "resource_type": "provider",
  "resource_id": "ip-openai-001",
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

---

## CLI Integration

### iron providers create

```bash
iron providers create \
  --name openai \
  --endpoint https://api.openai.com/v1 \
  --api-key sk-proj-xyz789... \
  --models gpt-4,gpt-4-turbo,gpt-3.5-turbo

# Output:
# Provider created: ip-openai-001
# Name: openai
# Endpoint: https://api.openai.com/v1
# Models: gpt-4, gpt-4-turbo, gpt-3.5-turbo
# Status: active
```

### iron providers list

```bash
iron providers list
iron providers list --name openai
iron providers list --status active

# Output:
# ID               NAME        AGENTS  STATUS
# ip-openai-001    openai      12      active
# ip-anthropic-001 anthropic   8       active
```

### iron providers get

```bash
iron providers get ip-openai-001

# Output:
# ID:       ip-openai-001
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

### iron providers update

```bash
iron providers update ip-openai-001 \
  --endpoint https://api.openai.com/v2 \
  --models gpt-4,gpt-4-turbo,gpt-4o,gpt-3.5-turbo

# Rotate credentials
iron providers update ip-openai-001 \
  --api-key sk-proj-new-key-abc123...

# Output:
# Provider updated: ip-openai-001
```

### iron providers delete

```bash
iron providers delete ip-openai-001

# If provider in use:
# Error: Cannot delete provider: 3 agents are using this provider
#   - agent-abc123 (Production Agent 1)
#   - agent-def456 (Test Agent)
#   - agent-ghi789 (Dev Agent)
#
# Suggested workflow:
#   1. Remove provider from agents:
#      iron agents update agent-abc123 --remove-provider ip-openai-001
#   2. Verify provider not in use:
#      iron providers get ip-openai-001
#   3. Delete provider:
#      iron providers delete ip-openai-001

# Success output:
# Provider deleted: ip-openai-001
```

### iron agents assign-providers

```bash
# Replace provider list
iron agents assign-providers agent-abc123 \
  --providers ip-openai-001,ip-anthropic-001

# Add provider (read current, append new, update)
iron agents assign-providers agent-abc123 --add ip-anthropic-001

# Remove provider
iron agents assign-providers agent-abc123 --remove ip-openai-001

# Output:
# Providers updated for agent-abc123
# Current providers:
#   - ip-openai-001 (openai)
#   - ip-anthropic-001 (anthropic)
```

---

## References

**Related Protocols:**
- [010: Agents API](010_agents_api.md) - Agent management
- [012: Analytics API](012_analytics_api.md) - Provider usage analytics
- [002: REST API Protocol](002_rest_api_protocol.md) - General standards

**Related Documents:**
- [007: Entity Model](../architecture/007_entity_model.md) - Provider entity definition
- [003: Security Architecture](../architecture/003_security_architecture.md) - Credential encryption

---

**Protocol 011 Version:** 1.0.0
**Status:** Specification
**Last Updated:** 2025-12-10
