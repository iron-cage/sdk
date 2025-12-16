# Iron Token Manager Test Suite

## Organization

This directory contains ALL tests for iron_token_manager crate following domain-based organization with comprehensive coverage of token lifecycle, cost calculation, usage tracking, rate limiting, and budget enforcement.

### Directory Structure

```
tests/
├── readme.md (this file)
├── token_generator.rs          # Token generation and hashing tests (16 tests)
├── token_storage.rs            # Token database operations tests (9 tests)
├── cost_calculator.rs          # Cost calculation tests (9 tests)
├── usage_tracker.rs            # Usage tracking tests (8 tests)
├── limit_enforcer.rs           # Limit enforcement tests (15 tests)
├── rate_limiter.rs             # Rate limiting tests (7 tests)
├── database_schema.rs          # Database schema validation tests (5 tests)
├── database_initialization.rs  # Database initialization tests (1 test)
├── seed_data_validation.rs     # Seed data validation tests (20 tests)
├── common/                     # Shared test utilities
├── fixtures/                   # Test data and reference docs
└── manual/                     # Manual testing procedures
```

## Responsibility Table

| File | Responsibility | Input→Output | Scope | Out of Scope |
|------|----------------|--------------|-------|--------------|
| `token_generator.rs` | Test cryptographic token generation and hashing | Generation requests → Tokens + Hashes | Token generation, Base64 encoding, SHA-256 hashing, hash verification, uniqueness, entropy, constant-time comparison, randomness distribution | NOT database storage (token_storage.rs), NOT token usage tracking (usage_tracker.rs), NOT cost calculation (cost_calculator.rs), NOT rate limiting (rate_limiter.rs) |
| `token_storage.rs` | Test token database CRUD operations | Token operations → Database state | Token creation, retrieval, verification, expiration, deactivation, deletion, hash storage, metadata management | NOT token generation (token_generator.rs), NOT usage tracking (usage_tracker.rs), NOT cost calculation (cost_calculator.rs), NOT schema creation (database_schema.rs) |
| `cost_calculator.rs` | Test token-to-cost conversion | Provider + Model + Tokens → Cost (cents) | OpenAI pricing, Anthropic pricing, Gemini pricing, unknown providers, zero tokens, cost accuracy | NOT usage recording (usage_tracker.rs), NOT token generation (token_generator.rs), NOT database operations (token_storage.rs) |
| `usage_tracker.rs` | Test usage event recording and retrieval | Usage events → Database records + Aggregations | Usage recording, time-based filtering, provider filtering, aggregation, cascade deletion, timestamp tracking | NOT cost calculation (cost_calculator.rs), NOT token storage (token_storage.rs), NOT rate limiting (rate_limiter.rs) |
| `limit_enforcer.rs` | Test usage limit enforcement | Limit scenarios → Allow/Deny decisions | Token quotas, request limits, cost caps, daily resets, project-level limits, create/update limits, unlimited checks | NOT rate limiting (rate_limiter.rs), NOT usage recording (usage_tracker.rs), NOT cost calculation (cost_calculator.rs), NOT database schema (database_schema.rs) |
| `rate_limiter.rs` | Test token bucket rate limiting | Request patterns → Rate decisions | Requests per second, burst handling, per-user isolation, project-level rates, quota tracking, recovery over time, zero quota rejection | NOT quota limits (limit_enforcer.rs), NOT usage tracking (usage_tracker.rs), NOT token operations (token_storage.rs), NOT cost calculation (cost_calculator.rs) |
| `database_schema.rs` | Test database schema correctness | Schema creation → Table validation | Table creation, column structure, indexes, foreign keys, CASCADE DELETE, uniqueness constraints, CHECK constraints | NOT business logic (limit_enforcer.rs, usage_tracker.rs), NOT token operations (token_storage.rs), NOT cost calculations (cost_calculator.rs), NOT rate limiting (rate_limiter.rs) |
| `database_initialization.rs` | Test database initialization and isolation | Database setup → Isolated test databases | Test database creation, schema application, fixture loading, test isolation, parallel test safety | NOT schema definition (database_schema.rs), NOT seed data validation (seed_data_validation.rs), NOT business logic tests |
| `seed_data_validation.rs` | Test seed data correctness and consistency | Seed script execution → Data validation | Seed data counts, user properties, provider keys, foreign key integrity, password hashes, consistent reproducibility | NOT seed implementation (src/seed.rs), NOT schema validation (database_schema.rs), NOT database initialization (database_initialization.rs) |
| `common/` | Provide shared test utilities and fixtures | Test utilities → Reusable test helpers | Test database creation, common assertions, test data builders, shared setup/teardown | NOT actual tests (test files), NOT production code (src/), NOT seed data implementation |
| `fixtures/` | Store test data and reference documentation | Test scenarios → Static test data | Seed data reference documentation, expected test outputs, sample payloads | NOT test implementations, NOT production fixtures, NOT dynamic test data generation |
| `manual/` | Document manual testing procedures | Manual test scenarios → Testing instructions | Manual verification scripts, layer verification procedures (task 1.3), operational testing | NOT automated tests (test files), NOT production procedures (docs/operations/)

## Test Coverage Summary

**Total Tests:** 129 (100% passing)

**Last Verified:** 2025-12-12
- ✅ 129/129 tests passing
- ✅ 0 clippy warnings
- ✅ All doc tests passing
- ✅ SHA-256 hashing for API tokens (high-entropy random values)
- ✅ Database schema with proper constraints and indexes
- ✅ Real implementations (no mocking)
- ✅ Loud failure patterns
- ✅ Bug reproducer test for BCrypt→SHA-256 migration (issue-bcrypt-revert)
- ✅ Token generator security tests (10K uniqueness, chi-squared randomness, constant-time comparison)
- ✅ Seed data validation tests (20 tests validating seed.rs correctness)

### By Functional Domain

- **Token Generation (`token_generator.rs`):** 16 tests
  - Token generation, Base64 encoding, minimum length, uniqueness, entropy
  - SHA-256 hashing (64 hex chars, deterministic)
  - Hash verification (correct/wrong hash validation)
  - Bug reproducer: BCrypt non-determinism issue (issue-bcrypt-revert)
  - Security tests: 10K uniqueness, chi-squared randomness distribution, constant-time comparison
  - Edge cases: Unique token generation (100 iterations), prefix support

- **Token Storage (`token_storage.rs`):** 9 tests
  - CRUD operations: Create, retrieve, update, delete, deactivate
  - Hash storage (never plaintext)
  - Token verification, expiration handling
  - Metadata management (user_id, project_id, name)
  - Edge cases: List by user, last-used timestamp updates

- **Cost Calculator (`cost_calculator.rs`):** 9 tests
  - OpenAI pricing (GPT-4 Turbo, GPT-3.5 Turbo)
  - Anthropic pricing (Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku)
  - Gemini pricing (Gemini 1.5 Pro, Gemini 1.5 Flash)
  - Provider/model enumeration
  - Edge cases: Unknown providers, unknown models, zero tokens

- **Usage Tracker (`usage_tracker.rs`):** 8 tests
  - Usage recording with timestamps
  - Time-based filtering, provider filtering
  - Aggregation across tokens
  - Cascade deletion on token deletion
  - Edge cases: Multiple records, cost tracking, timestamp verification

- **Limit Enforcer (`limit_enforcer.rs`):** 15 tests
  - Limit creation, updates
  - Token quota checks, request limit checks, cost cap checks
  - Daily token resets
  - Project-level limits
  - Unlimited (no limit set) handling
  - Edge cases: Within limits, exceeding limits, increment operations

- **Rate Limiter (`rate_limiter.rs`):** 7 tests
  - Token bucket algorithm
  - Requests per second enforcement
  - Burst handling, per-user isolation
  - Project-level rate limiting
  - Quota tracking, recovery over time
  - Edge cases: Zero quota rejection, rate limit recovery

- **Database Schema (`database_schema.rs`):** 5 tests
  - Table creation (api_tokens, token_usage, usage_limits)
  - Foreign key constraints and CASCADE DELETE behavior
  - Uniqueness constraints (token_hash, usage_limits per user/project)

- **Database Initialization (`database_initialization.rs`):** 1 test
  - Test database creation with schema and fixtures
  - Isolated test databases for parallel test safety
  - Proper cleanup and resource management

- **Seed Data Validation (`seed_data_validation.rs`):** 20 tests
  - User count and properties (admin, demo, viewer, tester, guest)
  - Provider key count and validation (OpenAI, Anthropic)
  - Foreign key integrity verification
  - Password hash consistency (bcrypt cost=12)
  - API token relationships and deactivation states
  - Edge cases: Inactive users, token limits, reproducibility

## Test Methodology

### No Mocking
All tests use real implementations:
- Real SQLite databases (temporary files via `tempfile::TempDir`)
- Real cryptographic functions (`sha2::Sha256`, `getrandom`)
- Real rate limiter (`governor` token bucket)
- Real budget tracker (`iron_cost::BudgetTracker`)

### Loud Failures
Every assertion includes explicit failure message:
```rust
assert_eq!(
  hash.len(), 64,
  "SHA-256 hash should be 64 hex characters"
);
```

### Test Matrices
Each test file documents corner cases and boundary conditions:
- Happy path (successful operations)
- Edge cases (empty inputs, zero values, maximum values)
- Error conditions (invalid inputs, missing data)
- State transitions (token creation → usage → expiration)
- Security (hash storage, SQL injection prevention)

### Self-Contained
No external dependencies:
- Temporary SQLite databases (no shared state)
- Explicit test data (no environment variables)
- Deterministic behavior (no network calls, no system time dependencies)

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
cargo nextest run --test token_generator
cargo nextest run --test cost_calculator
cargo nextest run --test usage_tracker

# With output
cargo nextest run --test token_storage --nocapture

# Single test
cargo nextest run --test token_generator test_hash_token_produces_sha256_hash
```

## Test Organization Principles

1. **Domain-Based** - Organized by functional domain (token generation, storage, cost calculation, etc.)
2. **No Mocking** - Real implementations only, temporary databases for isolation
3. **Loud Failures** - All assertions use descriptive failure messages
4. **Test Matrices** - Each file documents scenarios and edge cases
5. **Self-Contained** - No external dependencies, fully reproducible
6. **Integration-First** - Tests exercise real database operations, not mocked interfaces

## Test File Documentation Standards

Each test file includes header documentation with:
- **Purpose:** Why these tests exist
- **Implementation:** What real components are tested (no mocks)
- **Edge Cases:** Boundary conditions and corner cases covered

Example from `tests/token_generator.rs`:
```rust
//! Token generator tests
//!
//! Tests for cryptographically secure token generation.
//! Uses REAL cryptographic functions (no mocks).
```

## Manual Testing

Currently no manual testing required for iron_token_manager. All functionality is testable through automated tests with temporary databases and real implementations.

If manual testing becomes necessary:
- Create `tests/manual/` directory
- Document procedures in `tests/manual/readme.md`
- Follow test_organization.rulebook.md standards

## Known Test Gaps

None currently identified. All 129 tests passing with comprehensive coverage:
- ✅ Token generation and hashing (including security tests)
- ✅ Token storage CRUD operations
- ✅ Cost calculation for all major providers
- ✅ Usage tracking and aggregation
- ✅ Limit enforcement
- ✅ Rate limiting
- ✅ Database schema validation
- ✅ Database initialization and isolation
- ✅ Seed data validation (20 tests)

## Test Maintenance

### When Adding New Features

1. Consult this readme.md Responsibility Table to determine correct test file
2. If new domain emerges, create new test file and update Responsibility Table
3. Add test matrix documentation to test file header
4. Write tests BEFORE implementation (TDD)
5. Update test count in this readme
6. Verify all tests pass at level 3

### When Fixing Bugs

Follow bug-fixing workflow (code_design.rulebook.md):
1. Create failing MRE test marked `bug_reproducer(issue-NNN)`
2. Implement fix
3. Document in test (5 sections) and source (3-field comment)
4. Verify fix passes and run full test suite
5. Update test count if new tests added

### When Tests Fail

1. Check test output for "LOUD FAILURE" messages
2. Identify which domain is affected (token generation, storage, cost, etc.)
3. Review test file documentation for edge cases and known pitfalls
4. Never disable tests - fix them or remove them
5. If test reveals design flaw, document in module-level Known Pitfalls

## Verification

Last verified: 2025-12-12
- ✅ 129/129 tests passing
- ✅ 0 clippy warnings
- ✅ All doc tests passing
- ✅ All domains have comprehensive test coverage
- ✅ Real implementations (no mocking)
- ✅ Loud failure pattern consistently applied
- ✅ SHA-256 hashing correctly implemented for API tokens
- ✅ Database schema with proper constraints
- ✅ Token generator security tests (10K uniqueness, randomness, constant-time)
- ✅ Seed data validation (20 tests)
- ✅ Responsibility Table documents all 12 test files/directories
