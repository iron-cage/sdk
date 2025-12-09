# Development Workflow

**Purpose:** Development process principles ensuring quality and consistency.

---

## User Need

Understand the required workflow for adding features, fixing bugs, and making architectural changes.

## Core Idea

**Documentation-driven development with explicit decision tracking:**

```
Idea → ADR (if significant) → Spec → Implementation → Tests → Documentation Update
```

## Workflow Principles

### 1. Spec-First Approach

**Principle:** Write specification before implementation.

**Process:**
1. Identify need (feature request, bug report)
2. Write spec in module/*/spec.md
3. Review spec for completeness
4. Implement to spec
5. Verify implementation matches spec

**Rationale:** Spec clarifies requirements before code investment.

### 2. ADR-Driven Decisions

**Principle:** Document significant architectural decisions as ADRs.

**When to create ADR:**
- Multiple valid approaches exist
- Decision has long-term impact
- Choice affects multiple modules
- Trade-offs need explanation

**Current ADRs:**
- ADR-001: Two-Repository Split
- ADR-002: Rust-Python Boundary (PyO3)
- ADR-003: Client-Side Execution Primary
- ADR-004: Crate Renaming
- ADR-005: CLI Wrapper Architecture
- ADR-006: Package Consolidation
- ADR-007: Testing Philosophy (No Mocking)

### 3. Documentation Before Code

**Principle:** Update documentation atomically with code changes.

**For new features:**
1. Update spec/requirements.md (if needed)
2. Create/update Design Collection concept file
3. Update vocabulary.md with new terms
4. Implement feature
5. Update module spec.md and readme.md

**For bug fixes:**
1. Create failing test (bug_reproducer marker)
2. Fix bug
3. Document in test (5 sections: Root Cause, Why Not Caught, Fix Applied, Prevention, Pitfall)
4. Document in source (3 fields: Fix(issue-NNN), Root cause, Pitfall)

### 4. File Creation Protocol

**Principle:** Check readme.md before creating any file.

**Process:**
1. Open [directory]/readme.md
2. Check Responsibility Table Input→Output column
3. If your file's I/O matches existing row → Use that file, don't create new
4. If no match → Add row with File and Input→Output
5. Commit readme.md update + new file together (atomic change)

**Prohibited filenames:**
- utils.rs, helpers.rs, common.rs, misc.rs

### 5. Knowledge Preservation

**Principle:** Capture all development insights immediately.

**Priority (highest to lowest):**
1. Test file doc comments (most preferred)
2. Source code doc comments
3. Documentation files (docs/)
4. Specification (spec.md) for requirements

**Forbidden:**
- Main readme.md for knowledge (it's for onboarding only)
- Creating files without updating directory readme.md

---

*Related: [001_design_philosophy.md](001_design_philosophy.md) | [../decisions/](../decisions/)*
