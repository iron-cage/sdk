# Protocol: Authentication API



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


### Purpose

**User Need**: Developers (building agent applications via iron_cli), admins (managing organizational infrastructure via iron_dashboard), and super users (performing elevated operations) need secure authentication to Control Panel enabling initial login with email/password credentials to receive User Token (JWT) for API access, token refresh before expiration to extend session without re-authenticating (transparent in CLI, automatic in dashboard), token invalidation on logout to revoke access immediately, and token validation before operations to verify current session validity, with clear separation from IC Token (agent authentication) to avoid confusion where User Token authenticates users for Control Panel access while IC Token authenticates agents for budget protocol handshakes, supporting multiple concurrent sessions per user (multi-device: laptop CLI + desktop dashboard simultaneously) without one logout affecting other active sessions.

**Solution**: JWT-based authentication API with 4 REST endpoints implementing User Token lifecycle management. Provide POST /api/v1/auth/login (email/password → User Token JWT signed with HS256, 30-day expiration, returns user object with role for permission enforcement, rate limited to 5 attempts per 5 minutes per IP with account lockout after 10 failed attempts for defense in depth), POST /api/v1/auth/logout (blacklist current User Token in redis/database checked on every authenticated request, return 204 No Content, preserve other user tokens for multi-device support), POST /api/v1/auth/refresh (generate new User Token with extended 30-day expiration, invalidate old token atomically, CLI auto-refreshes when < 7 days remaining transparently during any command, dashboard auto-refreshes every 24 hours while user active), POST /api/v1/auth/validate (return 200 OK always with valid: true/false in body plus reason for invalidity: TOKEN_EXPIRED or TOKEN_REVOKED, used for pre-flight checks before batch operations). Authenticate all endpoints with User Token in Authorization: Bearer header (except login which uses email/password). Store CLI tokens encrypted in ~/.iron/credentials (AES-256), store dashboard tokens in sessionStorage (HTTPS only, NEVER localStorage due to XSS risk). Adhere to ID Format Standards (user_<uuid>), Data Format Standards (ISO 8601 timestamps, 30 days as 2592000 seconds integer), Error Format Standards (machine-readable codes: AUTH_INVALID_CREDENTIALS, AUTH_TOKEN_EXPIRED, AUTH_INVALID_TOKEN, AUTH_ACCOUNT_DISABLED, RATE_LIMIT_EXCEEDED), API Design Standards (standard REST conventions, POST for all auth operations).

**Key Insight**: Dual-layer security (IP-based rate limiting + account-based lockout) provides defense in depth against brute force attacks where IP rate limiting (5 attempts per 5 minutes) prevents automated attacks from single source while account lockout (10 failed attempts total) prevents distributed attacks across multiple IPs attempting same account credentials, operating independently so attacker hitting IP limit can't make 5 more attempts from new IP if already at 8 account-level failures. User Token vs IC Token separation prevents circular authentication dependency where IC Token management API (Protocol 006) requires User Token authentication (not IC Token) avoiding paradox where IC Tokens would manage themselves (POST /api/tokens with IC Token header creating token to create token). JWT structure (includes jti claim for token ID) enables selective revocation via blacklist while maintaining stateless verification for non-revoked tokens, balancing performance (no database lookup for valid tokens) with security (blacklist checked for revoked tokens, expired tokens rejected via exp claim). Multiple tokens per user (tracked independently, logout only invalidates current token) enables multi-device workflow where developer uses CLI on laptop + dashboard on desktop simultaneously, logout from laptop doesn't break dashboard session. Atomic token refresh (generate new, invalidate old, update database, return new in single transaction) eliminates race condition window where both old and new tokens temporarily valid preventing security gap where compromised old token still usable after refresh. Auto-refresh in CLI (when < 7 days remaining) balances user experience (no interruption during long-running operations) with security (tokens eventually expire), implemented transparently during any command execution.

---

**Status**: Specification
**Version**: 1.0.0
**Last Updated**: 2025-12-13
**Priority**: MUST-HAVE


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
- Machine-readable error codes: `AUTH_INVALID_CREDENTIALS`, `AUTH_TOKEN_EXPIRED`, `AUTH_INVALID_TOKEN`, `AUTH_ACCOUNT_DISABLED`, `RATE_LIMIT_EXCEEDED`
- HTTP status codes: 200, 400, 401

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- URL structure: `/api/v1/auth/login`, `/api/v1/auth/logout`, `/api/v1/auth/refresh`
- Standard HTTP methods: POST for all authentication operations


### Protocol Definition

#### Login

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
    "id": "user_abc123",
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
    "details": {"user_id": "user_abc123"}
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
  "sub": "user_abc123",      // User ID
  "email": "dev@example.com",
  "role": "developer",
  "iat": 1733740800,         // Issued at (Unix timestamp)
  "exp": 1736332800,         // Expires at (Unix timestamp)
  "jti": "token_xyz789"      // Token ID (for revocation)
}
```

**Security:**
- Password never logged or exposed in responses
- Rate limiting: 5 attempts per 5 minutes per IP
- Failed attempts logged for security monitoring
- Account lockout after 10 failed attempts (manual unlock by admin)

#### Logout

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

#### Refresh User Token

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
    "id": "user_abc123",
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

#### Validate User Token

```http
POST /api/v1/auth/validate
Authorization: Bearer <USER_TOKEN>

Response: 200 OK
{
  "valid": true,
  "user": {
    "id": "user_abc123",
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


### CLI-API Parity

| API Endpoint | CLI Command | Notes |
|--------------|-------------|-------|
| `POST /api/v1/auth/login` | `iron login` | Prompts for email/password, stores User Token |
| `POST /api/v1/auth/logout` | `iron logout` | Invalidates User Token, deletes credentials |
| `POST /api/v1/auth/refresh` | (automatic) | CLI auto-refreshes when < 7 days remaining |
| `POST /api/v1/auth/validate` | `iron auth status` | Shows token validity and expiration |

**Parity Details:** See [features/004_token_management_cli_api_parity.md](../features/004_token_management_cli_api_parity.md) for complete mapping.


### Cross-References

#### Related Principles Documents
- [Principles: Design Philosophy](../principles/001_design_philosophy.md) - API-First Design principle reflected in RESTful authentication API with standard HTTP semantics (POST for state-changing operations), Separation of Concerns via distinct User Token (users authenticate to Control Panel) vs IC Token (agents authenticate for budget protocol)
- [Principles: Quality Attributes](../principles/002_quality_attributes.md) - Security via dual-layer defense (IP rate limiting + account lockout), JWT signing with HS256, token blacklisting for revocation, encrypted storage (CLI AES-256, dashboard sessionStorage HTTPS-only); Usability via automatic token refresh in CLI (transparent when < 7 days remaining) and dashboard (every 24 hours), multi-device support (multiple tokens per user)

#### Related Architecture Documents
- [Architecture: Resource Catalog](../architecture/009_resource_catalog.md) - Authentication as Operation Resource in catalog, /api/auth endpoints implementing Operation Resource pattern (stateless operations on User Token lifecycle)
- [Architecture: Entity Model](../architecture/007_entity_model.md) - User entity definition, 1:N relationship with User Token (one user can have multiple active tokens for multi-device sessions)
- [Architecture: Roles and Permissions](../architecture/006_roles_and_permissions.md) - Admin, Super User, Developer role definitions returned in login response user object, role used for permission enforcement in other API endpoints

#### Used By
- `iron_cli` - CLI tool calls these endpoints for `iron login` (stores encrypted User Token in ~/.iron/credentials), `iron logout` (deletes credentials + blacklists token), automatic refresh (when < 7 days remaining during any command), `iron auth status` (validates token)
- `iron_dashboard` - Web UI calls these endpoints for login form authentication (stores User Token in sessionStorage), automatic refresh (every 24 hours while user active), logout (clears sessionStorage + blacklists token)
- All authenticated API endpoints - Require User Token in Authorization: Bearer header for authentication (Token Management API, User Management API, Agents API, etc.)

#### Dependencies
- [Protocol: REST API Protocol](002_rest_api_protocol.md) - Overall API overview, rate limiting standards (5 attempts per 5 minutes per IP, 429 Too Many Requests response format), error response format standards, authentication pattern guidance
- [Protocol: Token Management API](006_token_management_api.md) - IC Token management (different token type from User Token), User Token required for IC Token API authentication to avoid circular dependency
- [Protocol: Budget Control Protocol](005_budget_control_protocol.md) - IC Token used for budget protocol handshakes (not User Token), clarifies token type separation
- [Standards: ID Format Standards](../standards/id_format_standards.md) - Entity ID formats: `user_<uuid>`, `session_<uuid>` (if applicable) with underscore separator
- [Standards: Data Format Standards](../standards/data_format_standards.md) - ISO 8601 timestamp format with Z suffix, token lifetime as integer seconds (2592000 for 30 days), JSON boolean true/false (not strings)
- [Standards: Error Format Standards](../standards/error_format_standards.md) - Machine-readable error codes (AUTH_INVALID_CREDENTIALS, AUTH_TOKEN_EXPIRED, AUTH_INVALID_TOKEN, AUTH_ACCOUNT_DISABLED, RATE_LIMIT_EXCEEDED), consistent error response structure
- [Standards: API Design Standards](../standards/api_design_standards.md) - URL structure conventions (/api/v1/auth/login, /api/v1/auth/logout, /api/v1/auth/refresh, /api/v1/auth/validate), standard HTTP methods (POST for all authentication operations)

#### Implementation
- Module: `module/iron_token_manager/` - JWT generation with HS256 signing, token validation, token blacklist management (redis/database), User Token format definition
- API: `module/iron_control_api/src/routes/auth.rs` - Endpoint handlers for 4 authentication operations (login, logout, refresh, validate), bcrypt password hashing (cost factor 12), rate limiting enforcement, account lockout logic
- Tests: `module/iron_control_api/tests/auth_test.rs` - Integration tests covering login success/failure (invalid credentials, account disabled, rate limit), logout (blacklist verification), refresh (atomic token replacement), validate (valid/expired/revoked scenarios), multi-device sessions
- Specification: `module/iron_token_manager/spec.md` - Detailed implementation requirements for token manager backend, JWT claims structure (sub, email, role, iat, exp, jti), blacklist storage strategy

