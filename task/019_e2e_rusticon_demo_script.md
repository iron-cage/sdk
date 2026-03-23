# Task 019: End-to-end Rusticon demo script

## Dependencies
- Task 014
- Task 015
- Task 016
- Task 017

## Context
The Rusticon presentation described a demo with budget enforcement ($1 hard cap), PII detection, circuit breaker, and audit trail. This task creates a reproducible script that validates all promises work together.

Critical areas:
- All runtime modules working together
- Python SDK integration

## Implementation plan
1. Create demo script that exercises:
   - Budget enforcement with $1 hard limit
   - PII detection enabled on responses
   - Circuit breaker enabled
   - Audit trail capturing all events
2. Verify 90% budget warning fires.
3. Verify PII redaction in responses.
4. Verify circuit breaker activates when real test provider returns errors (per ADR-007: no mocking).
5. Export audit trail and verify completeness.

## Acceptance criteria
- Single script reproduces the Rusticon demo scenario.
- All four features (budget, PII, circuit breaker, audit) activate correctly.
- Audit export contains all events from the run.
- Script can be used as basis for documentation examples.
