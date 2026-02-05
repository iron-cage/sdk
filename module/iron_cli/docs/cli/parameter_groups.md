# Parameter Groups

Shared parameters reused across command namespaces.

## Overview

| Group | Parameters | Used By |
|-------|------------|---------|
| `output` | `format::`, `v::` | all commands |
| `pagination` | `page::`, `limit::` | list commands |
| `dry_run` | `dry::` | mutation commands |
| `authentication` | `email::`, `password::`, `username::` | auth commands |

---

## output

| Parameter | Type | Description | Env |
|-----------|------|-------------|-----|
| `format::` | [format_output](parameter_types.md#format_output) | Output format | `IRON_FORMAT` |
| `v::` | int (0-5) | Verbosity level | `IRON_VERBOSITY` |

**Verbosity levels**:

| Level | Description |
|-------|-------------|
| 0 | Silent (errors only) |
| 1 | Normal (default) |
| 2 | Verbose |
| 3 | Debug |
| 4 | Trace |
| 5 | All |

**Example**:
```bash
iron .tokens.list format::json
iron .tokens.list v::2
```

---

## pagination

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `page::` | int | Page number (1-based) | 1 |
| `limit::` | int | Results per page | 20 |
| `per_page::` | int | Alias for limit | 20 |

Used by `.tokens.list`, `.agent.list`, `.provider.list`, `.user.list`.

**Example**:
```bash
iron .tokens.list page::2 limit::50
# Returns page 2 with up to 50 results

iron .agent.list per_page::10
# Returns first 10 agents
```

---

## dry_run

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `dry::` | int (0\|1) | Dry run flag | 0 |

Used by create, update, delete, assign, remove operations.

- `dry::0` - Execute the operation (default)
- `dry::1` - Preview only, show what would happen

**Example**:
```bash
# preview deletion
iron .agent.delete id::abc123 dry::1
# Agent would be deleted: abc123 (dry run)

# actually delete
iron .agent.delete id::abc123
# Agent deleted: abc123
```

---

## authentication

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `email::` | [email](parameter_types.md#email) | Email for login | - |
| `username::` | string | Username for login | - |
| `password::` | string (sensitive) | User password | - |

Used by `.auth.login`.

**Notes**:
- `password::` is marked as sensitive and supports interactive input
- Tokens are stored in system keyring after successful login
- Use either `email::` or `username::` for login

**Example**:
```bash
# login with email
iron .auth.login email::user@example.com password::secret

# login with username (control API)
iron .auth.login username::admin password::secret

# interactive password prompt
iron .auth.login email::user@example.com
# Password: [hidden input]
```

---

## filter

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `filter::` | string | Filter criteria | - |
| `sort::` | string | Sort field | - |

Used by list commands for filtering and sorting results.

**Example**:
```bash
iron .tokens.list filter::active sort::created_at
```

---

## export

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `export_format::` | [export_format](parameter_types.md#export_format) | Export data format | json |
| `output::` | string | Output file path | stdout |
| `output_file::` | string | Alias for output | stdout |

Used by `.usage.export`, `.traces.export`, `.analytics.export_*`.

**Example**:
```bash
# export to file
iron .usage.export export_format::json output::usage.json

# export to stdout
iron .traces.export export_format::csv

# analytics export
iron .analytics.export_usage output_file::usage_report.csv format::csv
```
