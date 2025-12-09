# ADR-004: Crate Renaming Strategy

**Status:** Accepted
**Date:** 2025-12

*Source: Consolidated from pilot/decisions/001-crate-renaming.md*

---

## Context

Initial crate names (iron_token_*, iron_sandbox_*) were verbose and inconsistent.
Some crates had ambiguous names that didn't reflect their purpose.

Needed a consistent, semantic naming convention across the platform.

## Decision

Adopt unified naming convention:
- **iron_** prefix for all crates
- Single descriptive word (iron_safety, iron_cost, iron_reliability)
- No redundant prefixes (iron_token_manager -> iron_tokens)

## Renamed Crates

| Old Name | New Name | Rationale |
|----------|----------|-----------|
| iron_token_manager | iron_tokens | Simpler, standard |
| iron_sandbox_runtime | iron_sandbox | Clear purpose |
| iron_guardrails | iron_safety | Industry terminology |

## Consequences

**Positive:**
- Consistent, predictable naming
- Shorter import paths
- Clearer purpose from name

**Negative:**
- Breaking change for existing users
- Documentation updates required
- Potential confusion during transition

**Mitigations:**
- Clear migration guide
- Deprecation warnings for 2 releases
- Automated migration script

---

*Original: pilot/decisions/001-crate-renaming.md*
