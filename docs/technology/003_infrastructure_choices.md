# Infrastructure Choices

**Purpose:** Rationale for database, cache, and messaging infrastructure.

---

## User Need

Understand why PostgreSQL/Redis/S3 instead of alternatives.

## Core Idea

**Boring technology for reliability:**

## Database: PostgreSQL

| Alternative | Why PostgreSQL Wins |
|-------------|-------------------|
| MySQL | Better JSON support, ACID compliance |
| MongoDB | Relational data, SQL familiarity |
| CockroachDB | Simpler ops, sufficient scale |

**Use cases:** Tokens, policies, audit logs, user data

## Cache: Redis

| Alternative | Why Redis Wins |
|-------------|---------------|
| Memcached | Richer data structures, pub/sub |
| In-process | Shared state across replicas |
| DynamoDB | Lower latency, simpler |

**Use cases:** Rate limits, circuit breaker state, session cache

## Object Storage: S3

| Alternative | Why S3 Wins |
|-------------|------------|
| GCS | AWS ecosystem primary |
| Azure Blob | Same |
| MinIO | Self-hosted option available |

**Use cases:** Audit log archives, large artifacts

## Message Queue: None (Initially)

| Alternative | Why Skip For Now |
|-------------|-----------------|
| Kafka | Complexity not justified yet |
| RabbitMQ | Same |
| SQS | Same |

**Decision:** PostgreSQL LISTEN/NOTIFY + Redis pub/sub sufficient for v1.

---

*Related: [004_dependency_strategy.md](004_dependency_strategy.md)*
