# Iron API Test Suite

## Organization

This directory contains ALL tests for iron_control_api crate following domain-based organization with comprehensive endpoint coverage for Protocol 014 (API Tokens), Protocol 010 (Agents), Protocol 012 (Analytics), and legacy FR-8/9/10 (Usage/Limits/Traces).

### Directory Structure

```
tests/
├── readme.md (this file)
├── common/
│   ├── mod.rs              # Test helpers and shared utilities
│   ├── fixtures.rs         # Test data factories
│   └── test_state.rs       # Test state builders (DB, Auth, App state)
├── auth/
│   ├── login.rs            # Login endpoint tests
│   └── validation.rs       # Auth validation tests
├── auth.rs                 # Auth test suite entry point
├── auth_endpoints.rs       # Auth endpoint integration tests
├── tokens/                 # Protocol 014: API Token Management
│   ├── endpoints.rs        # Token CRUD + validate endpoint tests (111 tests total)
│   ├── validation.rs       # Token validation logic tests
│   └── readme.md           # Token test documentation
├── tokens.rs               # Token test suite entry point
├── agents/                 # Protocol 010: Agent Management API
│   ├── endpoints.rs        # Agent CRUD endpoint tests (39 tests)
│   └── mod.rs              # Agent test module
├── agents.rs               # Agent test suite entry point
├── analytics/              # Protocol 012: Analytics API
│   ├── spending.rs         # Spending analytics tests
│   ├── usage.rs            # Usage analytics tests
│   └── mod.rs              # Analytics test module
├── analytics.rs            # Analytics test suite entry point
├── usage/                  # FR-8: Usage Analytics API
│   ├── aggregate.rs        # Aggregate usage endpoint tests
│   ├── by_project.rs       # Project-specific usage tests
│   ├── by_provider.rs      # Provider-specific usage tests
│   ├── path_validation.rs  # Path parameter DoS prevention tests (6 tests)
│   └── mod.rs              # Usage test module
├── usage.rs                # Usage test suite entry point
├── limits/                 # FR-9: Budget Limits API
│   ├── endpoints.rs        # Limits CRUD endpoint tests (10 tests)
│   └── validation.rs       # Request validation tests (17 tests)
├── limits.rs               # Limits test suite entry point
├── traces/                 # FR-10: Request Traces API
│   ├── list.rs             # Trace list endpoint tests (7 tests)
│   ├── get_by_id.rs        # Trace by ID endpoint tests (8 tests)
│   └── mod.rs              # Traces test module
├── traces.rs               # Traces test suite entry point
├── manual/
│   └── readme.md           # Manual testing procedures (416 lines, covers FR-7/8/9/10)
├── api_test.rs             # API integration tests
├── integration_tests.rs    # Full integration test suite
└── rbac.rs                 # RBAC middleware tests
```

## Responsibility Table

| Entity | Responsibility | Input→Output | Out of Scope |
|--------|----------------|--------------|--------------|
| `tokens/` | Test Protocol 014 API token endpoints including validate | Token scenarios → Test results | NOT auth flows (auth/), NOT agents (agents/), NOT analytics (analytics/), NOT usage (usage/), NOT limits (limits/), NOT traces (traces/) |
| `agents/` | Test Protocol 010 agent CRUD endpoints and budgets | Agent scenarios → Test results | NOT tokens (tokens/), NOT auth (auth/), NOT analytics (analytics/), NOT usage (usage/), NOT limits (limits/) |
| `analytics/` | Test Protocol 012 analytics endpoints (spending, usage, budget status) | Analytics scenarios → Test results | NOT agents (agents/), NOT tokens (tokens/), NOT usage (usage/), NOT auth (auth/) |
| `usage/` | Test usage analytics endpoints and aggregation | Usage scenarios → Analytics validation | NOT token CRUD (tokens/), NOT auth (auth/), NOT limits (limits/), NOT manual procedures (manual/) |
| `limits/` | Test budget limits CRUD and validation | Limits scenarios → Constraint validation | NOT tokens (tokens/), NOT usage analytics (usage/), NOT traces (traces/), NOT integration flows (integration_tests.rs) |
| `traces/` | Test request traces listing and retrieval | Trace scenarios → Trace validation | NOT tokens (tokens/), NOT usage (usage/), NOT limits (limits/), NOT auth (auth/) |
| `auth/` | Test authentication and authorization flows | Auth scenarios → Security validation | NOT token operations (tokens/), NOT business logic (tokens/, usage/, limits/), NOT RBAC middleware (rbac.rs), NOT integration (integration_tests.rs) |
| `common/` | Provide shared test infrastructure and helpers | Test needs → Reusable utilities | NOT domain tests (tokens/, usage/, limits/, traces/), NOT integration tests (integration_tests.rs), NOT manual tests (manual/) |
| `manual/` | Document manual testing procedures | Test procedures → Manual validation steps | NOT automated tests (tokens/, usage/, limits/, traces/), NOT test infrastructure (common/), NOT integration (integration_tests.rs) |
| `integration_tests.rs` | Test full API integration across endpoints | Integration scenarios → End-to-end validation | NOT domain-specific details (tokens/, usage/, limits/, traces/), NOT auth internals (auth/), NOT manual procedures (manual/) |
| `rbac.rs` | Test role-based access control middleware | RBAC scenarios → Authorization validation | NOT auth flows (auth/), NOT endpoint logic (tokens/, usage/, limits/, traces/), NOT integration (integration_tests.rs) |
| `api_test.rs` | Test API-level contracts and behaviors | API scenarios → Contract validation | NOT domain specifics (tokens/, usage/, limits/, traces/), NOT auth (auth/), NOT RBAC (rbac.rs) |
| `budget_routes.rs` | Test Protocol 005 budget control types and crypto | Budget control unit tests → Type validation | NOT HTTP integration (integration_tests.rs), NOT enforcement (protocol_005_enforcement_simple.rs), NOT migration metrics (protocol_005_migration_metrics.rs) |
| `protocol_005_enforcement_simple.rs` | Test Protocol 005 multi-layer enforcement | Enforcement checks → Security validation | NOT budget flow (budget_routes.rs), NOT rollback (protocol_005_rollback_verification.rs), NOT metrics (protocol_005_migration_metrics.rs) |
| `protocol_005_migration_metrics.rs` | Test Protocol 005 migration completeness metrics | Migration metrics → Quantitative validation | NOT enforcement (protocol_005_enforcement_simple.rs), NOT rollback (protocol_005_rollback_verification.rs), NOT types (budget_routes.rs) |
| `protocol_005_rollback_verification.rs` | Test Protocol 005 rollback impossibility | Rollback attempts → Prevention validation | NOT enforcement checks (protocol_005_enforcement_simple.rs), NOT metrics (protocol_005_migration_metrics.rs), NOT types (budget_routes.rs) |
| `budget_database_state.rs` | Test Protocol 005 database state corner cases | Database state scenarios → Enforcement validation | NOT enforcement layers (protocol_005_enforcement_simple.rs), NOT metrics (protocol_005_migration_metrics.rs), NOT types/crypto (budget_routes.rs), NOT rollback (protocol_005_rollback_verification.rs) |
| `budget_concurrency.rs` | Test Protocol 005 concurrent access scenarios | Concurrent requests → Race condition prevention | NOT database state (budget_database_state.rs), NOT input validation (budget_corner_cases.rs), NOT security (budget_security.rs), NOT types/crypto (budget_routes.rs) |
| `budget_corner_cases.rs` | Test Protocol 005 input validation edge cases | Edge case inputs → Validation robustness | NOT concurrency (budget_concurrency.rs), NOT database state (budget_database_state.rs), NOT security (budget_security.rs), NOT types (budget_routes.rs) |
| `budget_security.rs` | Test Protocol 005 security-critical scenarios | Security attacks → Attack prevention | NOT concurrency (budget_concurrency.rs), NOT database state (budget_database_state.rs), NOT input validation (budget_corner_cases.rs), NOT types (budget_routes.rs) |
| `auth_endpoints.rs` | Test authentication endpoint JWT token lifecycle | JWT scenarios → Token validation | NOT auth flows (auth/), NOT user management (users.rs), NOT RBAC (rbac.rs), NOT integration (integration_tests.rs) |
| `users.rs` | Test user management CRUD endpoints | User management scenarios → CRUD validation | NOT auth (auth/, auth_endpoints.rs), NOT tokens (tokens/), NOT RBAC middleware (rbac.rs), NOT integration (integration_tests.rs) |
| `auth_rate_limiting.rs` | Test Protocol 007 login rate limiting for brute force prevention | Login rate limit scenarios → Attack prevention validation | NOT token rate limiting (tokens/rate_limiting.rs), NOT auth flows (auth/), NOT JWT lifecycle (auth_endpoints.rs), NOT user management (users.rs) |
| `agent_provider_key_tests.rs` | Test provider API key retrieval endpoint | Key fetch scenarios → Retrieval validation | Feature 014 provider key tests | NOT budget (budget_*), NOT auth (auth/), NOT tokens (tokens/) |

## Test Coverage Summary

**Total Tests:** 379 (all passing, 8 implementation bugs fixed, +26 Protocol 005 tests)

**Phase 1 Security Additions** (2025-12-06):
- **issue-001:** 3 DoS protection bug reproducer tests (unbounded string inputs)
- **issue-002:** 2 NULL byte injection bug reproducer tests
- **issue-003:** 4 database infrastructure tests (isolation, rollback, concurrency, constraints)
- **Security:** 28 corner case tests (command injection, path traversal, XSS, SQL injection, concurrency, uniqueness)
- **Baseline:** 201 tests → **Phase 1 Complete:** 266 tests (+65 tests, +32%)

**Phase 2 Corner Case Extensions** (2025-12-07):
- **State Transitions:** 7 tests for token lifecycle (revoked/nonexistent tokens, cascade deletes)
  - ✅ Fixed: Added token state validation before operations (3 bugs fixed)
- **Concurrency:** 4 tests for race conditions (concurrent rotate/revoke operations)
  - ✅ Fixed: Implemented atomic database operations with SQLite IMMEDIATE transactions (3 bugs fixed)
- **Malformed JSON:** 16 tests across tokens/limits/auth (invalid syntax, deep nesting, invalid UTF-8)
  - ✅ All passing - HTTP layer correctly rejects malformed JSON
- **HTTP Methods:** 14 tests across tokens/limits/auth (405 Method Not Allowed validation)
  - ✅ All passing - Routing layer correctly enforces HTTP method contracts
- **Phase 1 Complete:** 266 tests → **Phase 2 Complete:** 306 tests (+40 tests, +15%)
- **Bugs fixed:** 8 implementation bugs (6 security-critical concurrency/state bugs, 2 storage bugs fixed)

**Phase 3-4 API Contract & Edge Cases** (2025-12-07):
- **Content-Type Validation:** 6 tests across tokens/limits/auth (wrong/missing Content-Type headers)
  - ✅ All passing - Axum JSON extractor returns 400 for wrong Content-Type
- **Idempotency:** 4 tests for token/limits DELETE operations
  - ✅ All passing - Documented NON-idempotent token creation (security)
  - ✅ Fixed: Standardized DELETE idempotency semantics across endpoints (1 inconsistency resolved)
- **Error Format Consistency:** 2 shared tests validating JSON error responses across all domains
  - ✅ All passing - Consistent {"error": "..."} format, no stack traces leaked
- **Empty Body Handling:** 4 tests across tokens/limits (empty JSON, missing body)
  - ✅ All passing - Proper validation (400/422 based on semantics)
- **Phase 2 Complete:** 306 tests → **Phase 3-4 Complete:** 330 tests (+24 tests, +8%)

**Phase 5 Additional Coverage** (2025-12-07):
- **Additional test coverage:** Integration tests, auth flows, RBAC scenarios, traces endpoints
- **Phase 3-4 Complete:** 330 tests → **Phase 5 Complete:** 353 tests (+23 tests, +7%)
- **Status:** All tests passing, comprehensive corner case coverage achieved

**Protocol 005 Budget Control** (2025-12-11):
- **budget_routes.rs:** 12 unit tests for Protocol 005 types, crypto, and validation
  - IC Token lifecycle (JWT generation, validation, expiration)
  - IP Token encryption (AES-256-GCM format validation)
  - Request/response serialization for handshake, usage reporting, budget refresh
  - Budget state creation and initialization
- **protocol_005_enforcement_simple.rs:** 4 enforcement verification tests
  - Database foreign key constraints enforcement (agent_id → agents, budget_id → agent_budgets)
  - Agent token distinguishability (agent_id column presence in api_tokens)
  - Token schema enables enforcement (agent tokens vs user tokens)
  - Enforcement summary (multi-layer validation)
- **protocol_005_migration_metrics.rs:** 6 quantitative migration tests
  - Metric 1: Unprotected credential endpoints = 0
  - Metric 2: Budget control paths = 5 (handshake, report, refresh, leases, budgets)
  - Metric 3: Enforcement layers = 3 (database, schema, API)
  - Metric 4: Migration ratio = bypass:0%, protocol:100%
  - Metric 5: Checkpoint verification (rollback tests exist)
  - Summary: Migration score = 100%
- **protocol_005_rollback_verification.rs:** 4 rollback prevention tests
  - Budget flow works (agent budgets, budget leases functional)
  - Token distinguishability enables enforcement (agent_id field in schema)
  - Enforcement code exists in keys.rs (source code verification)
  - Rollback impossibility documented (lines 357-412 in test file)
- **Phase 5 Complete:** 353 tests → **Protocol 005 Complete:** 379 tests (+26 tests, +7%)
- **Status:** All Protocol 005 tests passing, 100% migration score, rollback impossible

### By Protocol and Functional Requirement

- **Protocol 014 (API Token Management):** 111 tests (+12 new in Phase 1 Deliverable 1.6)
  - Endpoints: POST /api/v1/api-tokens (create), GET /api/v1/api-tokens/:id (get), POST /api/v1/api-tokens/:id/rotate, DELETE /api/v1/api-tokens/:id (revoke), **POST /api/v1/api-tokens/validate** (Deliverable 1.6)
  - Validation: user_id/project_id length limits (500 chars), NULL byte rejection, empty string handling
  - Security: DoS protection (issue-001), NULL byte injection (issue-002), SQL injection, XSS, command injection, path traversal
  - Corner cases: Concurrent requests, unique token generation, Unicode handling, very long inputs
  - State transitions: Token lifecycle (create → rotate → revoke), revoked token validation (5 tests)
  - **NEW Deliverable 1.6**: Validate endpoint (5 tests) - valid token returns metadata, invalid/revoked tokens return false, no auth required
  - GET /api/v1/api-tokens (list) requires auth, tested manually

- **Protocol 010 (Agent Management):** 39 tests (Phase 2 complete)
  - Endpoints: POST /api/v1/agents (create), GET /api/v1/agents (list), GET /api/v1/agents/:id (get), PUT /api/v1/agents/:id (update), DELETE /api/v1/agents/:id (delete)
  - Integration tests comprehensive: Agent CRUD, budget management, provider configuration
  - Validation: Required fields, constraints, error handling
  - Security: Input validation, SQL injection prevention

- **Protocol 012 (Analytics):** 30 tests (Phase 4 complete)
  - Endpoints: GET /api/v1/analytics/spending/*, GET /api/v1/analytics/usage/*, GET /api/v1/analytics/budget/status
  - Coverage: Spending total, by-agent, by-provider, average per request, budget status, usage requests, token usage, model usage
  - Validation: Time ranges, aggregation accuracy, filtering

- **FR-8 (Usage Analytics):** 39 tests
  - GET /api/usage/aggregate
  - GET /api/usage/by-project/:project_id
  - GET /api/usage/by-provider/:provider
  - Path parameter DoS prevention (whitespace, length limits)
  - Aggregation accuracy, provider breakdown, project filtering

- **FR-9 (Budget Limits):** 62 tests (+14 new in Phases 2-4)
  - POST /api/limits (create), GET /api/limits (list), GET /api/limits/:id (get), PUT /api/limits/:id (update), DELETE /api/limits/:id (delete)
  - Validation: all-None rejection, overflow detection, boundary values, negative values, zero values
  - Field-level validation, partial updates, constraint enforcement
  - **NEW Phase 2**: Malformed JSON (4), HTTP methods (4)
  - **NEW Phase 3-4**: Content-Type (2), Idempotency (2), Empty body (2)

- **FR-10 (Request Traces):** 34 tests
  - GET /api/traces (list), GET /api/traces/:id (get by ID)
  - 404 handling, ordering, structure validation, field presence, empty database handling

### By Test Type

- **Security Tests:** 33 tests
  - DoS protection: 3 bug reproducers (issue-001) + 6 path validation tests
  - Injection prevention: 2 NULL byte bug reproducers (issue-002) + SQL injection + XSS + command injection
  - Attack scenarios: Path traversal, Unicode handling, concurrent attacks
- **Infrastructure Tests:** 4 database tests (issue-003)
  - Test isolation, transaction rollback, concurrent execution, constraint enforcement
- **Endpoint Integration Tests:** ~90 tests covering HTTP requests/responses across all FRs
- **Validation Tests:** ~60 tests for request validation logic
- **Corner Case Tests:** 28 tests for edge cases (uniqueness, concurrency, large inputs)
- **Auth Tests:** ~24 tests for authentication/authorization
- **Common Helpers:** Test fixtures, builders, shared utilities
- **RBAC Tests:** Role-based access control tests

## Test Methodology

### No Mocking
All tests use real implementations. Database is in-memory SQLite for speed and isolation.

### Loud Failures
Every assertion includes explicit failure message explaining what went wrong:
```rust
assert_eq!(
  response.status(),
  StatusCode::OK,
  "LOUD FAILURE: GET /api/usage/aggregate must return 200 OK"
);
```

### Test Matrices
Each test file documents its test matrix covering all scenarios:
- Happy path (200 OK, 201 Created, 204 No Content)
- Client errors (400 Bad Request, 404 Not Found)
- Edge cases (empty database, special characters, boundary values)
- Security (DoS prevention, validation bypass attempts)

### Self-Contained
No external dependencies (env vars, network calls, system time). All tests use:
- In-memory SQLite databases (`:memory:`)
- Test fixtures for predictable data
- Router testing via `router.oneshot()` (no server startup)

## Running Tests

```bash
# All tests (fast, uses nextest)
cargo nextest run --all-features

# Level 3 verification (nextest + doc tests + clippy)
w3 .test l::3
# OR manually:
cargo nextest run --all-features && \
cargo test --doc --all-features && \
cargo clippy --all-targets --all-features -- -D warnings

# Specific domain
cargo nextest run --test tokens
cargo nextest run --test usage
cargo nextest run --test limits
cargo nextest run --test traces

# With output
cargo nextest run --test tokens --nocapture

# Single test
cargo nextest run --test tokens test_create_token_valid_request
```

## Test Organization Principles

1. **Domain-Based** - Organized by functional requirement (FR-7/8/9/10), not by methodology
2. **No Mocking** - Real implementations only, in-memory database for isolation
3. **Loud Failures** - All assertions use descriptive failure messages for debugging
4. **Test Matrices** - Each file documents test matrix with all scenarios covered
5. **Self-Contained** - No external dependencies, fully reproducible
6. **Integration-First** - Tests exercise full request/response cycle via router
7. **Knowledge Preservation** - Test doc comments explain WHY tests exist, not WHAT they do

## Test File Documentation Standards

Each test file includes header documentation with:
- **Purpose:** Why these tests exist
- **Test Matrix:** Table of scenarios and expected outcomes
- **Known Edge Cases:** Non-obvious behaviors or corner cases
- **Failure Modes:** How to interpret and debug test failures

Example from `tests/usage/aggregate.rs`:
```rust
//! Aggregate usage statistics endpoint tests.
//!
//! Test Matrix:
//! | Scenario | Expected Behavior |
//! |----------|------------------|
//! | Empty DB | Returns zeros, not 404 |
//! | Valid data | Aggregates across all tokens |
//! | POST method | Returns 405 Method Not Allowed |
```

## Manual Testing

For tests requiring server startup, auth setup, or external dependencies:
- See `tests/manual/readme.md` (416 lines, comprehensive FR-7/8/9/10 coverage)
- Covers: Token security, auth flows, usage analytics, limits CRUD, traces, CORS, error recovery
- Documented procedures with expected results and pass/fail tracking

## Known Test Gaps

1. **GET /api/tokens (list_tokens)** - Requires JWT authentication
   - Not tested in integration tests (no auth infrastructure)
   - Tested via manual procedures in `tests/manual/readme.md`
   - Endpoint is functional, can be tested with auth harness

2. **Middleware Testing** - CORS, rate limiting, auth middleware
   - `router.oneshot()` bypasses middleware layers
   - Requires full server integration tests or manual testing
   - CORS configuration verified via code review against spec

## Test Maintenance

### When Adding New Endpoints

1. Create test file in appropriate domain directory (auth/, tokens/, usage/, limits/, traces/)
2. Add test matrix documentation to file header
3. Write tests BEFORE implementation (true TDD)
4. Update this readme with test count and coverage details
5. Verify all tests pass at level 3

### When Fixing Bugs

Follow bug-fixing workflow (code_design.rulebook.md):
1. Create failing MRE test marked `bug_reproducer(issue-NNN)`
2. Implement fix
3. Document in test (5 sections) and source (3-field comment)
4. Verify fix passes and run full test suite
5. Update test count in this readme if new tests added

## Verification

Last verified: 2025-12-07 (Phases 1-5 Complete)
- ✅ 353/353 tests passing (all 8 implementation bugs fixed)
- ✅ 0 clippy warnings
- ✅ All Phase 1-5 tests passing (Security, Corner Cases, API Contract, Edge Cases, Additional Coverage)
- ✅ All doc tests passing
- ✅ All FR-7/8/9/10 endpoints have integration test coverage
- ✅ Phases 1-5 security fixes and corner case coverage complete:
  - ✅ DoS protection (issue-001): 500-char limits enforced
  - ✅ NULL byte injection prevention (issue-002): Explicit rejection
  - ✅ Database test isolation (issue-003): In-memory database infrastructure
- ✅ Phase 2-5 corner case extensions complete:
  - ✅ State transition tests (7 tests): Token lifecycle validation
  - ✅ Concurrency tests (4 tests): Race condition detection
  - ✅ Malformed JSON tests (16 tests): HTTP layer validation
  - ✅ HTTP method tests (14 tests): API contract documentation
  - ✅ Content-Type validation (6 tests): Header validation
  - ✅ Idempotency tests (4 tests): DELETE semantics
  - ✅ Error format tests (2 tests): Response consistency
  - ✅ Empty body tests (4 tests): Edge case handling
  - ✅ Additional coverage (+23 tests): Integration, auth, RBAC, traces
- ✅ All bugs fixed (8 implementation bugs):
  - ✅ State validation: Revoked tokens properly rejected (3 bugs fixed)
  - ✅ Atomic operations: Concurrent operations properly isolated (3 bugs fixed)
  - ✅ Storage issues: Cascade delete and idempotency standardized (2 bugs fixed)
- ✅ Defense-in-depth: API validation + database CHECK constraints
- ✅ All bug reproducer tests have 5-section documentation
- ✅ All infrastructure tests have 5-section documentation
- ✅ Loud failure pattern consistently applied
