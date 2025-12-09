# Scaling Patterns

**Purpose:** How to scale Iron Cage horizontally.

---

## User Need

Scale beyond single instance as agent workload grows.

## Core Idea

**Stateless services + shared state = horizontal scaling**

## Scaling by Component

| Component | Scaling Method | State Location |
|-----------|---------------|----------------|
| Control Panel | Multiple replicas + LB | Database |
| Agent Runtime | K8s HPA | Cache + Database |
| Services | Per-service replicas | Cache |

*Note: This describes production multi-replica deployment. Pilot uses single instance (no replicas, in-memory cache, no shared state needed). See [technology/003](../technology/003_infrastructure_choices.md#cache) for pilot vs production cache choice.*

## Architecture (Production Multi-Replica)

```
                    +-------------+
                    | Load        |
                    | Balancer    |
                    +------+------+
         +-----------------+-----------------+
         v                 v                 v
   +----------+     +----------+     +----------+
   | API Pod  |     | API Pod  |     | API Pod  |
   | Replica 1|     | Replica 2|     | Replica 3|
   +----+-----+     +----+-----+     +----+-----+
        +----------------+----------------+
                         v
              +---------------------+
              | Database + Cache    |
              | (shared state)      |
              | PostgreSQL + Redis  |
              +---------------------+
```

**Pilot Architecture:** Single process, in-memory cache, no load balancer (simpler).

## Scaling Triggers

| Metric | Threshold | Action |
|--------|-----------|--------|
| CPU | >70% | Add replica |
| Memory | >80% | Add replica |
| Request latency | >500ms | Add replica |
| Queue depth | >100 | Add worker |

---

*Related: [003_distribution_strategy.md](003_distribution_strategy.md)*
