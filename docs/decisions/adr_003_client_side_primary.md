# ADR-003: Client-Side Execution as Primary Model

**Status:** Accepted
**Date:** 2025-01

---

## Context

AI agents need access to local resources (files, APIs, databases).
Two deployment models possible:
1. Client-side: Agent runs on user's machine
2. Server-side: Agent runs on Iron Cage infrastructure

## Decision

Client-side execution is the primary model (95% of deployments).
Server-side is optional for Phase 2 (5% of deployments).

**Key insight:** Governance doesn't require running agent code ON Iron Cage servers. Agent stays local, but Control Panel is required infrastructure for budget management and IP Token storage.

## Consequences

**Positive:**
- User keeps code and data local (privacy)
- Minimal infrastructure: Control Panel only (no agent hosting servers)
- Simple integration (just add SDK + IC Token)
- Agents have full local access (files, APIs)

**Negative:**
- Can't enforce execution isolation (user's machine)
- Harder to debug (can't see agent state remotely)
- SDK must be trusted (runs in user's process)
- Requires Control Panel deployment (admin service infrastructure)

**Mitigations:**
- Optional server-side for managed execution
- Rich logging/tracing for debugging
- SDK is open-source for auditability
- Control Panel can be localhost (pilot) or cloud (production)

---

*Related: [architecture/001_execution_models.md](../architecture/001_execution_models.md)*
