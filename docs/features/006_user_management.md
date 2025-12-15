# User Management - Feature Architecture

**Version:** 1.0.0
**Date:** 2025-12-10
**Status:** Implemented
**Related Protocol:** [docs/protocol/008_user_management_api.md](../protocol/008_user_management_api.md)

---

### Scope

**Responsibility:** Complete architecture and lifecycle documentation for admin-level user account management.

**In Scope:**
- User account lifecycle (creation → suspension → deletion)
- Role-based access control (viewer/user/admin)
- User audit logging (comprehensive operation tracking)
- Admin-only API endpoints (8 REST endpoints)
- CLI command handlers (8 pure validation functions)
- Password management (BCrypt hashing, admin reset)
- Self-modification prevention (admins can't delete own account)

**Out of Scope:**
- User self-service (password change, profile edit)
- User registration flow (admin creates all accounts)
- Email verification (not required for pilot)
- Multi-factor authentication (future enhancement)
- Session management (handled by JWT authentication)

---

## 1. Overview

User management provides admin-level control over user accounts within Iron Cage. This feature enables administrators to:
- Create user accounts with assigned roles
- Monitor user activity through audit logs
- Suspend/activate users for policy enforcement
- Soft-delete users while preserving audit history
- Change user roles (viewer → user → admin)
- Reset user passwords (admin-initiated)

**Key Design Principles:**
- **Admin-only access:** All user management operations require Admin role
- **Audit everything:** Every operation creates audit log entry
- **Soft delete:** User deletion preserves audit trail
- **Self-protection:** Admins cannot modify their own accounts
- **RBAC enforcement:** Permission checks on all endpoints

---

## 2. User Lifecycle

### 2.1 Lifecycle States

```
┌─────────────────────────────────────────────────────────────┐
│                      USER LIFECYCLE                         │
└─────────────────────────────────────────────────────────────┘

  [Admin Creates User]
         │
         ▼
   ┌──────────┐
   │ CREATED  │ (is_active=true, deleted_at=null)
   │  Active  │ ← User can login, perform actions
   └────┬─────┘
        │
        │ [Admin Suspends] ───────────────┐
        ▼                                  │
   ┌──────────┐                            │
   │SUSPENDED │ (is_active=false)          │
   │ Inactive │ ← User cannot login        │
   └────┬─────┘                            │
        │                                  │
        │ [Admin Activates] ───────────────┘
        │
        │ [Admin Deletes]
        ▼
   ┌──────────┐
   │ DELETED  │ (deleted_at=timestamp)
   │ Archived │ ← Soft delete, audit preserved
   └──────────┘
```

### 2.2 State Transitions

**1. Creation → Active**
- **Operation:** `POST /api/v1/users`
- **Preconditions:** None
- **Postconditions:** User active, can login
- **Audit Log:** action="create"

**2. Active → Suspended**
- **Operation:** `PUT /api/v1/users/{id}/suspend`
- **Preconditions:** User is active
- **Postconditions:** User cannot login, is_active=false
- **Audit Log:** action="suspend", reason captured

**3. Suspended → Active**
- **Operation:** `PUT /api/v1/users/{id}/activate`
- **Preconditions:** User is suspended
- **Postconditions:** User can login, is_active=true
- **Audit Log:** action="activate"

**4. Any → Deleted (+ Agent Reassignment)**
- **Operation:** `DELETE /api/v1/users/{id}`
- **Preconditions:** User exists, not self, not last admin
- **Postconditions:**
  - User cannot login, deleted_at set
  - All owned agents reassigned to admin
  - All agents moved to "Orphaned Agents" project (proj-orphaned)
  - Pending budget requests auto-cancelled
  - All API tokens revoked
  - Agents continue working (IC Tokens valid, budgets active)
- **Audit Log:** action="delete" with agents_affected, budget_requests_cancelled, api_tokens_revoked
- **Response:** Includes agents_affected array, agents_count, budget_requests_cancelled, api_tokens_revoked

**5. Role Change**
- **Operation:** `PUT /api/v1/users/{id}/role`
- **Preconditions:** User exists, not self
- **Postconditions:** User role changed
- **Audit Log:** action="change_role", old_value + new_value

**6. Password Reset**
- **Operation:** `PUT /api/v1/users/{id}/password`
- **Preconditions:** User exists
- **Postconditions:** Password hash updated, force_change flag set
- **Audit Log:** action="reset_password"

### 2.3 Lifecycle Constraints

**Self-Modification Prevention:**
- Admins cannot suspend their own account
- Admins cannot delete their own account
- Admins cannot change their own role
- Admins CAN reset their own password

**Soft Delete Behavior:**
- User record remains in database
- deleted_at timestamp set
- User cannot login
- Audit logs preserved
- No cascade deletion (preserves foreign key references)

**Password Security:**
- BCrypt cost factor 12
- Min length: 8 characters
- Max length: 1000 characters
- No plaintext storage
- Admin can reset, user must change on next login

---

## 3. Database Schema

### 3.1 Users Table

```sql
CREATE TABLE users
(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  email TEXT UNIQUE NOT NULL,
  role TEXT NOT NULL CHECK(role IN ('viewer', 'user', 'admin')),
  is_active BOOLEAN NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER,
  deleted_at INTEGER,
  suspended_at INTEGER,
  suspended_by INTEGER REFERENCES users(id),
  deleted_by INTEGER REFERENCES users(id),
  force_password_change BOOLEAN NOT NULL DEFAULT 0
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_deleted_at ON users(deleted_at);
```

**Fields:**
- `id` - Primary key
- `username` - Unique identifier (max 255 chars)
- `password_hash` - BCrypt hash (never plaintext)
- `email` - Unique email (max 255 chars)
- `role` - viewer, user, or admin
- `is_active` - true=active, false=suspended
- `created_at` - Unix timestamp (milliseconds)
- `updated_at` - Last update timestamp
- `deleted_at` - Soft delete timestamp
- `suspended_at` - Suspension timestamp
- `suspended_by` - Admin user ID who suspended
- `deleted_by` - Admin user ID who deleted
- `force_password_change` - Force password change on next login

### 3.2 User Audit Log Table

```sql
CREATE TABLE user_audit_log
(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  admin_id INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  action TEXT NOT NULL,
  old_value TEXT,
  new_value TEXT,
  reason TEXT,
  created_at INTEGER NOT NULL
);

CREATE INDEX idx_user_audit_log_user_id ON user_audit_log(user_id);
CREATE INDEX idx_user_audit_log_admin_id ON user_audit_log(admin_id);
CREATE INDEX idx_user_audit_log_action ON user_audit_log(action);
CREATE INDEX idx_user_audit_log_created_at ON user_audit_log(created_at);
```

**Fields:**
- `id` - Primary key
- `user_id` - Target user ID (ON DELETE RESTRICT)
- `admin_id` - Admin who performed action (ON DELETE RESTRICT)
- `action` - create, suspend, activate, delete, change_role, reset_password
- `old_value` - JSON serialized old state
- `new_value` - JSON serialized new state
- `reason` - Optional reason text
- `created_at` - Unix timestamp (milliseconds)

**Foreign Key Constraints:**
- `ON DELETE RESTRICT` ensures audit log is never lost
- Cannot delete user if they have audit log entries
- Cannot delete admin if they performed audited actions

---

## 4. API Endpoints

### 4.1 Complete Endpoint List

| Endpoint | Method | Purpose | RBAC |
|----------|--------|---------|------|
| `/api/v1/users` | POST | Create user | Admin |
| `/api/v1/users` | GET | List users (with filters) | Admin |
| `/api/v1/users/{id}` | GET | Get user details | Admin |
| `/api/v1/users/{id}/suspend` | PUT | Suspend user | Admin |
| `/api/v1/users/{id}/activate` | PUT | Activate user | Admin |
| `/api/v1/users/{id}` | DELETE | Soft delete user | Admin |
| `/api/v1/users/{id}/role` | PUT | Change user role | Admin |
| `/api/v1/users/{id}/password` | PUT | Reset user password | Admin |

**Authentication:** All endpoints require valid User Token (JWT)
**Authorization:** All endpoints require Admin role (ManageUsers permission)

### 4.2 Request/Response Examples

**Create User:**
```http
POST /api/v1/users
Authorization: Bearer <USER_TOKEN>
Content-Type: application/json

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
```

**List Users:**
```http
GET /api/v1/users?role=admin&is_active=true&page=1&page_size=20
Authorization: Bearer <USER_TOKEN>

Response: 200 OK
{
  "users": [
    {
      "id": 1,
      "username": "admin",
      "email": "admin@example.com",
      "role": "admin",
      "is_active": true,
      "created_at": 1733740800000
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20
}
```

**Suspend User:**
```http
PUT /api/v1/users/1001/suspend
Authorization: Bearer <USER_TOKEN>
Content-Type: application/json

{
  "reason": "Policy violation - multiple failed login attempts"
}

Response: 200 OK
{
  "id": 1001,
  "username": "john_doe",
  "is_active": false,
  "suspended_at": 1733740900000
}
```

For complete API specification, see [docs/protocol/008_user_management_api.md](../protocol/008_user_management_api.md).

---

## 5. CLI Commands

### 5.1 Command Structure

```bash
iron users create --username <name> --password <pass> --email <email> --role <role>
iron users list [--role <role>] [--is-active <bool>] [--search <term>]
iron users get <user_id>
iron users suspend <user_id> [--reason <text>]
iron users activate <user_id>
iron users delete <user_id>
iron users change-role <user_id> --role <role>
iron users reset-password <user_id> --new-password <pass> [--force-change]
```

### 5.2 Handler Architecture

**Pattern:** Pure functions for validation only
**Signature:** `HashMap<String, String> → Result<String, CliError>`
**Location:** `module/iron_cli/src/handlers/user_handlers.rs`

**Handler Functions:**
- `create_user_handler` - Validates username, password, email, role
- `list_users_handler` - Validates filters, pagination
- `get_user_handler` - Validates user_id
- `suspend_user_handler` - Validates user_id, reason
- `activate_user_handler` - Validates user_id
- `delete_user_handler` - Validates user_id
- `change_user_role_handler` - Validates user_id, role
- `reset_password_handler` - Validates user_id, password, force_change

**No I/O:** Handlers perform validation only, no database or network calls
**No Async:** All handlers are synchronous functions
**Adapter Layer:** Actual API calls handled by separate adapter layer

---

## 6. Role-Based Access Control

### 6.1 Roles and Permissions

**Three Roles:**
1. **Viewer** - Read-only access (no user management)
2. **User** - Standard access (no user management)
3. **Admin** - Full access (all user management operations)

**Permission:** `ManageUsers`
- Required for all user management operations
- Only granted to Admin role
- Checked at API layer (middleware)
- Checked at service layer (business logic)

### 6.2 RBAC Enforcement

**API Layer (Middleware):**
```rust
async fn require_permission(
  user_context: UserContext,
  required: Permission,
) -> Result<(), StatusCode>
{
  if !user_context.has_permission(required)
  {
    return Err(StatusCode::FORBIDDEN);
  }
  Ok(())
}
```

**Service Layer (Business Logic):**
```rust
pub async fn create_user(
  &self,
  username: &str,
  password: &str,
  email: &str,
  role: &str,
  admin_id: i64,
) -> Result<User>
{
  // Business logic + audit logging
}
```

**Self-Modification Check:**
```rust
if user_id == admin_id
{
  return Err(TokenError::SelfModificationNotAllowed);
}
```

---

## 7. Audit Logging

### 7.1 Audit Log Format

**Every operation creates audit log entry with:**
- `user_id` - Target user
- `admin_id` - Admin who performed action
- `action` - Operation type
- `old_value` - JSON serialized old state (if applicable)
- `new_value` - JSON serialized new state (if applicable)
- `reason` - Optional reason text
- `created_at` - Timestamp

**Example Audit Entry:**
```json
{
  "id": 42,
  "user_id": 1001,
  "admin_id": 1,
  "action": "suspend",
  "old_value": "{\"is_active\": true}",
  "new_value": "{\"is_active\": false}",
  "reason": "Policy violation",
  "created_at": 1733740900000
}
```

### 7.2 Audited Actions

| Action | Captures |
|--------|----------|
| `create` | Full user object created |
| `suspend` | is_active: true → false, suspended_at, suspended_by |
| `activate` | is_active: false → true |
| `delete` | deleted_at, deleted_by |
| `change_role` | old_role → new_role |
| `reset_password` | Password hash changed (NOT the password itself) |

**Privacy:** Passwords never logged (only indication that password was changed)

### 7.3 Audit Log Queries

**Get all actions for user:**
```sql
SELECT * FROM user_audit_log
WHERE user_id = ?
ORDER BY created_at DESC;
```

**Get all actions by admin:**
```sql
SELECT * FROM user_audit_log
WHERE admin_id = ?
ORDER BY created_at DESC;
```

**Get specific action types:**
```sql
SELECT * FROM user_audit_log
WHERE action IN ('suspend', 'delete')
ORDER BY created_at DESC;
```

---

## 8. Security Considerations

### 8.1 Password Security

**BCrypt Hashing:**
- Cost factor: 12 (2^12 rounds)
- Salt automatically generated per password
- Hash length: 60 characters
- Never store plaintext passwords

**Password Requirements:**
- Min length: 8 characters
- Max length: 1000 characters
- No complexity requirements (rely on length)
- Admin can reset to temporary password
- Force password change on next login

### 8.2 Authentication Flow

**User Login:**
1. User submits username + password
2. System retrieves user by username
3. System verifies password with BCrypt
4. System checks is_active flag
5. System checks deleted_at is null
6. System generates JWT token
7. User authenticated

**Admin Operations:**
1. Admin submits request with JWT token
2. System verifies JWT signature
3. System extracts user_id from token
4. System loads user context (role, permissions)
5. System checks ManageUsers permission
6. System checks self-modification rules
7. System executes operation + audit log

### 8.3 SQL Injection Prevention

**SQLx Prepared Statements:**
```rust
sqlx::query("UPDATE users SET is_active = ? WHERE id = ?")
  .bind(false)
  .bind(user_id)
  .execute(&self.pool)
  .await?
```

All queries use parameter binding, never string concatenation.

### 8.4 Self-Protection Rules

**Admins CANNOT:**
- Suspend their own account
- Delete their own account
- Change their own role to non-admin

**Admins CAN:**
- Reset their own password
- View their own audit log
- List themselves in user list

**Enforcement:** Checked at service layer before database operations

---

## 9. Testing

### 9.1 Test Coverage

**API Endpoint Tests:** 40 tests
- Create user (valid, invalid username, invalid email, etc.)
- List users (filters, pagination, search)
- Get user (valid, not found)
- Suspend user (valid, already suspended, not found)
- Activate user (valid, already active, not found)
- Delete user (valid, self-deletion prevention, not found)
- Change role (valid, invalid role, self-modification)
- Reset password (valid, invalid password, force_change)

**CLI Handler Tests:** 27 tests
- Parameter validation (missing, empty, invalid format)
- Role validation (viewer, user, admin)
- Boolean validation (true, false)
- Integer validation (negative, non-numeric)
- Boundary conditions (max length, min length)

**Integration Tests:** 39 additional tests
- Database schema validation
- Foreign key constraints
- Audit log creation
- Self-modification prevention
- RBAC enforcement

**Total:** 106 tests across all layers

### 9.2 Test Strategy

**Unit Tests:** Pure logic testing (handlers, validation)
**Integration Tests:** Database + API layer together
**No Mocking:** Use real implementations (test database)
**Loud Failures:** Tests fail explicitly with clear error messages
**No Disabled Tests:** All tests enabled and passing

---

## 10. Implementation Status

### 10.1 Completed Components

✅ **Phase 1:** Database migrations (migrations 005, 006)
✅ **Phase 2:** API implementation (8 REST endpoints with 40 tests)
✅ **Phase 3:** Specifications (protocol doc + module specs updated)
✅ **Phase 4:** CLI handlers (8 handlers with 27 tests)

### 10.2 Pending Components

⏳ **Phase 5:** Documentation (resource catalog, lifecycle docs, CLI reference)
⏳ **Phase 6:** Security audit (rate limiting, validation, authorization, audit logging)

### 10.3 Test Results

```
✅ iron_cli: All 299 tests pass
✅ iron_control_api: All 445 tests pass (40 user management)
✅ iron_token_manager: All 79 tests pass
```

---

## 11. Future Enhancements

### 11.1 Potential Improvements

**User Self-Service:**
- User profile editing
- Self-service password change
- Email verification

**Enhanced Security:**
- Multi-factor authentication (TOTP)
- Password complexity requirements
- Account lockout after failed attempts
- Password expiration policies

**Advanced Features:**
- User groups/teams
- Granular permissions (beyond viewer/user/admin)
- Bulk user operations
- CSV import/export
- User activity dashboard

**Audit Enhancements:**
- Audit log retention policies
- Audit log export (CSV, JSON)
- Real-time audit alerts
- Compliance reporting

### 11.2 Deferred to Post-Pilot

All enhancements deferred to post-pilot based on user feedback and business requirements.

---

*Related: [protocol/008_user_management_api.md](../protocol/008_user_management_api.md) (API spec) | [architecture/009_resource_catalog.md](../architecture/009_resource_catalog.md) (resource catalog) | [architecture/006_roles_and_permissions.md](../architecture/006_roles_and_permissions.md) (RBAC model)*

**Last Updated:** 2025-12-10
