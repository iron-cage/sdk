# Parameter Types

## Overview

### Input Types

| Type | Kind | Description |
|------|------|-------------|
| [`token_id`](#token_id) | string | `tok_abc123` |
| [`agent_id`](#agent_id) | string | UUID format |
| [`provider_id`](#provider_id) | string | Provider identifier |
| [`user_id`](#user_id) | string | User identifier |
| [`project_id`](#project_id) | string | Project identifier |
| [`limit_id`](#limit_id) | integer | Numeric limit identifier |
| [`trace_id`](#trace_id) | integer | Numeric trace identifier |
| [`email`](#email) | string | `user@example.com` |
| [`scope`](#scope) | string | `action:resource` format |
| [`provider_type`](#provider_type) | enum | `openai \| anthropic \| gemini \| xai` |
| [`role`](#role) | enum | `admin \| user` |
| [`period`](#period) | enum | `day \| week \| month` |
| [`format_output`](#format_output) | enum | `json \| table \| yaml` |
| [`export_format`](#export_format) | enum | `json \| csv` |

### Response Types

| Type | Fields |
|------|--------|
| [`token_response`](#token_response) | `id`, `name`, `scope`, `created_at`, `expires_at`, `status` |
| [`agent_response`](#agent_response) | `id`, `name`, `providers`, `budget`, `budget_used`, `status` |
| [`provider_response`](#provider_response) | `id`, `provider`, `base_url`, `description`, `created_at` |
| [`user_response`](#user_response) | `id`, `username`, `email`, `role`, `created_at` |
| [`usage_response`](#usage_response) | `total_tokens`, `total_requests`, `total_cost`, `breakdown` |
| [`limit_response`](#limit_response) | `id`, `type`, `max_value`, `time_window`, `current_value` |
| [`trace_response`](#trace_response) | `id`, `token_id`, `provider`, `model`, `tokens`, `cost` |
| [`success_response`](#success_response) | `message` |
| [`error_response`](#error_response) | `error`, `code`, `details` |

---

## `token_id`

API token identifier with `tok_` prefix.

| Format | Example |
|--------|---------|
| `tok_*` | `tok_abc123xyz` |

```bash
token_id::tok_abc123xyz
```

---

## `agent_id`

Agent UUID identifier.

| Format | Example |
|--------|---------|
| UUID | `550e8400-e29b-41d4-a716-446655440000` |

```bash
id::550e8400-e29b-41d4-a716-446655440000
```

---

## `provider_id`

Provider identifier string.

```bash
provider_id::prov_openai_123
id::prov_openai_123
```

---

## `user_id`

User identifier string.

```bash
id::user_abc123
```

---

## `project_id`

Project identifier string.

```bash
project_id::proj_production
project::prod
```

---

## `limit_id`

Numeric limit identifier.

| Format | Example |
|--------|---------|
| integer | `789` |

```bash
limit_id::789
```

---

## `trace_id`

Numeric trace identifier.

| Format | Example |
|--------|---------|
| integer | `999` |

```bash
trace_id::999
```

---

## `email`

Email address format.

```bash
email::user@example.com
```

---

## `scope`

Token permission scope in `action:resource` format.

| Action | Resources | Example |
|--------|-----------|---------|
| `read` | tokens, usage, limits, traces | `read:tokens` |
| `write` | tokens, limits | `write:tokens` |
| `admin` | all | `admin:all` |

```bash
scope::read:tokens
scope::write:limits
scope::admin:all
```

---

## `provider_type`

LLM provider type.

| Value | Description |
|-------|-------------|
| `openai` | OpenAI API |
| `anthropic` | Anthropic Claude API |
| `gemini` | Google Gemini API |
| `xai` | xAI Grok API |

```bash
provider::openai
provider::anthropic
provider::gemini
provider::xai
```

---

## `role`

User role for access control.

| Value | Description |
|-------|-------------|
| `admin` | Full administrative access |
| `user` | Standard user access |

```bash
role::admin
role::user
```

---

## `period`

Time period for analytics.

| Value | Description |
|-------|-------------|
| `day` | Daily aggregation |
| `week` | Weekly aggregation |
| `month` | Monthly aggregation |

```bash
period::day
period::week
period::month
```

---

## `format_output`

Output format for CLI responses.

| Value | Description |
|-------|-------------|
| `json` | Machine-readable JSON |
| `table` | ASCII table (default) |
| `yaml` | YAML format |

```bash
format::json
format::table
format::yaml
```

---

## `export_format`

Data export format.

| Value | Description |
|-------|-------------|
| `json` | JSON export (default) |
| `csv` | CSV export |

```bash
export_format::json
export_format::csv
```

---

## `token_response`

Response from `.tokens.generate`, `.tokens.get`, `.tokens.list`, `.tokens.rotate`.

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Token ID (tok_xxx) |
| `name` | string | Token name |
| `scope` | string | Permission scope |
| `project_id` | string? | Associated project |
| `created_at` | ISO8601 | Creation timestamp |
| `expires_at` | ISO8601? | Expiration timestamp |
| `last_used_at` | ISO8601? | Last usage timestamp |
| `status` | string | `active` or `revoked` |

```bash
iron .tokens.get token_id::tok_abc123 format::json
# {
#   "id": "tok_abc123",
#   "name": "MyToken",
#   "scope": "read:tokens",
#   "project_id": null,
#   "created_at": "2024-01-15T10:30:00Z",
#   "expires_at": "2024-07-15T10:30:00Z",
#   "status": "active"
# }
```

---

## `agent_response`

Response from `.agent.create`, `.agent.get`, `.agent.list`, `.agent.update`.

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Agent UUID |
| `name` | string | Agent name |
| `providers` | string[] | Assigned providers |
| `budget` | integer | Budget in microdollars |
| `budget_used` | integer | Used budget |
| `budget_remaining` | integer | Remaining budget |
| `created_at` | ISO8601 | Creation timestamp |
| `updated_at` | ISO8601 | Last update timestamp |
| `status` | string | `active`, `paused`, or `deleted` |

```bash
iron .agent.get id::abc123 format::json
# {
#   "id": "abc123",
#   "name": "my-agent",
#   "providers": ["openai"],
#   "budget": 1000000,
#   "budget_used": 250000,
#   "budget_remaining": 750000,
#   "status": "active"
# }
```

---

## `provider_response`

Response from `.provider.create`, `.provider.get`, `.provider.list`.

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Provider ID |
| `provider` | string | Provider type (openai, anthropic) |
| `base_url` | string? | Custom API endpoint |
| `description` | string? | Provider description |
| `created_at` | ISO8601 | Creation timestamp |

---

## `user_response`

Response from `.user.create`, `.user.get`, `.user.list`.

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | User ID |
| `username` | string | Username |
| `email` | string | Email address |
| `role` | string | User role |
| `created_at` | ISO8601 | Creation timestamp |

---

## `usage_response`

Response from `.usage.show`, `.usage.by_project`, `.usage.by_provider`.

| Field | Type | Description |
|-------|------|-------------|
| `total_tokens` | integer | Total tokens consumed |
| `total_requests` | integer | Total API requests |
| `total_cost` | decimal | Total cost in dollars |
| `breakdown` | object | Usage by provider/project |
| `period` | object | Time range (from, to) |

```bash
iron .usage.show format::json
# {
#   "total_tokens": 1500000,
#   "total_requests": 450,
#   "total_cost": 15.50,
#   "breakdown": {
#     "by_provider": {"openai": {...}, "anthropic": {...}},
#     "by_project": {"prod": {...}}
#   }
# }
```

---

## `limit_response`

Response from `.limits.list`, `.limits.get`, `.limits.create`.

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | Limit ID |
| `type` | string | Limit type (tokens, requests, cost) |
| `max_value` | integer | Maximum allowed value |
| `time_window` | string | Time window (minute, day, month) |
| `current_value` | integer | Current usage |

---

## `trace_response`

Response from `.traces.list`, `.traces.get`.

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | Trace ID |
| `token_id` | string | Associated token |
| `provider` | string | Provider used |
| `model` | string | Model used |
| `prompt_tokens` | integer | Input tokens |
| `completion_tokens` | integer | Output tokens |
| `total_tokens` | integer | Total tokens |
| `cost` | decimal | Request cost |
| `timestamp` | ISO8601 | Request timestamp |

---

## `success_response`

Response for operations that don't return data.

| Field | Type | Description |
|-------|------|-------------|
| `message` | string | Human-readable status message |

```bash
iron .tokens.revoke token_id::tok_abc123
# {"message": "Token revoked: tok_abc123"}
```

---

## `error_response`

Error response structure.

| Field | Type | Description |
|-------|------|-------------|
| `error` | string | Error message |
| `code` | string | Error code |
| `details` | object? | Additional error details |

```bash
# {
#   "error": "Token not found",
#   "code": "not_found",
#   "details": {"token_id": "tok_invalid"}
# }
```

---

## Parameter Resolution

Parameters are resolved in order of priority:

| Priority | Source | Example |
|----------|--------|---------|
| 1 | Explicit value | `format::json` |
| 2 | Environment variable | `IRON_FORMAT` |
| 3 | Default value | `format::table` |
| 4 | Error | required param missing |

```bash
# explicit overrides env
export IRON_FORMAT=json
iron .tokens.list format::table  # uses "table"

# env used when explicit missing
iron .tokens.list  # uses IRON_FORMAT=json

# default used when both missing
unset IRON_FORMAT
iron .tokens.list  # format::table by default
```

---

## Environment Variables

Any parameter can be set via environment variable with `IRON_` prefix:

```
param::<value>  ->  IRON_<PARAM>=<value>
```

**Example**:
```bash
export IRON_FORMAT=json
export IRON_VERBOSITY=2

# all these use env values
iron .tokens.list
iron .usage.show
```
