# Roles and Permissions

**Purpose:** Define the three access roles in Iron Cage platform.

---

## User Need

Understand who can do what in the system and how access is controlled.

## Core Idea

**Three roles with increasing Control Panel access:**

```
Admin (Full) > Super User (Read-Only Own) > Developer (Read-Only Own)
```

| Role | Control Panel Access | Can Allocate Budgets | Can See IP Tokens |
|------|---------------------|----------------------|-------------------|
| **Admin** | Full (CLI + Dashboard) | Yes | Yes |
| **Super User** | Read-only (CLI + Dashboard, own data) | No | No |
| **Developer** | Read-only (CLI + Dashboard, own usage) | No | No |

## Role 1: Admin

**Control Panel Access:** Full (create, read, update, delete) via CLI + Dashboard (equivalent interface)

**Responsibilities:**
- Allocate budgets to developers
- Create and manage developer accounts
- Monitor spending across ALL developers
- Manage IP Tokens (provider credentials in vault)
- Configure safety policies
- Revoke access, adjust limits

**Token Management:**
- Regenerate any IC Token (any agent, any developer)
- Regenerate any User Token (any user)
- Manage IP Tokens in vault (add, rotate, remove)

**Typical Users:** Engineering manager, Platform team, FinOps team, Security team

## Role 2: Super User

**Control Panel Access:** Read-only dashboard (own data only) via CLI + Dashboard (equivalent interface)

**Responsibilities:**
- Everything Developer can do +
- View own budgets in Control Panel dashboard
- Monitor own spending real-time (graphs, charts)
- See budget allocation details

**Token Management:**
- Regenerate own IC Tokens
- Regenerate own User Tokens
- Cannot regenerate other users' tokens

**Restrictions:**
- Cannot allocate budgets (admin only)
- Cannot manage IP Tokens (admin only)
- Cannot view other developers' data
- Cannot create accounts or modify limits

**Typical Users:** Team leads, Senior developers, Developers needing budget visibility

## Role 3: Developer

**Control Panel Access:** CLI + Dashboard (equivalent interface)

**Responsibilities:**
- Run agents locally with IC Token
- View own usage via CLI or Dashboard (equivalent interface)
- Select LLM model (among allowed list)
- Select IP/provider (among allowed list)
- Request budget increases (admin approval required)

**Token Management:**
- Regenerate own IC Tokens
- Regenerate own User Tokens
- Cannot regenerate other users' tokens

**Restrictions:**
- Dashboard is read-only (can view own usage, cannot allocate budgets)
- Cannot allocate budgets
- Cannot see IP Tokens
- Cannot view other developers' data

**Typical Users:** AI engineers, Data scientists, ML developers

## Permission Matrix

| Permission | Admin | Super User | Developer |
|------------|-------|------------|-----------|
| Control Panel Dashboard | Full (all data) | Read-only (own) | Read-only (own usage) |
| Allocate Budgets | Yes | No | No |
| View All Developers | Yes | No | No |
| Manage IP Tokens | Yes | No | No |
| Regenerate Own IC Token | Yes | Yes | Yes |
| Regenerate Any IC Token | Yes | No | No |
| Regenerate Own User Token | Yes | Yes | Yes |
| Regenerate Any User Token | Yes | No | No |
| Run Agents | Yes | Yes | Yes |
| View Own Usage | Yes | Yes | Yes (CLI+Dashboard) |
| Select Model | Yes | Yes | Yes (among allowed) |
| Select IP | Yes | Yes | Yes (among allowed) |
| Request Budget Increase | Yes | Yes | Yes |

## Role Assignment

**How roles are assigned:**
- Admin assigns roles when creating accounts
- Default: Developer (most restrictive)
- Super User: Granted by admin for visibility needs
- Admin: Platform team only

**Role Changes:**
- Admin can promote Developer → Super User
- Admin can demote Super User → Developer
- Only other admins can create new admins

---

*Related: [003_service_boundaries.md](003_service_boundaries.md) | [001_execution_models.md](001_execution_models.md) | [../deployment/002_actor_model.md](../deployment/002_actor_model.md)*
