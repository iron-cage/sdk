# Principles: Testing Strategy

### Scope

This document defines the testing philosophy and strategy across the Iron Cage platform, translating the Observable Behavior principle from Principles 001 into concrete testing patterns. These testing principles ensure all system behavior is validated through real implementations rather than mocked approximations, supporting the Reliability quality attribute from Principles 002.

**In scope**:
- Three testing principles (No Mocking, Module-Owned Tests, Loud Test Failures)
- Real implementation testing approach (SQLite in-memory, test providers, localhost services)
- Module-owned test organization (tests/ directories per crate)
- Test quality requirements (fast, isolated, deterministic, complete)
- Loud test failure patterns with actionable error messages
- Integration testing philosophy and cross-module interaction validation

**Out of scope**:
- Detailed test organization format specifications (see test_organization.rulebook.md)
- Bug fix workflow and test documentation requirements (see code_design.rulebook.md)
- Specific testing framework API documentation (see nextest, cargo test docs)
- CI/CD pipeline configuration and test execution automation (see Integration documentation)
- Performance testing and benchmarking strategies (see Performance quality attribute)
- ADR-007 detailed decision rationale and alternatives analysis (see decisions/adr_007_testing_philosophy.md)

### Purpose

**User Need:** Understand how testing is performed across the Iron Cage platform, why mocking is forbidden, and how the testing strategy ensures all system behavior is validated through real implementations.

**Solution:** Three fundamental testing principles govern all test implementation:

1. **No Mocking** - Use real implementations (SQLite in-memory, test providers, localhost services) instead of mocks to catch integration bugs
2. **Module-Owned Tests** - Each module owns its test strategy in its own tests/ directory without shared test utilities
3. **Loud Test Failures** - Tests fail clearly with actionable error messages showing exact mismatch details

These principles directly implement the Observable Behavior design principle from Principles 001, ensuring all system behavior is validated and visible through comprehensive testing. Real implementations catch integration bugs, version mismatches, and real-world edge cases that mocks hide. Module-owned tests enforce decentralized ownership without central mocking frameworks. Loud failures align with the Loud Failures error principle from Principles 003.

**Test with real implementations, not mocks:**

```
FORBIDDEN: Mocking
let mock_runtime = MockRuntime::new();
mock_runtime.set_response("fake");

REQUIRED: Real implementations
let runtime = Runtime::new("./test.db")?;
let response = runtime.call_llm("test")?;
```

**Key Insight:** Testing is not about code coverage metrics - it's about validating real system behavior under real conditions. Mocks create false confidence by testing mock behavior instead of actual implementation behavior. By using real implementations (even in tests), every test validates actual integration contracts and catches real bugs that would otherwise appear in production.

**Status:** Specification
**Version:** 1.0.0
**Last Updated:** 2025-12-13

---

### The Three Testing Principles

### 1. No Mocking (ADR-007)

**Principle:** Use real implementations, not mocks.

| Component | Test With |
|-----------|-----------|
| Database | SQLite in-memory (fast + real) |
| LLM | Test provider or cached responses |
| Network | localhost services (real TCP) |
| Files | tempfile crate (real filesystem) |

**Why:** Mocks hide integration bugs, version mismatches, real-world edge cases.

### 2. Module-Owned Tests

**Principle:** Each module tests itself in its own tests/ directory.

```
module/iron_cost/
├── src/lib.rs
└── tests/
    ├── budget_tracking_test.rs
    ├── cost_calculation_test.rs
    └── integration_test.rs
```

**No shared test utilities** (removed iron_testing per ADR-007)
**Rationale:** Module owns its test strategy, no central mocking framework

### 3. Loud Test Failures

**Principle:** Tests must fail clearly with actionable errors.

```rust
// BAD: Silent pass
assert!(result.is_ok());

// GOOD: Loud failure with context
assert_eq!(
  result.unwrap_err(),
  Error::BudgetExceeded { spent: 10.5, limit: 10.0 }
);
```

### Test Organization

**Per rulebook standards:**
- All tests in tests/ directory of each crate
- Integration tests test cross-module interactions
- Unit tests test module internals
- No disabled/ignored tests (fix or remove)

### Test Quality

| Requirement | Implementation |
|-------------|----------------|
| **Fast** | SQLite in-memory, async tests |
| **Isolated** | Each test independent, no shared state |
| **Deterministic** | No flaky tests, consistent results |
| **Complete** | Cover success + all failure modes |

---

### Cross-References

#### Related Principles Documents

- [001_design_philosophy.md](001_design_philosophy.md) - Observable Behavior principle that testing strategy implements through comprehensive validation
- [002_quality_attributes.md](002_quality_attributes.md) - Reliability quality attribute validated and enforced by testing strategy
- [003_error_handling_principles.md](003_error_handling_principles.md) - Loud Failures principle implemented in test assertion patterns
- [005_development_workflow.md](005_development_workflow.md) - TDD cycle and bug fix workflow integrating testing strategy

#### Used By

- Architecture 002: [Layer Model](../architecture/002_layer_model.md) - Testing strategy validates layer isolation and interaction contracts
- Protocol: All API specifications include test coverage demonstrating behavior validation
- Capabilities: All capability specifications demonstrate testing approach with real implementations
- Security: Threat model validation through integration tests without mocks
- All module test suites: Each crate implements testing principles in its tests/ directory

#### Dependencies

- Principles 001: [Design Philosophy](001_design_philosophy.md) - Observable Behavior principle foundational to testing visibility
- Principles 002: [Quality Attributes](002_quality_attributes.md) - Reliability attribute enforced by testing strategy
- Principles 003: [Error Handling Principles](003_error_handling_principles.md) - Loud Failures pattern used in test assertions
- **ADR-007:** [Testing Philosophy](../decisions/adr_007_testing_philosophy.md) - Detailed decision rationale for no-mocking approach and alternatives analysis
- **Rulebook Standards:** test_organization.rulebook.md format requirements, code_design.rulebook.md bug fix workflow

#### Implementation

- No mocking enforced via code review and absence of mocking frameworks in dependencies
- Module-owned tests validated via tests/ directory presence in all crates
- Loud test failures enforced via assert_eq! patterns with explicit error matching
- Real implementations: SQLite in-memory (database), test providers (LLM), localhost services (network), tempfile crate (filesystem)
- Integration tests in tests/ directories validate cross-module interactions without mocks
