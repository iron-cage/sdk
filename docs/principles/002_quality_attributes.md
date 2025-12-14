# Principles: Quality Attributes

### Scope

This document defines system-wide non-functional requirements and quality targets across five key attributes: Performance, Reliability, Scalability, Security, and Usability. These quality attributes translate the design principles from Principles 001 into measurable targets and concrete constraints that guide all implementation choices.

**In scope**:
- Performance targets (latency, overhead, throughput) for pilot and production deployments
- Reliability metrics (availability, durability, fail-safe behavior, error recovery)
- Scalability dimensions (agents, requests, storage, tokens) with architectural approaches
- Security principles (defense in depth, least privilege, input validation, encryption, audit)
- Usability aspects (installation, configuration, API style, error messages, developer experience)
- Pilot vs production trade-offs for each quality attribute

**Out of scope**:
- Authoritative latency budget values (see Constraints 004: Trade-offs for complete latency reference)
- Implementation-specific performance optimization techniques (see Architecture collection)
- Security threat modeling and attack scenarios (see Security 001: Threat Model)
- Detailed API specifications and endpoints (see Protocol collection)
- Technology selection rationale for achieving quality targets (see Technology collection)
- Testing strategies for validating quality attributes (see Principles 004: Testing Strategy)

### Purpose

**User Need:** Understand performance, reliability, scalability, security, and usability targets that constrain implementation across the Iron Cage platform.

**Solution:** Five quality attributes define system constraints:

```
Performance + Reliability + Scalability + Security + Usability = System Quality
   <10ms        99.9%          10K agents      Defense      Pythonic API
```

Each quality attribute translates abstract design principles (from Principles 001) into concrete, measurable targets. Performance defines latency and overhead budgets. Reliability establishes availability and fail-safe guarantees. Scalability sets capacity targets for agents, requests, and storage. Security implements defense-in-depth principles. Usability ensures developer-friendly interfaces and transparent operation.

**Key Insight:** Quality attributes are not wish-list items - they are enforceable constraints with specific numeric targets. Pilot targets prioritize simplicity and demo-readiness (Simplicity First principle). Production targets prioritize scale and compliance (Fail-Safe Defaults principle). Every implementation choice must satisfy applicable quality targets or document the conscious trade-off.

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13

---

### Performance

| Metric | Pilot Target | Production Target | Rationale |
|--------|--------------|-------------------|-----------|
| **Total Overhead** | ~25ms | ~100ms | Governance shouldn't slow agents noticeably |
| **Budget Check** | <1ms | <1ms | Local check, no network call |
| **Safety Validation** | 10ms (Regex) | 50ms (ML) | Pilot: demo-adequate, Production: compliance-grade |
| **Token Translation** | <0.5ms | <0.5ms | Memory operation only |
| **Cost Tracking** | 5ms (per-request) | 0.5ms (batched) | Pilot: simpler implementation, Production: optimized for scale |

**See:** [constraints/004: Trade-offs](../constraints/004_trade_offs.md#latency-budget-summary) for complete latency budget and decision rationale.

### Reliability

| Metric | Target | Implementation |
|--------|--------|----------------|
| **Availability** | 99.9% | Circuit breakers, fallback chains |
| **Data Durability** | 99.999% | SQLite WAL, PostgreSQL replication |
| **Fail-Safe** | 100% | Safety layer down = block all |
| **Error Recovery** | Automatic | Retry logic, exponential backoff |

### Scalability

| Dimension | Target | Architecture |
|-----------|--------|--------------|
| **Agents** | 10,000+ per Control Panel | Horizontal scaling, stateless services |
| **Requests** | 1,000 RPS | Async runtime, connection pooling |
| **Storage** | Millions of audit records | Database (partitioned tables), Object Storage (archives) |
| **Tokens** | 10,000+ active tokens | Indexed lookups, Cache layer |

### Security

| Principle | Implementation |
|-----------|----------------|
| **Defense in Depth** | 4 isolation layers (process, syscall, filesystem, network) |
| **Least Privilege** | Scoped credentials, minimal access |
| **Never Trust Input** | Validate everything from users and agents |
| **Encrypt Secrets** | IP Token encrypted in memory, never on disk |
| **Audit Everything** | Immutable logs for compliance |

### Usability

| Aspect | Target | Example |
|--------|--------|---------|
| **Installation** | Single command | `uv pip install iron-sdk` |
| **Configuration** | Zero config defaults | Works with IC Token only |
| **API Style** | Pythonic | `@protect_agent` decorator |
| **Error Messages** | Actionable | "Budget exceeded: $10.50 of $10.00 spent" |
| **Developer Experience** | Transparent | Agent code unchanged, protection automatic |

---

### Cross-References

#### Related Principles Documents

- [001_design_philosophy.md](001_design_philosophy.md) - Seven core design principles that these quality attributes make measurable and enforceable
- [003_error_handling_principles.md](003_error_handling_principles.md) - Error handling philosophy supporting Reliability quality attribute
- [004_testing_strategy.md](004_testing_strategy.md) - Testing approach for validating quality attributes
- [005_development_workflow.md](005_development_workflow.md) - Development workflow ensuring quality attribute compliance

#### Used By

- Architecture 002: [Layer Model](../architecture/002_layer_model.md) - References performance targets for each processing layer
- Protocol: All API specifications reference usability and performance targets
- Security: Security documents reference defense-in-depth and encryption requirements
- Deployment: Scaling patterns reference scalability targets (10K agents, 1K RPS)
- Capabilities: All capability specifications demonstrate quality attribute compliance

#### Dependencies

- **Authoritative Reference:** Constraints 004: [Trade-offs](../constraints/004_trade_offs.md#latency-budget-summary) - Provides authoritative latency budget values referenced in Performance section
- Principles 001: [Design Philosophy](001_design_philosophy.md) - Foundational principles translated into measurable quality attributes
- Architecture 002: [Layer Model](../architecture/002_layer_model.md) - Layer architecture implementing quality targets

#### Implementation

- Performance targets validated via observability metrics and load testing
- Reliability metrics enforced via circuit breakers, fallback chains, and fail-safe defaults
- Scalability targets validated via horizontal scaling tests and capacity planning
- Security principles implemented via isolation layers (Security 002) and audit logging
- Usability targets validated via developer feedback and API ergonomics review
