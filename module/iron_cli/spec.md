# iron_cli - Specification

**Module:** iron_cli
**Layer:** 6 (Application)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

Binary CLI tool for Iron Cage token management, usage tracking, and limits. Provides authoritative implementation of token operations with iron_cli_py wrapper delegating to this binary (ADR-005).

---

## Scope

**In Scope:**
- Token management commands (create, list, revoke, show)
- Usage tracking commands (usage, limits)
- Trace viewing commands
- Configuration management
- Integration with iron_token_manager for operations

**Out of Scope:**
- Python CLI wrapper (see iron_cli_py)
- REST API (see iron_control_api)
- Dashboard UI (see iron_dashboard)
- Agent execution (see iron_runtime)

---

## Dependencies

**Required Modules:**
- iron_token_manager - Token operations
- iron_runtime_state - State persistence
- iron_telemetry - Logging

**Required External:**
- clap - CLI argument parsing
- unilang - Command definitions

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **Command Router:** Parses CLI args, routes to handlers
- **Token Operations:** Delegates to iron_token_manager
- **Output Formatter:** Tables and JSON output
- **Config Manager:** Handles CLI configuration

---

## CLI Architecture

**Layer Structure:**
- **Routing Layer:** 22 CLI commands → adapter function calls (src/bin/iron_token_unilang.rs)
- **Adapter Layer:** 22 adapter functions → Token Manager API endpoints (src/adapters/*.rs)
- **Handler Layer:** Business logic and output formatting (src/handlers/*.rs)

**Adapter Organization:**
- auth_adapters.rs: 3 adapters (login, logout, whoami)
- token_adapters.rs: 5 adapters (create, list, revoke, show, rotate)
- usage_adapters.rs: 4 adapters (show, by_project, by_provider, export)
- limits_adapters.rs: 5 adapters (list, show, create, update, delete)
- traces_adapters.rs: 3 adapters (list, show, export)
- health_adapters.rs: 2 adapters (health, status)

**Migration Hardening:**

All adapters verified to have valid API endpoints. Orphaned adapters (functions calling non-existent endpoints) eliminated through migration process (28→22 adapters, 6 deleted).

**Negative Criteria (Zero-Tolerance Checks):**
- NC-R.1: Zero routes calling orphaned adapters
- NC-A.1: Zero orphaned adapters exist in codebase
- NC-M.1/2/3: Migration metrics at target (0% orphaned, 100% correct)

**Test Coverage:**
- Routing tests (tests/routing/): Verify NC-R.1, all commands route correctly
- Adapter coverage tests (tests/adapters/coverage.rs): Verify NC-A.1, adapter counts
- Migration metrics tests (tests/migration/): Verify NC-M.1/2/3, trajectory correctness
- Manual testing (tests/manual/): 7 categories, 15+ test cases for real API integration
- Integration tests (tests/integration/): Command-level integration tests (207 minimum, 3 per command)
- Parameter tests (tests/integration/parameters/): Parameter-level validation tests (2,251 minimum)

**Integration Test Infrastructure:**
- TestServer: Real Axum HTTP server on random port for integration testing (✅ Implemented)
- TestData: Real SQLite database fixtures with SQL inserts (✅ Implemented)
- IntegrationTestHarness: Real CLI binary execution via process spawn (✅ Implemented)
- Infrastructure tests: 15 tests passing

**Test Quality Gates:**
- Command coverage: 100% (69/69 commands must have integration tests)
- Parameter coverage: 100% (250/250 parameters must have validation tests)
  - Phase 0 (Infrastructure): ✅ Complete (8 infrastructure tests passing)
  - Phase 1 (String params): 🔄 In Progress - Basic Coverage Complete (190 total tests: 171 passing, 19 RED failures)
    - Parameters covered (25/25): ✅ ALL unique string parameters tested
      - Batch 1: format (11), token_id (8), id (8), project (6), description (7), name (9), username (7), password (7), email (7), role (5)
      - Batch 2: api_key (7), endpoint (9), provider (7), export_format (6), output (6), project_id (6), status (4)
      - Batch 3: period (7), provider_id (7), provider_ids (8), agent_id (7), agent_ids (8), output_file (7), message (7), new_password (8)
    - Test results: 190 total (8 infra + 182 param), 171 passing, 19 RED failures (expected in TDD RED phase)
    - Progress: 25/25 string parameters tested (100% basic coverage), 182/850 tests created (21.4%)
    - Next: Expand to cross-command coverage (format: 68 cmds, id: 25 cmds, etc.) to reach 850-test target
  - Phase 2 (Integer params): Pending (420 tests planned)
  - Phase 3 (Attribute tests): Pending (705 tests planned)
  - Phase 4 (Subtype tests): Pending (226 tests planned)
- No mocking policy: All tests use real implementations (HTTP server, database, CLI binary)
- TDD enforcement: Tests written FIRST, then implementation (strict RED-GREEN-REFACTOR)

**Multi-Layer Defense:**
1. Syntactic: Compiler prevents calling deleted functions
2. Semantic: Routes map to correct API endpoints (404 prevention)
3. Architectural: Parameter alignment enforced
4. Operational: Process verification through tests

---

## Integration Points

**Used by:**
- iron_cli_py - Python wrapper delegates operations here (ADR-005)
- Developers - Direct CLI usage

**Uses:**
- iron_token_manager - Token CRUD operations
- iron_runtime_state - Data retrieval

---

*For detailed command specifications, see spec/-archived_detailed_spec.md*
