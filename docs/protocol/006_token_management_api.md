# Protocol 006: Token Management API

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-10

---

### Scope

REST API endpoints for IC Token lifecycle management (create, list, get, delete, rotate).

**In Scope:**
- IC Token CRUD operations (create, read, delete)
- IC Token rotation (regenerate while invalidating old)
- User Token authentication
- Permission-based access (admin vs developer)
- Request/response schemas

**Out of Scope:**
- User Token management (separate endpoint, see [007_authentication_api.md](007_authentication_api.md))
- IP Token management (vault-only, never exposed via API)
- Budget protocol (see [005_budget_control_protocol.md](005_budget_control_protocol.md))
- Implementation details (see `module/iron_token_manager/spec.md`)

---

### Purpose

Provide API for managing IC Token lifecycle, enabling developers to create/rotate tokens for their agents and admins to manage all tokens.

**Problem:**

Developers and admins need to:
- Create IC Tokens for new agents
- List IC Tokens (own tokens for developers, all tokens for admins)
- View IC Token details (without exposing the token value after creation)
- Delete IC Tokens when agents are decommissioned
- Rotate IC Tokens (regenerate) when compromised

**Solution:**

RESTful CRUD API with permission-based access:
- Developers: Can manage own IC Tokens (for agents they own)
- Admins: Can manage all IC Tokens (across all users/agents)
- Standard HTTP semantics (GET, POST, DELETE, PUT)
- User Token authentication (not IC Token - this API manages IC Tokens)
- Secure token rotation (atomic replace)

---

### Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- All entity IDs use `prefix_uuid` format with underscore separator
- `token_id`: `token_<uuid>` (e.g., `token_550e8400-e29b-41d4-a716-446655440000`)
- `agent_id`: `agent_<uuid>`
- `project_id`: `project_<uuid>`
- `user_id`: `user_<uuid>`

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Booleans: JSON boolean `true`/`false` (not strings)
- Nulls: Omit optional fields when empty (not `null`)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Consistent error response structure across all endpoints
- Machine-readable error codes: `VALIDATION_ERROR`, `UNAUTHORIZED`, `NOT_FOUND`, `DUPLICATE_NAME`
- HTTP status codes: 200, 201, 400, 401, 403, 404, 409

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- Pagination: Offset-based with `?page=N&per_page=M` (default 50 items/page)
- Filtering: Query parameters for `project_id`, `status`, `agent_id`
- URL structure: `/api/v1/tokens`, `/api/v1/tokens/{id}`

---

### Protocol Definition

### List IC Tokens

```http
GET /api/v1/tokens
Authorization: Bearer <USER_TOKEN>

Query Parameters:
- page (optional): Page number (default: 1)
- per_page (optional): Items per page (default: 50, max: 200)
- project_id (optional): Filter by project
- status (optional): Filter by status (active, revoked)
- agent_id (optional): Filter by agent

Response: 200 OK
{
  "data": [
    {
      "id": "tok-abc123",
      "agent_id": "agent_xyz789",
      "project_id": "proj_456",
      "status": "active",
      "created_at": "2025-12-09T09:00:00Z",
      "created_by": "user_admin",
      "last_used_at": "2025-12-09T12:30:00Z"
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

**Permission Rules:**
- Developer: Returns only IC Tokens for agents owned by the developer
- Admin: Returns all IC Tokens across all users/projects

### Get IC Token

```http
GET /api/v1/tokens/{token_id}
Authorization: Bearer <USER_TOKEN>

Response: 200 OK
{
  "id": "tok-abc123",
  "agent_id": "agent_xyz789",
  "project_id": "proj_456",
  "status": "active",
  "created_at": "2025-12-09T09:00:00Z",
  "created_by": "user_admin",
  "last_used_at": "2025-12-09T12:30:00Z",
  "usage_summary": {
    "total_requests": 1543,
    "total_cost_usd": 42.35
  }
}

Error: 404 Not Found
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "IC Token not found"
  }
}

Error: 403 Forbidden (Developer accessing another user's token)
{
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "Access denied to IC Token"
  }
}
```

**Permission Rules:**
- Developer: Can only access own IC Tokens
- Admin: Can access any IC Token

**Note:** Token value (`ic_abc123...`) NOT included in GET response (only in POST create response)

### Create IC Token

```http
POST /api/v1/tokens
Authorization: Bearer <USER_TOKEN>
Content-Type: application/json

Request:
{
  "agent_id": "agent_xyz789",
  "project_id": "proj_456",
  "description": "Production agent for project X" (optional)
}

Response: 201 Created
{
  "id": "tok-abc123",
  "token": "ic_abc123def456ghi789...",  // ⚠️ ONLY returned on creation
  "agent_id": "agent_xyz789",
  "project_id": "proj_456",
  "status": "active",
  "created_at": "2025-12-09T09:00:00Z",
  "created_by": "user_admin",
  "warning": "Save this token securely - it will NOT be shown again"
}

Error: 409 Conflict (Agent already has IC Token)
{
  "error": {
    "code": "RESOURCE_CONFLICT",
    "message": "IC Token already exists for agent",
    "details": {
      "agent_id": "agent_xyz789",
      "existing_token_id": "tok-old123"
    }
  }
}

Error: 400 Bad Request (Invalid agent_id)
{
  "error": {
    "code": "VALIDATION_INVALID_REFERENCE",
    "message": "Agent not found",
    "details": {"agent_id": "agent_nonexistent"}
  }
}

Error: 403 Forbidden (Developer creating token for agent not owned)
{
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "Cannot create IC Token for agent not owned by user"
  }
}
```

**Permission Rules:**
- Developer: Can only create IC Tokens for agents they own
- Admin: Can create IC Tokens for any agent

**Entity Constraint:** 1:1 relationship - one agent can have exactly one IC Token

### Delete IC Token

```http
DELETE /api/v1/tokens/{token_id}
Authorization: Bearer <USER_TOKEN>

Response: 204 No Content

Error: 404 Not Found
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "IC Token not found"
  }
}

Error: 403 Forbidden (Developer deleting another user's token)
{
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "Access denied to IC Token"
  }
}
```

**Permission Rules:**
- Developer: Can only delete own IC Tokens
- Admin: Can delete any IC Token

**Side Effects:**
- Token immediately invalidated
- Agent can no longer authenticate with Control Panel
- Budget protocol calls with deleted token return 401 Unauthorized

### Rotate IC Token

```http
PUT /api/v1/tokens/{token_id}/rotate
Authorization: Bearer <USER_TOKEN>

Response: 200 OK
{
  "id": "tok-abc123",  // Same token ID
  "token": "ic_new456def789ghi012...",  // ⚠️ NEW token value
  "agent_id": "agent_xyz789",
  "project_id": "proj_456",
  "status": "active",
  "created_at": "2025-12-09T09:00:00Z",  // Original creation date preserved
  "rotated_at": "2025-12-09T14:00:00Z",
  "rotated_by": "user_admin",
  "warning": "Old token invalidated - save new token securely"
}

Error: 404 Not Found
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "IC Token not found"
  }
}

Error: 403 Forbidden (Developer rotating another user's token)
{
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "Access denied to IC Token"
  }
}
```

**Permission Rules:**
- Developer: Can only rotate own IC Tokens
- Admin: Can rotate any IC Token

**Atomic Operation:**
1. Generate new IC Token value
2. Invalidate old IC Token value
3. Update database record with new value + rotated_at timestamp
4. Return new token value

**Critical:** Old token invalidated immediately, all in-flight requests with old token will fail

---

### Authentication

**All endpoints require User Token (NOT IC Token):**

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Why User Token?**
- This API manages IC Tokens (can't use IC Token to manage itself)
- User Token provides user identity for permission checks
- Maps to CLI usage pattern (`iron login` → get User Token → `iron tokens create`)

---

### HTTP Status Codes

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | OK | Successful GET, PUT (rotate) |
| 201 | Created | Successful POST (token created) |
| 204 | No Content | Successful DELETE |
| 400 | Bad Request | Invalid request body, malformed JSON, invalid references |
| 401 | Unauthorized | Missing or invalid User Token |
| 403 | Forbidden | Valid token but insufficient permissions (developer accessing other user's tokens) |
| 404 | Not Found | IC Token not found |
| 409 | Conflict | IC Token already exists for agent |
| 422 | Unprocessable Entity | Validation errors |
| 500 | Internal Server Error | Unexpected server error |

---

### Security Considerations

**Token Value Exposure:**
- Token value ONLY returned in POST (create) and PUT (rotate) responses
- GET endpoints NEVER include token value
- LIST endpoint NEVER includes token values
- After creation/rotation, token value cannot be retrieved again

**Permission Enforcement:**
- Developer role: Can only manage IC Tokens for owned agents
- Admin role: Can manage all IC Tokens
- Permission checks happen on EVERY request
- 403 Forbidden returned for unauthorized access attempts

**Token Rotation Security:**
- Old token invalidated atomically with new token generation
- No grace period (old token fails immediately)
- Rotation logged with timestamp and user
- Agent must be updated with new token immediately

**Rate Limiting:**
- See [002: Rate Limiting](002_rest_api_protocol.md#rate-limiting) for standard limits and response format

---

### CLI-API Parity

| API Endpoint | CLI Command | Notes |
|--------------|-------------|-------|
| `GET /api/v1/tokens` | `iron tokens list` | Developer sees own, admin sees all |
| `GET /api/v1/tokens/{id}` | `iron tokens get <id>` | Show token details |
| `POST /api/v1/tokens` | `iron tokens create --agent <agent-id>` | Create IC Token |
| `DELETE /api/v1/tokens/{id}` | `iron tokens delete <id>` | Delete IC Token |
| `PUT /api/v1/tokens/{id}/rotate` | `iron tokens rotate <id>` | Rotate IC Token |

**Parity Details:** See [features/004_token_management_cli_api_parity.md](../features/004_token_management_cli_api_parity.md) for complete mapping (24 operations).

---

### Cross-References

**Resource Organization:**
- [architecture/009: Resource Catalog](../architecture/009_resource_catalog.md) - IC Token as entity resource

**Entities:**
- [architecture/007: Entity Model](../architecture/007_entity_model.md) - IC Token entity definition (1:1 with agent)

**Protocols:**
- [002: REST API Protocol](002_rest_api_protocol.md) - Overall API overview
- [005: Budget Control Protocol](005_budget_control_protocol.md) - IC Token used for budget authentication
- [007: Authentication API](007_authentication_api.md) - User Token management (different from IC Token)

**Permissions:**
- [architecture/006: Roles and Permissions](../architecture/006_roles_and_permissions.md) - Admin vs Developer permissions

**Used By:**
- `iron_cli` - CLI tool calls these endpoints after `iron login`
- `iron_dashboard` - Web UI calls these endpoints for token management
- Developers - Create/rotate IC Tokens for their agents
- Admins - Manage all IC Tokens across organization

**Implementation:**
- Module: `module/iron_token_manager/` - Token management backend
- API: `module/iron_control_api/src/routes/tokens.rs` - Endpoint handlers
- Tests: `module/iron_control_api/tests/tokens_test.rs` - Integration tests

---

**Last Updated:** 2025-12-09
**Document Version:** 1.0
**API Version:** v1 (`/api/v1/`)
**Status:** ✅ Certain (required for Pilot)
