# .agent

## .agent.list

List all agents.

```bash
iron .agent.list
iron .agent.list format::json v::2
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `v::` | integer (0-5) | Verbosity level | 1 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |
| `page::` | integer | Page number | 1 |
| `limit::` | integer | Results per page | 20 |

**Response**: Array of [agent_response](../parameter_types.md#agent_response)

**Example**:
```bash
iron .agent.list
# ┌──────────────┬────────────┬───────────┬──────────┬────────┐
# │ ID           │ Name       │ Providers │ Budget   │ Status │
# ├──────────────┼────────────┼───────────┼──────────┼────────┤
# │ abc123       │ my-agent   │ openai    │ $1.00    │ active │
# │ def456       │ prod-agent │ anthropic │ $10.00   │ active │
# └──────────────┴────────────┴───────────┴──────────┴────────┘

iron .agent.list format::json
```

---

## .agent.create

Create new agent.

```bash
iron .agent.create name::<name> providers::<list> provider_key_ids::<list> budget::<amount>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `name::` | string | Agent name (3-100 chars) | - |
| `providers::` | string | Comma-separated provider list | - |
| `provider_key_ids::` | string | Comma-separated provider key IDs (one per provider type) | - |
| `budget::` | integer | Initial budget in microdollars | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Created [agent_response](../parameter_types.md#agent_response)

**Example**:
```bash
iron .agent.create name::my-agent providers::openai provider_key_ids::1 budget::1000000
# Agent created
# ID: abc123
# Name: my-agent
# Providers: openai
# Budget: $1.00

# multiple provider keys (one per provider type)
iron .agent.create name::multi-agent providers::openai,anthropic provider_key_ids::1,2 budget::1000000

# dry run
iron .agent.create name::test-agent providers::openai provider_key_ids::1 budget::500000 dry::1
# Would create agent: test-agent (dry run)
```

---

## .agent.get

Get agent details by ID.

```bash
iron .agent.get id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [agent_id](../parameter_types.md#agent_id) | Agent ID (UUID) | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: [agent_response](../parameter_types.md#agent_response) with complete configuration

**Example**:
```bash
iron .agent.get id::abc123
# Agent: abc123
# ───────────────────────────────
# Name: my-agent
# Providers: openai
# Status: active
#
# Budget
# ┌────────────┬───────────┐
# │ Field      │ Value     │
# ├────────────┼───────────┤
# │ Total      │ $1.00     │
# │ Used       │ $0.25     │
# │ Remaining  │ $0.75     │
# └────────────┴───────────┘
#
# Created: 2024-01-15T10:30:00Z
# Updated: 2024-01-15T14:00:00Z
```

---

## .agent.update

Update agent.

```bash
iron .agent.update id::<id> name::<name>
iron .agent.update id::<id> budget::<amount>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [agent_id](../parameter_types.md#agent_id) | Agent ID | - |
| `name::` | string | New agent name | - |
| `budget::` | integer | New budget in microdollars | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Updated [agent_response](../parameter_types.md#agent_response)

**Example**:
```bash
iron .agent.update id::abc123 name::renamed-agent
# Agent updated: abc123
# Name: renamed-agent

iron .agent.update id::abc123 budget::2000000
# Agent updated: abc123
# Budget: $2.00
```

---

## .agent.delete

Delete agent.

```bash
iron .agent.delete id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [agent_id](../parameter_types.md#agent_id) | Agent ID | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: [success_response](../parameter_types.md#success_response)

**Example**:
```bash
iron .agent.delete id::abc123
# Agent deleted: abc123

iron .agent.delete id::abc123 dry::1
# Would delete agent: abc123 (dry run)
```

---

## .agent.assign_providers

Assign providers to agent.

```bash
iron .agent.assign_providers id::<id> provider_ids::<list>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [agent_id](../parameter_types.md#agent_id) | Agent ID | - |
| `provider_ids::` | string | Comma-separated provider IDs | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Updated agent with assigned providers

**Example**:
```bash
iron .agent.assign_providers id::abc123 provider_ids::prov1,prov2
# Providers assigned to agent: abc123
# Providers: prov1, prov2
```

---

## .agent.list_providers

List agent's providers.

```bash
iron .agent.list_providers id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [agent_id](../parameter_types.md#agent_id) | Agent ID | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Array of assigned provider objects

**Example**:
```bash
iron .agent.list_providers id::abc123
# ┌──────────┬──────────┬─────────────────┐
# │ ID       │ Provider │ Endpoint        │
# ├──────────┼──────────┼─────────────────┤
# │ prov1    │ openai   │ api.openai.com  │
# │ prov2    │ anthropic│ api.anthropic.com│
# └──────────┴──────────┴─────────────────┘
```

---

## .agent.remove_provider

Remove provider from agent.

```bash
iron .agent.remove_provider id::<id> provider_id::<provider_id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [agent_id](../parameter_types.md#agent_id) | Agent ID | - |
| `provider_id::` | string | Provider ID to remove | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Updated agent with provider removed

**Example**:
```bash
iron .agent.remove_provider id::abc123 provider_id::prov2
# Provider removed from agent: abc123
# Removed: prov2
```

