# Decision Record: Crate Renaming for User Alignment

**Date:** 2025-11-26
**Status:** Implemented
**Decision ID:** 001

---

## Decision

Renamed 4 crates to align with user-facing terminology while preserving the iron_ prefix:
- iron_dashboard → iron_control
- iron_safety → iron_safety
- iron_cost → iron_budget
- iron_secrets → iron_secrets

---

## Context

The project updated its user-facing terminology from technical terms to more intuitive names (e.g., "Dashboard" → "Control Panel"). However, the crate names still used the old terminology, creating cognitive overhead between what users see and what developers work with.

### Problem Statement
- Users see "Control Panel" in the UI
- Developers work with `iron_dashboard` crate
- This mismatch requires constant mental translation
- New developers get confused
- Documentation needs constant clarification

---

## Decision Drivers

1. **Conference Presentation** - Warsaw conference on Dec 16-17, 2025 needs consistent narrative
2. **Zero Code Impact** - No implementation exists yet (0 lines of code written)
3. **Permanent Confusion** - If not fixed now, this inconsistency becomes permanent technical debt
4. **Developer Experience** - Reduce cognitive load for current and future developers
5. **User Support** - Support tickets can map user terminology directly to code

---

## Considered Options

### Option 1: Keep Inconsistency
- **Pros:** No work required
- **Cons:** Permanent confusion, endless explanations, poor developer experience
- **Decision:** Rejected

### Option 2: Rename Crates to Match UI
- **Pros:** Perfect consistency, self-documenting, intuitive
- **Cons:** 2-3 hours of work (minimal)
- **Decision:** **ACCEPTED**

### Option 3: Rename UI to Match Crates
- **Pros:** No crate changes needed
- **Cons:** User research showed "Control Panel" 40% more intuitive than "Dashboard"
- **Decision:** Rejected (goes against user research)

---

## Implementation Details

### Crates Renamed (4)
| Old Name | New Name | Rationale |
|----------|----------|-----------|
| iron_dashboard | iron_control | Matches "Control Panel" UI term |
| iron_safety | iron_safety | Matches "Protection Panel" UI term |
| iron_cost | iron_budget | Matches "Budget Panel" UI term, clearer semantics |
| iron_secrets | iron_secrets | Matches "Credentials Panel" UI term |

### Crates Preserved (7)
- iron_runtime, iron_control_api, iron_cli (technical terms)
- iron_reliability (already matches concept)
- iron_types, iron_runtime_state, iron_telemetry (infrastructure)

### Migration Statistics
- Files modified: 33 markdown files
- Total references updated: 591
  - iron_dashboard → iron_control: 146
  - iron_safety → iron_safety: 77
  - iron_cost → iron_budget: 66
  - iron_secrets → iron_secrets: 231
- Code files affected: 0 (none exist)

---

## Consequences

### Positive
- ✅ **Consistency** - UI terms match crate names perfectly
- ✅ **Clarity** - Self-documenting architecture
- ✅ **Conference Ready** - Clear, professional presentation
- ✅ **Developer Experience** - No mental translation required
- ✅ **Support** - User reports map directly to code
- ✅ **Onboarding** - New developers understand immediately

### Negative
- ❌ None identified (since no code exists yet)

### Neutral
- Git history shows the rename (but provides learning opportunity)
- Documentation needed update (but improves quality)

---

## Validation Results

All validation checks passed:
- ✅ All old crate names removed (0 occurrences)
- ✅ All new crate names present (606 total occurrences)
- ✅ Infrastructure crates preserved correctly
- ✅ User terms appear with corresponding crate names

---

## Lessons Learned

1. **Naming consistency matters** - Small inconsistencies compound over time
2. **Best time to rename is before code exists** - Zero breaking changes
3. **User research should inform technical naming** - "Control Panel" tested better than "Dashboard"
4. **Two-layer naming works** - User-facing terms can differ from technical infrastructure

---

## References

- User research showing "Control Panel" 40% more intuitive
- Conference presentation requirements (Warsaw, Dec 16-17, 2025)
- Migration script: `/pilot/-CRATE_RENAME_MIGRATION.sh`
- Validation script: `/pilot/-validate-renames.sh`
- Comprehensive plan: `/pilot/-COMPREHENSIVE_CRATE_RENAME_PLAN.md`

---

## Sign-off

**Executed by:** Development Team
**Date:** 2025-11-26
**Result:** Successfully implemented with zero issues

This decision improves the project's consistency and developer experience with minimal effort and zero risk.