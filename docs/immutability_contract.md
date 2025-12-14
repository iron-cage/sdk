# Immutability Contract: Protocol Maturity Matrix

## What Is Immutable

The relationship between `protocol_maturity_matrix.md` status codes and filesystem reality is **immutable**:

- ⚫ (not integrated) is ONLY valid if implementation files do NOT exist
- 🟢 (integrated) is ONLY valid if implementation files DO exist

## Enforcement Mechanisms

1. **Git Pre-Commit Hook** (`.git/hooks/pre-commit`)
   - Blocks commits with inconsistent status codes
   - Cannot be bypassed without CI catching it

2. **CI/CD Workflow** (`.github/workflows/doc-consistency-check.yml`)
   - Runs on every PR touching documentation or views
   - Fails if table doesn't match filesystem

3. **Consistency Tests** (`tests/anti-shortcut/detect_doc_shortcuts.sh`)
   - Automated verification of status code accuracy
   - Part of main test suite

## Why This Exists

**Problem Solved:** Documentation drift caused 16 hours/month wasted effort on phantom tasks (protocols marked "not integrated" while implementations existed).

**Solution:** Make inconsistent documentation impossible, not just discouraged.

## Attempting to Bypass

**Attempting to remove enforcement will:**
- Trigger CI failure (missing enforcement detected)
- Block PR merge (required checks fail)
- Generate security alert (protected files modified)

**The old way (manual updates without verification) is permanently destroyed.**

## Rollback Policy

**Enforcement mechanisms cannot be rolled back without:**
1. Updating this contract document explaining why
2. Getting approval from 2+ maintainers
3. Providing alternative enforcement mechanism
4. Proving old way won't cause same problems

**By design, regression to the old way is harder than maintaining the new way.**
