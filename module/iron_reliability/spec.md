# iron_reliability - Specification

**Module:** iron_reliability
**Layer:** 3 (Feature)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

Circuit breaker patterns and retry logic for LLM provider reliability. Prevents cascade failures, implements automatic fallback chains, handles transient errors with exponential backoff.

---

## Scope

**In Scope:**
- Circuit breaker (open/half-open/closed states)
- Retry logic with exponential backoff
- Fallback chain management (primary → secondary → tertiary providers)
- Timeout enforcement for LLM calls
- Failure tracking and recovery

**Out of Scope:**
- LLM provider integration (see iron_lang)
- Cost tracking (see iron_cost)
- Safety validation (see iron_safety)
- Observability export (see iron_telemetry)

---

## Dependencies

**Required Modules:**
- iron_types - Foundation types
- iron_telemetry - Error logging

**Required External:**
- tokio - Async runtime for timeouts

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **Circuit Breaker:** Prevents cascade failures, tracks failure rate
- **Retry Manager:** Handles transient errors with exponential backoff
- **Fallback Chain:** Routes to backup providers when primary fails
- **Timeout Enforcer:** Prevents hanging requests

---

## Integration Points

**Used by:**
- iron_runtime - Wraps all LLM calls with reliability patterns

**Uses:**
- iron_telemetry - Logs circuit breaker state changes and retry attempts

---

*For detailed circuit breaker logic, see spec/-archived_detailed_spec.md*
*For architecture, see docs/architecture/002_layer_model.md*
