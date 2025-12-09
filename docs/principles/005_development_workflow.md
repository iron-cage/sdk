# Development Workflow

**Purpose:** Development process principles ensuring quality and consistency.

---

## User Need

Understand the required workflow for adding features, fixing bugs, and making architectural changes.

## Core Idea

**Test-driven development with documentation throughout:**

```
Idea → ADR (if significant) → Spec Update → Tests (RED) → Implementation (GREEN) → Refactor → docs/ Update (if needed)
```

**Key Principles:**
- **Tests before implementation** (TDD Red-Green-Refactor cycle)
- **Spec updated before coding** (requirements clarified upfront)
- **Documentation throughout** (spec/tests/source during development, docs/ after if needed)

## Workflow Principles

### 0. TDD Cycle (Red-Green-Refactor)

**Principle:** Write tests before implementation.

**Process:**
1. **RED:** Write failing test that defines expected behavior
   - Test describes what code should do (not what it does)
   - Test currently fails (feature not implemented yet)
   - Captures requirements in executable form
2. **GREEN:** Write minimal code to make test pass
   - Implement just enough to make test green
   - Don't add extra features or optimizations
   - Focus on making the test pass
3. **REFACTOR:** Improve code structure while keeping tests green
   - Clean up duplication
   - Improve naming and structure
   - Tests stay green throughout refactoring
4. **VERIFY:** Run `w3 .test l::3` to ensure all tests pass
   - Full test suite must be green
   - No broken tests allowed

**Rationale:**
- Test-first ensures tests define behavior (not confirm existing code)
- Forces complete edge case coverage upfront
- Code designed for testability from start
- Real validation (tests can actually fail, not just confirm implementation)

**See:** features/005_token_management_implementation_plan.md for detailed TDD workflow example.

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
- ADR-002: Rust-Python Boundary (PyO3)
- ADR-003: Client-Side Execution Primary
- ADR-005: CLI Wrapper Architecture
- ADR-006: Package Consolidation
- ADR-007: Testing Philosophy (No Mocking)

### 3. Documentation Throughout Development

**Principle:** Documentation is created incrementally during development, not deferred to end.

**Documentation Sites and When Updated:**

| Site | When Updated | What Gets Documented | Example |
|------|-------------|----------------------|---------|
| **Spec (module/*/spec.md)** | Step 3: BEFORE coding | Requirements, API contracts, behavior definitions | Functional requirements, API signatures |
| **Test docs (tests/*.rs)** | Step 4: DURING Tests (RED) | Test doc comments, bug fix 5 sections, knowledge capture | Root cause, prevention strategies, pitfalls |
| **Source docs (src/*.rs)** | Step 5: DURING Implementation (GREEN) | Why decisions made, design rationale, bug fix 3 fields | Fix(issue-NNN), root cause, pitfall |
| **Design Collections (docs/)** | Step 7: AFTER implementation | ONLY if new architectural concepts introduced | New protocols, capabilities, architecture patterns |
| **Vocabulary (docs/vocabulary.md)** | Step 7: AFTER implementation | ONLY if new domain terms introduced | New entities, processes, technical terms |

**Key Point:** When workflow says "docs/ Update" at end, this refers ONLY to docs/ collection files (step 7). Spec (step 3), test documentation (step 4), and source documentation (step 5) are already complete by that point.

**For new features:**
1. **Before coding:** Update module/*/spec.md (define requirements)
2. **During Tests:** Write test doc comments (capture expected behavior)
3. **During Implementation:** Write source doc comments (explain design decisions)
4. **After implementation (if needed):** Update docs/ Design Collections, vocabulary.md

**For bug fixes:**
1. **During Tests:** Create failing test (bug_reproducer marker)
2. **During Fix:** Implement fix
3. **During Documentation:**
   - Test file: 5 sections (Root Cause, Why Not Caught, Fix Applied, Prevention, Pitfall)
   - Source file: 3 fields (Fix(issue-NNN), Root cause, Pitfall)
4. **Quality requirement:** Specific, technical, actionable, traceable (not generic "fixed bug")

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
