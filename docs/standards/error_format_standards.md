# Error Format Standards

**Purpose:** Define canonical error response format for all HTTP APIs

**Responsibility:** Specify error structure, HTTP status codes, and validation details

**Status:** Normative (all APIs MUST follow this standard)

**Version:** 1.0.0

---

## TL;DR

All HTTP error responses use simple custom JSON format with field-level details:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Budget must be at least 0.01",
    "fields": {
      "budget": "Must be >= 0.01",
      "name": "Required field"
    }
  }
}
```

**HTTP status:** 400 Bad Request (validation), 401 Unauthorized (auth), 403 Forbidden (permissions), etc.

---

## Standard Error Response Format

### Structure

All error responses MUST include an `error` object with:

```json
{
  "error": {
    "code": string,      // Machine-readable error code (UPPER_SNAKE_CASE)
    "message": string,   // Human-readable error message
    "fields"?: object    // Optional: Field-level validation errors
  }
}
```

**Required fields:**
- `code`: Machine-readable error identifier
- `message`: Human-readable description

**Optional fields:**
- `fields`: Object mapping field names to error messages (validation errors only)

### Design Rationale

**Why this format?**
1. **Simple**: Easy to parse, minimal nesting
2. **Actionable**: `fields` object clearly identifies what's wrong
3. **Consistent**: Same structure for all error types
4. **Machine-readable**: `code` enables client-side error handling
5. **Human-readable**: `message` suitable for logging/debugging

**Why not RFC 7807?**
- RFC 7807 adds complexity (`type` URLs, `instance`, `detail` vs `message`)
- Simple format sufficient for Pilot scope
- Can migrate to RFC 7807 post-Pilot if needed

---

## HTTP Status Codes

### Success Responses

| Code | Name | Usage |
|------|------|-------|
| **200** | OK | Successful GET, PUT, DELETE (with response body) |
| **201** | Created | Successful POST (resource created) |
| **204** | No Content | Successful DELETE (no response body) |

### Client Errors (4xx)

| Code | Name | Usage |
|------|------|-------|
| **400** | Bad Request | Validation errors, malformed requests, invalid parameters |
| **401** | Unauthorized | Missing authentication, invalid token, expired token |
| **403** | Forbidden | Authenticated but insufficient permissions |
| **404** | Not Found | Resource doesn't exist |
| **409** | Conflict | Resource conflict (duplicate, state conflict, constraint violation) |
| **429** | Too Many Requests | Rate limit exceeded |

### Server Errors (5xx)

| Code | Name | Usage |
|------|------|-------|
| **500** | Internal Server Error | Unexpected server error (log and alert) |
| **503** | Service Unavailable | Service temporarily down (maintenance, overload) |

### Status Code Selection Guide

**Validation errors:** 400 Bad Request
```json
POST /api/v1/agents {"budget": -10}
→ 400 Bad Request
```

**Authentication errors:** 401 Unauthorized
```json
GET /api/v1/agents
Authorization: Bearer <invalid-token>
→ 401 Unauthorized
```

**Permission errors:** 403 Forbidden
```json
DELETE /api/v1/agents/{id}  # Non-admin user
→ 403 Forbidden
```

**Resource not found:** 404 Not Found
```json
GET /api/v1/agents/agent_nonexistent
→ 404 Not Found
```

**Resource conflict:** 409 Conflict
```json
POST /api/v1/agents {"name": "Agent1"}  # Name already exists
→ 409 Conflict
```

**Rate limit:** 429 Too Many Requests
```json
# 21st request in 1 minute (limit: 20/min)
→ 429 Too Many Requests
```

**Server error:** 500 Internal Server Error
```json
# Database connection failed
→ 500 Internal Server Error
```

---

## Error Codes

### Validation Errors (400)

| Code | Usage | Example |
|------|-------|---------|
| `VALIDATION_ERROR` | Generic validation failure | Budget too low, missing required field |
| `INVALID_FORMAT` | Format validation failure | Invalid ID format, malformed JSON |
| `INVALID_RANGE` | Value out of valid range | Budget negative, per_page > 100 |

**Example:**
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed for 2 fields",
    "fields": {
      "budget": "Must be >= 0.01",
      "name": "Required field"
    }
  }
}
```

### Authentication Errors (401)

| Code | Usage | Example |
|------|-------|---------|
| `TOKEN_EXPIRED` | Authentication token expired | IC Token or API Token expired |
| `UNAUTHORIZED` | Generic authentication failure | Invalid token, missing Authorization header |

**Example (expired token):**
```json
{
  "error": {
    "code": "TOKEN_EXPIRED",
    "message": "Token expired"
  }
}
```

**Example (invalid token):**
```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Authentication failed"
  }
}
```

**Security note:** Use generic "Authentication failed" for invalid tokens (don't reveal "token not found" vs "signature mismatch").

### Authorization Errors (403)

| Code | Usage | Example |
|------|-------|---------|
| `FORBIDDEN` | Insufficient permissions | Non-admin trying admin operation |
| `INSUFFICIENT_PERMISSIONS` | Specific permission missing | User can't modify others' agents |

**Example:**
```json
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient permissions"
  }
}
```

### Resource Errors (404, 409)

| Code | Usage | Example |
|------|-------|---------|
| `NOT_FOUND` | Resource doesn't exist | Agent ID not in database |
| `CONFLICT` | Resource conflict | Duplicate name, budget already changed |
| `RESOURCE_IN_USE` | Cannot delete (dependencies) | Provider assigned to agents |

**Example (not found):**
```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Agent not found"
  }
}
```

**Example (conflict):**
```json
{
  "error": {
    "code": "CONFLICT",
    "message": "Agent name already exists",
    "fields": {
      "name": "Must be unique"
    }
  }
}
```

**Example (resource in use):**
```json
{
  "error": {
    "code": "RESOURCE_IN_USE",
    "message": "Cannot delete provider: 3 agents are using this provider",
    "details": {
      "agent_count": 3,
      "agents": ["agent_abc", "agent_def", "agent_ghi"]
    }
  }
}
```

### Rate Limiting Errors (429)

| Code | Usage | Example |
|------|-------|---------|
| `RATE_LIMIT_EXCEEDED` | Too many requests | User exceeded rate limit |

**Example:**
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests"
  }
}
```

**Headers (required):**
```http
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 20
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

### Server Errors (500, 503)

| Code | Usage | Example |
|------|-------|---------|
| `INTERNAL_ERROR` | Unexpected server error | Database connection failed, uncaught exception |
| `SERVICE_UNAVAILABLE` | Service temporarily down | Maintenance mode, overload |

**Example (internal error):**
```json
{
  "error": {
    "code": "INTERNAL_ERROR",
    "message": "Internal server error"
  }
}
```

**Security note:** NEVER expose stack traces, database errors, or internal details in error messages. Log details server-side only.

---

## Validation Error Details

### Single Field Error

```json
POST /api/v1/agents
{
  "budget": -10
}

Response: 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Budget must be at least 0.01",
    "fields": {
      "budget": "Must be >= 0.01"
    }
  }
}
```

### Multiple Field Errors (Batch Validation)

**Strategy:** Return ALL validation errors at once (not fail-fast).

**Rationale:** Better developer experience, fewer round trips.

```json
POST /api/v1/agents
{
  "budget": -10,
  "providers": ["invalid-id"]
}

Response: 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed for 3 fields",
    "fields": {
      "budget": "Must be >= 0.01",
      "name": "Required field",
      "providers[0]": "Invalid provider ID format"
    }
  }
}
```

**Array field notation:** Use `field[index]` for array element errors.

### Nested Field Errors

```json
PUT /api/v1/agents/{id}
{
  "metadata": {
    "tags": ["", "valid-tag"]
  }
}

Response: 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed",
    "fields": {
      "metadata.tags[0]": "Tag cannot be empty"
    }
  }
}
```

**Nested field notation:** Use dot notation (`parent.child`) and array notation (`field[index]`).

### No Field-Specific Errors

For errors without field mapping (e.g., business logic validation):

```json
POST /api/v1/agents
{
  "budget": 1000000
}

Response: 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Budget exceeds maximum allowed value ($100,000)"
  }
}
```

---

## Authentication Error Details

### Hybrid Strategy

**Specific for actionable errors:**
- Token expired → User can refresh token
- Token revoked → User can create new token

**Generic for security-sensitive errors:**
- Token invalid → Don't reveal "not found" vs "signature mismatch"
- Token malformed → Don't reveal token format details

### Token Expired (Specific)

```json
GET /api/v1/agents
Authorization: Bearer <expired-token>

Response: 401 Unauthorized
{
  "error": {
    "code": "TOKEN_EXPIRED",
    "message": "Token expired"
  }
}
```

**Client action:** Refresh token or prompt user to log in again.

### Token Invalid (Generic)

```json
GET /api/v1/agents
Authorization: Bearer <invalid-token>

Response: 401 Unauthorized
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Authentication failed"
  }
}
```

**Client action:** Prompt user to log in.

**Security rationale:** Generic message prevents enumeration attacks.

### Missing Authorization Header

```json
GET /api/v1/agents

Response: 401 Unauthorized
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Authentication required"
  }
}
```

### Insufficient Permissions (403)

```json
DELETE /api/v1/agents/{id}  # Non-admin user

Response: 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient permissions"
  }
}
```

**Security note:** Don't reveal "Admin role required" (information disclosure). Generic "Insufficient permissions" sufficient.

---

## Rate Limiting Response

### Headers + Body

**Headers (required):**
- `X-RateLimit-Limit`: Maximum requests per window
- `X-RateLimit-Remaining`: Requests remaining in current window
- `X-RateLimit-Reset`: Unix timestamp when window resets
- `Retry-After`: Seconds to wait before retrying

**Body (error object):**

```http
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 20
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1733830860
Retry-After: 60
Content-Type: application/json

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests"
  }
}
```

**Rationale:** Both headers (HTTP clients) and body (debugging, logging) for maximum compatibility.

### Successful Response with Rate Limit Info

Include rate limit headers on successful responses:

```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 20
X-RateLimit-Remaining: 15
X-RateLimit-Reset: 1733830860
Content-Type: application/json

{
  "id": "agent_550e8400-e29b-41d4-a716-446655440000",
  "name": "Production Agent"
}
```

**Rationale:** Clients can proactively throttle requests.

---

## Error Response Examples

### Example 1: Missing Required Field

```bash
POST /api/v1/agents
Content-Type: application/json

{
  "budget": 100.00
}

Response: 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed",
    "fields": {
      "name": "Required field"
    }
  }
}
```

### Example 2: Invalid ID Format

```bash
GET /api/v1/agents/invalid-id

Response: 400 Bad Request
{
  "error": {
    "code": "INVALID_FORMAT",
    "message": "Invalid agent ID format",
    "fields": {
      "id": "Expected format: agent_<uuid>"
    }
  }
}
```

### Example 3: Resource Not Found

```bash
GET /api/v1/agents/agent_nonexistent-uuid

Response: 404 Not Found
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Agent not found"
  }
}
```

### Example 4: Duplicate Resource

```bash
POST /api/v1/providers
{
  "name": "openai"  # Already exists
}

Response: 409 Conflict
{
  "error": {
    "code": "CONFLICT",
    "message": "Provider name already exists",
    "fields": {
      "name": "Must be unique"
    }
  }
}
```

### Example 5: Budget Conflict

```bash
PUT /api/v1/budget-requests/breq_abc/approve

# Budget already modified directly

Response: 409 Conflict
{
  "error": {
    "code": "CONFLICT",
    "message": "Budget has been modified since request was created"
  }
}
```

### Example 6: Internal Server Error

```bash
GET /api/v1/agents

# Database connection failed

Response: 500 Internal Server Error
{
  "error": {
    "code": "INTERNAL_ERROR",
    "message": "Internal server error"
  }
}
```

**Logging (server-side only):**
```
ERROR 2025-12-10T10:30:45Z [request_id=req_xyz789] Internal error: Database connection pool exhausted
  at iron_control_api::handlers::agents::list_agents (src/handlers/agents.rs:42)
  caused by: Connection pool timeout after 5s
```

**Security note:** NEVER expose stack traces or internal details in API response. Log details server-side for debugging.

---

## Implementation Guidelines

### Error Handling in Rust

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorDetails,
}

#[derive(Serialize)]
struct ErrorDetails {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<std::collections::HashMap<String, String>>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, fields) = match self {
            ApiError::ValidationError(field_errors) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Validation failed",
                Some(field_errors),
            ),
            ApiError::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                &format!("{} not found", resource),
                None,
            ),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Authentication failed",
                None,
            ),
            ApiError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Insufficient permissions",
                None,
            ),
            ApiError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
                None,
            ),
        };

        let error = ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message: message.to_string(),
                fields,
            },
        };

        (status, Json(error)).into_response()
    }
}
```

### Client-Side Error Handling

```typescript
// TypeScript client
async function createAgent(data: AgentData) {
    const response = await fetch('/api/v1/agents', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
    });

    if (!response.ok) {
        const error = await response.json();

        if (error.error.code === 'VALIDATION_ERROR') {
            // Display field-level errors in form
            Object.entries(error.error.fields).forEach(([field, message]) => {
                setFieldError(field, message);
            });
        } else if (error.error.code === 'TOKEN_EXPIRED') {
            // Redirect to login
            redirectToLogin();
        } else {
            // Generic error toast
            showErrorToast(error.error.message);
        }

        throw new Error(error.error.message);
    }

    return response.json();
}
```

---

## Testing

### Error Response Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_validation_error() {
        let response = create_agent_with_invalid_budget().await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body: ErrorResponse = response.json().await;
        assert_eq!(body.error.code, "VALIDATION_ERROR");
        assert!(body.error.fields.is_some());
        assert_eq!(body.error.fields.unwrap().get("budget"), Some(&"Must be >= 0.01".to_string()));
    }

    #[tokio::test]
    async fn test_not_found() {
        let response = get_agent("agent_nonexistent").await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body: ErrorResponse = response.json().await;
        assert_eq!(body.error.code, "NOT_FOUND");
        assert_eq!(body.error.message, "Agent not found");
    }
}
```

---

## References

### Related Documentation

- [REST API Protocol](../protocol/002_rest_api_protocol.md) - API implementation
- [Data Format Standards](./data_format_standards.md) - Response format standards
- [API Design Standards](./api_design_standards.md) - API conventions

### External Standards

- [RFC 7231 - HTTP Status Codes](https://tools.ietf.org/html/rfc7231) - Status code definitions
- [RFC 7807 - Problem Details](https://tools.ietf.org/html/rfc7807) - Alternative error format
- [OWASP - Error Handling](https://cheatsheetseries.owasp.org/cheatsheets/Error_Handling_Cheat_Sheet.html) - Security best practices

---

**Document Version:** 1.0.0
**Last Updated:** 2025-12-10
**Status:** Normative (must follow)
