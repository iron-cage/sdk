# Dependency Strategy

**Purpose:** Philosophy for external crate selection and management.

---

## User Need

Understand why certain crates are chosen and the criteria for adding new ones.

## Core Idea

**Minimal, audited dependencies for security-critical code:**

## Selection Criteria

| Criterion | Requirement |
|-----------|-------------|
| **Maintenance** | Active maintenance, <6 month since last commit |
| **Security** | No known CVEs, passed `cargo audit` |
| **Size** | Prefer smaller crates (less attack surface) |
| **Popularity** | >1000 downloads/month (battle-tested) |
| **License** | MIT, Apache-2.0, BSD (business-friendly) |

## Core Dependencies

| Crate | Purpose | Why This One |
|-------|---------|-------------|
| tokio | Async runtime | Industry standard, fastest |
| axum | HTTP server | Tower ecosystem, ergonomic |
| sqlx | Database | Compile-time SQL verification |
| serde | Serialization | De facto standard |
| tracing | Observability | Structured logging, spans |

## Avoided Patterns

| Pattern | Reason |
|---------|--------|
| ORM frameworks | SQL is simpler, more controllable |
| Heavy abstractions | Prefer explicit over magical |
| Kitchen-sink crates | Import only what's needed |

## Audit Process

1. New crate proposed -> Check criteria
2. Run `cargo audit` on dependency
3. Review source for obvious issues
4. Add to allowlist in `deny.toml`

---

*Related: [001_why_rust.md](001_why_rust.md) | [003_infrastructure_choices.md](003_infrastructure_choices.md)*
