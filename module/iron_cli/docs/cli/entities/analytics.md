# .analytics

Unified command for usage metrics, spending, and request traces.

```bash
iron .analytics
iron .analytics type::<type>
iron .analytics group_by::<dimension>
iron .analytics agent_id::<id> provider_id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `type::` | enum | Data type: `metrics`, `usage`, `spending`, `traces` | `metrics` |
| `id::` | integer | Get specific trace by ID (when type::traces) | - |
| `group_by::` | enum | Group results: `agent`, `provider`, `period` | - |
| `agent_id::` | string | Filter by agent ID | - |
| `provider_id::` | string | Filter by provider ID | - |
| `project_id::` | string | Filter by project ID | - |
| `period::` | enum | Time period: `day`, `week`, `month` | - |
| `limit::` | integer | Max results (for traces) | 20 |
| `export::` | string | Export to file path | - |
| `export_format::` | enum | Export format: `json`, `csv`, `yaml` | `json` |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

---

## Metrics (default)

Combined usage and spending statistics.

```bash
# all metrics
iron .analytics
# Analytics
# ───────────────────────────────
# Total Tokens: 5,000,000
# Total Requests: 1,200
# Total Cost: $50.00

# only usage
iron .analytics type::usage

# only spending
iron .analytics type::spending
```

---

## Grouping

Group results by dimension.

```bash
# by agent
iron .analytics group_by::agent
# ┌────────────┬──────────┬──────────┬────────┐
# │ Agent      │ Tokens   │ Requests │ Cost   │
# ├────────────┼──────────┼──────────┼────────┤
# │ my-agent   │ 3,000,000│ 800      │ $30.00 │
# │ prod-agent │ 2,000,000│ 400      │ $20.00 │
# └────────────┴──────────┴──────────┴────────┘

# by provider
iron .analytics group_by::provider
# ┌───────────┬──────────┬──────────┬────────┐
# │ Provider  │ Tokens   │ Requests │ Cost   │
# ├───────────┼──────────┼──────────┼────────┤
# │ openai    │ 4,000,000│ 1,000    │ $40.00 │
# │ anthropic │ 1,000,000│ 200      │ $10.00 │
# └───────────┴──────────┴──────────┴────────┘

# by period
iron .analytics group_by::period period::month
# ┌────────────┬──────────┬────────┐
# │ Period     │ Tokens   │ Cost   │
# ├────────────┼──────────┼────────┤
# │ 2024-01    │ 3,500,000│ $35.00 │
# │ 2023-12    │ 1,500,000│ $15.00 │
# └────────────┴──────────┴────────┘
```

---

## Filtering

Filter by agent, provider, project, or combine filters.

```bash
# specific agent
iron .analytics agent_id::my-agent

# specific provider
iron .analytics provider_id::openai

# specific project
iron .analytics project_id::prod

# combine: agent + provider
iron .analytics agent_id::my-agent provider_id::openai

# combine: agent + provider + period
iron .analytics agent_id::my-agent provider_id::openai period::week

# group + filter: agents using openai
iron .analytics group_by::agent provider_id::openai
```

---

## Traces

Request logs showing individual API calls.

```bash
# list recent traces
iron .analytics type::traces
# ┌─────┬─────────────┬───────────┬─────────────────┬────────┬────────┐
# │ ID  │ Token       │ Provider  │ Model           │ Tokens │ Cost   │
# ├─────┼─────────────┼───────────┼─────────────────┼────────┼────────┤
# │ 999 │ tok_abc123  │ openai    │ gpt-4           │ 1,500  │ $0.045 │
# │ 998 │ tok_abc123  │ openai    │ gpt-3.5-turbo   │ 800    │ $0.002 │
# │ 997 │ tok_def456  │ anthropic │ claude-3-sonnet │ 2,000  │ $0.030 │
# └─────┴─────────────┴───────────┴─────────────────┴────────┴────────┘

# more traces
iron .analytics type::traces limit::50

# specific trace
iron .analytics type::traces id::999
# Trace ID: 999
# ───────────────────────────────
# Token: tok_abc123
# Provider: openai
# Model: gpt-4
# Timestamp: 2024-01-15T14:22:00Z
# Prompt tokens: 500
# Completion tokens: 1,000
# Total tokens: 1,500
# Cost: $0.045

# filter traces by agent
iron .analytics type::traces agent_id::my-agent

# filter traces by provider
iron .analytics type::traces provider_id::openai
```

---

## Export

Export data to file.

```bash
# export metrics
iron .analytics export::report.json

# export as CSV
iron .analytics export::report.csv export_format::csv

# export with filters
iron .analytics agent_id::my-agent export::agent-report.csv export_format::csv

# export traces
iron .analytics type::traces export::traces.json
```
