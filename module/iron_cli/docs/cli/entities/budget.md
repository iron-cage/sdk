# .budget

## .budget.status

Get budget status across agents.

```bash
iron .budget.status
iron .budget.status threshold::80
iron .budget.status status::active
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `agent_id::` | integer | Filter by specific agent | - |
| `threshold::` | integer | Filter agents above % used | - |
| `status::` | string | Filter by status (active\|exhausted) | - |
| `page::` | integer | Page number | 1 |
| `per_page::` | integer | Results per page | 10 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Budget status array with usage, remaining amounts, risk levels

**Example**:
```bash
iron .budget.status
# ┌────────────┬──────────┬──────────┬───────────┬────────┬──────────┐
# │ Agent      │ Budget   │ Used     │ Remaining │ Usage  │ Risk     │
# ├────────────┼──────────┼──────────┼───────────┼────────┼──────────┤
# │ my-agent   │ $1.00    │ $0.85    │ $0.15     │ 85%    │ high     │
# │ prod-agent │ $10.00   │ $2.50    │ $7.50     │ 25%    │ low      │
# │ test-agent │ $0.50    │ $0.50    │ $0.00     │ 100%   │ exhausted│
# └────────────┴──────────┴──────────┴───────────┴────────┴──────────┘

# filter by threshold (agents above 80% usage)
iron .budget.status threshold::80
# Shows only: my-agent, test-agent

# filter by status
iron .budget.status status::exhausted
# Shows only: test-agent

iron .budget.status format::json
# [
#   {"agent": "my-agent", "budget": 1000000, "used": 850000, "remaining": 150000, "usage_pct": 85, "risk": "high"},
#   ...
# ]
```

---

# .budget_limit

## .budget_limit.get

Get global budget limit.

```bash
iron .budget_limit.get
iron .budget_limit.get format::json
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Global budget limit object

**Example**:
```bash
iron .budget_limit.get
# Global Budget Limit
# ───────────────────────────────
# Limit: $100.00
# Used: $52.50
# Remaining: $47.50
# Usage: 52.5%

iron .budget_limit.get format::json
# {
#   "limit": 100000000,
#   "used": 52500000,
#   "remaining": 47500000,
#   "usage_pct": 52.5
# }
```

---

## .budget_limit.set

Set global budget limit.

```bash
iron .budget_limit.set limit::<amount>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `limit::` | integer | New budget limit in microdollars | - |
| `dry::` | integer (0\|1) | Dry run flag | 0 |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Updated budget limit object

**Example**:
```bash
iron .budget_limit.set limit::200000000
# Global budget limit updated
# New limit: $200.00

iron .budget_limit.set limit::150000000 dry::1
# Would set global budget limit to: $150.00 (dry run)
```
