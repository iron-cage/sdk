# Token Management API Reference

**Version:** 1.1.0
**Base URL:** `http://localhost:3000` (development) / `https://api.example.com` (production)
**Authentication:** JWT Bearer Token (most endpoints) / API Token (Key Fetch API)
**Date:** 2025-12-09

---

### Table of Contents

1. [Authentication](#authentication)
2. [Token Management](#token-management)
3. [Usage Analytics](#usage-analytics)
4. [Limits Management](#limits-management)
5. [Call Tracing](#call-tracing)
6. [AI Provider Key Management](#ai-provider-key-management)
7. [Key Fetch API](#key-fetch-api)
8. [Error Codes](#error-codes)
9. [Rate Limiting](#rate-limiting)

---

### Authentication

### POST /api/auth/login

Authenticate user and receive JWT tokens.

**Request:**
```json
{
  "username": "string",
  "password": "string"
}
```

**Response (200 OK):**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

**Errors:**
- `400 Bad Request` - Missing credentials
- `401 Unauthorized` - Invalid credentials

---

### POST /api/auth/refresh

Refresh access token using refresh token.

**Request:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

**Response (200 OK):**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

**Errors:**
- `401 Unauthorized` - Invalid/expired refresh token

---

### POST /api/auth/logout

Invalidate current tokens (blacklist).

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "message": "Logged out successfully"
}
```

---

### Token Management

### POST /api/tokens

Generate new API token.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request:**
```json
{
  "user_id": "string",
  "project_id": "string (optional)",
  "description": "string (optional)"
}
```

**Response (201 Created):**
```json
{
  "id": 123,
  "token": "sk_live_abc123...",
  "user_id": "user_001",
  "project_id": "project_001",
  "description": "Production API token",
  "created_at": 1701648000
}
```

**Errors:**
- `400 Bad Request` - Invalid request body
- `401 Unauthorized` - Missing/invalid JWT
- `403 Forbidden` - Insufficient permissions
- `429 Too Many Requests` - Rate limit exceeded

---

### GET /api/tokens

List all tokens for authenticated user.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
[
  {
    "id": 123,
    "user_id": "user_001",
    "project_id": "project_001",
    "name": "Production API token",
    "created_at": 1701648000,
    "last_used_at": 1701734400,
    "is_active": true
  }
]
```

---

### GET /api/tokens/{id}

Get specific token metadata (token value NOT returned).

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "id": 123,
  "user_id": "user_001",
  "project_id": "project_001",
  "name": "Production API token",
  "created_at": 1701648000,
  "last_used_at": 1701734400,
  "is_active": true
}
```

**Errors:**
- `404 Not Found` - Token ID not found

---

### POST /api/tokens/{id}/rotate

Rotate token (invalidate old, generate new).

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "id": 123,
  "token": "sk_live_xyz789...",
  "user_id": "user_001",
  "project_id": "project_001",
  "description": "Production API token",
  "created_at": 1701820800
}
```

**Errors:**
- `404 Not Found` - Token ID not found
- `409 Conflict` - Token already revoked

---

### POST /api/tokens/{id}/revoke

Revoke token (cannot be undone).

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "message": "Token revoked successfully"
}
```

**Errors:**
- `404 Not Found` - Token ID not found
- `409 Conflict` - Token already revoked

---

### Usage Analytics

### GET /api/usage

Get all usage records for authenticated user.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Query Parameters:**
- `start_date` (optional): Unix timestamp (filter by date range)
- `end_date` (optional): Unix timestamp
- `provider` (optional): Filter by provider (openai, anthropic, gemini)
- `limit` (optional): Max records to return (default: 100)

**Response (200 OK):**
```json
[
  {
    "id": 456,
    "token_id": 123,
    "provider": "openai",
    "model": "gpt-4",
    "input_tokens": 150,
    "output_tokens": 50,
    "cost": 0.0045,
    "timestamp": 1701734400
  }
]
```

---

### GET /api/usage/stats

Get aggregated usage statistics.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Query Parameters:**
- `start_date` (optional): Unix timestamp
- `end_date` (optional): Unix timestamp

**Response (200 OK):**
```json
{
  "total_requests": 1234,
  "total_input_tokens": 50000,
  "total_output_tokens": 25000,
  "total_cost": 15.75,
  "by_provider": [
    {
      "provider": "openai",
      "requests": 800,
      "cost": 12.50
    },
    {
      "provider": "anthropic",
      "requests": 434,
      "cost": 3.25
    }
  ],
  "by_model": [
    {
      "model": "gpt-4",
      "requests": 500,
      "cost": 10.00
    },
    {
      "model": "claude-3-opus",
      "requests": 300,
      "cost": 5.00
    }
  ]
}
```

---

### GET /api/usage/token/{token_id}

Get usage records for specific token.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
[
  {
    "id": 456,
    "token_id": 123,
    "provider": "openai",
    "model": "gpt-4",
    "input_tokens": 150,
    "output_tokens": 50,
    "cost": 0.0045,
    "timestamp": 1701734400
  }
]
```

---

### Limits Management

### POST /api/limits

Create usage limit.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request:**
```json
{
  "user_id": "user_001",
  "project_id": "project_001 (optional)",
  "limit_type": "budget",
  "limit_value": 100.00,
  "period": "monthly"
}
```

**Limit Types:**
- `budget` - Dollar amount ($)
- `tokens` - Total token count
- `requests` - Request count

**Periods:**
- `hourly`, `daily`, `weekly`, `monthly`, `yearly`

**Response (201 Created):**
```json
{
  "id": 789,
  "user_id": "user_001",
  "project_id": "project_001",
  "limit_type": "budget",
  "limit_value": 100.00,
  "period": "monthly",
  "created_at": 1701648000
}
```

---

### GET /api/limits

List all limits for authenticated user.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
[
  {
    "id": 789,
    "user_id": "user_001",
    "project_id": "project_001",
    "limit_type": "budget",
    "limit_value": 100.00,
    "period": "monthly",
    "created_at": 1701648000
  }
]
```

---

### PUT /api/limits/{id}

Update existing limit.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request:**
```json
{
  "limit_value": 150.00,
  "period": "monthly"
}
```

**Response (200 OK):**
```json
{
  "id": 789,
  "user_id": "user_001",
  "project_id": "project_001",
  "limit_type": "budget",
  "limit_value": 150.00,
  "period": "monthly",
  "created_at": 1701648000
}
```

---

### DELETE /api/limits/{id}

Delete limit.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "message": "Limit deleted successfully"
}
```

---

### Call Tracing

### GET /api/traces

Get API call traces (detailed request logs).

**Headers:**
```
Authorization: Bearer <access_token>
```

**Query Parameters:**
- `token_id` (optional): Filter by token ID
- `provider` (optional): Filter by provider
- `start_date` (optional): Unix timestamp
- `end_date` (optional): Unix timestamp
- `limit` (optional): Max records (default: 100)

**Response (200 OK):**
```json
[
  {
    "id": 999,
    "token_id": 123,
    "request_id": "req_abc123xyz",
    "provider": "openai",
    "model": "gpt-4",
    "input_tokens": 150,
    "output_tokens": 50,
    "cost": 0.0045,
    "timestamp": 1701734400,
    "metadata": {
      "temperature": 0.7,
      "max_tokens": 100
    }
  }
]
```

---

### GET /api/traces/{id}

Get specific trace details.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "id": 999,
  "token_id": 123,
  "request_id": "req_abc123xyz",
  "provider": "openai",
  "model": "gpt-4",
  "input_tokens": 150,
  "output_tokens": 50,
  "cost": 0.0045,
  "timestamp": 1701734400,
  "metadata": {
    "temperature": 0.7,
    "max_tokens": 100
  }
}
```

---

### AI Provider Key Management

Manage AI provider API keys (OpenAI, Anthropic) with encrypted storage and per-project assignment.

### POST /api/providers

Create a new AI provider key.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request:**
```json
{
  "provider": "openai",
  "api_key": "sk-proj-xxx...",
  "base_url": "https://api.openai.com/v1 (optional)",
  "description": "Production API key (optional)"
}
```

**Response (201 Created):**
```json
{
  "id": 1,
  "provider": "openai",
  "base_url": null,
  "description": "Production API key",
  "is_enabled": true,
  "created_at": 1733788800,
  "last_used_at": null,
  "masked_key": "***",
  "assigned_projects": []
}
```

**Errors:**
- `400 Bad Request` - Invalid provider or missing api_key
- `401 Unauthorized` - Missing/invalid JWT

---

### GET /api/providers

List all provider keys for authenticated user.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
[
  {
    "id": 1,
    "provider": "openai",
    "base_url": null,
    "description": "Production API key",
    "is_enabled": true,
    "created_at": 1733788800,
    "last_used_at": 1733875200,
    "masked_key": "***",
    "assigned_projects": ["lupo", "my-project"]
  }
]
```

---

### GET /api/providers/{id}

Get specific provider key (masked).

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "id": 1,
  "provider": "openai",
  "base_url": null,
  "description": "Production API key",
  "is_enabled": true,
  "created_at": 1733788800,
  "last_used_at": 1733875200,
  "masked_key": "***",
  "assigned_projects": ["lupo"]
}
```

**Errors:**
- `404 Not Found` - Provider key ID not found

---

### PUT /api/providers/{id}

Update provider key settings.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request:**
```json
{
  "base_url": "https://custom-endpoint.example.com (optional)",
  "description": "Updated description (optional)",
  "is_enabled": true
}
```

**Response (200 OK):**
```json
{
  "id": 1,
  "provider": "openai",
  "base_url": "https://custom-endpoint.example.com",
  "description": "Updated description",
  "is_enabled": true,
  "created_at": 1733788800,
  "last_used_at": 1733875200,
  "masked_key": "***",
  "assigned_projects": ["lupo"]
}
```

**Errors:**
- `404 Not Found` - Provider key ID not found

---

### DELETE /api/providers/{id}

Delete provider key.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (204 No Content)**

**Errors:**
- `404 Not Found` - Provider key ID not found

---

### POST /api/projects/{project_id}/provider

Assign provider key to a project.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request:**
```json
{
  "provider_key_id": 1
}
```

**Response (204 No Content)**

**Errors:**
- `404 Not Found` - Provider key ID not found
- `400 Bad Request` - Invalid request body

---

### DELETE /api/projects/{project_id}/provider

Unassign provider key from project.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (204 No Content)**

---

### Key Fetch API

External applications can fetch decrypted AI provider keys using API tokens. This endpoint uses **API token authentication** (not JWT).

### GET /api/keys

Fetch decrypted provider key for token's project.

**Headers:**
```
Authorization: Bearer <api_token>
```

**Note:** This endpoint uses the API token (from `/api/tokens`), NOT JWT. The token must be assigned to a project.

**Response (200 OK):**
```json
{
  "provider": "openai",
  "api_key": "sk-proj-actual-decrypted-key-here",
  "base_url": null
}
```

**Errors:**
- `400 Bad Request` - Token not assigned to a project
- `401 Unauthorized` - Invalid or expired API token
- `403 Forbidden` - Provider key is disabled
- `404 Not Found` - No provider key assigned to project
- `500 Internal Server Error` - Decryption failed

**Security Notes:**
1. This endpoint returns the **full decrypted API key** - treat responses carefully
2. Uses API tokens (not JWT) for authentication
3. Keys are scoped to projects via token's `project_id`
4. All fetches logged to audit_log for security review

**Example Usage:**
```bash
# Get API token's project key
curl -H "Authorization: Bearer pP/IDEeDNxQC/Kr3UHk8bWXjKAEkI1IgsGN465c9x88=" \
  http://localhost:3000/api/keys

# Response:
{
  "provider": "openai",
  "api_key": "sk-proj-actual-decrypted-key-here",
  "base_url": null
}
```

---

### Error Codes

### Standard HTTP Status Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | OK | Request successful |
| 201 | Created | Resource created successfully |
| 400 | Bad Request | Invalid request body/parameters |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Insufficient permissions (RBAC) |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Resource state conflict (e.g., already revoked) |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error (check logs) |

### Error Response Format

```json
{
  "error": "Error description",
  "code": "ERROR_CODE",
  "details": {
    "field": "Additional context"
  }
}
```

### Custom Error Codes

| Code | Description |
|------|-------------|
| `INVALID_TOKEN` | JWT token invalid/expired |
| `TOKEN_BLACKLISTED` | JWT token has been revoked |
| `INSUFFICIENT_PERMISSIONS` | User lacks required RBAC role |
| `RATE_LIMIT_EXCEEDED` | Too many requests in time window |
| `LIMIT_EXCEEDED` | Usage limit reached |
| `TOKEN_ALREADY_REVOKED` | Cannot operate on revoked token |

---

### Rate Limiting

### Authentication Endpoints
- **Limit:** 10 requests per minute per IP
- **Header:** `X-RateLimit-Remaining: <count>`
- **Reset:** `X-RateLimit-Reset: <unix_timestamp>`

### Token Operations
- **Limit:** 100 requests per minute per user
- **Burst:** 20 simultaneous requests

### Usage Queries
- **Limit:** 60 requests per minute per user

### Response When Limited

**Status:** `429 Too Many Requests`

```json
{
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT_EXCEEDED",
  "retry_after": 45
}
```

**Headers:**
```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1701734445
Retry-After: 45
```

---

### RBAC Permissions

### Roles

| Role | Permissions |
|------|-------------|
| **Admin** | Full access to all operations |
| **User** | Manage own tokens, view own usage, create limits |
| **Viewer** | Read-only access to own data |

### Permission Matrix

| Operation | Admin | User | Viewer |
|-----------|-------|------|--------|
| Create Token | ✅ | ✅ | ❌ |
| List Own Tokens | ✅ | ✅ | ✅ |
| Rotate Token | ✅ | ✅ | ❌ |
| Revoke Token | ✅ | ✅ | ❌ |
| View Usage | ✅ | ✅ | ✅ |
| Create Limit | ✅ | ✅ | ❌ |
| Delete Limit | ✅ | ✅ | ❌ |
| View Traces | ✅ | ✅ | ✅ |

---

### Best Practices

### Security
1. **Always use HTTPS** in production
2. **Store JWT securely** (httpOnly cookies or secure storage)
3. **Rotate tokens regularly** (every 90 days recommended)
4. **Never log token values** (only metadata)
5. **Revoke tokens immediately** when compromised

### Performance
1. **Cache usage statistics** (update every 5 minutes)
2. **Use pagination** for large result sets
3. **Filter by date range** to reduce query load
4. **Batch operations** when creating multiple tokens

### Rate Limiting
1. **Implement exponential backoff** on 429 responses
2. **Monitor rate limit headers** to avoid hitting limits
3. **Distribute requests** across time windows

---

### Examples

### Complete Authentication Flow

```bash
# 1. Login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "user@example.com", "password": "secret"}'

# Response:
# {
#   "access_token": "eyJ...",
#   "refresh_token": "eyJ...",
#   "expires_in": 3600
# }

# 2. Use access token for API calls
curl -X GET http://localhost:3000/api/tokens \
  -H "Authorization: Bearer eyJ..."

# 3. Refresh token when expired
curl -X POST http://localhost:3000/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "eyJ..."}'

# 4. Logout
curl -X POST http://localhost:3000/api/auth/logout \
  -H "Authorization: Bearer eyJ..."
```

### Token Lifecycle

```bash
# 1. Create token
curl -X POST http://localhost:3000/api/tokens \
  -H "Authorization: Bearer eyJ..." \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_001",
    "project_id": "project_001",
    "description": "Production API"
  }'

# 2. List tokens
curl -X GET http://localhost:3000/api/tokens \
  -H "Authorization: Bearer eyJ..."

# 3. Rotate token (security best practice)
curl -X POST http://localhost:3000/api/tokens/123/rotate \
  -H "Authorization: Bearer eyJ..."

# 4. Revoke token (when compromised)
curl -X POST http://localhost:3000/api/tokens/123/revoke \
  -H "Authorization: Bearer eyJ..."
```

---

**Document Version:** 1.1.0
**Last Updated:** 2025-12-09
**Maintained By:** Token Management Team

**Changelog:**
- 1.1.0 (2025-12-09): Added AI Provider Key Management (FR-11) and Key Fetch API (FR-12)
- 1.0.0 (2025-12-03): Initial release with Token Management, Usage Analytics, Limits, and Traces
