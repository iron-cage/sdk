# Module Distribution

**Purpose:** Which code modules belong to which deployment package.

---

## User Need

Know where to find code for a specific feature and which package ships it.

## Core Idea

**Map modules to packages based on runtime dependencies:**

## Package 1: Control Panel

| Module | Purpose |
|--------|---------|
| iron_api | REST API + WebSocket server |
| iron_dashboard | Vue 3 SPA |
| iron_db | PostgreSQL schemas |

## Package 3: Agent Runtime

| Module | Purpose |
|--------|---------|
| iron_sdk | Python SDK |
| iron_safety | Input/output validation |
| iron_cost | Budget tracking |
| iron_reliability | Circuit breakers |

## Package 4: Sandbox

| Module | Purpose |
|--------|---------|
| iron_sandbox | OS isolation (Landlock, seccomp) |
| iron_executor | Code runner |

## Package 5: CLI Tool

| Module | Purpose |
|--------|---------|
| iron_cli | Binary CLI |
| iron_tokens | Token management |

## Shared Modules (crates.io)

| Module | Used By | Published |
|--------|---------|-----------|
| iron_types | All packages | crates.io |
| iron_telemetry | All packages | crates.io |
| iron_config | CLI, Runtime | crates.io |

---

*Related: [package_model.md](package_model.md)*
