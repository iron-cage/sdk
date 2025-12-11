# Protocol 007: Authentication API

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-10

---

### Scope

REST API endpoints for User authentication and User Token lifecycle management (login, logout, refresh, validate).

**In Scope:**
- User login (email/password → User Token)
- User logout (invalidate User Token)
- User Token refresh (extend expiration)
- User Token validation (check if valid)
- Request/response schemas
- User Token format and lifetime

**Out of Scope:**
- IC Token authentication (agent authentication, see [005_budget_control_protocol.md](005_budget_control_protocol.md))
- User registration/account creation (admin-managed, separate endpoint)
- Password reset (future feature)
- Multi-factor authentication (future feature)
- Implementation details (see `module/iron_token_manager/spec.md`)

---

### Purpose

Provide secure authentication for Control Panel access via CLI and web dashboard, issuing User Tokens for subsequent API calls.

**Problem:**

Users (admin, super user, developer) need to:
- Authenticate with Control Panel using credentials
- Receive User Token for API access (iron_cli, iron_dashboard)
- Refresh User Token before expiration
- Invalidate User Token on logout
- Verify User Token validity

**Solution:**

JWT-based authentication with token lifecycle management:
- Login: Email/password → User Token (JWT, 30 days)
- Refresh: Extend User Token expiration before it expires
- Logout: Invalidate User Token (blacklist)
- Validate: Check if User Token is valid and not expired
- Standard HTTP semantics (POST for state-changing operations)

**Token Type:** User Token (JWT) - NOT IC Token (IC Token is for agents, User Token is for users)

---

### Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- All entity IDs use `prefix_uuid` format with underscore separator
- `user_id`: `user_<uuid>` (e.g., `user_550e8400-e29b-41d4-a716-446655440000`)
- `session_id`: `session_<uuid>` (if applicable)

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Token lifetime: Integer seconds (e.g., `2592000` for 30 days)
- Booleans: JSON boolean `true`/`false` (not strings)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Consistent error response structure across all endpoints
- Machine-readable error codes: `INVALID_CREDENTIALS`, `TOKEN_EXPIRED`, `TOKEN_INVALID`, `UNAUTHORIZED`
- HTTP status codes: 200, 400, 401

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- URL structure: `/api/v1/auth/login`, `/api/v1/auth/logout`, `/api/v1/auth/refresh`
- Standard HTTP methods: POST for all authentication operations

---

### Protocol Definition

### Login

```http
POST /api/v1/auth/login
Content-Type: application/json

Request:
{
  "email": "developer@example.com",
  "password": "secure_password_123"
}

Response: 200 OK
{
  "user_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 2592000,  // 30 days in seconds
  "expires_at": "2026-01-08T09:00:00Z",
  "refresh_token": "refresh_abc123def456...",  (optional, future)
  "user": {
    "id": "user-abc123",
    "email": "developer@example.com",
    "role": "developer",
    "name": "John Doe"
  }
}

Error: 401 Unauthorized (Invalid credentials)
{
  "error": {
    "code": "AUTH_INVALID_CREDENTIALS",
    "message": "Invalid email or password"
  }
}

Error: 403 Forbidden (Account disabled)
{
  "error": {
    "code": "AUTH_ACCOUNT_DISABLED",
    "message": "Account has been disabled",
    "details": {"user_id": "user-abc123"}
  }
}

Error: 429 Too Many Requests (Rate limit)
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many login attempts",
    "details": {
      "retry_after": 300,  // seconds
      "limit": 5,
      "window": "300s"
    }
  }
}
```

**User Token Format (JWT):**
```json
{
  "sub": "user-abc123",      // User ID
  "email": "dev@example.com",
  "role": "developer",
  "iat": 1733740800,         // Issued at (Unix timestamp)
  "exp": 1736332800,         // Expires at (Unix timestamp)
  "jti": "token-xyz789"      // Token ID (for revocation)
}
```

**Security:**
- Password never logged or exposed in responses
- Rate limiting: 5 attempts per 5 minutes per IP
- Failed attempts logged for security monitoring
- Account lockout after 10 failed attempts (manual unlock by admin)

### Logout

```http
POST /api/v1/auth/logout
Authorization: Bearer <USER_TOKEN>

Response: 204 No Content

Error: 401 Unauthorized (Invalid token)
{
  "error": {
    "code": "AUTH_INVALID_TOKEN",
    "message": "Invalid or expired authentication token"
  }
}
```

**Implementation:**
- User Token added to blacklist (redis/database)
- Blacklist checked on every authenticated request
- Token remains blacklisted until original expiration time
- Multiple User Tokens per user supported (logout only invalidates current token)

**Side Effects:**
- Logged out User Token immediately invalid
- All subsequent requests with logged out token return 401 Unauthorized
- Other User Tokens for same user remain valid (if user has multiple sessions)

### Refresh User Token

```http
POST /api/v1/auth/refresh
Authorization: Bearer <USER_TOKEN>

Response: 200 OK
{
  "user_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",  // NEW token
  "token_type": "Bearer",
  "expires_in": 2592000,  // 30 days from now
  "expires_at": "2026-01-08T15:00:00Z",
  "user": {
    "id": "user-abc123",
    "email": "developer@example.com",
    "role": "developer",
    "name": "John Doe"
  }
}

Error: 401 Unauthorized (Token expired)
{
  "error": {
    "code": "AUTH_TOKEN_EXPIRED",
    "message": "Authentication token has expired",
    "details": {
      "expired_at": "2025-12-09T09:00:00Z"
    }
  }
}
```

**Refresh Window:**
- Can refresh anytime before expiration
- Recommended: Refresh when < 7 days remaining
- Old token invalidated when new token issued
- Atomic operation (old invalidated, new generated)

**CLI Behavior:**
- `iron_cli` automatically refreshes token when < 7 days remaining
- Refresh happens transparently during any CLI command
- User prompted to re-login if token expired

### Validate User Token

```http
POST /api/v1/auth/validate
Authorization: Bearer <USER_TOKEN>

Response: 200 OK
{
  "valid": true,
  "user": {
    "id": "user-abc123",
    "email": "developer@example.com",
    "role": "developer"
  },
  "expires_at": "2026-01-08T09:00:00Z",
  "expires_in": 2500000  // seconds remaining
}

Response: 200 OK (Invalid token)
{
  "valid": false,
  "reason": "TOKEN_EXPIRED",
  "expired_at": "2025-12-09T09:00:00Z"
}

Response: 200 OK (Blacklisted token)
{
  "valid": false,
  "reason": "TOKEN_REVOKED",
  "revoked_at": "2025-12-09T10:00:00Z"
}
```

**Note:** Validate returns 200 OK even for invalid tokens (result in response body)

**Use Cases:**
- CLI checks token validity before operations
- Dashboard validates token on page load
- Pre-flight check before batch operations

---

### Authentication Flow

**Initial Authentication (CLI):**
```
1. User runs: iron login
2. CLI prompts for email/password
3. CLI sends: POST /api/v1/auth/login
4. Control Panel returns User Token
5. CLI stores User Token in ~/.iron/credentials (encrypted)
6. Subsequent commands use stored User Token
```

**Token Refresh (Automatic):**
```
1. User runs any command: iron tokens list
2. CLI checks User Token expiration
3. If < 7 days remaining: POST /api/v1/auth/refresh
4. CLI updates stored User Token
5. CLI proceeds with original command
```

**Logout:**
```
1. User runs: iron logout
2. CLI sends: POST /api/v1/auth/logout (with current User Token)
3. Control Panel blacklists User Token
4. CLI deletes stored credentials
```

**Web Dashboard:**
```
1. User enters email/password in login form
2. Dashboard sends: POST /api/v1/auth/login
3. Dashboard stores User Token in sessionStorage (secure)
4. Dashboard includes User Token in Authorization header for all API calls
5. Dashboard auto-refreshes token every 24 hours (while user active)
6. Logout clears sessionStorage and calls POST /api/v1/auth/logout
```

---

### HTTP Status Codes

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | OK | Successful login, refresh, validate |
| 204 | No Content | Successful logout |
| 400 | Bad Request | Malformed request, missing required fields |
| 401 | Unauthorized | Invalid credentials, expired token, blacklisted token |
| 403 | Forbidden | Account disabled, insufficient permissions |
| 429 | Too Many Requests | Rate limit exceeded (login attempts) |
| 500 | Internal Server Error | Unexpected server error |

---

### Security Considerations

**Password Security:**
- Passwords hashed with bcrypt (cost factor 12)
- Never logged or stored in plaintext
- Never included in responses
- Password complexity requirements enforced

**Token Security:**
- JWT signed with HS256 (HMAC SHA-256)
- Secret key rotated quarterly
- Token includes jti (JWT ID) for revocation
- Blacklist checked on every authenticated request

**Rate Limiting:**
- See [002: Rate Limiting](002_rest_api_protocol.md#rate-limiting) for standard limits and response format

**Session Management:**
- Multiple User Tokens per user allowed (multi-device support)
- Each token tracked independently
- Logout only invalidates current token
- Admin can revoke all tokens for a user

**Storage:**
- CLI: User Token encrypted in `~/.iron/credentials` (AES-256)
- Dashboard: User Token in sessionStorage (HTTPS only, secure flag)
- Never store in localStorage (XSS risk)

---

### CLI-API Parity

| API Endpoint | CLI Command | Notes |
|--------------|-------------|-------|
| `POST /api/v1/auth/login` | `iron login` | Prompts for email/password, stores User Token |
| `POST /api/v1/auth/logout` | `iron logout` | Invalidates User Token, deletes credentials |
| `POST /api/v1/auth/refresh` | (automatic) | CLI auto-refreshes when < 7 days remaining |
| `POST /api/v1/auth/validate` | `iron auth status` | Shows token validity and expiration |

**Parity Details:** See [features/004_token_management_cli_api_parity.md](../features/004_token_management_cli_api_parity.md) for complete mapping.

---

### Cross-References

**Resource Organization:**
- [architecture/009: Resource Catalog](../architecture/009_resource_catalog.md) - Authentication as operation resource

**Entities:**
- [architecture/007: Entity Model](../architecture/007_entity_model.md) - User entity definition
- User Token (1:N with user) - Multiple tokens per user allowed

**Protocols:**
- [002: REST API Protocol](002_rest_api_protocol.md) - Overall API overview
- [006: Token Management API](006_token_management_api.md) - IC Token management (different from User Token)
- [005: Budget Control Protocol](005_budget_control_protocol.md) - IC Token used for budget (not User Token)

**Permissions:**
- [architecture/006: Roles and Permissions](../architecture/006_roles_and_permissions.md) - Admin, Super User, Developer roles

**Used By:**
- `iron_cli` - CLI login/logout/auto-refresh
- `iron_dashboard` - Web UI authentication
- All authenticated API endpoints - Require User Token in Authorization header

**Implementation:**
- Module: `module/iron_token_manager/` - JWT generation/validation
- API: `module/iron_control_api/src/routes/auth.rs` - Authentication endpoints
- Tests: `module/iron_control_api/tests/auth_test.rs` - Integration tests

---

**Last Updated:** 2025-12-09
**Document Version:** 1.0
**API Version:** v1 (`/api/v1/`)
**Status:** ✅ Certain (required for Pilot)
