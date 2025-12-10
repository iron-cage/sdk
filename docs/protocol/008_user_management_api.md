# Protocol: User Management API

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

### Delete User (Soft Delete)

```http
DELETE /api/v1/users/1001
Authorization: Bearer <USER_TOKEN>

Response: 200 OK
{
  "id": 1001,
  "username": "john_doe",
  "email": "john.doe@example.com",
  "role": "user",
  "is_active": false,
  "created_at": 1733740800000,
  "last_login": 1733745000000,
  "suspended_at": null,
  "deleted_at": 1733755000000
}

Error: 500 Internal Server Error (Self-deletion)
{
  "error": "failed to delete user: Token management error"
}
```

**Side Effects:**
- `is_active` set to 0 (false)
- `deleted_at` set to current timestamp
- `deleted_by` set to admin user ID
- Audit log entry created
- User cannot authenticate
- Existing User Tokens remain valid (not automatically revoked)
- User record preserved (soft delete, not hard delete)
- Audit trail preserved (foreign keys with ON DELETE RESTRICT)

**Self-Deletion Prevention:**
- Admin cannot delete their own account
- Prevents accidental lockout

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
