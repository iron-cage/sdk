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


### Purpose

**User Need**: Administrators (managing team members with diverse access requirements), super users (overseeing organizational infrastructure), and system operators (maintaining platform security and compliance) need comprehensive user account management for Iron Cage Control Panel enabling initial user provisioning (create accounts for developers/admins/viewers with appropriate roles, set initial passwords following BCrypt hashing cost factor 12, assign email for identification), lifecycle state management (suspend accounts immediately for policy violations without deleting audit trail, activate suspended accounts restoring access, soft-delete departed users preserving historical data via deleted_at timestamp instead of hard deletion), role-based access control administration (change user roles between viewer/user/admin affecting permission grants, prevent self-modification of own role avoiding accidental privilege loss), emergency access recovery (admin-initiated password resets for locked-out users, optional force_change flag requiring password update on next login), operational visibility (list all users with filtering by role/active status/search query, paginate results with page/page_size parameters defaulting to 20 per page max 100, retrieve individual user details by numeric ID), and complete audit trail (immutable user_audit_log table recording every create/suspend/activate/delete/role_change/password_reset operation with who/what/when/why, JSON previous_state and new_state for updates, optional reason field for suspensions).

**Solution**: RESTful CRUD API with 8 Admin-only endpoints implementing user lifecycle management. Provide POST /api/v1/users (create user with username/password/email/role, validate email contains @, enforce password min 8 chars max 1000, hash with BCrypt cost 12, default is_active=1, return 201 Created with user object including numeric ID, audit log operation="create"), GET /api/v1/users (list with optional filters role/is_active/search, pagination via page/page_size query params, offset calculated as (page-1)*page_size, results ordered by created_at DESC newest first, return 200 OK with users array + total/page/page_size), GET /api/v1/users/{id} (retrieve single user by numeric ID, return 200 OK with full user object including last_login/suspended_at/deleted_at timestamps as null when empty, 404 Not Found if user doesn't exist), PUT /api/v1/users/{id}/suspend (set is_active=0 and suspended_at timestamp, require reason in request body, preserve existing User Tokens not auto-revoked, user cannot authenticate while suspended, audit log with reason), PUT /api/v1/users/{id}/activate (set is_active=1 and clear suspended_at/suspended_by to NULL, user can authenticate again, audit log operation="activate"), DELETE /api/v1/users/{id} (soft delete setting deleted_at timestamp and is_active=0, reassign all owned agents to admin in "Orphaned Agents" special project proj_orphaned, cancel pending budget requests with review_notes "Auto-cancelled: user deleted", revoke all API tokens setting revoked_at, return 200 OK with agents_affected array if agents_count > 0 showing agent_id/name/new_owner_id/new_project_id/budget/providers for each reassigned agent, prevent self-deletion with 500 error, prevent last admin deletion with 400 error, preserve User Tokens until natural expiration), PUT /api/v1/users/{id}/role (change role to viewer/user/admin, audit log with previous and new roles, existing User Tokens retain old role until reissued, prevent self-modification with 500 error), POST /api/v1/users/{id}/reset-password (admin sets new_password with min 8 chars validation, optional force_change boolean requiring password update on next login via force_password_change flag, hash with BCrypt cost 12, existing User Tokens remain valid not revoked, audit log operation="password_reset"). Authenticate all endpoints with User Token (JWT) in Authorization: Bearer header. Enforce Admin role with ManageUsers permission (403 Forbidden for non-admins). Implement soft deletion pattern (deleted_at timestamp, ON DELETE RESTRICT foreign keys, preserve audit trail). Adhere to ID Format Standards (numeric auto-incrementing user IDs), Data Format Standards (Unix epoch millisecond timestamps as integers, JSON booleans true/false, null for empty timestamp fields), Error Format Standards (machine-readable codes VALIDATION_ERROR/UNAUTHORIZED/FORBIDDEN/NOT_FOUND/DUPLICATE_EMAIL/DUPLICATE_USERNAME/SELF_MODIFICATION_FORBIDDEN/LAST_ADMIN_DELETION_FORBIDDEN, consistent error response structure, field-level validation in error.fields object), API Design Standards (pagination with page/page_size default 20 max 100, filtering via query params, URL structure /api/v1/users and /api/v1/users/{id}).

**Key Insight**: Soft deletion with agent reassignment (DELETE sets deleted_at timestamp, reassigns all owned agents to admin in special "Orphaned Agents" project proj_orphaned, cancels pending budget requests, revokes API tokens) prevents service disruption where agents continue running normally with existing IC Tokens and budgets valid ensuring no downtime for production workloads while preserving complete audit trail via deleted users retained in database with ON DELETE RESTRICT foreign keys. Dual prevention rules (self-modification forbidden, last admin deletion forbidden) protect system integrity where admin cannot delete own account preventing accidental lockout and cannot delete last active admin ensuring at least one admin exists always, both returning errors not silent failures. Existing User Tokens behavior during account changes (suspension sets is_active=0 blocking new logins but existing tokens remain valid until natural expiration, role change updates database immediately but existing tokens retain old role until reissued, password reset leaves existing sessions active not forcing logout) balances security (unauthorized users eventually locked out within token expiration window max 30 days) with user experience (legitimate admin operations don't disrupt active sessions mid-workflow). Audit log immutability (append-only user_audit_log table with operation/target_user_id/performed_by/timestamp/previous_state/new_state/reason, FOREIGN KEY ON DELETE RESTRICT preventing audit record deletion) ensures compliance traceability for security investigations, regulatory audits, and dispute resolution where every user management action has permanent record of who did what when and why. Variable response format for DELETE endpoint (returns agents_affected array with full agent details when agents_count > 0, returns minimal response when agents_count = 0) provides operational transparency showing exact reassignment outcomes for admins managing deleted users with production agents versus simple confirmation for users without agents.

---

**Status**: Specification
**Version**: 1.0.0
**Last Updated**: 2025-12-13
**Priority**: MUST-HAVE


### Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- User IDs use auto-incrementing integers (INTEGER PRIMARY KEY)
- `user_id`: Positive integer (e.g., `1001`, `1002`)

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Timestamps: Unix epoch milliseconds as integer (e.g., `1733740800000` for 2025-12-10T00:00:00Z)
- Booleans: JSON boolean `true`/`false` (not strings)
- Nulls: Include optional timestamp fields as `null` when empty (suspended_at, deleted_at, last_login), omit other optional fields
- Arrays: Empty array `[]` when no items (not `null`)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Consistent error response structure across all endpoints
- Machine-readable error codes: `VALIDATION_ERROR`, `UNAUTHORIZED`, `FORBIDDEN`, `NOT_FOUND`, `DUPLICATE_EMAIL`, `DUPLICATE_USERNAME`, `SELF_MODIFICATION_FORBIDDEN`, `LAST_ADMIN_DELETION_FORBIDDEN`
- HTTP status codes: 200, 201, 400, 401, 403, 404, 409, 500
- Field-level validation details in `error.fields` object

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- Pagination: Offset-based with `?page=N&page_size=M` (default 20 items/page, max 100)
- Filtering: Query parameters for `role`, `status`, `search`
- Sorting: Optional `?sort=created_at` or `?sort=-created_at`
- URL structure: `/api/v1/users`, `/api/v1/users/{id}`


### Protocol Definition

#### Create User

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

#### List Users

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

#### Get User by ID

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

#### Suspend User

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

#### Activate User

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

#### Delete User (Soft Delete + Agent Reassignment)

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

#### Change User Role

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

#### Reset User Password

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


#### Audit Logging

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


#### HTTP Status Codes

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | OK | Successful get, suspend, activate, delete, role change, password reset |
| 201 | Created | Successful user creation |
| 400 | Bad Request | Validation error (invalid email, weak password, invalid role) |
| 403 | Forbidden | Insufficient permissions (non-admin user) |
| 404 | Not Found | User not found by ID |
| 500 | Internal Server Error | Database error, duplicate username, self-modification |


#### Security Considerations

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


#### RBAC Requirements

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


#### Database Schema

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


#### CLI-API Parity

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


### Cross-References

#### Related Principles Documents
- [Principles: Design Philosophy](../principles/001_design_philosophy.md) - API-First Design principle reflected in RESTful user management API with standard CRUD operations, Separation of Concerns via distinct user management (this protocol) vs user authentication (Protocol 007)
- [Principles: Quality Attributes](../principles/002_quality_attributes.md) - Security via BCrypt password hashing cost factor 12, soft deletion preserving audit trail, RBAC with ManageUsers permission Admin-only, self-modification prevention; Maintainability via comprehensive audit logging (immutable user_audit_log table with operation/performed_by/timestamp/previous_state/new_state/reason)

#### Related Architecture Documents
- [Architecture: Resource Catalog](../architecture/009_resource_catalog.md) - User Management as Operation Resource in catalog, /api/users endpoints implementing Operation Resource pattern
- [Architecture: Entity Model](../architecture/007_entity_model.md) - User entity definition, numeric ID format (INTEGER PRIMARY KEY AUTOINCREMENT), relationships with agents (1:N), budget requests (1:N), API tokens (1:N), audit log (1:N)
- [Architecture: Roles and Permissions](../architecture/006_roles_and_permissions.md) - Admin/User/Viewer role definitions, ManageUsers permission granted to Admin role only, role hierarchy (Admin > User > Viewer)

#### Used By
- `iron_cli` - CLI tool calls these endpoints via `iron users create/list/get/suspend/activate/delete/set-role/reset-password` commands, requires User Token authentication, displays confirmation prompts for destructive operations (delete/suspend), colorized output for status (active=green, suspended=yellow, deleted=red), table format for list, optional --json flag
- `iron_dashboard` - Web UI calls these endpoints for user management interface, admin panel for creating/editing/suspending/deleting users, role assignment dropdown, password reset forms, user listing with filters (role/status/search), pagination controls
- System administrators - Direct API usage for automated user provisioning, bulk user imports via scripts, integration with HR systems, compliance reporting via audit log queries

#### Dependencies
- [Protocol: REST API Protocol](002_rest_api_protocol.md) - Overall API overview, standard rate limiting (applies to all user management endpoints), error response format standards, authentication pattern guidance
- [Protocol: Authentication API](007_authentication_api.md) - User Token (JWT) authentication required for all user management endpoints, login provides User Token with role claim, Admin role required for ManageUsers permission check
- [Protocol: Agents API](010_agents_api.md) - Agent reassignment on user deletion, agents moved to "Orphaned Agents" project (proj_orphaned) when owner deleted, IC Tokens remain valid ensuring no service disruption
- [Protocol: Budget Requests API](017_budget_requests_api.md) - Pending budget requests auto-cancelled when user deleted (status=cancelled, review_notes="Auto-cancelled: user deleted"), historical requests preserved with requester_id set to NULL
- [Protocol: API Tokens API](014_api_tokens_api.md) - User's API tokens revoked when user deleted (revoked_at timestamp set, revoked_by=admin_id), existing requests with revoked tokens fail with 401 Unauthorized
- [Standards: ID Format Standards](../standards/id_format_standards.md) - User ID format: numeric auto-incrementing integers (e.g., 1001, 1002) via INTEGER PRIMARY KEY AUTOINCREMENT
- [Standards: Data Format Standards](../standards/data_format_standards.md) - Unix epoch millisecond timestamps as integers (created_at, last_login, suspended_at, deleted_at), JSON boolean true/false for is_active, null for empty timestamp fields
- [Standards: Error Format Standards](../standards/error_format_standards.md) - Machine-readable error codes (VALIDATION_ERROR, UNAUTHORIZED, FORBIDDEN, NOT_FOUND, DUPLICATE_EMAIL, DUPLICATE_USERNAME, SELF_MODIFICATION_FORBIDDEN, LAST_ADMIN_DELETION_FORBIDDEN), consistent error response structure with error.fields for validation details
- [Standards: API Design Standards](../standards/api_design_standards.md) - Pagination with page/page_size query parameters (default 20, max 100), URL structure /api/v1/users and /api/v1/users/{id}, filtering via query params (role, is_active, search), POST for creation (201 Created), PUT for updates (200 OK), DELETE for soft deletion (200 OK)

#### Implementation
- Module: `module/iron_token_manager/src/user_service.rs` - User management business logic (create/suspend/activate/delete/role change/password reset), BCrypt hashing (cost factor 12), soft deletion with agent reassignment, audit log creation
- API: `module/iron_control_api/src/routes/users.rs` - 8 endpoint handlers (POST /users, GET /users, GET /users/{id}, PUT /users/{id}/suspend, PUT /users/{id}/activate, DELETE /users/{id}, PUT /users/{id}/role, POST /users/{id}/reset-password), ManageUsers permission enforcement, placeholder admin_id=999 extraction from JWT
- Tests: `module/iron_control_api/tests/users/endpoints.rs` - Integration tests covering 40 test cases (create success/validation errors, list with filters/pagination, get by ID/not found, suspend/activate, delete with agent reassignment/edge cases, role change/self-modification prevention, password reset/weak password)
- Migrations: `module/iron_token_manager/migrations/005_enhance_users_table.sql` (adds suspended_at, suspended_by, deleted_at, deleted_by, force_password_change columns), `006_create_user_audit_log.sql` (creates user_audit_log table with operation CHECK constraint)
- Specification: `module/iron_token_manager/spec.md` - Detailed implementation requirements for user service backend, database schema, audit logging strategy

