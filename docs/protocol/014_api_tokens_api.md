# Protocol: API Tokens API

Manages persistent authentication tokens for accessing the Iron Control Panel API.

### Scope

#### In Scope

- API token creation with user permissions inheritance (SAME-AS-USER scope)
- Token lifecycle management (create, list, get, validate, revoke)
- Token authentication for dashboard and automation scripts
- Token value security (shown once on creation, GitHub pattern)
- User-scoped token management (users see own tokens only)
- Admin-scoped token listing (admins can list all users' tokens)
- Token usage tracking and statistics
- Token validation endpoint (public, for external services)

#### Out of Scope

- IC Token management (agent authentication tokens - see Protocol 006)
- Token expiration and automatic rotation (post-pilot feature)
- Fine-grained permission scopes (post-pilot feature)
- Token renewal without revocation (post-pilot feature)
- User session management (see Protocol 007)
- Password reset workflows (see Protocol 007)

### Purpose

**User Need:** Users need secure, long-lived credentials for API authentication without exposing passwords in dashboard configurations or automation scripts. Applications need programmatic access tokens that work across sessions without requiring repeated login prompts. Monitoring systems need to validate tokens without their own authentication.

**Solution:** Protocol 014 provides RESTful API for creating, managing, and validating API tokens with SAME-AS-USER permission inheritance. Tokens use Bearer authentication with SHA-256 hashing for secure storage. Token values are shown only once at creation (GitHub pattern) to prevent accidental exposure, and validation endpoint allows external services to verify tokens without authentication. Users manage their own tokens with full lifecycle support (create, list, get, revoke).

**Key Insight:** API tokens separate authentication identity (who you are) from authorization credentials (what you can access). The SAME-AS-USER scope ensures tokens inherit user's role and permissions, enabling principle of least privilege while maintaining simplicity. The one-time display pattern (show token value once, never again) forces secure storage practices and prevents token leakage through logs or repeated API calls. The public validation endpoint enables external services to verify tokens without requiring their own authentication credentials.

---

**Status:** Certain (Required for dashboard and automation workflows)
**Version:** 1.2.0
**Last Updated:** 2025-12-14
**Priority:** MUST-HAVE

### Standards Compliance

This protocol defines the following ID formats:

- `token_id`: `at_<alphanumeric>` (e.g., `at_abc123`)
  - Pattern: `^at_[a-z0-9]{6,32}$`
  - Source: Protocol 014 (API Tokens API) - defined here
  - Usage: Database entity identifier for API tokens
  - Appears in: API responses (`id` field), database primary key

- Token value: `apitok_<base62_64chars>`
  - Pattern: `^apitok_[a-zA-Z0-9]{64}$`
  - Source: Protocol 014 (API Tokens API) - defined here
  - Usage: Authentication credential sent in Bearer header
  - Example: `apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yzABCDEFGH`
  - Security: Shown once at creation, hashed for storage, never retrievable

- `user_id`: `user_<alphanumeric>` (e.g., `user_xyz789`)
  - Pattern: `^user_[a-z0-9_]{3,32}$`
  - Source: Protocol 007 (Authentication API)
  - Usage: Owner of API token

- `project_id`: `project_<alphanumeric>` (e.g., `project_abc456`)
  - Pattern: `^project_[a-z0-9_]{3,32}$`
  - Source: Protocol 015 (Projects API)
  - Usage: Associated project for scoped API tokens (nullable)

**Data Format Standards:**
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45Z`)
- Booleans: JSON boolean `true`/`false` (not strings)
- Pagination: Offset-based with `?page=N&per_page=M` (default 50 items/page, max 100)

**Error Format Standards:**
- Consistent error response structure with `error.code` and `error.message`
- HTTP status codes: 200, 201, 400, 401, 403, 404, 409, 429, 500

### Token vs IC Token Comparison

This protocol defines **API Tokens** (user authentication for Control Panel API). For agent authentication tokens, see Protocol 006 (IC Tokens).

| Feature | API Token | IC Token |
|---------|-----------|----------|
| **Purpose** | User authentication for Control Panel API | Agent authentication for inference requests |
| **Created by** | User via `POST /api/v1/api-tokens` | Auto-created with agent |
| **Scope** | Inherits user permissions (SAME-AS-USER) | Agent-specific (1:1 with agent) |
| **Use case** | Dashboard, admin scripts | AI inference requests |
| **Prefix** | `apitok_` | `ic_` |
| **Token ID prefix** | `at_` | (varies) |

### Endpoints

#### Create API Token

**Endpoint:** `POST /api/v1/api-tokens`

**Description:** Creates a new API token with user's permissions. Token value returned ONLY on creation (never retrievable again).

##### Authentication

Requires user authentication via `Authorization: Bearer <user-token>` header.

##### Request

```json
POST /api/v1/api-tokens
Authorization: Bearer <user-token>
Content-Type: application/json

{
  "name": "Dashboard Token",
  "description": "Token for production dashboard"
}
```

##### Request Parameters

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `name` | string | Yes | 1-100 chars | Human-readable token name |
| `description` | string | No | Max 500 chars | Optional token description |

##### Success Response

```json
HTTP 201 Created
Content-Type: application/json

{
  "id": "at_abc123",
  "token": "apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yzABCDEFGH",
  "name": "Dashboard Token",
  "description": "Token for production dashboard",
  "user_id": "user_xyz789",
  "created_at": "2025-12-10T10:30:45Z",
  "last_used": null,
  "message": "⚠️  Save this token now. You won't be able to see it again."
}
```

##### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique token identifier (at_ prefix) |
| `token` | string | Token value (shown ONCE, never again) |
| `name` | string | Token name |
| `description` | string | Token description (omitted if empty) |
| `user_id` | string | User who created token |
| `created_at` | string | ISO 8601 timestamp |
| `last_used` | string\|null | Last usage timestamp (null for new tokens) |
| `message` | string | Warning to save token |

**Important:** The `token` field is returned ONLY in this response. It cannot be retrieved later via `GET /api/v1/api-tokens/{id}`. If lost, the token must be revoked and a new one created.

##### Error Responses

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

##### Authorization

- **Any authenticated user:** Can create API tokens for themselves
- **Admins:** Can create API tokens for themselves (not for other users)

##### Audit Log

Yes (mutation operation, token value EXCLUDED from log for security)

#### List API Tokens

**Endpoint:** `GET /api/v1/api-tokens`

**Description:** Returns paginated list of API tokens. Users see only their own tokens; admins see all tokens. Token values NOT included (only metadata).

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```
GET /api/v1/api-tokens?page=1&per_page=50&sort=-created_at
Authorization: Bearer <user-token or api-token>

# Admin filtering by specific user:
GET /api/v1/api-tokens?user_id=user_xyz789&page=1&per_page=50
Authorization: Bearer <admin-user-token>
```

##### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number (1-indexed) |
| `per_page` | integer | 50 | Results per page (max 100) |
| `user_id` | string | - | Filter by user ID (admin-only, ignored for regular users) |
| `sort` | string | `-created_at` | Sort field: `name`, `created_at`, `last_used` (prefix `-` for desc) |

##### Success Response

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

##### Response Fields

- **`data[]`:** Array of token metadata objects (NO token values)
- Token value NEVER included in list response
- **Scoping:** Users see only `user_id` matching their own; admins see tokens from all users

##### Empty Results

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

##### Error Responses

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

##### Authorization

- **User:** Can list own API tokens only
- **Admin:** Can list all API tokens (from all users)

##### Audit Log

No (read operation)

#### Get API Token Details

**Endpoint:** `GET /api/v1/api-tokens/{id}`

**Description:** Returns metadata for a specific API token. Token value NOT included.

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```
GET /api/v1/api-tokens/at_abc123
Authorization: Bearer <user-token or api-token>
```

##### Success Response

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

##### Response Fields (additional vs List)

| Field | Type | Description |
|-------|------|-------------|
| `usage_stats` | object | Token usage statistics |
| `usage_stats.total_requests` | integer | Total requests since creation |
| `usage_stats.requests_today` | integer | Requests today |
| `usage_stats.requests_last_hour` | integer | Requests in last 60 minutes |

##### Error Responses

```json
HTTP 404 Not Found
{
  "error": {
    "code": "TOKEN_NOT_FOUND",
    "message": "API token 'at_invalid' does not exist"
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

##### Authorization

- **Token Owner:** Can view own tokens
- **Other Users:** 403 Forbidden (including admins - token privacy)

##### Audit Log

No (read operation)

#### Revoke API Token

**Endpoint:** `DELETE /api/v1/api-tokens/{id}`

**Description:** Revokes (soft deletes) an API token. Token becomes invalid immediately. Audit trail preserved.

##### Authentication

Requires authentication via `Authorization: Bearer <user-token or api-token>` header.

##### Request

```
DELETE /api/v1/api-tokens/at_abc123
Authorization: Bearer <user-token or api-token>
```

##### Success Response

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

##### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Token identifier |
| `name` | string | Token name |
| `revoked` | boolean | Always true on success |
| `revoked_at` | string | ISO 8601 timestamp of revocation |
| `message` | string | Confirmation message |

##### Error Responses

```json
HTTP 404 Not Found
{
  "error": {
    "code": "TOKEN_NOT_FOUND",
    "message": "API token 'at_invalid' does not exist"
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
    "message": "Token 'at_abc123' is already revoked",
    "revoked_at": "2025-12-09T10:00:00Z"
  }
}
```

##### Authorization

- **Token Owner:** Can revoke own tokens
- **Other Users:** 403 Forbidden (including admins - token privacy)

##### Audit Log

Yes (mutation operation)

#### Validate API Token

**Endpoint:** `POST /api/v1/api-tokens/validate`

**Description:** Validates an API token without authentication. Returns token validity status and metadata if valid. This public endpoint allows external services to verify tokens without requiring their own authentication.

##### Authentication

**No authentication required** - public endpoint for external validation.

##### Request

```json
POST /api/v1/api-tokens/validate
Content-Type: application/json

{
  "token": "apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yzABCDEFGH"
}
```

##### Request Parameters

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `token` | string | Yes | 1-500 chars | API token value to validate |

##### Success Response (Valid Token)

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

##### Success Response (Invalid Token)

```json
HTTP 200 OK
Content-Type: application/json

{
  "valid": false
}
```

##### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `valid` | boolean | Token validity status |
| `user_id` | string | User who created token (only if valid) |
| `project_id` | string | Associated project ID (only if valid, nullable) |
| `token_id` | integer | Internal database ID (only if valid) |

##### Important Notes

- **Public endpoint:** No authentication required (allows external services to validate tokens)
- **Always 200 OK:** Returns success status even for invalid tokens (prevents information leakage)
- **Minimal metadata:** Only returns essential information for valid tokens
- **Security:** Uses constant-time comparison to prevent timing attacks

##### Error Responses

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Missing required field: token"
  }
}
```

##### Authorization

No authentication required - public endpoint for external validation.

##### Audit Log

No (read-only operation, public endpoint)

##### Rate Limiting

Recommend external rate limiting at reverse proxy (e.g., 100 requests/min per IP)

##### Use Cases

1. **External services:** Validate tokens before processing requests
2. **Load balancers:** Health check token validity before routing
3. **Monitoring systems:** Verify token status without authentication

### Data Models

#### API Token Object

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

#### API Token Creation Response

```json
{
  "id": "at_abc123",
  "token": "apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yzABCDEFGH",
  "name": "Dashboard Token",
  "description": "Token for production dashboard",
  "user_id": "user_xyz789",
  "created_at": "2025-12-10T10:30:45Z",
  "last_used": null,
  "message": "⚠️  Save this token now. You won't be able to see it again."
}
```

### Security

#### Token Storage

**At Rest:**
- Stored as hash (bcrypt or Argon2)
- Never stored in plain text
- Cannot be retrieved (only compared during authentication)

**In Transit:**
- HTTPS required for all requests (TLS 1.2+)
- Token value transmitted only during creation
- Token value never included in logs

**Token Value Format:**
- Prefix: `apitok_` (identifies as API token)
- Length: 64 characters (after prefix)
- Character set: Base62 (alphanumeric, case-sensitive)
- Example: `apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yzABCDEFGH`

#### Token Lifecycle

1. **Creation:** User creates token via `POST /api/v1/api-tokens`
2. **Initial Display:** Token value shown once in creation response
3. **Storage:** User saves token (dashboard config, script, etc.)
4. **Usage:** Token sent in `Authorization: Bearer apitok_...` header
5. **Validation:** API verifies token hash on each request
6. **Tracking:** Last usage timestamp updated
7. **Revocation:** User revokes token via `DELETE /api/v1/api-tokens/{id}`
8. **Invalidation:** Token immediately invalid, requests fail with 401

#### Token Permissions

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

#### Authorization Matrix

| Operation | Token Owner | Other User | Admin |
|-----------|-------------|------------|-------|
| Create token | ✅ | ❌ | ✅ (own) |
| List tokens | ✅ (own) | ❌ | ✅ (all) |
| Get token details | ✅ (own) | ❌ | ❌ |
| Revoke token | ✅ (own) | ❌ | ❌ |

**Note:** Admins can LIST all users' tokens (metadata only) but can only view details or revoke their own tokens (privacy/security).

### Error Handling

#### Error Codes

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

#### Authentication with Revoked Token

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

### Rate Limiting

#### Limits (per user)

| Endpoint | Limit | Window | Reasoning |
|----------|-------|--------|-----------|
| `POST /api/v1/api-tokens` | 10 | 1 minute | Token creation rare, prevent abuse |
| `GET /api/v1/api-tokens` | 60 | 1 minute | Standard read rate |
| `GET /api/v1/api-tokens/{id}` | 60 | 1 minute | Standard read rate |
| `DELETE /api/v1/api-tokens/{id}` | 10 | 1 minute | Token revocation rare |

### Audit Logging

#### Logged Operations

| Endpoint | Method | Logged | Special Fields |
|----------|--------|--------|----------------|
| `POST /api/v1/api-tokens` | POST | ✅ Yes | Token value EXCLUDED |
| `GET /api/v1/api-tokens` | GET | ❌ No | N/A |
| `GET /api/v1/api-tokens/{id}` | GET | ❌ No | N/A |
| `DELETE /api/v1/api-tokens/{id}` | DELETE | ✅ Yes | N/A |

#### Audit Log Entry (Token Creation)

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

### CLI Integration

#### iron api-tokens create

```bash
iron api-tokens create \
  --name "Dashboard Token" \
  --description "Token for production dashboard"

# Output:
# API Token created: at_abc123
# Token: apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yzABCDEFGH
#
# ⚠️  IMPORTANT: Save this token now. You won't be able to see it again.
#
# To use this token:
#   export IRON_API_TOKEN="apitok_xyz789..."
#   iron agents list
```

#### iron api-tokens list

```bash
iron api-tokens list
iron api-tokens list --sort -last_used

# Output:
# ID          NAME               CREATED              LAST USED
# at_abc123   Dashboard Token    2025-12-10 10:30:45  2025-12-10 15:22:10
# at_def456   Monitoring Script  2025-12-09 14:20:30  2025-12-10 15:00:00
# at_ghi789   Old Token          2025-11-01 08:15:00  Never used
```

#### iron api-tokens get

```bash
iron api-tokens get at_abc123

# Output:
# ID:          at_abc123
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

#### iron api-tokens revoke

```bash
iron api-tokens revoke at_abc123

# Output:
# API Token revoked: at_abc123 (Dashboard Token)
# Revoked at: 2025-12-10 15:30:45
#
# ⚠️  All requests using this token will now fail.
```

### Use Case Examples

#### Example 1: Dashboard Authentication

**Scenario:** Web dashboard needs persistent authentication

**Setup:**
1. User creates API token via CLI or web UI
2. Dashboard stores token in configuration
3. Dashboard uses token for all API requests

**Dashboard config:**
```json
{
  "api_endpoint": "https://api.ironcage.ai/v1",
  "api_token": "apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yzABCDEFGH"
}
```

**Dashboard API requests:**
```bash
# All requests use same token
curl -H "Authorization: Bearer apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yzABCDEFGH" \
  https://api.ironcage.ai/v1/agents

curl -H "Authorization: Bearer apitok_xyz789abc123def456ghi789jkl012mno345pqr678stu901vwx234yzABCDEFGH" \
  https://api.ironcage.ai/v1/analytics/spending/total
```

#### Example 2: Admin Automation Script

**Scenario:** Monitoring script checks budget status every 5 minutes

**Script:**
```bash
#!/bin/bash
# budget-monitor.sh

IRON_API_TOKEN="apitok_M0n1t0r1ngScr1ptXyz789abc123def456ghi789jkl012mno345pqr678uvw901"
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

#### Example 3: Token Rotation

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
iron api-tokens revoke at_old_token_xyz
# ✅ Old token revoked
```

### Future Enhancements (Post-Pilot)

#### Token Expiration

**Feature:** Tokens expire after N days (configurable)

**Benefits:**
- Automatic token rotation
- Reduced risk from leaked tokens
- Compliance with security policies

**Implementation:**
- Add `expires_at` field to token
- Return 401 TOKEN_EXPIRED for expired tokens
- Add `POST /api/v1/api-tokens/{id}/renew` endpoint

#### Fine-Grained Permissions

**Feature:** Tokens with custom permission sets (not SAME-AS-USER)

**Benefits:**
- Least-privilege access (e.g., read-only tokens)
- Resource-specific tokens (e.g., specific agent only)
- Reduced risk from compromised tokens

**Implementation:**
- Add `permissions` field to token creation
- Support scopes: `agents:read`, `agents:write`, `analytics:read`, etc.
- Add `?agent_id=agent_abc` scope for agent-specific tokens

#### Token Rotation

**Endpoint:** `POST /api/v1/api-tokens/{id}/rotate`

**Feature:** Generate new token value, invalidate old value

**Benefits:**
- No downtime during rotation (old and new both valid temporarily)
- Automatic credential rotation for compliance

### Cross-References

#### Related Principles Documents

None

#### Related Architecture Documents

None

#### Used By

- Dashboard applications (web UI for Iron Control Panel)
- CLI tools (iron command-line interface)
- Admin automation scripts (monitoring, alerts, reporting)
- Third-party integrations (external services using Iron Control Panel API)

#### Dependencies

- Protocol 007: Authentication API (User authentication, user roles)
- Protocol 006: Token Management API (IC Tokens comparison, agent authentication)
- Protocol 010: Agents API (API tokens used for agent management access)
- Protocol 002: REST API Protocol (General REST standards, pagination, error formats)

#### Implementation

**Status:** Specified (Not yet implemented)

**Planned Files:**
- `module/iron_control_api/src/routes/api_tokens.rs` - Endpoint implementation
- `module/iron_control_api/src/services/token_service.rs` - Token business logic
- `module/iron_control_api/tests/api_tokens/endpoints.rs` - Integration tests
- `module/iron_control_api/tests/api_tokens/security.rs` - Security tests
