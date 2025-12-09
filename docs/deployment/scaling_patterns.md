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
| Control Panel | Multiple replicas + LB | PostgreSQL |
| Agent Runtime | K8s HPA | Redis + PostgreSQL |
| Services | Per-service replicas | Redis (cache) |

## Architecture

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
              | PostgreSQL + Redis  |
              | (shared state)      |
              +---------------------+
```

## Scaling Triggers

| Metric | Threshold | Action |
|--------|-----------|--------|
| CPU | >70% | Add replica |
| Memory | >80% | Add replica |
| Request latency | >500ms | Add replica |
| Queue depth | >100 | Add worker |

---

*Related: [distribution_strategy.md](distribution_strategy.md)*
