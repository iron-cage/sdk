# .project

## .project.list

List all projects.

```bash
iron .project.list
iron .project.list format::json
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Array of project objects

**Example**:
```bash
iron .project.list
# ┌──────────┬──────────────┬─────────────────────┐
# │ ID       │ Name         │ Created             │
# ├──────────┼──────────────┼─────────────────────┤
# │ proj_1   │ Production   │ 2024-01-10T08:00:00Z│
# │ proj_2   │ Development  │ 2024-01-05T12:00:00Z│
# │ proj_3   │ Testing      │ 2024-01-01T10:00:00Z│
# └──────────┴──────────────┴─────────────────────┘

iron .project.list format::json
# [
#   {"id": "proj_1", "name": "Production", "created_at": "2024-01-10T08:00:00Z"},
#   ...
# ]
```

---

## .project.get

Get project details by ID.

```bash
iron .project.get id::<id>
```

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `id::` | [project_id](../parameter_types.md#project_id) | Project ID | - |
| `format::` | [format_output](../parameter_types.md#format_output) | Output format | `table` |

**Response**: Project configuration object

**Example**:
```bash
iron .project.get id::proj_1
# Project: proj_1
# ───────────────────────────────
# Name: Production
# Created: 2024-01-10T08:00:00Z
#
# Agents: 5
# Providers: 2
# Total Usage: $35.00

iron .project.get id::proj_1 format::json
# {
#   "id": "proj_1",
#   "name": "Production",
#   "created_at": "2024-01-10T08:00:00Z",
#   "agents_count": 5,
#   "providers_count": 2,
#   "total_usage": 35000000
# }
```
