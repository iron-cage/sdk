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

Client-side execution is the primary model (95% of users).
Server-side is optional for Phase 2 (5% of users).

**Key insight:** Governance doesn't require running agent code. The SDK intercepts LLM calls and routes through Iron Cage for validation, while agent stays local.

## Consequences

**Positive:**
- User keeps code and data local (privacy)
- No infrastructure needed from Iron Cage
- Simple integration (just add SDK)
- Agents have full local access (files, APIs)

**Negative:**
- Can't enforce execution isolation (user's machine)
- Harder to debug (can't see agent state)
- SDK must be trusted (runs in user's process)

**Mitigations:**
- Optional server-side for managed execution
- Rich logging/tracing for debugging
- SDK is open-source for auditability

---

*Related: [architecture/001_execution_models.md](../architecture/001_execution_models.md)*
