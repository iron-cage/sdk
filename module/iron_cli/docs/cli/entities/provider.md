# .provider

## .provider.list

List all providers.

```bash
iron .provider.list
iron .provider.list format::json
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `v::` | integer (0-5) | Verbosity level | 1 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Array of [provider_response](../parameter_types.md#provider_response)

**Example**:
```bash
iron .provider.list
# ┌──────────┬───────────┬─────────────────────┬─────────────┐
# │ ID       │ Provider  │ Endpoint            │ Description │
# ├──────────┼───────────┼─────────────────────┼─────────────┤
# │ prov1    │ openai    │ api.openai.com      │ Production  │
# │ prov2    │ anthropic │ api.anthropic.com   │ Claude API  │
# └──────────┴───────────┴─────────────────────┴─────────────┘

iron .provider.list format::json
```

---

## .provider.create

Create new provider.

```bash
iron .provider.create provider::<type> ip_token::<key>
iron .provider.create provider::<type> ip_token::<key> base_url::<url> description::<desc>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `provider::` | [provider_type](../parameter_types.md#provider_type) | Provider type (openai\|anthropic\|gemini\|xai) | - |
| `ip_token::` | string (sensitive) | Inference Provider token | - |
| `base_url::` | string | Custom API endpoint | - |
| `description::` | string | Provider description | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Created [provider_response](../parameter_types.md#provider_response)

**Example**:
```bash
iron .provider.create provider::openai ip_token::sk-xxx
# Provider created
# ID: prov_abc123
# Provider: openai
# Endpoint: api.openai.com (default)

iron .provider.create provider::anthropic ip_token::sk-ant-xxx description::"Claude production"

iron .provider.create provider::gemini ip_token::AIza-xxx description::"Gemini"

iron .provider.create provider::xai ip_token::xai-xxx description::"Grok"

# with custom endpoint
iron .provider.create provider::openai ip_token::sk-xxx base_url::https://custom.openai.azure.com
```

---

## .provider.get

Get provider details by ID.

```bash
iron .provider.get id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [provider_id](../parameter_types.md#provider_id) | Provider ID | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: [provider_response](../parameter_types.md#provider_response)

**Example**:
```bash
iron .provider.get id::prov_abc123
# Provider: prov_abc123
# ───────────────────────────────
# Type: openai
# Endpoint: api.openai.com
# Description: Production OpenAI
# Created: 2024-01-15T10:30:00Z
#
# Assigned Agents: 3
```

---

## .provider.update

Update provider.

```bash
iron .provider.update id::<id> ip_token::<key>
iron .provider.update id::<id> name::<name> endpoint::<url>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [provider_id](../parameter_types.md#provider_id) | Provider ID | - |
| `name::` | string | New name | - |
| `ip_token::` | string (sensitive) | New IP token | - |
| `endpoint::` | string | New endpoint | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Updated [provider_response](../parameter_types.md#provider_response)

**Example**:
```bash
iron .provider.update id::prov_abc123 ip_token::sk-new-xxx
# Provider updated: prov_abc123
# IP token updated

iron .provider.update id::prov_abc123 endpoint::https://new-endpoint.openai.com
# Provider updated: prov_abc123
# Endpoint: https://new-endpoint.openai.com
```

---

## .provider.delete

Delete provider.

```bash
iron .provider.delete id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [provider_id](../parameter_types.md#provider_id) | Provider ID | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: [success_response](../parameter_types.md#success_response)

**Example**:
```bash
iron .provider.delete id::prov_abc123
# Provider deleted: prov_abc123

iron .provider.delete id::prov_abc123 dry::1
# Would delete provider: prov_abc123 (dry run)
```

---

## .provider.assign_agents

Assign agents to provider.

```bash
iron .provider.assign_agents id::<id> agent_ids::<list>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [provider_id](../parameter_types.md#provider_id) | Provider ID | - |
| `agent_ids::` | string | Comma-separated agent IDs | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Updated provider with assigned agents

**Example**:
```bash
iron .provider.assign_agents id::prov_abc123 agent_ids::agent1,agent2
# Agents assigned to provider: prov_abc123
# Agents: agent1, agent2
```

---

## .provider.list_agents

List provider's agents.

```bash
iron .provider.list_agents id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [provider_id](../parameter_types.md#provider_id) | Provider ID | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Array of assigned agent objects

**Example**:
```bash
iron .provider.list_agents id::prov_abc123
# ┌──────────┬────────────┬────────┐
# │ ID       │ Name       │ Status │
# ├──────────┼────────────┼────────┤
# │ agent1   │ my-agent   │ active │
# │ agent2   │ prod-agent │ active │
# └──────────┴────────────┴────────┘
```

---

## .provider.remove_agent

Remove agent from provider.

```bash
iron .provider.remove_agent id::<id> agent_id::<agent_id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [provider_id](../parameter_types.md#provider_id) | Provider ID | - |
| `agent_id::` | string | Agent ID to remove | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Updated provider with agent removed

**Example**:
```bash
iron .provider.remove_agent id::prov_abc123 agent_id::agent2
# Agent removed from provider: prov_abc123
# Removed: agent2
```
