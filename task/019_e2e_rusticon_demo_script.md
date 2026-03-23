# Task 019: End-to-end Rusticon demo script

## Goal
Create a single reproducible script that exercises all four Rusticon demo features - budget enforcement, PII detection, circuit breaker, and audit trail - working together in an end-to-end scenario. This is observable when the script runs to completion, activating each feature and producing a verifiable audit export. Scoped to script creation and validation against the integrated runtime.

## Dependencies
- Task 014
- Task 015
- Task 016
- Task 017

## In Scope
- Creating a self-contained demo script exercising budget enforcement, PII detection, circuit breaker, and audit trail
- Verifying the 90% budget warning fires
- Verifying PII redaction in responses
- Verifying circuit breaker activates on provider errors
- Exporting and validating the audit trail
- Making the script usable as a basis for documentation examples

## Out of Scope
- Adding new runtime features beyond what Tasks 014-017 deliver
- Performance benchmarking of the integrated system
- UI or visualization of the demo results
- Deployment or hosting of the demo environment

## Description
The Rusticon presentation promised a demo where all four core features - budget enforcement with a $1 hard cap, PII detection and redaction, circuit breaker activation, and persistent audit trail - work together seamlessly. Currently no single script validates this integrated scenario, making it impossible to verify that the promises hold when features interact.

This task creates a reproducible end-to-end script that exercises the full stack. It sets up a budget with a $1 hard limit, enables PII detection on responses, activates the circuit breaker, and captures all events to the audit trail. The script verifies each feature fires correctly: the 90% budget warning appears, PII is redacted, the circuit breaker activates when providers return errors, and the exported audit trail contains all events. Per ADR-007, the script uses real test providers rather than mocks.

## Context
The Rusticon presentation described a demo with budget enforcement ($1 hard cap), PII detection, circuit breaker, and audit trail. This task creates a reproducible script that validates all promises work together.

Critical areas:
- All runtime modules working together
- Python SDK integration

## Work Procedure
1. Verify all prerequisite tasks (014, 015, 016, 017) are complete and their features work individually.
2. Design the script flow: initialization, budget setup, PII-laden prompts, error injection for circuit breaker, audit export.
3. Set up the runtime with a $1 hard budget cap and 90% warning threshold.
4. Configure PII detection with all available patterns enabled.
5. Enable the circuit breaker with appropriate thresholds for the demo.
6. Send prompts that will trigger PII detection in responses.
7. Inject provider errors to trigger circuit breaker activation.
8. Continue sending requests until the 90% budget warning fires.
9. Export the audit trail and verify all events are present.
10. Add assertions/checks throughout the script to verify each feature activated correctly.

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

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Budget set to $1 hard cap | Requests succeed until budget exhausted | Budget enforcement prevents overspend |
| 90% of budget consumed | Warning event fires | Audit log contains 90% budget warning event |
| Response contains PII (e.g., SSN, email) | PII is detected and redacted | Redacted response contains no raw PII |
| Provider returns repeated errors | Circuit breaker activates | Subsequent requests short-circuit without hitting provider |
| Circuit breaker recovery | Circuit breaker closes after provider recovers | Requests resume flowing to provider |
| Audit trail export after run | JSON/CSV export contains all events | Export has budget, PII, circuit breaker, and request events |
| Full demo script execution | All four features activate in sequence | Script exits 0 with all assertions passing |

## Validation List
- [ ] Demo script exists as a single executable file
- [ ] Budget enforcement activates at $1 hard cap
- [ ] 90% budget warning event is recorded
- [ ] PII detection identifies and redacts PII in responses
- [ ] Circuit breaker activates on repeated provider errors
- [ ] Audit trail export is produced and contains all event types
- [ ] Script uses real test providers, not mocks (per ADR-007)
- [ ] Script can serve as documentation example (clean, commented code)
- [ ] Script exits with code 0 on success

## Validation Procedure
1. Run the demo script and verify it completes without errors (exit code 0).
2. Inspect the script output for the 90% budget warning message.
3. Check the script output for PII redaction markers in responses.
4. Verify the circuit breaker activation message appears when providers return errors.
5. Open the exported audit trail (JSON or CSV) and confirm it contains events for all four features.
6. Count events in the export and verify none are missing from the run.
7. Review the script code to confirm no mocks are used and all providers are real.

## Acceptance criteria
- Single script reproduces the Rusticon demo scenario.
- All four features (budget, PII, circuit breaker, audit) activate correctly.
- Audit export contains all events from the run.
- Script can be used as basis for documentation examples.
