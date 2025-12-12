# Protocol 014: API Tokens API

**Status:** Specification
**Version:** 1.1.0
**Last Updated:** 2025-12-12
**Priority:** NICE-TO-HAVE

---

## Overview

The API Tokens API manages persistent authentication tokens for accessing the Iron Control Panel API. API Tokens are used primarily by dashboards and admin automation scripts. Each token inherits the permissions of the user who created it (SAME-AS-USER scope).

**Key characteristics:**
- **SAME-AS-USER scope:** Token inherits user's role and permissions
- **Primary use case:** Dashboard authentication (persistent sessions)
- **Secondary use case:** Admin automation (scripts, monitoring)
- **Security:** Token value shown ONLY on creation (GitHub pattern)
- **Lifecycle:** Create, List, Get, Validate, Revoke operations

---

## Token vs IC Token

**Clarification:** This protocol defines **API Tokens** (user authentication for Control Panel API). For agent authentication tokens, see [006: Token Management API](006_token_management_api.md) (IC Tokens).

| Feature | API Token | IC Token |
|---------|-----------|----------|
| **Purpose** | User authentication for Control Panel API | Agent authentication for inference requests |
| **Created by** | User via `POST /api/v1/api-tokens` | Auto-created with agent |
| **Scope** | Inherits user permissions (SAME-AS-USER) | Agent-specific (1:1 with agent) |
| **Use case** | Dashboard, admin scripts | AI inference requests |
| **Prefix** | `apitok_` | `ic_` |

---

## Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- All entity IDs use `prefix_uuid` format with underscore separator
- `token_id`: `apitoken_<uuid>` (e.g., `apitoken_550e8400-e29b-41d4-a716-446655440000`)
- `user_id`: `user_<uuid>`

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Token format: `at_<random_base64_32chars>` (e.g., `at_rY8xKpQm3nZ5vD9wF2sL7h`)
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Booleans: JSON boolean `true`/`false` (not strings)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Consistent error response structure across all endpoints
- Machine-readable error codes: `VALIDATION_ERROR`, `UNAUTHORIZED`, `NOT_FOUND`, `TOKEN_REVOKED`
- HTTP status codes: 200, 201, 400, 401, 403, 404

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- Pagination: Offset-based with `?page=N&per_page=M` (default 50 items/page)
- Filtering: Query parameters for `status`
- URL structure: `/api/v1/tokens/api`, `/api/v1/tokens/api/{id}`

---

## Endpoints

### Create API Token

**Endpoint:** `POST /api/v1/api-tokens`

**Description:** Creates a new API token with user's permissions. Token value returned ONLY on creation (never retrievable again).

**Request:**

```json
POST /api/v1/api-tokens
Authorization: Bearer <user-token>
Content-Type: application/json

{
  "name": "Dashboard Token",
  "description": "Token for production dashboard"
}
```

**Request Parameters:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `name` | string | Yes | 1-100 chars | Human-readable token name |
| `description` | string | No | Max 500 chars | Optional token description |

**Success Response:**

```json
HTTP 201 Created
Content-Type: application/json

{
  "id": "at_abc123",
  "token": "apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yz",
  "name": "Dashboard Token",
  "description": "Token for production dashboard",
  "user_id": "user_xyz789",
  "created_at": "2025-12-10T10:30:45Z",
  "last_used": null,
  "message": "⚠️  Save this token now. You won't be able to see it again."
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique token identifier (at- prefix) |
| `token` | string | Token value (shown ONCE, never again) |
| `name` | string | Token name |
| `description` | string | Token description (omitted if empty) |
| `user_id` | string | User who created token |
| `created_at` | string | ISO 8601 timestamp |
| `last_used` | string | Last usage timestamp (null for new tokens) |
| `message` | string | Warning to save token |

**Important:** The `token` field is returned ONLY in this response. It cannot be retrieved later via `GET /api/v1/api-tokens/{id}`. If lost, the token must be revoked and a new one created.

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "name": "Required field",
      "description": "Maximum 500 characters"
    }
  }
}
```

```json
HTTP 401 Unauthorized
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Authentication required"
  }
}
```

**Authorization:**
- **Any authenticated user:** Can create API tokens for themselves

**Audit Log:** Yes (mutation operation, token value EXCLUDED from log)

---

### List API Tokens

**Endpoint:** `GET /api/v1/api-tokens`

**Description:** Returns paginated list of API tokens. Users see only their own tokens; admins see all tokens. Token values NOT included (only metadata).

**Request:**

```
GET /api/v1/api-tokens?page=1&per_page=50&sort=-created_at
Authorization: Bearer <user-token or api-token>

# Admin filtering by specific user:
GET /api/v1/api-tokens?user_id=user_xyz789&page=1&per_page=50
Authorization: Bearer <admin-user-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number (1-indexed) |
| `per_page` | integer | 50 | Results per page (max 100) |
| `user_id` | string | - | Filter by user ID (admin-only, ignored for regular users) |
| `sort` | string | `-created_at` | Sort field: `name`, `created_at`, `last_used` (prefix `-` for desc) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "data": [
    {
      "id": "at_abc123",
      "name": "Dashboard Token",
      "description": "Token for production dashboard",
      "user_id": "user_xyz789",
      "created_at": "2025-12-10T10:30:45Z",
      "last_used": "2025-12-10T15:22:10Z"
    },
    {
      "id": "at_def456",
      "name": "Monitoring Script",
      "description": "Token for budget monitoring automation",
      "user_id": "user_xyz789",
      "created_at": "2025-12-09T14:20:30Z",
      "last_used": "2025-12-10T15:00:00Z"
    },
    {
      "id": "at_ghi789",
      "name": "Old Token",
      "user_id": "user_xyz789",
      "created_at": "2025-11-01T08:15:00Z",
      "last_used": null
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 3,
    "total_pages": 1
  }
}
```

**Response Fields:**

- **`data[]`:** Array of token metadata objects (NO token values)
- Token value NEVER included in list response
- **Scoping:** Users see only `user_id` matching their own; admins see tokens from all users

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
- **User:** Can list own API tokens only
- **Admin:** Can list all API tokens (from all users)

**Audit Log:** No (read operation)

---

### Get API Token Details

**Endpoint:** `GET /api/v1/api-tokens/{id}`

**Description:** Returns metadata for a specific API token. Token value NOT included.

**Request:**

```
GET /api/v1/api-tokens/at-abc123
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "at_abc123",
  "name": "Dashboard Token",
  "description": "Token for production dashboard",
  "user_id": "user_xyz789",
  "created_at": "2025-12-10T10:30:45Z",
  "last_used": "2025-12-10T15:22:10Z",
  "usage_stats": {
    "total_requests": 1247,
    "requests_today": 89,
    "requests_last_hour": 12
  }
}
```

**Response Fields (additional vs List):**

| Field | Type | Description |
|-------|------|-------------|
| `usage_stats` | object | Token usage statistics |
| `usage_stats.total_requests` | integer | Total requests since creation |
| `usage_stats.requests_today` | integer | Requests today |
| `usage_stats.requests_last_hour` | integer | Requests in last 60 minutes |

**Error Responses:**

```json
HTTP 404 Not Found
{
  "error": {
    "code": "TOKEN_NOT_FOUND",
    "message": "API token 'at-invalid' does not exist"
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
- **Token Owner:** Can view own tokens
- **Other Users:** 403 Forbidden (including admins - token privacy)

**Audit Log:** No (read operation)

---

### Revoke API Token

**Endpoint:** `DELETE /api/v1/api-tokens/{id}`

**Description:** Revokes (soft deletes) an API token. Token becomes invalid immediately. Audit trail preserved.

**Request:**

```
DELETE /api/v1/api-tokens/at-abc123
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "at_abc123",
  "name": "Dashboard Token",
  "revoked": true,
  "revoked_at": "2025-12-10T15:30:45Z",
  "message": "Token revoked. All requests using this token will now fail."
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Token identifier |
| `name` | string | Token name |
| `revoked` | boolean | Always true on success |
| `revoked_at` | string | ISO 8601 timestamp of revocation |
| `message` | string | Confirmation message |

**Error Responses:**

```json
HTTP 404 Not Found
{
  "error": {
    "code": "TOKEN_NOT_FOUND",
    "message": "API token 'at-invalid' does not exist"
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
HTTP 409 Conflict
{
  "error": {
    "code": "TOKEN_ALREADY_REVOKED",
    "message": "Token 'at-abc123' is already revoked",
    "revoked_at": "2025-12-09T10:00:00Z"
  }
}
```

**Authorization:**
- **Token Owner:** Can revoke own tokens
- **Other Users:** 403 Forbidden (including admins - token privacy)

**Audit Log:** Yes (mutation operation)

---

### Validate API Token

**Endpoint:** `POST /api/v1/api-tokens/validate`

**Description:** Validates an API token without authentication. Returns token validity status and metadata if valid. This public endpoint allows external services to verify tokens without requiring their own authentication.

**Request:**

```json
POST /api/v1/api-tokens/validate
Content-Type: application/json

{
  "token": "apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yz"
}
```

**Request Parameters:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `token` | string | Yes | 1-500 chars | API token value to validate |

**Success Response (Valid Token):**

```json
HTTP 200 OK
Content-Type: application/json

{
  "valid": true,
  "user_id": "user_xyz789",
  "project_id": "project_abc456",
  "token_id": 123
}
```

**Success Response (Invalid Token):**

```json
HTTP 200 OK
Content-Type: application/json

{
  "valid": false
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `valid` | boolean | Token validity status |
| `user_id` | string | User who created token (only if valid) |
| `project_id` | string | Associated project ID (only if valid, nullable) |
| `token_id` | integer | Token database ID (only if valid) |

**Important:**
- **Public endpoint:** No authentication required (allows external services to validate tokens)
- **Always 200 OK:** Returns success status even for invalid tokens (prevents information leakage)
- **Minimal metadata:** Only returns essential information for valid tokens
- **Security:** Uses constant-time comparison to prevent timing attacks

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Missing required field: token"
  }
}
```

**Authorization:**
- **No authentication required:** Public endpoint for external validation

**Audit Log:** No (read-only operation, public endpoint)

**Rate Limiting:** Recommend external rate limiting at reverse proxy (e.g., 100 requests/min per IP)

**Use Cases:**
1. **External services:** Validate tokens before processing requests
2. **Load balancers:** Health check token validity before routing
3. **Monitoring systems:** Verify token status without authentication

---

## Data Models

### API Token Object

```json
{
  "id": "at_abc123",
  "name": "Dashboard Token",
  "description": "Token for production dashboard",
  "user_id": "user_xyz789",
  "created_at": "2025-12-10T10:30:45Z",
  "last_used": "2025-12-10T15:22:10Z",
  "usage_stats": {
    "total_requests": 1247,
    "requests_today": 89,
    "requests_last_hour": 12
  }
}
```

### API Token Creation Response

```json
{
  "id": "at_abc123",
  "token": "apitok_xyz789abc123def456...",
  "name": "Dashboard Token",
  "description": "Token for production dashboard",
  "user_id": "user_xyz789",
  "created_at": "2025-12-10T10:30:45Z",
  "last_used": null,
  "message": "⚠️  Save this token now. You won't be able to see it again."
}
```

---

## Security

### Token Storage

**At Rest:**
- Stored as hash (bcrypt or Argon2)
- Never stored in plain text
- Cannot be retrieved (only compared during authentication)

**In Transit:**
- HTTPS required for all requests (TLS 1.2+)
- Token value transmitted only during creation
- Token value never included in logs

**Token Format:**
- Prefix: `apitok_` (identifies as API token)
- Length: 64 characters (after prefix)
- Character set: Base62 (alphanumeric, case-sensitive)
- Example: `apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yz`

### Token Lifecycle

1. **Creation:** User creates token via `POST /api/v1/api-tokens`
2. **Initial Display:** Token value shown once in creation response
3. **Storage:** User saves token (dashboard config, script, etc.)
4. **Usage:** Token sent in `Authorization: Bearer apitok_...` header
5. **Validation:** API verifies token hash on each request
6. **Tracking:** Last usage timestamp updated
7. **Revocation:** User revokes token via `DELETE /api/v1/api-tokens/{id}`
8. **Invalidation:** Token immediately invalid, requests fail with 401

### Token Permissions

**SAME-AS-USER scope:**
- Token inherits user's role (User, Admin)
- Token inherits user's permissions (view own agents, etc.)
- Token permissions updated if user role changes
- Token invalidated if user deleted/deactivated

**Example:**
- **User role:** Regular user (not admin)
- **Token permissions:**
  - ✅ View own agents
  - ✅ Create own agents
  - ✅ Modify own agents
  - ❌ View other users' agents
  - ❌ Modify other users' agents

**Example:**
- **User role:** Admin
- **Token permissions:**
  - ✅ View all agents
  - ✅ Create agents for any user
  - ✅ Modify any agent
  - ✅ Access admin-only endpoints

### Authorization Matrix

| Operation | Token Owner | Other User | Admin |
|-----------|-------------|------------|-------|
| Create token | ✅ | ❌ | ✅ (own) |
| List tokens | ✅ (own) | ❌ | ✅ (all) |
| Get token details | ✅ (own) | ❌ | ✅ (own) |
| Revoke token | ✅ (own) | ❌ | ✅ (own) |

**Note:** Admins can LIST all users' tokens (metadata only) but can only view details or revoke their own tokens (privacy/security).

---

## Error Handling

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Field validation failed |
| `UNAUTHORIZED` | 401 | Missing/invalid authentication |
| `TOKEN_EXPIRED` | 401 | User token expired (during token creation) |
| `FORBIDDEN` | 403 | Cannot access other users' tokens |
| `TOKEN_NOT_FOUND` | 404 | API token does not exist |
| `TOKEN_ALREADY_REVOKED` | 409 | Token already revoked |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

### Authentication with Revoked Token

When a revoked token is used:

```
Authorization: Bearer apitok_revoked...

HTTP 401 Unauthorized
{
  "error": {
    "code": "TOKEN_REVOKED",
    "message": "API token has been revoked",
    "revoked_at": "2025-12-09T10:00:00Z"
  }
}
```

---

## Rate Limiting

### Limits (per user)

| Endpoint | Limit | Window | Reasoning |
|----------|-------|--------|-----------|
| `POST /api/v1/api-tokens` | 10 | 1 minute | Token creation rare, prevent abuse |
| `GET /api/v1/api-tokens` | 60 | 1 minute | Standard read rate |
| `GET /api/v1/api-tokens/{id}` | 60 | 1 minute | Standard read rate |
| `DELETE /api/v1/api-tokens/{id}` | 10 | 1 minute | Token revocation rare |

---

## Audit Logging

### Logged Operations

| Endpoint | Method | Logged | Special Fields |
|----------|--------|--------|----------------|
| `POST /api/v1/api-tokens` | POST | ✅ Yes | Token value EXCLUDED |
| `GET /api/v1/api-tokens` | GET | ❌ No | N/A |
| `GET /api/v1/api-tokens/{id}` | GET | ❌ No | N/A |
| `DELETE /api/v1/api-tokens/{id}` | DELETE | ✅ Yes | N/A |

### Audit Log Entry (Token Creation)

```json
{
  "timestamp": "2025-12-10T10:30:45Z",
  "user_id": "user_xyz789",
  "endpoint": "POST /api/v1/api-tokens",
  "method": "POST",
  "resource_type": "api_token",
  "resource_id": "at_abc123",
  "action": "create",
  "parameters": {
    "name": "Dashboard Token",
    "description": "Token for production dashboard"
    // Note: token value EXCLUDED
  },
  "status": "success",
  "ip_address": "203.0.113.42",
  "user_agent": "iron-cli/1.0.0"
}
```

---

## CLI Integration

### iron api-tokens create

```bash
iron api-tokens create \
  --name "Dashboard Token" \
  --description "Token for production dashboard"

# Output:
# API Token created: at-abc123
# Token: apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yz
#
# ⚠️  IMPORTANT: Save this token now. You won't be able to see it again.
#
# To use this token:
#   export IRON_API_TOKEN="apitok_xyz789..."
#   iron agents list
```

### iron api-tokens list

```bash
iron api-tokens list
iron api-tokens list --sort -last_used

# Output:
# ID          NAME               CREATED              LAST USED
# at-abc123   Dashboard Token    2025-12-10 10:30:45  2025-12-10 15:22:10
# at-def456   Monitoring Script  2025-12-09 14:20:30  2025-12-10 15:00:00
# at-ghi789   Old Token          2025-11-01 08:15:00  Never used
```

### iron api-tokens get

```bash
iron api-tokens get at-abc123

# Output:
# ID:          at-abc123
# Name:        Dashboard Token
# Description: Token for production dashboard
# User:        user_xyz789
# Created:     2025-12-10 10:30:45
# Last Used:   2025-12-10 15:22:10
#
# Usage Stats:
#   Total Requests: 1,247
#   Requests Today: 89
#   Requests Last Hour: 12
```

### iron api-tokens revoke

```bash
iron api-tokens revoke at-abc123

# Output:
# API Token revoked: at-abc123 (Dashboard Token)
# Revoked at: 2025-12-10 15:30:45
#
# ⚠️  All requests using this token will now fail.
```

---

## Use Case Examples

### Example 1: Dashboard Authentication

**Scenario:** Web dashboard needs persistent authentication

**Setup:**
1. User creates API token via CLI or web UI
2. Dashboard stores token in configuration
3. Dashboard uses token for all API requests

**Dashboard config:**
```json
{
  "api_endpoint": "https://api.ironcage.ai/v1",
  "api_token": "apitok_xyz789abc123def456..."
}
```

**Dashboard API requests:**
```bash
# All requests use same token
curl -H "Authorization: Bearer apitok_xyz789..." \
  https://api.ironcage.ai/v1/agents

curl -H "Authorization: Bearer apitok_xyz789..." \
  https://api.ironcage.ai/v1/analytics/spending/total
```

---

### Example 2: Admin Automation Script

**Scenario:** Monitoring script checks budget status every 5 minutes

**Script:**
```bash
#!/bin/bash
# budget-monitor.sh

IRON_API_TOKEN="apitok_monitoring_script_xyz..."
API_URL="https://api.ironcage.ai/v1"

# Check budget status
response=$(curl -s -H "Authorization: Bearer $IRON_API_TOKEN" \
  "$API_URL/analytics/budget/status?threshold=80")

# Parse and alert if agents near limit
critical_count=$(echo "$response" | jq '.summary.critical')
if [ "$critical_count" -gt 0 ]; then
  echo "⚠️  WARNING: $critical_count agents near budget limit"
  # Send alert email/slack notification
fi
```

**Cron job:**
```
*/5 * * * * /path/to/budget-monitor.sh
```

---

### Example 3: Token Rotation

**Scenario:** Security policy requires token rotation every 90 days

**Steps:**
1. Create new token
2. Update dashboard/script with new token
3. Verify new token works
4. Revoke old token

**CLI:**
```bash
# Create new token
iron api-tokens create --name "Dashboard Token (2025-Q1)"
# Token: apitok_new_token_abc...

# Update dashboard config with new token

# Test new token
export IRON_API_TOKEN="apitok_new_token_abc..."
iron agents list
# ✅ Works

# Revoke old token
iron api-tokens revoke at-old-token_xyz
# ✅ Old token revoked
```

---

## Future Enhancements (Post-Pilot)

### Token Expiration

**Feature:** Tokens expire after N days (configurable)

**Benefits:**
- Automatic token rotation
- Reduced risk from leaked tokens
- Compliance with security policies

**Implementation:**
- Add `expires_at` field to token
- Return 401 TOKEN_EXPIRED for expired tokens
- Add `POST /api/v1/api-tokens/{id}/renew` endpoint

---

### Fine-Grained Permissions

**Feature:** Tokens with custom permission sets (not SAME-AS-USER)

**Benefits:**
- Least-privilege access (e.g., read-only tokens)
- Resource-specific tokens (e.g., specific agent only)
- Reduced risk from compromised tokens

**Implementation:**
- Add `permissions` field to token creation
- Support scopes: `agents:read`, `agents:write`, `analytics:read`, etc.
- Add `?agent_id=agent_abc` scope for agent-specific tokens

---

### Token Rotation

**Endpoint:** `POST /api/v1/api-tokens/{id}/rotate`

**Feature:** Generate new token value, invalidate old value

**Benefits:**
- No downtime during rotation (old and new both valid temporarily)
- Automatic credential rotation for compliance

---

## References

**Related Protocols:**
- [006: Token Management API](006_token_management_api.md) - IC Tokens (agent authentication)
- [007: Authentication API](007_authentication_api.md) - User authentication (login, logout)
- [010: Agents API](010_agents_api.md) - Agent management (API tokens used for access)
- [002: REST API Protocol](002_rest_api_protocol.md) - General standards

**Related Documents:**
<!-- TODO: Add Security Architecture and Authentication Model docs for token security and types -->

---
