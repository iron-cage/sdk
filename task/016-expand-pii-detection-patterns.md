# Task 016: Expand PII detection patterns

## Dependencies
- None

## Context
PII detection covers only email and US phone regex. This contradicts promise #2 ("PII detected and redacted"). Production compliance requires broader pattern coverage.

Critical areas:
- `module/iron_safety/src/lib.rs`

## Implementation plan
1. Add SSN pattern (XXX-XX-XXXX).
2. Add credit card number pattern with Luhn validation.
3. Add E.164 international phone format.
4. Add IP address detection.
5. Add AWS access key pattern (AKIA prefix).
6. Add configurable pattern registry for user-defined rules.
7. Emit PII detection events via existing `EventStore::record()` mechanism.

## Acceptance criteria
- Each new pattern type has dedicated unit tests with positive and negative cases.
- Configurable registry allows adding/removing patterns at runtime.
- PII detections produce audit events.
- False positive rate documented for each pattern.
