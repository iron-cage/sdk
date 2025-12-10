# Actor Model

**Purpose:** Who and what interacts with Iron Cage.

---

## User Need

Understand system boundaries and who uses which packages.

## Core Idea

**Three actor types: Human, Software, Package**

## Human Actors

**Note:** All roles use both CLI and Dashboard (equivalent interface).

| Actor | Role | Primary Package |
|-------|------|-----------------|
| **Admin** | Manages Control Panel (CLI + Dashboard), allocates budgets, oversees all developers | Control Panel (REQUIRED) |
| **Super User** | Developer + read-only Control Panel dashboard access (CLI + Dashboard, own budgets) | Control Panel + Agent Runtime |
| **Developer** | Builds AI agents with IC Token (CLI + Dashboard, read-only own usage) | Agent Runtime |
| Operations | Monitors execution | Control Panel |
| Security | Implements isolation | Sandbox |
| Visitor | Views marketing | Marketing Site |

## Software Actors

| Actor | Role | Interacts With |
|-------|------|----------------|
| Python Agent | User's AI code | Agent Runtime |
| Web Browser | Dashboard access | Control Panel |
| Terminal | CLI commands | CLI Tool |
| LLM API | External providers | Agent Runtime |
| Database | State storage | Control Panel |

## Interaction Patterns

```
Developer --uv pip install--> Agent Runtime --HTTPS--> LLM APIs
                                |
Operations --browser--> Control Panel --WebSocket--+
                                |
Administrator --CLI--> CLI Tool --REST API--+
```

---

*Related: [001_package_model.md](001_package_model.md)*
