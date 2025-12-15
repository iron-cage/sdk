# Principles: Development Workflow

### Scope

This document defines the development workflow and process principles across the Iron Cage platform, integrating all previous principles (Design Philosophy 001, Quality Attributes 002, Error Handling 003, Testing Strategy 004) into a cohesive development process. This workflow ensures quality and consistency by prescribing when and how to apply each principle during feature development and bug fixes.

**In scope**:
- Six workflow principles (TDD cycle, spec-first approach, ADR-driven decisions, documentation throughout, file creation protocol, knowledge preservation)
- Test-Driven Development cycle (RED → GREEN → REFACTOR → VERIFY) implementing Testing Strategy 004
- Specification-first approach ensuring requirements clarity before coding
- ADR-driven architectural decision documentation for significant choices
- Documentation sites and timing (spec before coding, tests during RED, source during GREEN, docs/ after if architectural)
- File creation protocol with responsibility table validation preventing duplication
- Knowledge preservation hierarchy (test docs → source docs → documentation files → specifications)

**Out of scope**:
- Specific ADR content and decision rationale (see decisions/adr_*.md individual files)
- Detailed test documentation format specifications (see test_organization.rulebook.md)
- Bug fix workflow detailed requirements (see code_design.rulebook.md § Bug-Fixing Workflow)
- File structure and hierarchy rules (see files_structure.rulebook.md)
- Codebase hygiene standards (see codebase_hygiene.rulebook.md)
- Specific w3 tool commands and configuration (see w3 tool documentation)
- Detailed TDD workflow example (see features/005_token_management_implementation_plan.md)

### Purpose

**User Need:** Understand the required workflow for adding features, fixing bugs, and making architectural changes across the Iron Cage platform, ensuring all development work applies the established principles consistently.

**Solution:** Six workflow principles integrate all previous Principles documents into a cohesive development process:

1. **TDD Cycle (Red-Green-Refactor)** - Tests before implementation, implementing Testing Strategy 004
2. **Spec-First Approach** - Specification before coding, ensuring requirements clarity
3. **ADR-Driven Decisions** - Architectural decisions documented for significant choices
4. **Documentation Throughout** - Incremental documentation at each workflow stage (spec → tests → source → docs/)
5. **File Creation Protocol** - Responsibility table validation before creating files
6. **Knowledge Preservation** - Development insights captured immediately in appropriate sites

**Test-driven development with documentation throughout:**

```
Idea → ADR (if significant) → Spec Update → Tests (RED) → Implementation (GREEN) → Refactor → docs/ Update (if needed)
```

This workflow integrates Design Philosophy 001 (Simplicity First through minimal workflow steps, Observable Behavior through TDD verification), Quality Attributes 002 (Reliability through comprehensive testing, Performance through early optimization consideration), Error Handling 003 (Loud Failures in test assertions, Proper Fixes through bug fix documentation requirements), and Testing Strategy 004 (No Mocking through real implementations in TDD, Module-Owned Tests through tests/ directory organization, Loud Test Failures through explicit error matching).

**Key Insight:** Development workflow is not bureaucracy - it's a forcing function that ensures principles are applied at the right time. By prescribing when to write specs (before coding), when to write tests (before implementation), and when to document (throughout, not after), the workflow prevents common failure modes like retroactive test writing, missing specifications, and undocumented design decisions. Each workflow step enforces a specific principle at the moment it provides maximum value.

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13

---

### Workflow Principles

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

**Principle:** Check readme.md Responsibility Table before creating any file.

**Process:**
1. Open [directory]/readme.md
2. Check Responsibility Table (3 columns: ID, Entity, Responsibility)
3. Review existing file responsibilities
4. If your intended file responsibility matches existing file → Use that file, don't create new
5. If no match → Add row to table: `| NNN | **NNN_file_name.md** | Describe what file is responsible for |`
6. Commit readme.md update + new file together (atomic change)

**Responsibility Table Format:**
```markdown
| ID | Entity | Responsibility |
|----|--------|----------------|
| 001 | **001_file_name.md** | Brief description (key details in parentheses) |
```

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

### Cross-References

#### Related Principles Documents

- [001_design_philosophy.md](001_design_philosophy.md) - Design principles implemented through workflow (Simplicity First in minimal steps, Observable Behavior through TDD, Fail-Safe Defaults through spec-first)
- [002_quality_attributes.md](002_quality_attributes.md) - Quality attributes enforced through workflow (Reliability via TDD, Performance via early consideration, Security via review points)
- [003_error_handling_principles.md](003_error_handling_principles.md) - Error handling integrated in bug fix workflow (Loud Failures in assertions, Proper Fixes through documentation requirements)
- [004_testing_strategy.md](004_testing_strategy.md) - Testing strategy enforced through TDD cycle (No Mocking in real implementations, Module-Owned Tests in tests/ directories, Loud Test Failures in explicit assertions)

#### Used By

- All module development: Each crate follows this workflow for features and bug fixes
- Architecture documents: Architectural decisions follow ADR-Driven Decisions principle
- Protocol specifications: Protocol definitions follow Spec-First Approach principle
- Capability implementations: Capability development follows TDD Cycle and Documentation Throughout principles
- All developers: Workflow prescribes consistent development process across team
- Code review process: Reviews validate workflow compliance (spec present, tests first, documentation complete)

#### Dependencies

- Principles 001: [Design Philosophy](001_design_philosophy.md) - Simplicity First, Observable Behavior, Fail-Safe Defaults principles implemented in workflow
- Principles 002: [Quality Attributes](002_quality_attributes.md) - Quality targets enforced through workflow stages
- Principles 003: [Error Handling Principles](003_error_handling_principles.md) - Bug fix workflow integrates Loud Failures and Proper Fixes requirements
- Principles 004: [Testing Strategy](004_testing_strategy.md) - TDD cycle implements all three testing principles
- **ADR Collection:** [decisions/](../decisions/readme.md) - ADR-002, ADR-003, ADR-005, ADR-006, ADR-007 referenced as examples
- **Detailed TDD Example:** [features/005_token_management_implementation_plan.md](../features/005_token_management_implementation_plan.md) - Complete TDD workflow demonstration
- **Rulebook Standards:** code_design.rulebook.md (bug fix workflow), test_organization.rulebook.md (test documentation), files_structure.rulebook.md (file creation protocol), codebase_hygiene.rulebook.md (documentation quality)

#### Implementation

- TDD cycle enforced via code review checking test-before-implementation order
- Spec-first validated via spec.md file presence before feature implementation
- ADR-driven enforced for significant decisions (multiple approaches, long-term impact, multi-module effects)
- Documentation throughout validated via incremental documentation at each stage (spec → tests → source → docs/)
- File creation protocol enforced via responsibility table presence in directory readme.md files
- Knowledge preservation enforced via prioritized documentation sites (test docs highest priority, source docs second, documentation files third)
- w3 test command (l::3) used for comprehensive verification after implementation
