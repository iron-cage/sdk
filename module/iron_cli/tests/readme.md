# tests/

Contains all automated tests for iron_cage CLI.

---

## Responsibility Table

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| `handlers.rs` | Test pure handler business logic | Handler function calls → Validation results | 100 tests across 6 handler categories (auth, token, usage, limits, traces, health), pure function validation, no I/O | NOT adapter testing (adapters.rs), NOT integration testing (integration_test.rs), NOT configuration (config_test.rs) |
| `adapters.rs` | Test adapter layer with service integration | Adapter calls + InMemoryAdapter → Integration results | 110 tests across 22 commands, async I/O validation, service integration via InMemoryAdapter | NOT handler logic (handlers.rs), NOT end-to-end workflows (integration_test.rs), NOT configuration (config_test.rs) |
| `integration_test.rs` | Test complete end-to-end workflows | Workflow scenarios → System validation | Complete stack integration (handler + adapter + service + config + formatter), InMemoryAdapter for isolation | NOT handler-only testing (handlers.rs), NOT adapter-only testing (adapters.rs), NOT configuration-only (config_test.rs) |
| `config_test.rs` | Test configuration hierarchy and precedence | Config scenarios → Config validation | 13 tests for 6-level hierarchy (CLI, env, local temp, local, global, defaults), precedence order, validation | NOT handler logic (handlers.rs), NOT adapter logic (adapters.rs), NOT formatting (formatting.rs) |
| `formatting.rs` | Test universal output formatter | Format requests → Formatted output validation | 23 tests for 4 formats (table, expanded, json, yaml), formatter correctness | NOT handler logic (handlers.rs), NOT adapter logic (adapters.rs), NOT configuration (config_test.rs) |
| `api_parity.rs` | Verify CLI/API command parity | CLI commands + API endpoints → Parity validation | Count parity (all endpoints have CLI), operation parity, structure parity | NOT handler testing (handlers.rs), NOT integration testing (integration_test.rs), NOT formatting (formatting.rs) |

---

## Organization

Tests organized by functional area (CLI parsing, configuration, integration).

Flat structure maintained (< 20 test files expected).

## Running Tests

```bash
cd cli
cargo test --all-features
```

## Test Principles

- All tests in tests/ directory (NO #[cfg(test)] in src/)
- Real implementations only (NO mocking)
- Tests fail loudly (NO silent failures)
- Domain-based organization (NOT methodology-based)
