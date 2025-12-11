# Protocol 008: User Management API

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-10

---

### Scope

REST API endpoints for administrator user account management (create, suspend, activate, delete, role changes, password resets).

**In Scope:**
- User creation (admin creates user accounts)
- User listing with filters (role, status, search)
- User retrieval (get user details by ID)
- User suspension and activation
- User soft deletion
- Role management (change user roles)
- Password reset (admin-initiated)
- Audit logging for all operations
- RBAC enforcement (Admin-only access)

**Out of Scope:**
- User self-registration (admin-managed only)
- User self-service password reset (future feature)
- User profile editing (future feature)
- Multi-factor authentication (future feature)
- User authentication (see [007_authentication_api.md](007_authentication_api.md))
- Implementation details (see `module/iron_token_manager/spec.md`)

---

### Purpose

Provide secure administrative control over user accounts in the Control Panel, enabling admins to manage user lifecycle, roles, and access.

**Problem:**

Administrators need to:
- Create user accounts for team members
- Suspend or activate accounts (instead of deletion)
- Change user roles (Admin, User, Viewer)
- Reset passwords when users are locked out
- Soft-delete accounts (preserve audit trail)
- Track all user management operations
- Filter and search users efficiently

**Solution:**

RESTful API with comprehensive CRUD operations:
- Role-Based Access Control (ManageUsers permission, Admin-only)
- Soft deletion pattern (deleted_at timestamp, not hard delete)
- Comprehensive audit logging (who did what, when, why)
- Flexible filtering (role, active status, search by name/email)
- Validation (email format, password strength, role values)
- Self-modification prevention (can't delete/change own role)

**Authorization:** All endpoints require Admin role with ManageUsers permission.

---

### Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- All entity IDs use `prefix_uuid` format with underscore separator
- `user_id`: `user_<uuid>` (e.g., `user_550e8400-e29b-41d4-a716-446655440000`)

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Booleans: JSON boolean `true`/`false` (not strings)
- Nulls: Omit optional fields when empty (not `null`)
- Arrays: Empty array `[]` when no items (not `null`)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Consistent error response structure across all endpoints
- Machine-readable error codes: `VALIDATION_ERROR`, `UNAUTHORIZED`, `FORBIDDEN`, `NOT_FOUND`, `DUPLICATE_EMAIL`
- HTTP status codes: 200, 201, 400, 401, 403, 404, 409
- Field-level validation details in `error.fields` object

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- Pagination: Offset-based with `?page=N&per_page=M` (default 50 items/page)
- Filtering: Query parameters for `role`, `status`, `search`
- Sorting: Optional `?sort=created_at` or `?sort=-created_at`
- URL structure: `/api/v1/users`, `/api/v1/users/{id}`

---

### Protocol Definition

### Create User

```http
POST /api/v1/users
Authorization: Bearer <USER_TOKEN>
Content-Type: application/json

Request:
{
  "username": "john_doe",
  "password": "SecurePass123!",
  "email": "john.doe@example.com",
  "role": "user"
}

Response: 201 Created
{
  "id": 1001,
  "username": "john_doe",
  "email": "john.doe@example.com",
  "role": "user",
  "is_active": true,
  "created_at": 1733740800000
}

Error: 400 Bad Request (Validation)
{
  "error": "email must contain @ symbol"
}

Error: 403 Forbidden (Insufficient permissions)
{
  "error": "insufficient permissions"
}

Error: 500 Internal Server Error (Duplicate username)
{
  "error": "failed to create user: Token management error"
}
```

**Validation Rules:**
- `username`: Required, non-empty, max 255 chars
- `password`: Required, min 8 chars, max 1000 chars
- `email`: Required, non-empty, must contain @, max 255 chars
- `role`: Required, must be one of: "viewer", "user", "admin"

**Side Effects:**
- New user created with `is_active=1` (active by default)
- Password hashed with BCrypt (cost factor 12)
- Audit log entry created (operation="create", performed_by=admin_id)
- User can immediately authenticate with credentials

### List Users

```http
GET /api/v1/users?role=admin&is_active=true&search=john&page=1&page_size=20
Authorization: Bearer <USER_TOKEN>

Response: 200 OK
{
  "users": [
    {
      "id": 1001,
      "username": "john_doe",
      "email": "john.doe@example.com",
      "role": "admin",
      "is_active": true,
      "created_at": 1733740800000,
      "last_login": 1733745000000,
      "suspended_at": null,
      "deleted_at": null
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20
}
```

**Query Parameters:**
- `role` (optional): Filter by role (viewer, user, admin)
- `is_active` (optional): Filter by active status (true, false)
- `search` (optional): Search by username or email (partial match)
- `page` (optional): Page number (default: 1)
- `page_size` (optional): Items per page (default: 20, max: 100)

**Pagination:**
- Results ordered by `created_at DESC` (newest first)
- `offset` calculated as `(page - 1) * page_size`
- `limit` enforced with max 100 items per page

### Get User by ID

```http
GET /api/v1/users/1001
Authorization: Bearer <USER_TOKEN>

Response: 200 OK
{
  "id": 1001,
  "username": "john_doe",
  "email": "john.doe@example.com",
  "role": "user",
  "is_active": true,
  "created_at": 1733740800000,
  "last_login": 1733745000000,
  "suspended_at": null,
  "deleted_at": null
}

Error: 404 Not Found
{
  "error": "failed to get user: Token management error"
}
```

### Suspend User

```http
PUT /api/v1/users/1001/suspend
Authorization: Bearer <USER_TOKEN>
Content-Type: application/json

Request:
{
  "reason": "Violation of terms of service"
}

Response: 200 OK
{
  "id": 1001,
  "username": "john_doe",
  "email": "john.doe@example.com",
  "role": "user",
  "is_active": false,
  "created_at": 1733740800000,
  "last_login": 1733745000000,
  "suspended_at": 1733750000000,
  "deleted_at": null
}

Error: 500 Internal Server Error (Already suspended)
{
  "error": "failed to suspend user: Token management error"
}
```

**Side Effects:**
- `is_active` set to 0 (false)
- `suspended_at` set to current timestamp
- `suspended_by` set to admin user ID
- Audit log entry created with reason
- User cannot authenticate while suspended
- Existing User Tokens remain valid (not automatically revoked)

### Activate User

```http
PUT /api/v1/users/1001/activate
Authorization: Bearer <USER_TOKEN>

Response: 200 OK
{
  "id": 1001,
  "username": "john_doe",
  "email": "john.doe@example.com",
  "role": "user",
  "is_active": true,
  "created_at": 1733740800000,
  "last_login": 1733745000000,
  "suspended_at": null,
  "deleted_at": null
}

Error: 500 Internal Server Error (Already active)
{
  "error": "failed to activate user: Token management error"
}
```

**Side Effects:**
- `is_active` set to 1 (true)
- `suspended_at` and `suspended_by` cleared (set to NULL)
- Audit log entry created
- User can authenticate again

### Delete User (Soft Delete + Agent Reassignment)

**Endpoint:** `DELETE /api/v1/users/{id}`

**Description:** Soft-deletes a user and automatically reassigns all owned agents to admin in the "Orphaned Agents" special project. Deleting a user sets deleted_at timestamp (user cannot login), reassigns agent ownership, cancels pending budget requests, and revokes all API tokens. Agents continue working normally with existing IC Tokens and budgets.

**Request:**
```http
DELETE /api/v1/users/1001
Authorization: Bearer <USER_TOKEN>
```

**Success Response (User with Agents):**
```json
HTTP 200 OK
Content-Type: application/json

{
  "id": 1001,
  "username": "john_doe",
  "email": "john.doe@example.com",
  "role": "user",
  "is_active": false,
  "created_at": 1733740800000,
  "last_login": 1733745000000,
  "suspended_at": null,
  "deleted_at": 1733755000000,
  "agents_affected": [
    {
      "agent_id": "agent_abc123",
      "name": "Production Agent 1",
      "new_owner_id": "admin_001",
      "new_project_id": "proj_orphaned",
      "budget": 100.00,
      "providers": ["ip_openai_001", "ip_anthropic_001"]
    },
    {
      "agent_id": "agent_def456",
      "name": "Test Agent",
      "new_owner_id": "admin_001",
      "new_project_id": "proj_orphaned",
      "budget": 10.00,
      "providers": ["ip_openai_001"]
    }
  ],
  "agents_count": 2,
  "budget_requests_cancelled": 3,
  "api_tokens_revoked": 2
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `agents_affected` | array | List of agents reassigned with new ownership details |
| `agents_count` | integer | Number of agents reassigned to admin |
| `budget_requests_cancelled` | integer | Number of pending budget requests auto-cancelled |
| `api_tokens_revoked` | integer | Number of API tokens revoked |

**Success Response (User with No Agents):**
```json
HTTP 200 OK
{
  "id": 1002,
  "username": "jane_doe",
  "deleted_at": 1733755100000,
  "agents_count": 0,
  "budget_requests_cancelled": 0,
  "api_tokens_revoked": 1
}
```

**Behavior Note:** Response format changes based on whether user owned agents. If `agents_count > 0`, response includes `agents_affected` array with full details.

**Error Responses:**

```json
HTTP 500 Internal Server Error (Self-deletion)
{
  "error": "failed to delete user: Token management error"
}
```

```json
HTTP 400 Bad Request (Last admin)
{
  "error": "Cannot delete last admin user"
}
```

**Side Effects:**

**User Record:**
- `is_active` set to 0 (false)
- `deleted_at` set to current timestamp
- `deleted_by` set to admin user ID
- User cannot authenticate (login blocked)
- User record preserved (soft delete, not hard delete)

**Owned Agents (if any):**
- All agents reassigned to admin (`owner_id` changed)
- All agents moved to "Orphaned Agents" project (`project_id` = `proj_orphaned`)
- Tags added to each agent: `orphaned`, `original-owner:{user_id}`
- IC Tokens remain valid (agents continue working)
- Budgets remain active (no service disruption)
- Provider access unchanged (agents can still make requests)

**Budget Change Requests:**
- Pending requests: Auto-cancelled (status = `cancelled`, review_notes = "Auto-cancelled: user deleted")
- Historical requests: Preserved (requester_id set to NULL if user deleted)

**API Tokens:**
- All user's API tokens revoked (`revoked_at` set, `revoked_by` = admin_id)
- Existing requests with these tokens fail with 401 Unauthorized

**User Tokens (Session):**
- Existing sessions remain valid until natural expiration
- Authentication layer checks `deleted_at IS NULL`, blocking new logins

**Audit Log:**
- User deletion logged with full reassignment details
- Includes: agents_affected list, budget_requests_cancelled count, api_tokens_revoked count

**Self-Deletion Prevention:**
- Admin cannot delete their own account
- Prevents accidental lockout
- Error: 500 Internal Server Error

**Last Admin Prevention:**
- Cannot delete last active admin user
- At least one admin must exist in system
- Error: 400 Bad Request

**Orphaned Agents Project:**
- Special project `proj_orphaned` ("Orphaned Agents")
- Contains all deleted users' agents
- Owned by system admin
- Admin can view all orphaned agents, reassign to new users, or delete
- Agents continue running normally (budgets active, IC Tokens valid)

**Edge Cases:**

1. **User with Zero Agents:**
   - Normal soft delete
   - No agent reassignment
   - `agents_count: 0`, `agents_affected: []`

2. **User with Many Agents (100+):**
   - All agents reassigned in single transaction
   - Bulk UPDATE efficient (<100ms for 100 agents)
   - Response includes all agents in `agents_affected` array

3. **Pending Budget Requests:**
   - All auto-cancelled with reason "Auto-cancelled: user deleted"
   - Requester gone, approval context lost

4. **Agent Service Continuity:**
   - NO SERVICE DISRUPTION
   - IC Tokens remain valid
   - Budgets continue working
   - Agents can make requests to providers

5. **Concurrent Agent Creation:**
   - If user deleted during agent creation for that user
   - Agent creation fails with 404 USER_NOT_FOUND

**Authorization:**
- Requires Admin role with ManageUsers permission
- Self-deletion always prevented (even for admins)
- Last admin deletion prevented (system integrity)

**Audit Log:** Yes (comprehensive mutation operation with reassignment details)

### Change User Role

```http
PUT /api/v1/users/1001/role
Authorization: Bearer <USER_TOKEN>
Content-Type: application/json

Request:
{
  "role": "admin"
}

Response: 200 OK
{
  "id": 1001,
  "username": "john_doe",
  "email": "john.doe@example.com",
  "role": "admin",
  "is_active": true,
  "created_at": 1733740800000,
  "last_login": 1733745000000,
  "suspended_at": null,
  "deleted_at": null
}

Error: 400 Bad Request (Invalid role)
{
  "error": "role must be one of: viewer, user, admin"
}

Error: 500 Internal Server Error (Self-modification)
{
  "error": "failed to change user role: Token management error"
}
```

**Valid Roles:**
- `viewer`: Read-only access (view tokens, usage, limits)
- `user`: Standard access (create tokens, view usage)
- `admin`: Full access (manage users, all operations)

**Side Effects:**
- User role changed immediately
- Audit log entry created with old and new roles
- Existing User Tokens retain old role (not refreshed)
- New User Tokens issued after login have new role

**Self-Modification Prevention:**
- Admin cannot change their own role
- Prevents accidental privilege loss

### Reset User Password

```http
POST /api/v1/users/1001/reset-password
Authorization: Bearer <USER_TOKEN>
Content-Type: application/json

Request:
{
  "new_password": "NewSecurePass456!",
  "force_change": true
}

Response: 200 OK
{
  "id": 1001,
  "username": "john_doe",
  "email": "john.doe@example.com",
  "role": "user",
  "is_active": true,
  "created_at": 1733740800000,
  "last_login": 1733745000000,
  "suspended_at": null,
  "deleted_at": null
}

Error: 400 Bad Request (Weak password)
{
  "error": "password must be at least 8 characters"
}
```

**Validation:**
- `new_password`: Required, min 8 chars, max 1000 chars
- `force_change`: Required boolean

**Side Effects:**
- Password hashed with BCrypt (cost factor 12)
- `force_password_change` flag set (if force_change=true)
- Audit log entry created
- User must change password on next login (if force_change=true)
- Existing User Tokens remain valid (not revoked)

---

### Audit Logging

All user management operations create audit log entries in the `user_audit_log` table:

```sql
CREATE TABLE user_audit_log
(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  operation TEXT NOT NULL,  -- create, suspend, activate, delete, role_change, password_reset
  target_user_id INTEGER NOT NULL,
  performed_by INTEGER NOT NULL,
  timestamp INTEGER NOT NULL,  -- Unix epoch milliseconds
  previous_state TEXT,  -- JSON (for updates)
  new_state TEXT,  -- JSON (for updates)
  reason TEXT  -- Optional reason (suspend)
);
```

**Operations:**
- `create`: User account created
- `suspend`: User account suspended
- `activate`: User account activated
- `delete`: User account soft-deleted
- `role_change`: User role changed
- `password_reset`: Password reset by admin

**Example Audit Entries:**

```json
// User creation
{
  "operation": "create",
  "target_user_id": 1001,
  "performed_by": 999,
  "timestamp": 1733740800000,
  "previous_state": null,
  "new_state": "{\"username\":\"john_doe\",\"email\":\"john.doe@example.com\",\"role\":\"user\"}",
  "reason": null
}

// User suspension
{
  "operation": "suspend",
  "target_user_id": 1001,
  "performed_by": 999,
  "timestamp": 1733750000000,
  "previous_state": "{\"is_active\":true}",
  "new_state": "{\"is_active\":false}",
  "reason": "Violation of terms of service"
}

// Role change
{
  "operation": "role_change",
  "target_user_id": 1001,
  "performed_by": 999,
  "timestamp": 1733760000000,
  "previous_state": "{\"role\":\"user\"}",
  "new_state": "{\"role\":\"admin\"}",
  "reason": null
}
```

---

### HTTP Status Codes

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | OK | Successful get, suspend, activate, delete, role change, password reset |
| 201 | Created | Successful user creation |
| 400 | Bad Request | Validation error (invalid email, weak password, invalid role) |
| 403 | Forbidden | Insufficient permissions (non-admin user) |
| 404 | Not Found | User not found by ID |
| 500 | Internal Server Error | Database error, duplicate username, self-modification |

---

### Security Considerations

**Authorization:**
- All endpoints require User Token (JWT) in Authorization header
- All endpoints require Admin role with ManageUsers permission
- Non-admin users receive 403 Forbidden
- Placeholder admin_id=999 in current implementation (TODO: extract from JWT)

**Password Security:**
- Passwords hashed with BCrypt (cost factor 12)
- Never logged or stored in plaintext
- Never included in responses
- Min 8 characters enforced

**Self-Modification Prevention:**
- Admins cannot delete their own account (prevents lockout)
- Admins cannot change their own role (prevents privilege loss)

**Soft Deletion:**
- User records never hard-deleted (preserve audit trail)
- Foreign keys use ON DELETE RESTRICT (preserve references)
- Deleted users marked with deleted_at timestamp

**Audit Trail:**
- All operations logged in user_audit_log table
- Audit logs immutable (append-only)
- Includes who, what, when, why for compliance

**Rate Limiting:**
- Standard API rate limits apply (see [002_rest_api_protocol.md](002_rest_api_protocol.md))

---

### RBAC Requirements

**ManageUsers Permission:**
- Required for all user management endpoints
- Granted to Admin role only
- Not granted to User or Viewer roles

**Role Hierarchy:**
```
Admin > User > Viewer
  |
  ├── ManageUsers (user management)
  ├── ManageTokens (token management)
  └── ManageConfig (system configuration)
```

**See:** [architecture/006_roles_and_permissions.md](../architecture/006_roles_and_permissions.md) for full RBAC design.

---

### Database Schema

**users table (enhanced in migration 005):**
```sql
CREATE TABLE users
(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  email TEXT,
  role TEXT NOT NULL DEFAULT 'user',
  is_active INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  last_login INTEGER,
  suspended_at INTEGER,
  suspended_by INTEGER REFERENCES users(id),
  deleted_at INTEGER,
  deleted_by INTEGER REFERENCES users(id),
  force_password_change INTEGER NOT NULL DEFAULT 0
);
```

**user_audit_log table (migration 006):**
```sql
CREATE TABLE user_audit_log
(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  operation TEXT NOT NULL CHECK (operation IN ('create', 'suspend', 'activate', 'delete', 'role_change', 'password_reset')),
  target_user_id INTEGER NOT NULL,
  performed_by INTEGER NOT NULL,
  timestamp INTEGER NOT NULL,
  previous_state TEXT,
  new_state TEXT,
  reason TEXT,
  FOREIGN KEY (target_user_id) REFERENCES users(id) ON DELETE RESTRICT,
  FOREIGN KEY (performed_by) REFERENCES users(id) ON DELETE RESTRICT
);
```

---

### CLI-API Parity

| API Endpoint | CLI Command | Notes |
|--------------|-------------|-------|
| `POST /api/v1/users` | `iron users create` | Create new user account |
| `GET /api/v1/users` | `iron users list` | List all users (with filters) |
| `GET /api/v1/users/:id` | `iron users get <id>` | Get user details |
| `PUT /api/v1/users/:id/suspend` | `iron users suspend <id>` | Suspend user account |
| `PUT /api/v1/users/:id/activate` | `iron users activate <id>` | Activate user account |
| `DELETE /api/v1/users/:id` | `iron users delete <id>` | Soft-delete user account |
| `PUT /api/v1/users/:id/role` | `iron users set-role <id> <role>` | Change user role |
| `POST /api/v1/users/:id/reset-password` | `iron users reset-password <id>` | Reset user password |

**CLI Requirements:**
- Confirmation prompts for destructive operations (delete, suspend)
- Colorized output for status (active=green, suspended=yellow, deleted=red)
- Table format for list command
- JSON output option (`--json` flag)

---

### Cross-References

**Resource Organization:**
- [architecture/009: Resource Catalog](../architecture/009_resource_catalog.md) - User management as resource

**Entities:**
- [architecture/007: Entity Model](../architecture/007_entity_model.md) - User entity definition

**Protocols:**
- [002: REST API Protocol](002_rest_api_protocol.md) - Overall API overview
- [007: Authentication API](007_authentication_api.md) - User authentication (login/logout)

**Permissions:**
- [architecture/006: Roles and Permissions](../architecture/006_roles_and_permissions.md) - ManageUsers permission

**Implementation:**
- Module: `module/iron_token_manager/src/user_service.rs` - User management business logic
- API: `module/iron_control_api/src/routes/users.rs` - User management endpoints
- Tests: `module/iron_control_api/tests/users/endpoints.rs` - Integration tests (40 tests)
- Migrations: `module/iron_token_manager/migrations/005_enhance_users_table.sql`, `006_create_user_audit_log.sql`

---

**Last Updated:** 2025-12-10
**Document Version:** 1.0
**API Version:** v1 (`/api/v1/`)
**Status:** ✅ Implemented (Phase 2 complete, Phase 4 CLI pending)
