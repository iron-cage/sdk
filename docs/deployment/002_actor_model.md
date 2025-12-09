# Actor Model

**Purpose:** Who and what interacts with Iron Cage.

---

## User Need

Understand system boundaries and who uses which packages.

## Core Idea

**Three actor types: Human, Software, Package**

## Human Actors

| Actor | Role | Primary Package |
|-------|------|-----------------|
| Developer | Builds AI agents | Agent Runtime |
| Operations | Monitors execution | Control Panel |
| Security | Implements isolation | Sandbox |
| Administrator | Manages tokens/secrets | CLI Tools |
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
Developer --pip install--> Agent Runtime --HTTPS--> LLM APIs
                                |
Operations --browser--> Control Panel --WebSocket--+
                                |
Administrator --CLI--> CLI Tool --REST API--+
```

---

*Related: [001_package_model.md](001_package_model.md)*
