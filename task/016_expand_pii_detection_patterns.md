# Task 016: Expand PII detection patterns

## Goal
Expand the PII detection system from two patterns (email and US phone) to cover SSN, credit cards, international phone numbers, IP addresses, and AWS access keys, plus a configurable pattern registry for user-defined rules. This is observable when the safety module detects and redacts these new PII types in LLM responses. Scoped to the `iron_safety` module's detection and redaction logic.

## Dependencies
- None

## In Scope
- Adding SSN pattern (XXX-XX-XXXX format)
- Adding credit card number pattern with Luhn checksum validation
- Adding E.164 international phone format detection
- Adding IPv4 and IPv6 address detection
- Adding AWS access key pattern (AKIA prefix)
- Building a configurable pattern registry for user-defined rules
- Emitting PII detection events via `EventStore::record()`

## Out of Scope
- Natural language PII detection (names, addresses as free text)
- PII detection in binary or image content
- GDPR compliance workflows beyond detection and redaction
- Performance optimization for high-throughput streaming

## Description
The current PII detection in `iron_safety` only covers email addresses and US phone numbers via regex. This is insufficient for production compliance, where sensitive data like Social Security Numbers, credit card numbers, international phone numbers, IP addresses, and cloud credentials must also be detected and redacted. Promise #2 ("PII detected and redacted") requires broader coverage.

This task adds five new built-in pattern types, each with dedicated positive and negative test cases and documented false positive rates. Beyond built-in patterns, a configurable pattern registry allows users to add or remove detection rules at runtime, supporting domain-specific PII requirements. All detections emit audit events through the existing `EventStore::record()` mechanism for traceability.

## Context
PII detection covers only email and US phone regex. This contradicts promise #2 ("PII detected and redacted"). Production compliance requires broader pattern coverage.

Critical areas:
- `module/iron_safety/src/lib.rs`

## Work Procedure
1. Review the existing PII detection code in `iron_safety/src/lib.rs` to understand the pattern structure.
2. Define a `PiiPattern` trait or struct that encapsulates regex, optional validation (e.g., Luhn), and metadata.
3. Implement the SSN pattern with XXX-XX-XXXX format regex.
4. Implement the credit card pattern with digit grouping regex and Luhn checksum validation.
5. Implement E.164 international phone pattern.
6. Implement IPv4 and IPv6 address detection patterns.
7. Implement AWS access key pattern (AKIA prefix, 20-char alphanumeric).
8. Build the configurable pattern registry with add/remove operations.
9. Wire all detections to emit events via `EventStore::record()`.
10. Write unit tests with positive and negative cases for each pattern and document false positive rates.

## Implementation plan
1. Add SSN pattern (XXX-XX-XXXX).
2. Add credit card number pattern with Luhn validation.
3. Add E.164 international phone format.
4. Add IP address detection.
5. Add AWS access key pattern (AKIA prefix).
6. Add configurable pattern registry for user-defined rules.
7. Emit PII detection events via existing `EventStore::record()` mechanism.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| SSN "123-45-6789" in text | Detected and redacted | Pattern match found, redacted output contains no SSN |
| Invalid SSN "000-00-0000" | Not detected (invalid range) | No false positive match |
| Valid credit card "4111111111111111" | Detected, Luhn validates | Pattern match found, Luhn check passes |
| Invalid credit card failing Luhn | Not detected | No match due to Luhn failure |
| E.164 phone "+44 20 7946 0958" | Detected as international phone | Pattern match found |
| IPv4 "192.168.1.1" in text | Detected as IP address | Pattern match found |
| IPv6 "2001:0db8::1" in text | Detected as IP address | Pattern match found |
| AWS key "AKIAIOSFODNN7EXAMPLE" | Detected as AWS credential | Pattern match found |
| User-defined custom pattern | Detected after registry addition | Custom pattern triggers match |
| PII detection event | Audit event emitted via EventStore | Event recorded with correct type and payload |

## Validation Checklist
- [ ] SSN pattern detects valid SSNs and rejects invalid ones
- [ ] Credit card pattern includes Luhn validation
- [ ] E.164 international phone format is detected
- [ ] IPv4 and IPv6 addresses are detected
- [ ] AWS access key pattern (AKIA prefix) is detected
- [ ] Pattern registry supports runtime add/remove operations
- [ ] PII detections emit events via `EventStore::record()`
- [ ] Each pattern has positive and negative unit tests
- [ ] False positive rates are documented per pattern
- [ ] All tests pass with `cargo test --workspace`

## Validation Procedure
1. Run `cargo test -p iron_safety` and verify all new pattern tests pass.
2. Review each pattern's test file for both positive cases (should match) and negative cases (should not match).
3. Confirm the pattern registry allows adding a custom rule and detecting it in test input.
4. Verify PII detection events are recorded by checking `EventStore` in test output.
5. Check for false positive rate documentation in code comments or a dedicated section.
6. Run `cargo test --workspace` and confirm no regressions.

## Acceptance Criteria
- Each new pattern type has dedicated unit tests with positive and negative cases.
- Configurable registry allows adding/removing patterns at runtime.
- PII detections produce audit events.
- False positive rate documented for each pattern.
