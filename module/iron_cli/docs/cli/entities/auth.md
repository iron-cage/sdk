# .auth

## .auth.login

Authenticate user and obtain JWT tokens.

```bash
iron .auth.login email::<email> password::<password>
iron .auth.login username::<username> password::<password>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `email::` | [email](../parameter_types.md#email) | Email for authentication | - |
| `username::` | string | Username (Control API) | - |
| `password::` | string (sensitive) | User password | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: JWT tokens stored in system keyring

**Example**:
```bash
# login with email (Token API)
iron .auth.login email::user@example.com password::secret
# Authentication successful
# Access token expires: 2024-01-15T11:30:00Z

# login with username (Control API)
iron .auth.login username::admin password::secret

# interactive password prompt
iron .auth.login email::user@example.com
# Password: [hidden input]

# JSON output
iron .auth.login email::user@example.com password::secret format::json
# {
#   "access_token_expires": "2024-01-15T11:30:00Z",
#   "message": "Authentication successful"
# }
```

---

## .auth.refresh

Refresh access token using stored refresh token.

```bash
iron .auth.refresh
iron .auth.refresh format::json
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Updated tokens in keyring

**Example**:
```bash
iron .auth.refresh
# Access token refreshed
# New expiry: 2024-01-15T12:30:00Z

iron .auth.refresh format::json
# {
#   "access_token_expires": "2024-01-15T12:30:00Z",
#   "message": "Token refreshed successfully"
# }
```

---

## .auth.logout

Invalidate tokens and clear keyring storage.

```bash
iron .auth.logout
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: [success_response](../parameter_types.md#success_response)

**Example**:
```bash
iron .auth.logout
# Logged out successfully
# Credentials cleared from keyring
```
