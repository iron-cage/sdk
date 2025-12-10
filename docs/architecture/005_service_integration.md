# Service Integration

**Purpose:** How the five core services communicate at runtime.

---

## User Need

Understand service dependencies and communication patterns for debugging/operations.

## Core Idea

**Gateway orchestrates calls to specialized services:**

```
                    +-----------------+
                    |    Gateway      |
                    |   (Port 8084)   |
                    +--------+--------+
         +--------------+----+----+--------------+
         v              v         v              v
   +----------+  +----------+ +----------+ +----------+
   |  Safety  |  |   Cost   | |Tool Proxy| |  Audit   |
   |  :8080   |  |  :8081   | |  :8082   | |  :8083   |
   +----------+  +----------+ +----------+ +----------+
```

## The Five Services

| Service | Port | Purpose | Deps |
|---------|------|---------|------|
| Safety | 8080 | Input/output validation | Database |
| Cost | 8081 | Budget tracking | Database, Cache |
| Tool Proxy | 8082 | Tool authorization | Cache |
| Audit | 8083 | Compliance logging | Database, Object Storage |
| Gateway | 8084 | Orchestration | All above |

*Note: Cache = In-memory (pilot) or Redis (production). Database = SQLite/PostgreSQL. Object Storage = S3/compatible. See [technology/003](../technology/003_infrastructure_choices.md).*

## Call Sequence

1. Agent calls SDK -> SDK calls Gateway
2. Gateway -> Safety (validate input)
3. Gateway -> Cost (check budget)
4. Gateway -> OpenAI (forward request)
5. Gateway -> Safety (validate output)
6. Gateway -> Audit (log event, async)
7. Gateway -> Agent (return response)

## Failure Handling

| Service Down | Behavior |
|--------------|----------|
| Safety | BLOCK all (fail-safe) |
| Cost | ALLOW, track in memory |
| Tool Proxy | BLOCK tool execution |
| Audit | ALLOW, buffer in queue (in-memory or cache) |

---

*Related: [002_layer_model.md](002_layer_model.md) | [004_data_flow.md](004_data_flow.md)*
