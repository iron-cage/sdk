# Principles: Error Handling Principles

### Scope

This document defines the error handling philosophy across the Iron Cage platform, translating the Fail-Safe Defaults and Observable Behavior principles from Principles 001 into concrete error handling patterns. These principles ensure failures are visible, traceable, and properly resolved rather than hidden or worked around.

**In scope**:
- Three fundamental error principles (Fail-Fast, Loud Failures, Proper Fixes Only)
- Fail-fast philosophy with application examples and code patterns
- Loud failures implementation (logging, tracing, audit, dashboard alerts)
- Proper fixes requirements (root cause analysis, test documentation, prevention)
- Error type hierarchy using error_tools crate per rulebook standards
- Error recovery strategies for different failure scenarios

**Out of scope**:
- Specific error_tools crate API documentation (see crate documentation)
- Detailed logging infrastructure implementation (see Observability capability)
- Test documentation format specifications (see test_organization.rulebook.md)
- Bug fix workflow details (see code_design.rulebook.md and codebase_hygiene.rulebook.md)
- Circuit breaker and retry logic implementation (see Reliability architecture)
- Observability backend configuration (see Integration 004: Observability Backends)

### Purpose

**User Need:** Understand how errors are handled across the Iron Cage platform and why certain error handling patterns are required to ensure failures are visible, traceable, and properly resolved.

**Solution:** Three fundamental error handling principles govern all failure scenarios:

1. **Fail-Fast** - Errors propagate immediately (no silent failures)
2. **Loud Failures** - All errors logged, traced, visible
3. **Proper Fixes** - No workarounds, only root cause solutions

These principles directly implement the Fail-Safe Defaults and Observable Behavior design principles from Principles 001. Fail-Fast ensures unsafe operations are blocked immediately rather than continuing with degraded state. Loud Failures makes all error scenarios observable through structured logging, audit trails, and dashboard visibility. Proper Fixes enforces root cause analysis and prevention strategies rather than symptom suppression.

**Key Insight:** Error handling is not defensive programming - it's a security and reliability enforcement mechanism. Silent errors and workarounds are anti-patterns that hide systemic issues and create unobservable failure modes. By making all failures loud and enforcing proper fixes, the platform ensures that every error teaches a lesson and prevents future occurrences rather than creating technical debt.

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13

---

### Fail-Fast Philosophy

**Principle:** Detect errors as early as possible, propagate immediately.

```
BAD: Silent error handling
if error { log.warn("error occurred"); continue; }

GOOD: Fail-fast with context
if error { return Err(Error::BudgetExceeded { spent, limit }); }
```

**Application:**
- Budget exceeded → Block request immediately
- PII detected → Fail request (or redact and warn)
- Invalid token → Reject at gateway (401)
- Provider down → Try fallback, fail if none available

### Loud Failures

**Principle:** Every error must be observable and traceable.

| Error | Visibility |
|-------|------------|
| Budget exceeded | Log + dashboard alert + request blocked |
| PII detected | Log + audit record + redaction notice |
| Provider failure | Log + retry attempt + eventual error |
| Invalid token | Log + 401 response + audit entry |

**Tools:**
- Structured logging (tracing crate)
- Error context (error_tools crate per rulebook)
- Audit trail (iron_runtime_state persistence)
- Dashboard alerts (WebSocket updates)

### Proper Fixes Only

**Principle:** Fix root cause, not symptoms. No workarounds.

**Forbidden:**
- Silencing errors with empty catch blocks
- Ignoring test failures
- Disabling warnings
- Workarounds instead of fixes

**Required:**
- Root cause analysis for bugs
- Test documenting fix (5 sections per rulebook)
- Source comment (3 fields per rulebook)
- Prevention strategy

### Error Type Hierarchy

**Use error_tools per rulebook:**

```rust
use error_tools::prelude::*;

#[ derive( Error, Debug ) ]
pub enum RuntimeError
{
  #[ error( "Budget exceeded: ${spent} of ${limit}" ) ]
  BudgetExceeded { spent: f64, limit: f64 },

  #[ error( "PII detected: {pii_type}" ) ]
  PiiDetected { pii_type: String },
}
```

### Error Recovery

| Scenario | Recovery Strategy |
|----------|-------------------|
| Transient failures | Retry with exponential backoff |
| Provider down | Circuit breaker + fallback chain |
| Budget exceeded | Hard stop (no recovery) |
| Invalid config | Fail at startup (not runtime) |

---

### Cross-References

#### Related Principles Documents

- [001_design_philosophy.md](001_design_philosophy.md) - Fail-Safe Defaults and Observable Behavior principles that error handling implements
- [002_quality_attributes.md](002_quality_attributes.md) - Reliability quality attribute supported by error handling principles
- [004_testing_strategy.md](004_testing_strategy.md) - Testing approach validating error handling behavior and fail-fast patterns
- [005_development_workflow.md](005_development_workflow.md) - Bug fix workflow requiring proper fixes and comprehensive documentation

#### Used By

- Architecture 002: [Layer Model](../architecture/002_layer_model.md) - Each processing layer implements fail-fast error propagation
- Protocol: All API specifications reference loud failures and structured error responses
- Security: Threat model and isolation layers apply fail-safe defaults to security boundaries
- Capabilities: All capability specifications demonstrate fail-fast patterns and proper error handling
- Testing: Test organization follows bug fix workflow (5-section test documentation, 3-field source comments)

#### Dependencies

- Principles 001: [Design Philosophy](001_design_philosophy.md) - Foundational Fail-Safe Defaults and Observable Behavior principles
- **Rulebook Standards:** error_tools crate usage, 5-section test documentation format, 3-field source comment format
- code_design.rulebook.md - Bug fix workflow and root cause analysis requirements
- test_organization.rulebook.md - Test documentation format (Root Cause, Why Not Caught, Fix Applied, Prevention, Pitfall)
- codebase_hygiene.rulebook.md - Documentation quality standards (specific, technical, actionable, traceable)

#### Implementation

- Error handling enforced via error_tools crate throughout codebase
- Fail-fast patterns validated via integration tests demonstrating proper error propagation
- Loud failures implemented via structured logging (tracing crate) and audit trails
- Proper fixes workflow enforced via bug fix documentation requirements (test + source comments)
- Error recovery strategies implemented via circuit breakers, exponential backoff, and fallback chains
