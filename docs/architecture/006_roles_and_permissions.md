# Roles and Permissions

**Purpose:** Define the three access roles in Iron Cage platform.
**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-10

---

## User Need

Understand who can do what in the system and how access is controlled.

## Core Idea

**Three roles with increasing Control Panel access:**

```
Admin (Full) > User (Standard Access) > Viewer (Read-Only)
```

| Role | Control Panel Access | Can Manage Users | Can Allocate Budgets | Can See IP Tokens |
|------|---------------------|------------------|----------------------|-------------------|
| **Admin** | Full (CLI + Dashboard) | Yes | Yes | Yes |
| **User** | Standard (CLI + Dashboard) | No | No | No |
| **Viewer** | Read-only (CLI + Dashboard) | No | No | No |

**Note:** The implementation uses lowercase role identifiers (`admin`, `user`, `viewer`) in the database and API, while this documentation uses capitalized names for clarity.

## Role 1: Admin

**Control Panel Access:** Full (create, read, update, delete) via CLI + Dashboard (equivalent interface)

**Responsibilities:**
- **User Management:** Create, suspend, activate, delete user accounts (with comprehensive audit logging)
- **Role Management:** Change user roles, reset passwords
- Allocate budgets to developers
- Monitor spending across ALL users
- Manage IP Tokens (provider credentials in vault)
- Configure safety policies
- Revoke access, adjust limits

**Token Management:**
- Regenerate any IC Token (any agent, any user)
- Regenerate any User Token (any user)
- Manage IP Tokens in vault (add, rotate, remove)

**User Management Permissions:**
- ManageUsers permission (admin-only)
- Create new user accounts with role assignment
- Suspend/activate user accounts (with optional reason)
- Soft delete users (preserves audit trail)
- Change user roles (viewer ↔ user ↔ admin)
- Reset user passwords (with force_change flag)
- Cannot modify own account (self-modification prevention)

**Audit Trail:**
- All admin actions are logged to `user_audit_log` table
- Tracks: action, user_id, admin_id, old_value, new_value, reason, timestamp
- Append-only log with ON DELETE RESTRICT constraint (audit integrity guaranteed)

**Typical Users:** Engineering manager, Platform team, FinOps team, Security team

## Role 2: User

**Control Panel Access:** Standard access via CLI + Dashboard (equivalent interface)

**Responsibilities:**
- Run agents locally with IC Token
- View own usage via CLI or Dashboard
- Monitor own spending real-time (graphs, charts)
- Select LLM model (among allowed list)
- Select IP/provider (among allowed list)
- Request budget increases (admin approval required)

**Token Management:**
- Regenerate own IC Tokens
- Regenerate own User Tokens
- Cannot regenerate other users' tokens

**Restrictions:**
- Cannot manage other users' accounts
- Cannot allocate budgets (admin only)
- Cannot manage IP Tokens (admin only)
- Cannot view other users' data
- Cannot create accounts or modify limits
- Cannot perform admin operations (user management, role changes)

**Typical Users:** AI engineers, Data scientists, ML developers, Team leads

## Role 3: Viewer

**Control Panel Access:** Read-only via CLI + Dashboard (equivalent interface)

**Responsibilities:**
- View own usage via CLI or Dashboard
- View budgets and spending (own data only)
- View token information (own tokens only)
- Monitor spending real-time (graphs, charts, own data)

**Token Management:**
- Regenerate own IC Tokens
- Regenerate own User Tokens
- Cannot regenerate other users' tokens

**Restrictions:**
- **Read-only:** Cannot create, modify, or delete any resources
- Cannot run agents or create new tokens
- Cannot allocate budgets
- Cannot see IP Tokens
- Cannot view other users' data
- Cannot perform admin operations (user management, role changes)
- Cannot select models or providers (view-only role)

**Typical Users:** Auditors, Compliance team, Read-only stakeholders, External consultants

## Permission Matrix

| Permission | Admin | User | Viewer |
|------------|-------|------|--------|
| **User Management** |
| Create User Accounts | Yes (ManageUsers) | No | No |
| Suspend/Activate Users | Yes (ManageUsers) | No | No |
| Delete Users | Yes (ManageUsers) | No | No |
| Change User Roles | Yes (ManageUsers) | No | No |
| Reset User Passwords | Yes (ManageUsers) | No | No |
| View User Audit Log | Yes | No | No |
| **Control Panel Access** |
| Control Panel Dashboard | Full (all data) | Standard (own) | Read-only (own) |
| Allocate Budgets | Yes | No | No |
| View All Users | Yes | No | No |
| Manage IP Tokens | Yes | No | No |
| **Token Operations** |
| Regenerate Own IC Token | Yes | Yes | Yes |
| Regenerate Any IC Token | Yes | No | No |
| Regenerate Own User Token | Yes | Yes | Yes |
| Regenerate Any User Token | Yes | No | No |
| **Agent Operations** |
| Run Agents | Yes | Yes | No |
| View Own Usage | Yes | Yes | Yes |
| Select Model | Yes | Yes (among allowed) | No (view only) |
| Select IP | Yes | Yes (among allowed) | No (view only) |
| Request Budget Increase | Yes | Yes | Yes (view budgets) |

## Role Assignment

**How roles are assigned:**
- Admin assigns roles when creating accounts via `iron users create --role <viewer|user|admin>`
- Default: User (standard access)
- Viewer: Granted by admin for read-only stakeholders
- Admin: Platform team only

**Role Changes:**
- Admin can change any user's role via `iron users change-role <user_id> <new_role>`
- Admin can promote: Viewer → User → Admin
- Admin can demote: Admin → User → Viewer
- Only other admins can create new admins
- All role changes are logged to `user_audit_log` with admin_id and timestamp
- Admins cannot change their own role (self-modification prevention)

**Account Lifecycle:**
- **Created:** User account is active (is_active=true, deleted_at=null)
- **Suspended:** Admin can suspend account (is_active=false), user cannot login
- **Activated:** Admin can reactivate suspended account (is_active=true)
- **Deleted:** Admin soft-deletes account (deleted_at=timestamp), preserves audit trail

---

*Related: [003_service_boundaries.md](003_service_boundaries.md) | [001_execution_models.md](001_execution_models.md) | [../deployment/002_actor_model.md](../deployment/002_actor_model.md)*
