# Quality Attributes

**Purpose:** System-wide non-functional requirements and quality targets.

---

## User Need

Understand performance, reliability, scalability, security, and usability targets that constrain implementation.

## Core Idea

**Five quality attributes define system constraints:**

```
Performance + Reliability + Scalability + Security + Usability = System Quality
   <10ms        99.9%          10K agents      Defense      Pythonic API
```

## Performance

| Metric | Pilot Target | Production Target | Rationale |
|--------|--------------|-------------------|-----------|
| **Total Overhead** | ~25ms | ~100ms | Governance shouldn't slow agents noticeably |
| **Budget Check** | <1ms | <1ms | Local check, no network call |
| **Safety Validation** | 10ms (Regex) | 50ms (ML) | Pilot: demo-adequate, Production: compliance-grade |
| **Token Translation** | <0.5ms | <0.5ms | Memory operation only |
| **Cost Tracking** | 5ms (per-request) | 0.5ms (batched) | Pilot: simpler implementation, Production: optimized for scale |

**See:** [constraints/004: Trade-offs](../constraints/004_trade_offs.md#latency-budget-summary) for complete latency budget and decision rationale.

## Reliability

| Metric | Target | Implementation |
|--------|--------|----------------|
| **Availability** | 99.9% | Circuit breakers, fallback chains |
| **Data Durability** | 99.999% | SQLite WAL, PostgreSQL replication |
| **Fail-Safe** | 100% | Safety layer down = block all |
| **Error Recovery** | Automatic | Retry logic, exponential backoff |

## Scalability

| Dimension | Target | Architecture |
|-----------|--------|--------------|
| **Agents** | 10,000+ per Control Panel | Horizontal scaling, stateless services |
| **Requests** | 1,000 RPS | Async runtime, connection pooling |
| **Storage** | Millions of audit records | Database (partitioned tables), Object Storage (archives) |
| **Tokens** | 10,000+ active tokens | Indexed lookups, Cache layer |

## Security

| Principle | Implementation |
|-----------|----------------|
| **Defense in Depth** | 4 isolation layers (process, syscall, filesystem, network) |
| **Least Privilege** | Scoped credentials, minimal access |
| **Never Trust Input** | Validate everything from users and agents |
| **Encrypt Secrets** | IP Token encrypted in memory, never on disk |
| **Audit Everything** | Immutable logs for compliance |

## Usability

| Aspect | Target | Example |
|--------|--------|---------|
| **Installation** | Single command | `pip install iron-cage` |
| **Configuration** | Zero config defaults | Works with IC Token only |
| **API Style** | Pythonic | `@protect_agent` decorator |
| **Error Messages** | Actionable | "Budget exceeded: $10.50 of $10.00 spent" |
| **Developer Experience** | Transparent | Agent code unchanged, protection automatic |

---

*Related: [001_design_philosophy.md](001_design_philosophy.md) | [../architecture/002_layer_model.md](../architecture/002_layer_model.md)*
