# tests/

Contains all automated tests for iron_cost.

## Responsibility Table

| File | Responsibility | Input→Output | Out of Scope |
|------|----------------|--------------|--------------|
| `budget_test.rs` | Test CostController budget tracking and enforcement | Budget operations → Validation | NOT pricing (pricing_test.rs), NOT conversions (converter_test.rs) |
| `converter_test.rs` | Test USD/micros conversion functions | Conversion inputs → Precision validation | NOT budget logic (budget_test.rs), NOT pricing (pricing_test.rs) |
| `pricing_test.rs` | Test PricingManager model pricing lookup | Model names → Cost calculations | NOT budget (budget_test.rs), NOT conversions (converter_test.rs) |

## Running Tests

```bash
cd cost
cargo test --all-features
```

## Test Principles

- All tests in tests/ directory (NO #[cfg(test)] in src/)
- Real implementations only (NO mocking)
- Tests fail loudly (NO silent failures)
- Domain-based organization (NOT methodology-based)
