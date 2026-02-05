# .limits

## .limits.list

List all configured limits.

```bash
iron .limits.list
iron .limits.list format::json
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Array of [limit_response](../parameter_types.md#limit_response)

**Example**:
```bash
iron .limits.list
# ┌────┬──────────────┬───────────┬─────────────┬─────────┐
# │ ID │ Type         │ Max Value │ Time Window │ Current │
# ├────┼──────────────┼───────────┼─────────────┼─────────┤
# │ 1  │ tokens/day   │ 1,000,000 │ day         │ 250,000 │
# │ 2  │ requests/min │ 100       │ minute      │ 15      │
# │ 3  │ cost/month   │ 10000     │ month       │ 1550    │
# └────┴──────────────┴───────────┴─────────────┴─────────┘

iron .limits.list format::json
# [
#   {"id": 1, "type": "tokens/day", "max_value": 1000000, "time_window": "day", "current_value": 250000},
#   {"id": 2, "type": "requests/min", "max_value": 100, "time_window": "minute", "current_value": 15},
#   {"id": 3, "type": "cost/month", "max_value": 10000, "time_window": "month", "current_value": 1550}
# ]
```

---

## .limits.get

Get specific limit details by ID.

```bash
iron .limits.get limit_id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `limit_id::` | [limit_id](../parameter_types.md#limit_id) | Numeric limit identifier | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: [limit_response](../parameter_types.md#limit_response)

**Example**:
```bash
iron .limits.get limit_id::1
# Limit ID: 1
# Type: tokens/day
# Max Value: 1,000,000
# Time Window: day
# Current Value: 250,000
# Usage: 25%
# Resets: 2024-01-16T00:00:00Z

iron .limits.get limit_id::1 format::json
# {
#   "id": 1,
#   "type": "tokens/day",
#   "max_value": 1000000,
#   "time_window": "day",
#   "current_value": 250000,
#   "reset_at": "2024-01-16T00:00:00Z"
# }
```

---

## .limits.create

Create new usage limit(s).

```bash
iron .limits.create max_tokens::<n>
iron .limits.create max_requests::<n>
iron .limits.create max_cost::<n>
iron .limits.create project::<id> max_tokens::<n>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `project::` | string | Project ID | - |
| `max_tokens::` | integer | Max tokens per day | - |
| `max_requests::` | integer | Max requests per minute | - |
| `max_cost::` | integer | Max cost per month (cents) | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Created [limit_response](../parameter_types.md#limit_response)

**Example**:
```bash
# create token limit
iron .limits.create max_tokens::1000000
# Limit created
# ID: 4
# Type: tokens/day
# Max Value: 1,000,000

# create request rate limit
iron .limits.create max_requests::100
# Limit created
# ID: 5
# Type: requests/minute
# Max Value: 100

# create cost limit (in cents)
iron .limits.create max_cost::5000
# Limit created
# ID: 6
# Type: cost/month
# Max Value: $50.00

# create limit for specific project
iron .limits.create project::prod max_tokens::500000
# Limit created for project: prod
```

---

## .limits.update

Update existing limit.

```bash
iron .limits.update limit_id::<id> max_tokens::<n>
iron .limits.update limit_id::<id> max_requests::<n>
iron .limits.update limit_id::<id> max_cost::<n>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `limit_id::` | [limit_id](../parameter_types.md#limit_id) | Numeric limit identifier | - |
| `max_tokens::` | integer | New max tokens | - |
| `max_requests::` | integer | New max requests | - |
| `max_cost::` | integer | New max cost | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Updated [limit_response](../parameter_types.md#limit_response)

**Example**:
```bash
iron .limits.update limit_id::1 max_tokens::2000000
# Limit updated
# ID: 1
# Type: tokens/day
# Max Value: 2,000,000 (was: 1,000,000)

iron .limits.update limit_id::2 max_requests::200 format::json
# {
#   "id": 2,
#   "type": "requests/minute",
#   "max_value": 200,
#   "previous_max_value": 100
# }
```

---

## .limits.delete

Delete limit.

```bash
iron .limits.delete limit_id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `limit_id::` | [limit_id](../parameter_types.md#limit_id) | Numeric limit identifier | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: [success_response](../parameter_types.md#success_response)

**Example**:
```bash
iron .limits.delete limit_id::4
# Limit deleted: 4
```
