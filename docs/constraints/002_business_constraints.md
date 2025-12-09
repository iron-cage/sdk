# Business Constraints

**Purpose:** Business and timeline limitations that shaped project scope.

---

## User Need

Understand why certain features were deferred or simplified.

## Core Idea

**Pilot-first approach with tight timeline constraints:**

## Timeline Constraints

| Milestone | Duration | Impact |
|-----------|----------|--------|
| **Pilot Development** | 22 days | Simplified scope, single-agent focus |
| **Demo Length** | 5 minutes | 100 leads processed, 3 triggers visible |
| **Production Readiness** | Q1 2026 | Pilot validates approach first |

**Implication:** Feature set minimized for demo success, full platform deferred.

## Scope Constraints

| Feature | Pilot Scope | Full Platform |
|---------|-------------|---------------|
| **LLM Providers** | OpenAI only | Multi-provider |
| **Execution Model** | Client-side only | Client + Server |
| **Sandboxing** | Spec-only | Full implementation |
| **Budget Currency** | USD only | Multi-currency |
| **Agents** | Single agent | Multi-agent orchestration |

## Team Constraints

**Pilot team:**
- Small team (optimized for speed)
- Rust + Python expertise
- Focus on core governance features

**Deferred expertise:**
- DevOps (Kubernetes, multi-region)
- Security hardening (penetration testing)
- SRE (advanced monitoring, alerting)

## Budget Constraints

**Pilot priorities:**
- Minimal infrastructure (SQLite, single process)
- No paid services (self-hosted only)
- Developer machines for execution

**Production will require:**
- PostgreSQL hosting
- Redis clusters
- Multi-region deployment
- CDN for marketing site

---

*Related: [001_technical_constraints.md](001_technical_constraints.md) | [003_scope_boundaries.md](003_scope_boundaries.md)*
