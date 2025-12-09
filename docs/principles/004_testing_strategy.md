# Testing Strategy

**Purpose:** Testing philosophy and approach across Iron Cage platform.

---

## User Need

Understand how testing is done and why mocking is forbidden.

## Core Idea

**Test with real implementations, not mocks:**

```
FORBIDDEN: Mocking
let mock_runtime = MockRuntime::new();
mock_runtime.set_response("fake");

REQUIRED: Real implementations
let runtime = Runtime::new("./test.db")?;
let response = runtime.call_llm("test")?;
```

**Rationale:** Mocks test mock behavior, not real behavior. Real implementations catch integration bugs.

## The Three Testing Principles

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

## Test Organization

**Per rulebook standards:**
- All tests in tests/ directory of each crate
- Integration tests test cross-module interactions
- Unit tests test module internals
- No disabled/ignored tests (fix or remove)

## Test Quality

| Requirement | Implementation |
|-------------|----------------|
| **Fast** | SQLite in-memory, async tests |
| **Isolated** | Each test independent, no shared state |
| **Deterministic** | No flaky tests, consistent results |
| **Complete** | Cover success + all failure modes |

---

*Related: [003_error_handling_principles.md](003_error_handling_principles.md) | [../decisions/adr_007_testing_philosophy.md](../decisions/adr_007_testing_philosophy.md)*
