# Error Handling Principles

**Purpose:** Error handling philosophy ensuring failures are visible and recoverable.

---

## User Need

Understand how errors are handled across the platform and why certain patterns are required.

## Core Idea

**Three fundamental error principles:**

1. **Fail-Fast** - Errors propagate immediately (no silent failures)
2. **Loud Failures** - All errors logged, traced, visible
3. **Proper Fixes** - No workarounds, only root cause solutions

## Fail-Fast Philosophy

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

## Loud Failures

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

## Proper Fixes Only

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

## Error Type Hierarchy

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

## Error Recovery

| Scenario | Recovery Strategy |
|----------|-------------------|
| Transient failures | Retry with exponential backoff |
| Provider down | Circuit breaker + fallback chain |
| Budget exceeded | Hard stop (no recovery) |
| Invalid config | Fail at startup (not runtime) |

---

*Related: [001_design_philosophy.md](001_design_philosophy.md) | [004_testing_strategy.md](004_testing_strategy.md)*
