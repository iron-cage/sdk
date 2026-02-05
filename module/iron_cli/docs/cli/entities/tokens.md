# .tokens

Unified token management for all token types.

```bash
iron .tokens
iron .tokens type::<type>
iron .tokens id::<id>
iron .tokens generate type::<type> name::<name>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `type::` | enum | Token type: `api`, `ic` | - |
| `id::` | string | Token ID (for get/rotate/revoke) | - |
| `agent_id::` | string | Agent ID (required for type::ic) | - |
| `name::` | string | Token name | - |
| `scope::` | [scope](../parameter_types.md#scope) | Token scope (for type::api) | - |
| `description::` | string | Token description | - |
| `ttl::` | integer | Time-to-live in seconds | - |
| `generate::` | flag | Generate new token | - |
| `rotate::` | flag | Rotate existing token | - |
| `revoke::` | flag | Revoke token | - |
| `status::` | flag | Get token status (for IC tokens) | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Token types:**

| Type | Description |
|------|-------------|
| `api` | API tokens for service authentication |
| `ic` | IC tokens for agent authentication |

---

## List tokens

```bash
# list all tokens
iron .tokens
# ┌─────────────┬───────────┬──────────────┬────────┐
# │ ID          │ Name      │ Type         │ Status │
# ├─────────────┼───────────┼──────────────┼────────┤
# │ tok_abc123  │ MyToken   │ api          │ active │
# │ tok_def456  │ ProdKey   │ api          │ active │
# │ ic_xyz789   │ my-agent  │ ic           │ active │
# └─────────────┴───────────┴──────────────┴────────┘

# filter by type
iron .tokens type::api
iron .tokens type::ic
```

---

## Get token

```bash
iron .tokens id::tok_abc123
# ID: tok_abc123
# Name: MyToken
# Type: api
# Scope: read:tokens
# Created: 2024-01-15T10:30:00Z
# Expires: never
# Status: active

# IC token status
iron .tokens id::ic_xyz789 status
# ID: ic_xyz789
# Agent: my-agent
# Type: ic
# Created: 2024-01-15T10:30:00Z
# Last used: 2024-01-15T14:00:00Z
# Status: active
```

---

## Generate token

```bash
# API token
iron .tokens generate type::api name::MyToken scope::read:tokens
# Token created: tok_abc123
# Name: MyToken
# Type: api
# Scope: read:tokens
#
# WARNING: Token value shown only once. Store it securely.
# Token: sk_live_xxxxxxxxxxxxx

# API token with TTL
iron .tokens generate type::api name::TempToken scope::read:usage ttl::86400

# IC token for agent
iron .tokens generate type::ic agent_id::my-agent
# IC Token created for agent: my-agent
#
# WARNING: Token value shown only once. Store it securely.
# Token: ic_xxxxxxxxxxxxx
```

---

## Rotate token

Generate new value, revoke old.

```bash
iron .tokens id::tok_abc123 rotate
# Token rotated: tok_abc123
# Old token revoked
#
# WARNING: New token value shown only once. Store it securely.
# Token: sk_live_newxxxxxxxxxxxxx

# rotate with new TTL
iron .tokens id::tok_abc123 rotate ttl::604800

# rotate IC token
iron .tokens id::ic_xyz789 rotate
```

---

## Revoke token

```bash
iron .tokens id::tok_abc123 revoke
# Token revoked: tok_abc123

# revoke IC token
iron .tokens id::ic_xyz789 revoke
# IC Token revoked for agent: my-agent
```
