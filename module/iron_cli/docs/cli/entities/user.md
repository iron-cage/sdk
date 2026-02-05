# .user

## .user.list

List all users.

```bash
iron .user.list
iron .user.list format::json
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Array of [user_response](../parameter_types.md#user_response)

**Example**:
```bash
iron .user.list
# ┌──────────┬──────────┬────────────────────┬────────┐
# │ ID       │ Username │ Email              │ Role   │
# ├──────────┼──────────┼────────────────────┼────────┤
# │ user_1   │ admin    │ admin@example.com  │ admin  │
# │ user_2   │ alice    │ alice@example.com  │ user   │
# │ user_3   │ bob      │ bob@example.com    │ user   │
# └──────────┴──────────┴────────────────────┴────────┘

iron .user.list format::json
```

---

## .user.create

Create new user.

```bash
iron .user.create username::<name> email::<email> password::<password>
iron .user.create username::<name> email::<email> password::<password> role::admin
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `username::` | string | Username (3-50 chars) | - |
| `email::` | [email](../parameter_types.md#email) | User email | - |
| `password::` | string (sensitive) | Initial password | - |
| `role::` | [role](../parameter_types.md#role) | User role (admin\|user) | `user` |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Created [user_response](../parameter_types.md#user_response)

**Example**:
```bash
iron .user.create username::alice email::alice@example.com password::secret
# User created
# ID: user_xyz
# Username: alice
# Email: alice@example.com
# Role: user

# create admin user
iron .user.create username::admin2 email::admin2@example.com password::secret role::admin
# User created with admin role

# interactive password prompt
iron .user.create username::bob email::bob@example.com
# Password: [hidden input]
# User created
```

---

## .user.get

Get user details by ID.

```bash
iron .user.get id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [user_id](../parameter_types.md#user_id) | User ID | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: [user_response](../parameter_types.md#user_response) with permissions

**Example**:
```bash
iron .user.get id::user_1
# User: user_1
# ───────────────────────────────
# Username: admin
# Email: admin@example.com
# Role: admin
# Created: 2024-01-01T00:00:00Z
#
# Permissions: all

iron .user.get id::user_1 format::json
```

---

## .user.update

Update user.

```bash
iron .user.update id::<id> email::<email>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [user_id](../parameter_types.md#user_id) | User ID | - |
| `email::` | [email](../parameter_types.md#email) | New email | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Updated [user_response](../parameter_types.md#user_response)

**Example**:
```bash
iron .user.update id::user_2 email::newalice@example.com
# User updated: user_2
# Email: newalice@example.com

iron .user.update id::user_2 email::test@example.com dry::1
# Would update user: user_2 (dry run)
```

---

## .user.delete

Delete user.

```bash
iron .user.delete id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [user_id](../parameter_types.md#user_id) | User ID | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: [success_response](../parameter_types.md#success_response)

**Example**:
```bash
iron .user.delete id::user_3
# User deleted: user_3

iron .user.delete id::user_3 dry::1
# Would delete user: user_3 (dry run)
```

---

## .user.set_role

Set user role.

```bash
iron .user.set_role id::<id> role::<role>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [user_id](../parameter_types.md#user_id) | User ID | - |
| `role::` | [role](../parameter_types.md#role) | New role (admin\|user) | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Updated [user_response](../parameter_types.md#user_response)

**Example**:
```bash
iron .user.set_role id::user_2 role::admin
# User role updated: user_2
# Role: admin

iron .user.set_role id::user_2 role::user
# User role updated: user_2
# Role: user (demoted from admin)
```

---

## .user.reset_password

Reset user password.

```bash
iron .user.reset_password id::<id> new_password::<password>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [user_id](../parameter_types.md#user_id) | User ID | - |
| `new_password::` | string (sensitive) | New password | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Password reset confirmation

**Example**:
```bash
iron .user.reset_password id::user_2 new_password::newsecret
# Password reset for user: user_2

# interactive password prompt
iron .user.reset_password id::user_2
# New password: [hidden input]
# Password reset for user: user_2
```

---

## .user.get_permissions

Get user permissions.

```bash
iron .user.get_permissions id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [user_id](../parameter_types.md#user_id) | User ID | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: User permissions array

**Example**:
```bash
iron .user.get_permissions id::user_1
# Permissions for user: user_1 (admin)
# ┌─────────────────────────┐
# │ Permission              │
# ├─────────────────────────┤
# │ agents:read             │
# │ agents:write            │
# │ agents:delete           │
# │ providers:read          │
# │ providers:write         │
# │ users:read              │
# │ users:write             │
# │ analytics:read          │
# └─────────────────────────┘

iron .user.get_permissions id::user_2
# Permissions for user: user_2 (user)
# ┌─────────────────────────┐
# │ Permission              │
# ├─────────────────────────┤
# │ agents:read             │
# │ providers:read          │
# │ analytics:read          │
# └─────────────────────────┘
```
