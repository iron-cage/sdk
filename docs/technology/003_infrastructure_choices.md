# Infrastructure Choices

**Purpose:** Rationale for database, cache, and messaging infrastructure.

---

## User Need

Understand why PostgreSQL/Redis/S3 instead of alternatives.

## Core Idea

**Boring technology for reliability:**

## Database

**Trade-off:** Implementation simplicity vs distributed deployment

| Solution | Setup | Replication | Pilot Choice | Production Choice |
|----------|-------|-------------|--------------|-------------------|
| SQLite | File-based, zero config | No (single file) | ✅ **Chosen** | ❌ Not distributed |
| PostgreSQL | Server process, configuration | Yes (streaming replication) | ❌ Unnecessary | ✅ **Chosen** |
| MySQL | Server process | Yes | ❌ Weaker JSON | ❌ Weaker JSON than PostgreSQL |
| MongoDB | Server process | Yes | ❌ Rejected | ❌ Relational data needed |
| CockroachDB | Distributed | Built-in | ❌ Overkill | ❌ Complexity not justified |

**Rationale:**
- **Pilot:** SQLite sufficient for single-instance localhost demo. File-based (no server process), zero configuration, WAL mode for durability. All data fits in single file for 5-minute demo.
- **Production:** PostgreSQL required for distributed deployment. Network access for multiple replicas, streaming replication for durability, partitioned tables for scale. Better JSON support than MySQL, ACID compliance, proven reliability.

**Use Cases:** Tokens, policies, audit logs, user data

**See:** principles/001 § "SQLite before PostgreSQL" design principle.

## Cache

**Trade-off:** Implementation simplicity vs distributed state management

| Solution | Latency | Shared State | Pilot Choice | Production Choice |
|----------|---------|--------------|--------------|-------------------|
| In-memory (HashMap/LRU) | <0.1ms | No (single instance) | ✅ **Chosen** | ❌ Not distributed |
| Redis | ~1ms | Yes (replicas) | ❌ Unnecessary | ✅ **Chosen** |
| Memcached | ~1ms | Yes (replicas) | ❌ Unnecessary | ⚠️ Alternative to Redis |
| DynamoDB | ~5ms | Yes (distributed) | ❌ Higher latency | ❌ Higher cost |

**Rationale:**
- **Pilot:** In-memory structures sufficient for single-instance localhost deployment. HashMap for rate limits, LRU cache for IC Token validation. No external dependency, simpler setup, faster iteration.
- **Production:** Redis required for shared state across replicas. Provides distributed cache, pub/sub for circuit breaker coordination, persistent rate limit counters. Chosen over Memcached for richer data structures.

**Use Cases:**
- Rate limits: Track requests per IC Token (prevent abuse)
- Circuit breaker state: Coordinate failure detection across replicas
- Session cache: Cache IC Token validation results (avoid database on every request)
- Audit buffer: Queue events when Audit service temporarily down

**See:** [constraints/004: Trade-offs](../constraints/004_trade_offs.md) for pattern of pilot simplicity vs production optimization.

## Object Storage

**Trade-off:** Demo simplicity vs compliance archival

| Solution | Pilot Choice | Production Choice |
|----------|--------------|-------------------|
| None (database only) | ✅ **Chosen** | ❌ Need durable archive |
| Filesystem | ⚠️ Alternative | ❌ Not distributed |
| S3 | ❌ Unnecessary for demo | ✅ **Chosen** |
| GCS | ❌ Unnecessary | ⚠️ Alternative (non-AWS) |
| Azure Blob | ❌ Unnecessary | ⚠️ Alternative (Azure) |
| MinIO | ❌ Unnecessary | ⚠️ Alternative (self-hosted) |

**Rationale:**
- **Pilot:** No object storage needed for 5-minute demo. Audit logs stored in SQLite database (sufficient for demonstration). No long-term archival requirement.
- **Production:** S3 required for compliance. Immutable audit log archive, versioning for tamper detection, retention policies, cost-effective long-term storage. S3 chosen for AWS ecosystem compatibility, MinIO available for self-hosted.

**Use Cases:** Audit log archives (compliance requirement), large artifacts (model weights, training data)

## Message Queue: None (Initially)

| Alternative | Why Skip For Now |
|-------------|-----------------|
| Kafka | Complexity not justified yet |
| RabbitMQ | Same |
| SQS | Same |

**Decision:** PostgreSQL LISTEN/NOTIFY + Redis pub/sub sufficient for v1.

---

*Related: [004_dependency_strategy.md](004_dependency_strategy.md)*
