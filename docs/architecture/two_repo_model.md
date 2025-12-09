# Two-Repository Model

**Purpose:** Why iron_runtime and iron_cage are separate repositories.

---

## User Need

Understand codebase organization and where to find code for different concerns.

## Core Idea

**Split by deployment lifecycle, not by technology:**

| Repository | Purpose | Release Cycle |
|------------|---------|---------------|
| **iron_runtime** | Control Panel, Agent Runtime | Weekly |
| **iron_cage** | Sandboxing, CLI, foundation | Monthly |

## Repository Contents

**iron_runtime (this repo):**
- iron_api - REST API + WebSocket server
- iron_dashboard - Vue 3 control panel
- iron_runtime - Agent orchestrator + PyO3
- iron_sdk - Python SDK with decorators
- iron_safety, iron_cost, iron_reliability - Core services

**iron_cage:**
- iron_sandbox - OS-level isolation (Landlock, seccomp)
- iron_cli - Binary CLI tool
- iron_types, iron_telemetry - Foundation modules

## Sharing Pattern

```
iron_cage --publishes--> crates.io --consumed by--> iron_runtime
           (iron_types)              (iron_api uses iron_types)
```

- Foundation modules published to crates.io
- No path dependencies between repos
- Clear versioning and compatibility

## Why This Split

1. **Different stability:** Sandbox is stable, dashboard changes weekly
2. **Different audiences:** Security team vs product team
3. **Independent releases:** Can ship dashboard without touching sandbox

---

*Related: [service_boundaries.md](service_boundaries.md)*
