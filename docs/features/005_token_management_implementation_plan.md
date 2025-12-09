# LLM Token Management - Comprehensive Implementation Plan

**Version:** 3.2.0
**Date:** 2025-12-02
**Status:** Planning document (TDD enforced, no mocks, CLI/API parity)
**Architecture:** [002_token_management.md](002_token_management.md)
**Validation Framework:** [006_token_management_validation_framework.md](006_token_management_validation_framework.md)
**CLI/API Parity:** [004_token_management_cli_api_parity.md](004_token_management_cli_api_parity.md)
**Related Task:** [task/backlog/001_implement_llm_token_management_dashboard_and_backend.md](../../../task/backlog/001_implement_llm_token_management_dashboard_and_backend.md)

---

## Executive Summary

**Total Effort:** 75 days (11 weeks, ~385 person-hours)
**Recommended Start:** January 6, 2026 (post-pilot)
**Target Completion:** March 24, 2026
**Team Size:** 1-2 developers
**Risk Level:** Medium (new modules, cross-cutting concerns, CLI/API parity)

**Critical Decision:** This plan assumes **deferral to post-pilot** due to timeline conflict with December 17, 2025 deadline (23 days remaining vs 75 days required).

---

## Table of Contents

0. [Development Principles (MANDATORY)](#0-development-principles-mandatory)
1. [Project Overview](#1-project-overview)
2. [Timeline & Milestones](#2-timeline--milestones)
3. [Phase-by-Phase Breakdown](#3-phase-by-phase-breakdown)
4. [Risk Assessment & Mitigation](#4-risk-assessment--mitigation)
5. [Dependencies & Prerequisites](#5-dependencies--prerequisites)
6. [Resource Requirements](#6-resource-requirements)
7. [Quality Gates](#7-quality-gates)
8. [Rollback Strategy](#8-rollback-strategy)
9. [Success Criteria](#9-success-criteria)
10. [Post-Implementation](#10-post-implementation)
11. [Development Principles Summary (CRITICAL)](#11-development-principles-summary-critical)

---

## 0. Development Principles (MANDATORY)

### 0.1 Test-Driven Development (TDD)

**Every code change follows TDD red-green-refactor cycle:**

1. **RED:** Write failing test first (defines behavior)
2. **GREEN:** Write minimal code to make test pass
3. **REFACTOR:** Clean up code while keeping tests green
4. **VERIFY:** Run `w3 .test l::3` to ensure full green state

**Daily workflow:**
- Start each day: `w3 .test l::3` (verify green state)
- After each feature: `w3 .test l::3` (verify still green)
- End each day: `w3 .test l::3` (mandatory green state before commit)

**No code may be committed unless all tests pass.**

### 0.2 No Mocks - Real Implementation Testing

**Forbidden:**
- ❌ Mock API clients
- ❌ Mock database connections
- ❌ Mock LLM provider responses
- ❌ Fake token generation
- ❌ Stub secret managers

**Required:**
- ✅ Real database instances (PostgreSQL + SQLite for tests)
- ✅ Real HTTP requests (use actual provider APIs in integration tests)
- ✅ Real token generation (cryptographic RNG)
- ✅ Real secrets (test environment tokens/keys)
- ✅ Real rate limiting (actual timer-based token bucket)

**Integration test requirements:**
- Tests must use real OpenAI/Anthropic/Gemini API keys (from environment)
- Tests must make actual API calls (small, cheap requests)
- Tests must fail loudly if API keys missing (never skip/ignore)
- Test tokens stored in `.env.test` (gitignored, documented in readme)

### 0.3 Anti-Duplication Policy

**Before writing code, check for existing implementations:**

1. **Search for similar logic:**
   - `grep -r "similar_function_name" module/`
   - Check `iron_cost`, `iron_state`, `iron_api` for reusable code

2. **Consolidate or reference:**
   - If logic exists, use it (add dependency if needed)
   - If close but different, extract common parts to shared module
   - Document why duplication is necessary (if unavoidable)

3. **Forbidden patterns:**
   - ❌ Copy-pasting code between modules
   - ❌ Duplicating database queries
   - ❌ Re-implementing HTTP clients
   - ❌ Multiple implementations of same algorithm

**Quality gate enforcement:**
- Code review must verify no duplication
- Run `cargo clippy` to catch duplicated code patterns
- Document shared utilities in `module/*/spec.md`

### 0.4 Green State Enforcement

**Every phase ends with GREEN state:**

```bash
# Mandatory verification command
w3 .test l::3
# Expands to:
# - cargo nextest run --all-features (RUSTFLAGS="-D warnings")
# - cargo test --doc --all-features (RUSTDOCFLAGS="-D warnings")
# - cargo clippy --all-targets --all-features -- -D warnings
```

**Blocking quality gates:**
- ✅ All unit tests passing
- ✅ All integration tests passing
- ✅ All doc tests passing
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ No disabled/ignored tests

**If tests fail:**
1. Fix immediately (do not proceed to next stage)
2. Never disable/skip failing tests
3. Document root cause in test file
4. Add regression test if bug found

### 0.5 Real Token Testing Strategy

**Test environment setup:**

```bash
# .env.test (gitignored, template in .env.test.example)
OPENAI_API_KEY=sk-test-...
ANTHROPIC_API_KEY=sk-ant-...
GEMINI_API_KEY=...
TEST_DATABASE_URL=postgresql://localhost/token_mgmt_test
```

### 0.6 Workspace LLM API Crates (MANDATORY REUSE)

**CRITICAL: DO NOT re-implement LLM provider clients. Use existing workspace crates.**

**Available API crates** (located at `/home/user1/pro/lib/wip_iron/api_llm/dev`):

| Crate | Provider | Version | Features |
|-------|----------|---------|----------|
| `api_openai` | OpenAI | v0.3.0 | Retry, circuit breaker, rate limiting, caching, streaming |
| `api_claude` | Anthropic | v0.4.0 | Prompt caching, tool calling, vision, streaming |
| `api_gemini` | Google Gemini | Latest | Content caching, function calling, multimodal, streaming |
| `api_ollama` | Ollama (Local) | v0.2.0 | Local model inference |
| `api_xai` | xAI Grok | v0.3.0 | Latest Grok models |
| `api_huggingface` | HuggingFace | Latest | Inference API access |

**Key characteristics:**
- ✅ **No mocks policy**: "All tests use real API integration. No mocking allowed."
- ✅ **Enterprise reliability**: Retry, circuit breaker, rate limiting built-in
- ✅ **error_tools integration**: Uses same error handling framework
- ✅ **Stateless design**: Runtime-stateful, process-stateless (perfect for our use case)
- ✅ **Feature-gated**: Zero overhead when features disabled

**Usage in token management:**

```rust
// DO NOT create custom HTTP clients - use workspace crates
use api_openai::{ OpenAIClient, ChatRequest };
use api_claude::{ ClaudeClient, MessageRequest };
use api_gemini::{ GeminiClient, GenerateContentRequest };

// Token management wraps these clients for usage tracking
pub struct TrackedOpenAIClient
{
  client: OpenAIClient,
  usage_tracker: Arc< UsageTracker >,
}

impl TrackedOpenAIClient
{
  pub async fn chat_with_tracking( &self, request: &ChatRequest ) -> Result< ChatResponse >
  {
    // Use REAL api_openai client (no duplication)
    let response = self.client.chat( request ).await?;

    // Track usage (our responsibility)
    self.usage_tracker.record_usage(
      provider: "openai",
      tokens_used: response.usage.total_tokens,
      cost: calculate_cost( &response.usage ),
    ).await?;

    Ok( response )
  }
}
```

**Anti-duplication enforcement:**
- ❌ DO NOT implement HTTP clients for OpenAI/Claude/Gemini
- ❌ DO NOT implement retry logic (already in api_* crates)
- ❌ DO NOT implement rate limiting at HTTP level (already in api_* crates)
- ✅ DO add these crates as dependencies
- ✅ DO wrap them for usage tracking only
- ✅ DO reuse their testing patterns (real API integration)
- ✅ DO implement rate limiting at business logic level (token budget enforcement)

### 0.7 Workspace Utility Crates (MANDATORY REUSE)

**CRITICAL: Prefer workspace crates over external dependencies.**

**wTools workspace crates** (located at `/home/user1/pro/lib/wip_iron/wTools/dev/module/core`):

| Crate | Purpose | Replace External | Usage in Token Mgmt |
|-------|---------|------------------|---------------------|
| `error_tools` | Unified error handling (anyhow + thiserror facade) | `anyhow`, `thiserror` | Error types, Result aliases |
| `former` | Builder pattern with nested builders | `typed-builder`, `derive_builder` | Request/response builders |
| `mod_interface` | Module interface pattern | Manual `mod` + `pub use` | Clean module exports |
| `time_tools` | Time utilities (epoch, duration) | `chrono` partial | Timestamp tracking, TTL |
| `test_tools` | Testing utilities aggregator | `assert_eq!`, custom macros | Test organization, assertions |
| `derive_tools` | Derive macro utilities | Manual derives | Custom derives |
| `format_tools` | String formatting | `format!` extensions | Logging, display |
| `collection_tools` | Collection utilities | `vec!`, `hashmap!` manual | Collection construction |
| `async_tools` | Async utilities | Manual async patterns | Async helpers |
| `iter_tools` | Iterator utilities | `itertools` crate | Iterator operations |
| `workspace_tools` | Workspace utilities, secrets | Manual env var handling | API key management |
| `diagnostics_tools` | Debugging utilities | Manual debug | Diagnostics |
| `fs_tools` | File system utilities | `std::fs` wrappers | File operations |
| `process_tools` | Process utilities | `std::process` wrappers | Process management |

**willbe/iron_cage workspace** (already exists in iron_cage project):

| Crate | Purpose | Current Status | Integration |
|-------|---------|---------------|-------------|
| `iron_cost` | Cost tracking, budget management | ✅ Exists | Reuse for pricing calculations |
| `iron_state` | Database layer, migrations | ✅ Exists | Extend for token tables |
| `iron_api` | API framework, middleware | ✅ Exists | Add token endpoints |
| `iron_types` | Shared types | ✅ Exists | Use for common types |
| `iron_safety` | Safety utilities | ✅ Exists | Input validation |
| `iron_reliability` | Reliability features | ✅ Exists | Circuit breaker patterns |
| `iron_telemetry` | Telemetry, tracing | ✅ Exists | Usage tracking |
| `iron_runtime` | Runtime utilities | ✅ Exists | Runtime helpers |

**Dependency replacement strategy:**

```toml
# iron_token_manager/Cargo.toml

[dependencies]
# ❌ DON'T: Use external crates when workspace alternatives exist
# anyhow = "1.0"
# thiserror = "1.0"
# typed-builder = "0.18"

# ✅ DO: Use workspace crates
error_tools = { workspace = true }
former = { workspace = true }
mod_interface = { workspace = true }
time_tools = { workspace = true, features = ["enabled", "time_now"] }
test_tools = { workspace = true }
workspace_tools = { workspace = true, features = ["secrets"] }

# Workspace LLM API crates
api_openai = { version = "0.3.0", path = "../../api_llm/dev/api/openai" }
api_claude = { version = "0.4.0", path = "../../api_llm/dev/api/claude" }
api_gemini = { path = "../../api_llm/dev/api/gemini" }

# iron_cage crates (reuse existing infrastructure)
iron_cost = { workspace = true }
iron_state = { workspace = true }
iron_types = { workspace = true }
```

**Usage examples:**

```rust
// Error handling with error_tools (replaces anyhow + thiserror)
use error_tools::{ Result, err };

pub fn validate_token( token: &str ) -> Result< () >
{
  if token.is_empty()
  {
    return Err( err!( "Token cannot be empty" ) );
  }
  Ok( () )
}

// Builder pattern with former (replaces typed-builder)
use former::Former;

#[ derive( Former ) ]
pub struct TokenRequest
{
  user_id: i64,
  project_id: i64,
  provider: String,
  #[ former( default = 100 ) ]
  max_tokens: i32,
}

// Time utilities with time_tools (replaces chrono partially)
use time_tools::{ now, s };

let created_at = now();  // milliseconds since epoch
let expires_at = created_at + ( 3600 * 1000 );  // +1 hour

// Workspace secrets with workspace_tools
use workspace_tools::secrets::secret_env;

let api_key = secret_env( "OPENAI_API_KEY" )
  .expect( "OPENAI_API_KEY required" );
```

**Integration test pattern:**

```rust
#[tokio::test]
async fn test_real_openai_token_usage() {
  // NEVER mock - use real API
  let api_key = std::env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY required for integration tests");

  // Real API call (small/cheap request)
  let client = OpenAIClient::new(&api_key);
  let response = client.complete("test", 10).await.unwrap();

  // Verify real usage tracking
  assert!(response.usage.total_tokens > 0);
}
```

**Test must fail loudly if tokens missing:**
- Use `.expect()` not `.unwrap_or_default()`
- Never silently skip tests
- Document required env vars in test file header

---

## 1. Project Overview

### 1.1 Objectives

**Primary Goal:** Build production-ready LLM token management system with backend API and Vue.js dashboard.

**Key Deliverables:**
1. `iron_token_manager` crate (backend business logic)
2. Enhanced `iron_api` (REST endpoints + JWT auth)
3. Enhanced `iron_state` (database schema + migrations)
4. `iron_cli` binary (CLI tool with 24 commands, 100% API parity)
5. Vue 3 + TypeScript dashboard (4 views)
6. Comprehensive test suite (60% unit, 30% integration, 10% E2E + CLI/API parity tests)
7. Production deployment guide

### 1.2 Scope Boundaries

**In Scope:**
- Token generation/rotation/revocation
- Usage tracking per user/project/provider
- Hard limit enforcement with grace periods
- Rate limiting (requests per minute/hour/day)
- Dashboard UI with analytics
- JWT + RBAC authentication
- Security hardening

**Out of Scope:**
- Multi-tenancy isolation (future)
- Cost forecasting/predictions (future)
- Automated budget optimization (future)
- Mobile app (future)
- Third-party integrations beyond OpenAI/Anthropic/Gemini (future)

### 1.3 Success Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| API Latency (p95) | < 100ms | Load testing with k6 |
| Dashboard Load Time | < 2s | Lighthouse CI |
| Test Coverage | > 80% | cargo-tarpaulin |
| Security Vulnerabilities | 0 critical/high | cargo-audit + OWASP ZAP |
| Concurrent Requests | 10,000+ calls/min | Apache Bench |
| Token Uniqueness | 100% (collision-free) | Statistical testing |

---

## 2. Timeline & Milestones

### 2.1 High-Level Timeline

```
Week 1-2:  Phase 1 - Database Schema + Token Generation
Week 3-4:  Phase 2 - Usage Tracking
Week 5:    Phase 3 - Limits & Rate Limiting
Week 6-7:  Phase 4 - API Endpoints + Authentication + CLI Tool
Week 8-10: Phase 5 - Dashboard UI
Week 11:   Phase 6 - Security Hardening + Documentation
```

### 2.2 Milestone Gates

| Milestone | Date | Exit Criteria | Stakeholder Review |
|-----------|------|---------------|-------------------|
| M1: Database Ready | Week 2 End | Schema deployed, migrations tested, 20+ unit tests passing | Technical lead |
| M2: Backend Core Complete | Week 4 End | Token generation + usage tracking working, 50+ tests passing | Technical lead |
| M3: API + CLI Complete | Week 7 End | All 24 endpoints + 24 CLI commands functional, JWT auth working, CLI/API parity tests passing, 100+ tests passing | Technical lead + Product |
| M4: Dashboard Beta | Week 10 End | All 4 views functional, E2E tests passing | Product + Design |
| M5: Production Ready | Week 11 End | Security audit passed, load testing passed, docs complete | All stakeholders |

### 2.3 Parallel Work Streams

**Weeks 1-7:** Backend development (API + CLI, critical path)
**Weeks 6-10:** Frontend development (can start after API spec finalized)
**Weeks 1-11:** Documentation (continuous, not blocking)

```
Week:  1    2    3    4    5    6    7    8    9    10   11
──────────────────────────────────────────────────────────────
BE:   [==================================]
CLI:                           [========]
FE:                        [==========================]
Docs: [====================================================]
Test: [====================================================]
```

---

## 3. Phase-by-Phase Breakdown

### Phase 1: Database Schema + Token Generation (Weeks 1-2)

**Duration:** 10 working days
**Effort:** ~70 hours
**Risk:** Low (straightforward database work)

**TDD Enforcement:** Every code change follows red-green-refactor. Tests written before implementation.

#### Week 1: Database Schema & Migrations

**Day 1-2: Schema Design (TDD: RED phase)**
- [ ] **START:** Run `w3 .test l::3` (verify green baseline)
- [ ] Design database schema (5 tables: api_tokens, token_usage, usage_limits, api_call_traces, audit_log)
- [ ] Create ER diagram
- [ ] **Anti-duplication check:** Search for existing table definitions in `iron_state`
- [ ] Review schema with team
- [ ] Finalize column types, indexes, constraints
- [ ] **Write failing tests:** Schema validation tests (tables exist, columns correct, indexes present)
- [ ] **END:** Tests should FAIL (schema not created yet)

**Day 3-4: Migration Implementation (TDD: GREEN phase)**
- [ ] **START:** Run `w3 .test l::3` (verify tests still failing)
- [ ] Create SQLx migration files
- [ ] Implement up migrations (CREATE TABLE statements)
- [ ] Implement down migrations (DROP TABLE statements)
- [ ] Add indexes and constraints
- [ ] Test migrations on PostgreSQL (real database, no mocks)
- [ ] Test migrations on SQLite (real database, no mocks)
- [ ] **GREEN:** Run `w3 .test l::3` (all tests must pass)
- [ ] **END:** Verify green state before continuing

**Day 5: Storage Trait (TDD: RED → GREEN → REFACTOR)**
- [ ] **START:** Run `w3 .test l::3` (verify green state from Day 4)
- [ ] **RED:** Write failing tests for `TokenStorage` trait operations
- [ ] **Anti-duplication check:** Search for similar storage patterns in `iron_state`, `iron_cost`
- [ ] Define `TokenStorage` trait in `iron_state`
- [ ] **GREEN:** Implement PostgreSQL storage adapter (make tests pass)
- [ ] **GREEN:** Implement SQLite storage adapter (make tests pass)
- [ ] Add connection pooling configuration
- [ ] **REFACTOR:** Remove any code duplication between adapters
- [ ] Write integration tests using REAL databases (no mocks)
- [ ] **END:** Run `w3 .test l::3` (mandatory green state)

**Deliverables:**
- Migration files in `module/iron_state/migrations/`
- `TokenStorage` trait implementation
- 10+ integration tests

#### Week 2: Token Generator Implementation

**Day 6-7: Token Generator Core (TDD: RED → GREEN → REFACTOR)**
- [ ] **START:** Run `w3 .test l::3` (verify green state from Day 5)
- [ ] **RED:** Write failing unit tests for token generation (format, length, randomness)
- [ ] **Anti-duplication check:** Search for existing RNG/hashing utilities in codebase
- [ ] Create `iron_token_manager` crate skeleton
- [ ] Implement `TokenGenerator` struct
- [ ] **GREEN:** Add cryptographic RNG (use `rand` crate, REAL randomness, no mocks)
- [ ] **GREEN:** Implement SHA-256 hashing (real crypto, no fake hashes)
- [ ] **GREEN:** Add base64 encoding for tokens
- [ ] **REFACTOR:** Extract common crypto utilities if duplicated elsewhere
- [ ] **END:** Run `w3 .test l::3` (mandatory green state)

**Day 8-9: Token Operations (TDD: RED → GREEN → REFACTOR)**
- [ ] **START:** Run `w3 .test l::3` (verify green state from Day 7)
- [ ] **RED:** Write failing tests for token storage operations
- [ ] **RED:** Write failing tests for token lookup/verification
- [ ] **RED:** Write failing tests for rotation logic
- [ ] **RED:** Write failing tests for revocation logic
- [ ] **Anti-duplication check:** Check if `iron_state` has similar CRUD patterns
- [ ] **GREEN:** Implement token storage (hash insertion, REAL database)
- [ ] **GREEN:** Implement token lookup (hash verification, REAL database)
- [ ] **GREEN:** Implement token rotation logic
- [ ] **GREEN:** Implement token revocation logic
- [ ] **GREEN:** Add timestamp tracking (created_at, expires_at, revoked_at)
- [ ] **REFACTOR:** Consolidate repeated database query patterns
- [ ] **END:** Run `w3 .test l::3` (mandatory green state)

**Day 10: Integration & Testing (TDD: Integration + Performance)**
- [ ] **START:** Run `w3 .test l::3` (verify green state from Day 9)
- [ ] **RED:** Write failing integration tests: generate → store → lookup → revoke
- [ ] **GREEN:** Fix any integration issues (REAL end-to-end flow, no mocks)
- [ ] Statistical tests: verify token uniqueness (1M samples, REAL generation)
- [ ] Performance benchmarks: token generation speed (REAL crypto, not fake)
- [ ] Load testing: concurrent token generation (REAL database concurrency)
- [ ] **REFACTOR:** Optimize slow paths identified in benchmarks
- [ ] **END:** Run `w3 .test l::3` (mandatory green state, blocking gate)

**Deliverables:**
- `iron_token_manager/src/token_generator.rs`
- `iron_token_manager/src/storage.rs`
- 20+ unit tests
- Statistical uniqueness test
- Benchmark results

**Quality Gate (BLOCKING - must pass to continue):**
- ✅ All tests passing (`w3 .test l::3`) - GREEN state mandatory
- ✅ Zero mocks used (all tests use real databases, real crypto)
- ✅ Zero code duplication (verified via code review + clippy)
- ✅ Token generation: > 10,000 tokens/sec (measured with REAL crypto)
- ✅ Token collision probability: < 1 in 2^128 (statistical test with 1M samples)
- ✅ No disabled/ignored tests
- ✅ Code review approved

---

### Phase 2: Usage Tracking (Weeks 3-4)

**Duration:** 10 working days
**Effort:** ~70 hours
**Risk:** Medium (integration with iron_cost, provider SDKs, real API testing)

**TDD Enforcement:** Every code change follows red-green-refactor. Tests use REAL provider APIs.

#### Week 3: Usage Tracker Core

**Day 11-12: Cost Calculator (TDD: RED → GREEN → REFACTOR)**
- [ ] **START:** Run `w3 .test l::3` (verify green state from Phase 1)
- [ ] **RED:** Write failing unit tests for cost calculations (tokens → USD)
- [ ] **Anti-duplication check:** Check if `iron_cost` has similar pricing logic
- [ ] Create `CostCalculator` struct
- [ ] **GREEN:** Add provider pricing data (OpenAI, Anthropic, Gemini) - use REAL current prices
- [ ] **GREEN:** Implement cost calculation logic (input/output tokens → USD)
- [ ] **GREEN:** Add pricing versioning support (future price changes)
- [ ] **REFACTOR:** Extract common pricing patterns if duplicated
- [ ] Verify against actual provider pricing (compare with real invoices/docs)
- [ ] **END:** Run `w3 .test l::3` (mandatory green state)

**Day 13-14: Usage Tracker (TDD: RED → GREEN → REFACTOR)**
- [ ] **START:** Run `w3 .test l::3` (verify green state from Day 12)
- [ ] **RED:** Write failing tests for usage recording
- [ ] **RED:** Write failing tests for usage aggregation
- [ ] **Anti-duplication check:** Check for similar tracking patterns in `iron_cost`
- [ ] Implement `UsageTracker` struct
- [ ] **GREEN:** Implement `record_usage()` method (insert to token_usage table, REAL database)
- [ ] **GREEN:** Implement `aggregate_usage()` method (time-window aggregation)
- [ ] **GREEN:** Implement usage queries (by user, by project, by provider)
- [ ] **GREEN:** Add call tracing (insert to api_call_traces table, REAL database)
- [ ] **REFACTOR:** Consolidate duplicate query patterns
- [ ] **END:** Run `w3 .test l::3` (mandatory green state)

**Day 15: Integration with iron_cost (TDD: Integration)**
- [ ] **START:** Run `w3 .test l::3` (verify green state from Day 14)
- [ ] **Anti-duplication check:** Review existing `iron_cost` crate for reusable code
- [ ] **RED:** Write failing integration tests for cost tracking
- [ ] **GREEN:** Integrate `BudgetTracker` with `UsageTracker` (reuse existing code, no duplication)
- [ ] **GREEN:** Ensure consistent cost calculation (single source of truth)
- [ ] **GREEN:** Add usage metrics export
- [ ] **REFACTOR:** Remove any duplicated cost logic
- [ ] **END:** Run `w3 .test l::3` (mandatory green state)

**Deliverables:**
- `iron_token_manager/src/usage_tracker.rs`
- `iron_token_manager/src/cost_calculator.rs`
- Provider pricing configuration
- 30+ unit tests
- 5+ integration tests

#### Week 4: Provider Integration (USE WORKSPACE API CRATES)

**CRITICAL: Do NOT implement custom HTTP clients. Use existing `api_llm` workspace crates.**

**Day 16-17: Workspace API Crates Integration (TDD: RED → GREEN → REFACTOR)**
- [ ] **START:** Run `w3 .test l::3` (verify green state from Day 15)
- [ ] **WORKSPACE REUSE:** Add dependencies to `iron_token_manager/Cargo.toml`:
  - `api_openai = { version = "0.3.0", path = "../../api_llm/dev/api/openai" }`
  - `api_claude = { version = "0.4.0", path = "../../api_llm/dev/api/claude" }`
  - `api_gemini = { path = "../../api_llm/dev/api/gemini" }`
- [ ] **RED:** Write failing tests for usage tracking wrappers
- [ ] **ANTI-DUPLICATION:** Verify NO custom HTTP client implementation
- [ ] Create `TrackedClient` wrapper trait (wraps api_* clients)
- [ ] **GREEN:** Implement `TrackedOpenAIClient` (wraps `api_openai::OpenAIClient`)
- [ ] **GREEN:** Add usage extraction from `api_openai` response types
- [ ] **GREEN:** Record usage to `token_usage` table after each API call
- [ ] **NO MOCKS:** Integration tests use REAL api_openai client (small requests, ~$0.001 each)
- [ ] Tests must call `.expect("OPENAI_API_KEY required")` - fail loudly if missing
- [ ] **END:** Run `w3 .test l::3` (mandatory green state)

**Day 18-19: Claude & Gemini Integration (TDD: RED → GREEN → REFACTOR)**
- [ ] **START:** Run `w3 .test l::3` (verify green state from Day 17)
- [ ] **RED:** Write failing tests for Claude tracking wrapper
- [ ] **RED:** Write failing tests for Gemini tracking wrapper
- [ ] **ANTI-DUPLICATION:** Ensure wrapper pattern reuses common code (NO duplication)
- [ ] **GREEN:** Implement `TrackedClaudeClient` (wraps `api_claude::ClaudeClient`)
- [ ] **GREEN:** Implement `TrackedGeminiClient` (wraps `api_gemini::GeminiClient`)
- [ ] **GREEN:** Extract common tracking logic to shared trait implementation
- [ ] **VERIFY:** Retry/circuit breaker/rate limiting comes from `api_*` crates (DO NOT re-implement)
- [ ] **NO MOCKS:** Integration tests use REAL api_claude and api_gemini clients
- [ ] Tests must fail loudly if API keys missing (never skip)
- [ ] **REFACTOR:** Extract common tracking pattern to shared module
- [ ] **END:** Run `w3 .test l::3` (mandatory green state)

**Day 20: End-to-End Testing (TDD: Real Integration)**
- [ ] **START:** Run `w3 .test l::3` (verify green state from Day 19)
- [ ] Setup: Document required API keys in `.env.test.example`
- [ ] **NO MOCKS:** E2E test with REAL api_openai: generate token → make LLM call → verify usage recorded
- [ ] **NO MOCKS:** E2E test with REAL api_claude: same flow
- [ ] **NO MOCKS:** E2E test with REAL api_gemini: same flow
- [ ] Test cost calculation accuracy (compare with actual provider pricing docs)
- [ ] Test concurrent usage recording (100 requests/sec, REAL database)
- [ ] Performance testing: usage aggregation queries (REAL data)
- [ ] **VERIFY:** No custom HTTP client code exists (100% workspace crate reuse)
- [ ] **END:** Run `w3 .test l::3` (mandatory green state, blocking gate)

**Deliverables:**
- `iron_token_manager/src/tracked_client.rs` (wrapper trait, NOT HTTP client)
- `TrackedOpenAIClient`, `TrackedClaudeClient`, `TrackedGeminiClient` (wrappers only)
- `.env.test.example` (documents required API keys)
- Dependencies added for `api_openai`, `api_claude`, `api_gemini`
- 30+ unit tests
- 10+ integration tests (using REAL workspace API crates, no mocks)
- NO custom HTTP client code (verified via code review)

**Quality Gate (BLOCKING - must pass to continue):**
- ✅ All tests passing (`w3 .test l::3`) - GREEN state mandatory
- ✅ Zero mocks used (all integration tests use REAL workspace API crates)
- ✅ Tests fail loudly if API keys missing (never skip/ignore)
- ✅ Zero code duplication (100% workspace crate reuse - NO custom HTTP clients)
- ✅ Workspace API crates used: `api_openai`, `api_claude`, `api_gemini` (verified in Cargo.toml)
- ✅ Cost calculation accuracy: ±1% of actual provider pricing
- ✅ Usage recording latency: < 10ms (p95, measured with REAL database)
- ✅ No disabled/ignored tests
- ✅ Code review verifies NO custom HTTP client implementation exists
- ✅ Code review approved

---

### Phase 3: Limits & Rate Limiting (Week 5)

**Duration:** 5 working days
**Effort:** ~35 hours
**Risk:** Medium (complex validation logic, race conditions)

**TDD Enforcement:** Every code change follows red-green-refactor. Tests verify REAL concurrency behavior.

#### Week 5: Limit Enforcement & Rate Limiting

**Day 21-22: Limit Enforcer**
- [ ] Implement `LimitEnforcer` struct
- [ ] Add `check_limit()` method (validate-then-mutate pattern)
- [ ] Implement grace period logic
- [ ] Add notification triggers (80%, 90%, 100% thresholds)
- [ ] Add limit breach handling
- [ ] Write unit tests for all scenarios

**Day 23-24: Rate Limiter**
- [ ] Implement `RateLimiter` struct using `governor` crate
- [ ] Add token bucket algorithm
- [ ] Support multiple time windows (minute, hour, day)
- [ ] Add per-token rate limit state (DashMap)
- [ ] Implement rate limit exceeded handling (429 response)
- [ ] Write unit tests for rate limiting

**Day 25: Integration & Testing**
- [ ] Integration test: limit enforcement prevents overage
- [ ] Integration test: grace period allows temporary overage
- [ ] Integration test: rate limiting throttles requests
- [ ] Load test: concurrent limit checks (1000 requests/sec)
- [ ] Race condition testing: verify no budget overruns
- [ ] Run `w3 .test l::3` and verify all pass

**Deliverables:**
- `iron_token_manager/src/limit_enforcer.rs`
- `iron_token_manager/src/rate_limiter.rs`
- 20+ unit tests
- 10+ integration tests
- Race condition tests

**Quality Gate (BLOCKING - must pass to continue):**
- ✅ All tests passing (`w3 .test l::3`) - GREEN state mandatory
- ✅ Zero mocks used (REAL timer-based rate limiting, REAL concurrent access tests)
- ✅ Zero code duplication (shared validation patterns)
- ✅ Budget enforcement: 100% accurate (no overruns, tested with 1000 concurrent requests)
- ✅ Rate limiting accuracy: ±5% of configured limit (measured with REAL timing)
- ✅ No race conditions detected (verified with race detector, concurrent stress tests)
- ✅ No disabled/ignored tests
- ✅ Code review approved

---

### Phase 4: API Endpoints + Authentication + CLI Tool (Weeks 6-7)

**Duration:** 10 working days
**Effort:** ~70 hours
**Risk:** Medium (JWT security, RBAC complexity, CLI/API parity enforcement)

**TDD Enforcement:** API and CLI developed in parallel with parity tests. Tests use REAL HTTP requests, REAL JWT tokens.

#### Week 6, Days 26-30: REST API & Authentication

**Day 26: JWT Authentication**
- [ ] Add `jsonwebtoken` dependency to `iron_api`
- [ ] Implement `JwtAuth` middleware
- [ ] Add JWT signing/verification logic
- [ ] Implement access token (1hr) + refresh token (7 days)
- [ ] Add token blacklisting for logout
- [ ] Write unit tests for JWT operations

**Day 27: RBAC Authorization**
- [ ] Implement `RbacAuth` middleware
- [ ] Define roles: Admin, User, Agent
- [ ] Create authorization matrix
- [ ] Add role-based endpoint protection
- [ ] Implement permission checks
- [ ] Write unit tests for RBAC

**Day 28-29: API Endpoints**
- [ ] Implement authentication endpoints (login, refresh, logout)
- [ ] Implement token management endpoints (create, list, get, rotate, revoke)
- [ ] Implement usage analytics endpoints (aggregate, by-project, by-provider)
- [ ] Implement limits management endpoints (create, list, update, delete)
- [ ] Implement call tracing endpoints (query, get)
- [ ] Add health check endpoint

**Day 30: Testing & Documentation**
- [ ] Integration tests for all endpoints (happy paths)
- [ ] Integration tests for error cases (401, 403, 429, 500)
- [ ] Integration tests for RBAC enforcement
- [ ] Generate OpenAPI spec (use `utoipa` crate)
- [ ] Create Postman collection
- [ ] Run `w3 .test l::3` and verify all pass

#### Week 7, Days 31-35: CLI Tool Development

**Day 31: CLI Project Setup**
- [ ] Create `iron_cli` crate (binary)
- [ ] Add dependencies: clap 4.4+, reqwest 0.11+, tabled 0.14+, dialoguer 0.11+, keyring 2.2+
- [ ] Implement CLI structure with clap (`iron-token` binary)
- [ ] Create command groups: auth, tokens, usage, limits, traces, health, version
- [ ] Implement output formatters (table, JSON, CSV)
- [ ] Write CLI configuration management

**Day 32: Authentication Commands**
- [ ] Implement `iron-token auth login` (stores JWT in keyring)
- [ ] Implement `iron-token auth refresh`
- [ ] Implement `iron-token auth logout`
- [ ] Add credential storage via keyring crate
- [ ] Write unit tests for auth commands

**Day 33: Token & Usage Commands**
- [ ] Implement token commands: generate, list, get, rotate, revoke
- [ ] Implement usage commands: show, get, by-project, by-provider, export
- [ ] Add HTTP client wrapper (reqwest + JWT injection)
- [ ] Implement table formatting for list views
- [ ] Write unit tests for token/usage commands

**Day 34: Limits & Traces Commands**
- [ ] Implement limits commands: list, get, create, update, delete
- [ ] Implement traces commands: list, get, export
- [ ] Implement health and version commands
- [ ] Add interactive prompts for create/update operations
- [ ] Write unit tests for limits/traces commands

**Day 35: CLI/API Parity Testing**
- [ ] Write parity tests (count, operation, output, error)
- [ ] Verify all 24 API endpoints have CLI equivalents
- [ ] Test output parity (CLI JSON matches API JSON)
- [ ] Test error parity (same codes and messages)
- [ ] Add parity validation to CI/CD
- [ ] Run `w3 .test l::3` and verify all pass

**Deliverables:**
- `iron_api/src/middleware/jwt_auth.rs`
- `iron_api/src/middleware/rbac_auth.rs`
- `iron_api/src/routes/tokens.rs`
- `iron_api/src/routes/usage.rs`
- `iron_api/src/routes/limits.rs`
- `iron_api/src/routes/traces.rs`
- `iron_cli/src/main.rs` (CLI binary)
- `iron_cli/src/commands/` (24 CLI commands)
- OpenAPI specification
- Postman collection
- CLI user guide
- 30+ API integration tests
- 20+ CLI parity tests

**Quality Gate (BLOCKING - must pass to continue):**
- ✅ All tests passing (`w3 .test l::3`) - GREEN state mandatory
- ✅ Zero mocks used (API tests use REAL HTTP server, CLI tests use REAL API calls)
- ✅ Zero code duplication (API/CLI share common logic where possible)
- ✅ All 24 API endpoints functional (tested with REAL requests)
- ✅ All 24 CLI commands functional (tested with REAL API)
- ✅ JWT auth working correctly (REAL tokens, tested API + CLI)
- ✅ RBAC enforcement verified (integration tests with REAL role checks)
- ✅ CLI/API parity tests passing (100% coverage, automated verification)
- ✅ OpenAPI spec generated and validated
- ✅ No disabled/ignored tests
- ✅ Code review approved

---

### Phase 5: Dashboard UI (Weeks 8-10)

**Duration:** 15 working days
**Effort:** ~105 hours
**Risk:** Medium (frontend complexity, Vue.js learning curve)

**TDD Enforcement:** Component tests written first. E2E tests use REAL API (no mocks). Tests verify actual user workflows.

#### Week 8, Days 36-40: Project Setup & Authentication

**Day 36-37: Project Setup**
- [ ] Initialize Vue 3 + Vite project
- [ ] Configure TypeScript (strict mode)
- [ ] Add Tailwind CSS
- [ ] Install shadcn-vue components
- [ ] Configure Vue Router
- [ ] Install Pinia (state management)
- [ ] Install TanStack Query Vue
- [ ] Configure ESLint + Prettier

**Day 38-39: Authentication & Layout**
- [ ] Create login page
- [ ] Implement JWT storage (localStorage with security considerations)
- [ ] Create Pinia auth store
- [ ] Implement route guards (require authentication)
- [ ] Create main layout component (sidebar, header, footer)
- [ ] Add navigation menu
- [ ] Implement logout functionality

**Day 40: API Client**
- [ ] Create API client class
- [ ] Implement all API methods (tokens, usage, limits, traces)
- [ ] Add automatic JWT injection
- [ ] Add error handling (401 → redirect to login)
- [ ] Add request/response interceptors
- [ ] Write unit tests for API client

**Deliverables:**
- Vue 3 project structure
- Login/logout flow
- API client implementation
- Route guards
- Main layout

#### Week 9, Days 41-45: Core Views (Tokens & Usage)

**Day 41-42: Token Management View**
- [ ] Create TokensView component
- [ ] Implement tokens table (shadcn-vue Table)
- [ ] Add "Generate Token" modal
- [ ] Add "Rotate Token" action
- [ ] Add "Revoke Token" action with confirmation
- [ ] Implement copy-to-clipboard
- [ ] Add token metadata display
- [ ] Integrate with TanStack Query

**Day 43-44: Usage Analytics View**
- [ ] Create UsageView component
- [ ] Implement usage summary cards
- [ ] Add usage over time chart (Chart.js line chart)
- [ ] Add usage by provider chart (Chart.js pie chart)
- [ ] Add usage by project chart (Chart.js bar chart)
- [ ] Implement date range selector
- [ ] Add cost breakdown table
- [ ] Integrate with TanStack Query

**Day 45: Testing**
- [ ] Write component tests (Vitest)
- [ ] Write E2E tests for token management (Playwright)
- [ ] Write E2E tests for usage analytics (Playwright)
- [ ] Test responsive design (mobile, tablet, desktop)
- [ ] Run accessibility audit (axe-core)

**Deliverables:**
- TokensView component
- UsageView component
- Chart.js visualizations
- Component tests
- E2E tests

#### Week 10, Days 46-50: Remaining Views & Polish

**Day 46-47: Limits Management View**
- [ ] Create LimitsView component
- [ ] Implement limits table
- [ ] Add "Create Limit" form
- [ ] Add "Edit Limit" modal
- [ ] Add "Delete Limit" action with confirmation
- [ ] Implement grace period configuration
- [ ] Add period selection (hourly, daily, monthly)
- [ ] Integrate with TanStack Query

**Day 48-49: Call Tracing View**
- [ ] Create TracesView component
- [ ] Implement traces table with pagination
- [ ] Add call details drawer
- [ ] Implement filtering (provider, status, date range)
- [ ] Add CSV export functionality
- [ ] Add search functionality
- [ ] Integrate with TanStack Query

**Day 50: Polish & Testing**
- [ ] Add loading states (skeletons)
- [ ] Add error states (error boundaries)
- [ ] Add empty states (no data messages)
- [ ] Implement toast notifications (success, error)
- [ ] Run Lighthouse audit (performance, accessibility, SEO)
- [ ] Write E2E tests for limits and traces
- [ ] Fix all linting/type errors
- [ ] Run full E2E test suite

**Deliverables:**
- LimitsView component
- TracesView component
- Complete E2E test suite (20+ tests)
- Lighthouse audit report
- Production build

**Quality Gate (BLOCKING - must pass to continue):**
- ✅ All E2E tests passing (Playwright tests use REAL API, no mocks)
- ✅ All component tests passing (Vitest)
- ✅ Zero code duplication (shared components, utilities)
- ✅ Dashboard loads in < 2s (measured with Lighthouse)
- ✅ No console errors or warnings
- ✅ Lighthouse score: Performance > 90, Accessibility > 95, Best Practices > 90
- ✅ All 4 views functional (verified with E2E tests)
- ✅ No disabled/ignored tests
- ✅ Design review approved

---

### Phase 6: Security Hardening + Documentation (Week 11)

**Duration:** 5 working days
**Effort:** ~35 hours
**Risk:** Low (cleanup and documentation)

**TDD Enforcement:** Security tests verify REAL vulnerabilities. Load tests use REAL production-like scenarios.

#### Week 11, Days 51-55: Security & Documentation

**Day 51: Security Audit**
- [ ] Run OWASP ZAP scan on API
- [ ] Run cargo-audit on all crates
- [ ] Review JWT implementation (expiry, signature, claims)
- [ ] Review input validation (all endpoints)
- [ ] Review SQL injection prevention (prepared statements)
- [ ] Review XSS prevention (frontend sanitization)
- [ ] Review CSRF protection
- [ ] Document findings and fixes

**Day 52: Input Validation & Rate Limiting**
- [ ] Add JSON schema validation to all endpoints
- [ ] Add string sanitization for user inputs
- [ ] Implement rate limiting on auth endpoints (10/min)
- [ ] Add request size limits
- [ ] Add timeout configuration
- [ ] Write tests for validation logic

**Day 53: Documentation**
- [ ] Write API documentation (extend OpenAPI spec)
- [ ] Write deployment guide (Docker + K8s)
- [ ] Write developer guide (local setup, testing)
- [ ] Write user guide (dashboard usage, CLI usage)
- [ ] Document database schema
- [ ] Document security best practices
- [ ] Create architecture diagrams (update existing)

**Day 54: Load Testing**
- [ ] Create k6 load testing scripts
- [ ] Test token validation throughput (10K req/min)
- [ ] Test usage recording throughput (5K req/min)
- [ ] Test dashboard API load (2K req/min)
- [ ] Identify bottlenecks
- [ ] Optimize slow queries
- [ ] Document performance results

**Day 55: Final Review & Release**
- [ ] Run full test suite (`w3 .test l::5`)
- [ ] Run security scan (no critical/high vulnerabilities)
- [ ] Run load tests (all targets met)
- [ ] Review all documentation
- [ ] Create release notes
- [ ] Tag release (v1.0.0)
- [ ] Deploy to staging environment
- [ ] Final stakeholder demo

**Deliverables:**
- Security audit report
- API documentation (OpenAPI)
- Deployment guide
- Developer guide
- User guide
- Load testing results
- Release notes
- Staging deployment

**Quality Gate (BLOCKING - must pass to continue):**
- ✅ All tests passing (`w3 .test l::5`) - GREEN state mandatory (includes level 4-5: udeps + audit)
- ✅ Zero mocks used (security tests use REAL attack vectors, load tests use REAL traffic)
- ✅ Zero code duplication (verified across entire codebase)
- ✅ Security vulnerabilities: 0 critical/high (OWASP ZAP scan, cargo-audit)
- ✅ Load testing: all targets met (10K req/min token validation, 5K req/min usage recording)
- ✅ Documentation complete (API docs, CLI docs, deployment guide, user guide)
- ✅ No disabled/ignored tests in entire codebase
- ✅ Stakeholder approval (demo completed, sign-off received)

---

## 4. Risk Assessment & Mitigation

### 4.1 Technical Risks

| Risk | Probability | Impact | Mitigation Strategy |
|------|-------------|--------|-------------------|
| JWT token theft | Medium | High | Use httpOnly cookies, short expiry (1hr), refresh token rotation |
| Token collision | Very Low | Critical | Use 256-bit cryptographic RNG, statistical testing |
| Race condition in budget enforcement | Low | High | Use database transactions, validate-then-mutate pattern, write tests |
| Database migration failure | Low | Medium | Test migrations on staging, implement rollback scripts, backup data |
| Performance degradation under load | Medium | Medium | Load testing, database indexing, caching layer |
| Vue.js learning curve | Medium | Low | Allocate extra time for frontend, pair programming, code reviews |
| Provider API changes | Low | Medium | Version provider SDKs, add adapter abstraction layer |
| PostgreSQL vs SQLite differences | Low | Low | Test on both databases, use SQLx abstractions |

### 4.2 Project Risks

| Risk | Probability | Impact | Mitigation Strategy |
|------|-------------|--------|-------------------|
| Timeline slippage (pilot integration) | High | High | **DEFER to post-pilot** (primary mitigation) |
| Scope creep | Medium | Medium | Strict scope control, "future" backlog for enhancements |
| Developer availability | Low | High | Document all decisions, maintain clear README, pair programming |
| Dependency conflicts | Low | Medium | Lock dependency versions, test before upgrading |
| Security vulnerability discovered | Medium | High | Monthly cargo-audit, security scanning in CI/CD |

### 4.3 Mitigation Actions

**Before Start:**
- [ ] Confirm post-pilot timeline (Jan 6, 2026)
- [ ] Finalize scope (no changes after kickoff)
- [ ] Set up development environment
- [ ] Create backup schedule

**During Development:**
- [ ] Daily standup (async updates)
- [ ] Weekly progress review
- [ ] Run security scans weekly
- [ ] Test on both PostgreSQL and SQLite
- [ ] Document all design decisions

**After Completion:**
- [ ] Monitor production metrics
- [ ] Schedule security re-audit (monthly)
- [ ] Plan for future enhancements

---

## 5. Dependencies & Prerequisites

### 5.1 External Dependencies

**Infrastructure:**
- PostgreSQL 14+ (production database)
- SQLite 3.38+ (development/testing)
- Redis 7+ (optional, for session storage)
- Nginx/Caddy (reverse proxy)
- Docker + Docker Compose (development)
- Kubernetes (optional, production deployment)

**Rust Crates:**
- `sqlx` 0.7+ (database driver)
- `tokio` 1.35+ (async runtime)
- `axum` 0.7+ (HTTP framework)
- `jsonwebtoken` 9.2+ (JWT authentication)
- `governor` 0.6+ (rate limiting)
- `rand` 0.8+ (cryptographic RNG)
- `sha2` 0.10+ (SHA-256 hashing)
- `serde` 1.0+ (serialization)
- `error_tools` (workspace crate)

**Frontend Dependencies:**
- Node.js 20+ (build tooling)
- Vue 3.4+
- TypeScript 5.3+
- Vite 5+
- Vue Router 4+
- Pinia 2+
- TanStack Query Vue 5+
- Chart.js 4+
- shadcn-vue (latest)
- Tailwind CSS 3+

### 5.2 Internal Dependencies

**Must Complete Before Start:**
- [ ] Pilot project completed (Dec 17, 2025)
- [ ] `iron_cost` crate stable
- [ ] `iron_state` crate supports migrations
- [ ] `iron_api` crate supports middleware
- [ ] Development environment documented

**Nice to Have:**
- [ ] `iron_observability` crate (metrics export)
- [ ] `iron_secrets` crate (credential management)

### 5.3 Team Prerequisites

**Required Skills:**
- Rust (intermediate): async/await, traits, error handling
- Database design: PostgreSQL, SQLx, migrations
- REST API design: HTTP, JWT, RBAC
- Vue.js 3: Composition API, TypeScript
- Testing: unit, integration, E2E

**Optional Skills:**
- Security: OWASP Top 10, penetration testing
- DevOps: Docker, Kubernetes, CI/CD
- UI/UX: shadcn-vue, Tailwind CSS

---

## 6. Resource Requirements

### 6.1 Team Composition

**Option 1: Single Full-Stack Developer**
- Duration: 10 weeks (full-time)
- Pros: Clear ownership, no coordination overhead
- Cons: Knowledge concentration, slower progress
- Recommended for: Post-pilot implementation

**Option 2: Two Developers (Backend + Frontend)**
- Backend Developer: Weeks 1-6 (full-time), Weeks 7-10 (part-time)
- Frontend Developer: Weeks 5-10 (full-time)
- Pros: Parallel work streams, faster completion
- Cons: Coordination overhead, API contract negotiation
- Recommended for: Accelerated timeline

**Option 3: Three Developers (Backend + Frontend + DevOps)**
- Backend: Weeks 1-6
- Frontend: Weeks 5-10
- DevOps: Weeks 1-10 (part-time, infrastructure + CI/CD)
- Pros: Fastest completion, best quality
- Cons: High cost, coordination complexity
- Recommended for: Critical business priority

### 6.2 Infrastructure Costs

| Resource | Type | Monthly Cost | Notes |
|----------|------|--------------|-------|
| PostgreSQL (prod) | AWS RDS db.t3.medium | $80 | 2 vCPU, 4GB RAM |
| Redis (session) | AWS ElastiCache t3.micro | $15 | Optional |
| Application Server | AWS EC2 t3.medium | $35 | 2 vCPU, 4GB RAM |
| Load Balancer | AWS ALB | $25 | For HA setup |
| Domain + SSL | Route53 + ACM | $15 | DNS + certificates |
| **Total** | | **$170/mo** | Production environment |

**Development Environment:**
- Local Docker Compose: Free
- PostgreSQL container: Free
- SQLite: Free

### 6.3 Tool Licenses

| Tool | Cost | Purpose |
|------|------|---------|
| GitHub | Free (public repo) | Version control |
| Postman | Free tier | API testing |
| Lighthouse CI | Free | Performance monitoring |
| OWASP ZAP | Free | Security scanning |
| k6 | Free | Load testing |

---

## 7. Quality Gates

### 7.1 Code Quality Standards

**Rust Code:**
- No compiler warnings (RUSTFLAGS="-D warnings")
- All clippy lints pass (`cargo clippy -- -D warnings`)
- Code formatted with custom style (not `cargo fmt`)
- Test coverage > 80% (measured with cargo-tarpaulin)
- No `unsafe` code without justification
- All public APIs documented

**Vue.js Code:**
- No TypeScript errors (strict mode)
- ESLint passing (no errors, < 5 warnings)
- Prettier formatted
- Component tests for all components
- No console.log in production
- All components have type definitions

### 7.2 Testing Standards

**Unit Tests:**
- Every public function has at least 1 test
- Edge cases covered (empty input, null, overflow)
- Error cases tested
- Mock external dependencies

**Integration Tests:**
- All API endpoints tested (happy path + errors)
- Database transactions tested
- Authentication tested
- Authorization tested

**E2E Tests:**
- Critical user flows (token creation → usage → revocation)
- All 4 dashboard views tested
- Mobile responsive tested
- Error scenarios tested

### 7.3 Performance Standards

| Metric | Threshold | Action if Failed |
|--------|-----------|-----------------|
| API Latency (p50) | < 50ms | Profile slow queries, add caching |
| API Latency (p95) | < 100ms | Optimize database indexes |
| API Latency (p99) | < 200ms | Add rate limiting, scale horizontally |
| Dashboard Load | < 2s | Optimize bundle size, lazy loading |
| Database Query | < 10ms | Add indexes, optimize SQL |
| Test Suite Runtime | < 5min | Parallelize tests, remove flaky tests |

### 7.4 Security Standards

**OWASP Top 10 Compliance:**
- [ ] A01: Broken Access Control → RBAC enforced
- [ ] A02: Cryptographic Failures → TLS 1.3, SHA-256 hashing
- [ ] A03: Injection → Prepared statements, input validation
- [ ] A04: Insecure Design → Threat modeling completed
- [ ] A05: Security Misconfiguration → Security headers enabled
- [ ] A06: Vulnerable Components → cargo-audit passing
- [ ] A07: Authentication Failures → JWT with short expiry
- [ ] A08: Data Integrity Failures → HMAC signatures
- [ ] A09: Logging Failures → Audit logging implemented
- [ ] A10: SSRF → No user-controlled URLs

---

## 8. Rollback Strategy

### 8.1 Deployment Strategy

**Blue-Green Deployment:**
1. Deploy new version to "green" environment
2. Run smoke tests on green
3. Switch traffic to green (gradual, 10% → 50% → 100%)
4. Monitor error rates
5. If errors spike, switch back to blue

**Database Migrations:**
- Use forward-only migrations (no breaking changes)
- Deploy schema changes separately from code
- Test migrations on staging first
- Keep rollback scripts ready

### 8.2 Rollback Triggers

**Automatic Rollback:**
- Error rate > 5% for 5 minutes
- API latency (p95) > 500ms for 5 minutes
- Database connection failures > 10% for 1 minute

**Manual Rollback:**
- Security vulnerability discovered
- Data corruption detected
- Customer-facing bug (critical severity)

### 8.3 Rollback Procedure

**Step 1: Halt Deployment**
- [ ] Stop traffic switch (pause at current %)
- [ ] Alert team in Slack/Discord
- [ ] Document issue in incident log

**Step 2: Investigate**
- [ ] Check error logs
- [ ] Check metrics dashboard
- [ ] Identify root cause

**Step 3: Decision**
- If fixable in < 15 minutes → Apply hotfix
- If requires investigation → Rollback

**Step 4: Execute Rollback**
- [ ] Switch traffic back to blue (100%)
- [ ] Verify error rates return to normal
- [ ] Rollback database migrations (if needed)
- [ ] Post-mortem meeting (within 24 hours)

---

## 9. Success Criteria

### 9.1 Functional Acceptance

**Backend:**
- [ ] All API endpoints functional
- [ ] JWT authentication working
- [ ] RBAC authorization enforced
- [ ] Token generation: 10K+ unique tokens
- [ ] Usage tracking: accurate cost calculation
- [ ] Limit enforcement: 100% accurate (no overruns)
- [ ] Rate limiting: < 5% error margin

**Frontend:**
- [ ] All 4 views functional
- [ ] Token management CRUD operations work
- [ ] Charts display correctly
- [ ] Responsive design (mobile, tablet, desktop)
- [ ] No console errors
- [ ] Forms have validation

### 9.2 Non-Functional Acceptance

**Performance:**
- [ ] API latency (p95) < 100ms
- [ ] Dashboard load time < 2s
- [ ] Concurrent requests: 10K+ calls/min
- [ ] Database query time < 10ms

**Security:**
- [ ] No critical/high vulnerabilities (cargo-audit + OWASP ZAP)
- [ ] JWT tokens expire correctly
- [ ] RBAC prevents unauthorized access
- [ ] Input validation on all endpoints
- [ ] CSRF protection enabled

**Quality:**
- [ ] Test coverage > 80%
- [ ] All tests passing (`w3 .test l::5`)
- [ ] No compiler warnings
- [ ] No clippy warnings
- [ ] Documentation complete

### 9.3 User Acceptance

**Product Owner Checklist:**
- [ ] Token management workflow intuitive
- [ ] Usage analytics provide actionable insights
- [ ] Limits configuration is flexible
- [ ] Call tracing helps debugging
- [ ] Dashboard design matches brand
- [ ] All acceptance criteria from Task 001 met

---

## 10. Post-Implementation

### 10.1 Monitoring & Observability

**Metrics to Track:**
- API request rate (requests/sec)
- API error rate (%)
- API latency (p50, p95, p99)
- Database connection pool utilization (%)
- Token generation rate (tokens/hour)
- Usage tracking latency (ms)
- Dashboard page views
- User active sessions

**Alerts to Configure:**
- Error rate > 1% for 5 minutes
- API latency (p95) > 200ms for 5 minutes
- Database connection pool > 80% for 5 minutes
- Disk usage > 80%
- Security audit failures

### 10.2 Maintenance Plan

**Daily:**
- [ ] Check error logs for anomalies
- [ ] Review API metrics dashboard

**Weekly:**
- [ ] Run security scan (cargo-audit)
- [ ] Review database slow query log
- [ ] Check for dependency updates

**Monthly:**
- [ ] Security re-audit (OWASP ZAP)
- [ ] Performance review (compare against targets)
- [ ] Dependency updates (patch versions)
- [ ] Backup verification

**Quarterly:**
- [ ] Major dependency updates (minor versions)
- [ ] Load testing re-run
- [ ] Architecture review
- [ ] User feedback review

### 10.3 Future Enhancements

**Prioritized Backlog:**

**P0 (High Priority - Q2 2026):**
- Multi-tenancy isolation (tenant ID in all tables)
- Token usage forecasting (ML-based predictions)
- Automated budget alerts (email/Slack notifications)
- Webhook integration (notify external systems)

**P1 (Medium Priority - Q3 2026):**
- Cost optimization recommendations
- Token analytics dashboard (most used models, peak times)
- SSO integration (OAuth2, SAML)
- Mobile-responsive dashboard improvements

**P2 (Low Priority - Q4 2026):**
- Third-party integrations (Cohere, Mistral, etc.)
- Custom pricing models (volume discounts)
- Advanced analytics (cost attribution, chargebacks)
- GraphQL API (in addition to REST)

### 10.4 Documentation Updates

**As Code Evolves:**
- [ ] Update API documentation (OpenAPI spec)
- [ ] Update architecture diagrams
- [ ] Update deployment guide
- [ ] Update developer guide
- [ ] Document breaking changes

**Quarterly:**
- [ ] Review all documentation for accuracy
- [ ] Add FAQ section based on support tickets
- [ ] Update screenshots in user guide
- [ ] Add troubleshooting guide

---

## 11. Development Principles Summary (CRITICAL)

### 11.1 TDD Red-Green-Refactor Workflow

**Every single day of development follows this pattern:**

```
START OF DAY:
  └─> Run `w3 .test l::3` (verify GREEN baseline)

FOR EACH FEATURE:
  ├─> RED: Write failing test first
  ├─> GREEN: Write minimal code to pass test
  ├─> REFACTOR: Clean up while keeping tests green
  └─> VERIFY: Run `w3 .test l::3` (ensure still GREEN)

END OF DAY:
  └─> Run `w3 .test l::3` (mandatory GREEN before commit)
```

**No exceptions. No shortcuts. No "we'll add tests later."**

### 11.2 No Mocks - Real Implementation Only

**What this means in practice:**

| Component | ❌ FORBIDDEN (Mocks) | ✅ REQUIRED (Real) |
|-----------|---------------------|-------------------|
| Database | Mock DB / In-memory DB | PostgreSQL + SQLite (real instances) |
| LLM APIs | Mock responses / Fake clients | Real OpenAI/Anthropic/Gemini APIs (small requests) |
| HTTP Server | Mock HTTP / Fake responses | Real Axum server (integration tests) |
| JWT Tokens | Hardcoded tokens / Fake signing | Real jsonwebtoken library (crypto signing) |
| Rate Limiting | Fake timers / Mock clocks | Real governor crate (time-based) |
| Crypto RNG | Deterministic seeds / Fake random | Real cryptographic RNG (rand crate) |

**Environment setup required:**

```bash
# .env.test (gitignored, template in .env.test.example)
OPENAI_API_KEY=sk-proj-...          # Real API key (required)
ANTHROPIC_API_KEY=sk-ant-...        # Real API key (required)
GEMINI_API_KEY=AIza...              # Real API key (required)
TEST_DATABASE_URL=postgresql://localhost/token_mgmt_test  # Real DB

# Tests MUST fail loudly if missing:
let api_key = std::env::var("OPENAI_API_KEY")
  .expect("OPENAI_API_KEY required for integration tests");  // ✅ CORRECT

// ❌ FORBIDDEN:
let api_key = std::env::var("OPENAI_API_KEY")
  .unwrap_or("fake-key");  // NEVER DO THIS
```

**Cost management:**
- Integration tests use minimal API calls (1-10 tokens per request)
- Estimated cost: ~$0.10-0.50 per full test suite run
- Document expected costs in test file headers

### 11.3 Anti-Duplication Enforcement

**Before writing ANY code, check:**

```bash
# Search for similar functionality
grep -r "function_name" module/

# Check existing modules for reusable code
# - iron_cost: pricing, budget tracking
# - iron_state: database queries, storage traits
# - iron_api: HTTP clients, middleware patterns
# - api_llm workspace: LLM provider clients (CRITICAL - always check first!)
```

**CRITICAL: Three Workspace Sources**

1. **api_llm workspace** (`/home/user1/pro/lib/wip_iron/api_llm/dev`):
   - ❌ DO NOT implement custom LLM HTTP clients (OpenAI, Claude, Gemini, etc.)
   - ✅ USE `api_openai`, `api_claude`, `api_gemini` from workspace
   - ✅ These crates include retry, circuit breaker, rate limiting
   - ✅ Testing policy: "All tests use real API integration. No mocking allowed."

2. **wTools workspace** (`/home/user1/pro/lib/wip_iron/wTools/dev/module/core`):
   - ❌ DO NOT use external crates when workspace alternatives exist
   - ✅ USE `error_tools` (not anyhow/thiserror)
   - ✅ USE `former` (not typed-builder/derive_builder)
   - ✅ USE `time_tools` (not chrono for basic timestamps)
   - ✅ USE `mod_interface`, `test_tools`, `workspace_tools`, etc.

3. **iron_cage workspace** (current project, existing crates):
   - ❌ DO NOT create new crates when existing ones can be extended
   - ✅ EXTEND `iron_cost` for pricing calculations
   - ✅ EXTEND `iron_state` for token tables/migrations
   - ✅ EXTEND `iron_api` for token endpoints
   - ✅ REUSE `iron_types`, `iron_safety`, `iron_telemetry`

**Common duplication pitfalls:**

| ❌ Duplication Pattern | ✅ Correct Approach |
|----------------------|-------------------|
| Implement OpenAI HTTP client | Use `api_openai` workspace crate |
| Implement Claude HTTP client | Use `api_claude` workspace crate |
| Implement Gemini HTTP client | Use `api_gemini` workspace crate |
| Implement retry logic for LLM calls | Use retry built into `api_*` crates |
| Implement rate limiting for APIs | Use rate limiting built into `api_*` crates |
| Add anyhow/thiserror dependencies | Use `error_tools` workspace crate |
| Add typed-builder dependency | Use `former` workspace crate |
| Add chrono for timestamps | Use `time_tools` workspace crate |
| Implement custom error types | Use `error_tools::err!` macro |
| Create new cost tracking logic | Extend existing `iron_cost` crate |
| Create new database tables separately | Extend `iron_state` migrations |
| Duplicate database query patterns | Reuse `iron_state` storage traits |
| Copy-paste error handling | Use `error_tools` consistently |
| Duplicate validation logic | Reuse `iron_safety` utilities |

**Quality gate checkpoint:**
- Code review must verify no duplication
- Run `cargo clippy` (catches some duplication patterns)
- Use `cargo-duplicate` if available
- Document shared utilities in `module/*/spec.md`

### 11.4 Green State Blocking Gates

**BLOCKING means development STOPS if tests fail.**

```
Phase 1 End → Run `w3 .test l::3` → ❌ FAIL → FIX IMMEDIATELY → ✅ PASS → Continue
Phase 2 End → Run `w3 .test l::3` → ❌ FAIL → FIX IMMEDIATELY → ✅ PASS → Continue
Phase 3 End → Run `w3 .test l::3` → ❌ FAIL → FIX IMMEDIATELY → ✅ PASS → Continue
Phase 4 End → Run `w3 .test l::3` → ❌ FAIL → FIX IMMEDIATELY → ✅ PASS → Continue
Phase 5 End → Run `w3 .test l::3` → ❌ FAIL → FIX IMMEDIATELY → ✅ PASS → Continue
Phase 6 End → Run `w3 .test l::5` → ❌ FAIL → FIX IMMEDIATELY → ✅ PASS → Release
```

**If tests fail:**
1. ❌ DO NOT disable/ignore tests
2. ❌ DO NOT skip to next phase
3. ❌ DO NOT add `#[ignore]` annotation
4. ✅ FIX the underlying issue
5. ✅ Document root cause in test file
6. ✅ Add regression test if bug found

**Test levels:**
- **Level 3** (daily): `cargo nextest run` + `cargo test --doc` + `cargo clippy`
- **Level 5** (final): Level 3 + `cargo udeps` + `cargo audit`

### 11.5 Real Token Testing Requirements

**Integration test template:**

```rust
/// Integration test for OpenAI token usage tracking
///
/// **Required environment variables:**
/// - `OPENAI_API_KEY`: Real OpenAI API key
///
/// **Cost:** ~$0.001 per run (10 token completion)
///
/// **Failure mode:** Test fails loudly if API key missing
#[tokio::test]
async fn test_real_openai_usage_tracking() {
  // REQUIRED: Fail loudly if API key missing
  let api_key = std::env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY required - see .env.test.example");

  // Use REAL API (no mocks)
  let client = OpenAIClient::new(&api_key);
  let response = client.complete("test", 10).await
    .expect("API call failed - check API key validity");

  // Verify REAL usage recorded
  assert!(response.usage.total_tokens > 0);
  assert!(response.usage.total_tokens <= 20);  // Sanity check
}
```

**Documentation requirements:**
- Every integration test documents required env vars
- `.env.test.example` provides template
- `readme.md` explains how to set up test environment
- CI/CD documents how to configure secrets

### 11.6 Enforcement Checklist

**Use this checklist at every quality gate:**

```
□ Run `w3 .test l::3` - all tests passing?
□ Zero mocks used - verified by code review?
□ Zero disabled/ignored tests - verified with grep '#\[ignore\]'?
□ Zero code duplication - verified by code review?
□ Workspace API crates used - verified api_openai/api_claude/api_gemini in Cargo.toml?
□ Workspace utility crates used - verified error_tools/former/mod_interface/time_tools in Cargo.toml?
□ NO external crates when workspace alternatives exist - verified no anyhow/thiserror/chrono?
□ NO custom LLM HTTP clients - verified no reqwest/HTTP code for OpenAI/Claude/Gemini?
□ iron_cage crates reused - verified iron_cost/iron_state dependencies?
□ Integration tests use real APIs - checked .env.test usage?
□ Tests fail loudly if secrets missing - verified .expect() usage?
□ Green state achieved - blocking gate passed?
□ Code review approved - reviewer verified above checklist?
```

**Automated enforcement (CI/CD):**

```yaml
# .github/workflows/ci.yml
- name: Run tests (must pass)
  run: w3 .test l::3
  env:
    OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
    ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
    GEMINI_API_KEY: ${{ secrets.GEMINI_API_KEY }}
    # Tests fail if secrets missing - intentional

- name: Check for ignored tests (must be zero)
  run: |
    if grep -r "#\[ignore\]" tests/; then
      echo "ERROR: Ignored tests found"
      exit 1
    fi

- name: Check for mocks (forbidden patterns)
  run: |
    if grep -r "MockClient\|FakeDatabase\|mock_server" src/ tests/; then
      echo "ERROR: Mock usage detected"
      exit 1
    fi

- name: Check workspace API crate usage (must use api_openai/api_claude/api_gemini)
  run: |
    if ! grep -q "api_openai" module/iron_token_manager/Cargo.toml; then
      echo "ERROR: Missing api_openai workspace crate dependency"
      exit 1
    fi
    if ! grep -q "api_claude" module/iron_token_manager/Cargo.toml; then
      echo "ERROR: Missing api_claude workspace crate dependency"
      exit 1
    fi
    if ! grep -q "api_gemini" module/iron_token_manager/Cargo.toml; then
      echo "ERROR: Missing api_gemini workspace crate dependency"
      exit 1
    fi

- name: Check for custom LLM HTTP clients (forbidden - use workspace crates)
  run: |
    # Check for custom OpenAI/Claude/Gemini HTTP implementation
    if grep -r "reqwest.*openai\|OpenAI.*reqwest\|https://api.openai.com" module/iron_token_manager/src/; then
      echo "ERROR: Custom OpenAI HTTP client detected - use api_openai workspace crate"
      exit 1
    fi
    if grep -r "reqwest.*claude\|Claude.*reqwest\|https://api.anthropic.com" module/iron_token_manager/src/; then
      echo "ERROR: Custom Claude HTTP client detected - use api_claude workspace crate"
      exit 1
    fi
    if grep -r "reqwest.*gemini\|Gemini.*reqwest\|https://generativelanguage.googleapis.com" module/iron_token_manager/src/; then
      echo "ERROR: Custom Gemini HTTP client detected - use api_gemini workspace crate"
      exit 1
    fi

- name: Check workspace utility crate usage (must use wTools crates)
  run: |
    if ! grep -q "error_tools.*workspace.*true" module/iron_token_manager/Cargo.toml; then
      echo "ERROR: Missing error_tools workspace crate - don't use anyhow/thiserror directly"
      exit 1
    fi
    if ! grep -q "former.*workspace.*true" module/iron_token_manager/Cargo.toml; then
      echo "ERROR: Missing former workspace crate - don't use typed-builder"
      exit 1
    fi
    if ! grep -q "mod_interface.*workspace.*true" module/iron_token_manager/Cargo.toml; then
      echo "ERROR: Missing mod_interface workspace crate"
      exit 1
    fi
    if ! grep -q "time_tools.*workspace.*true" module/iron_token_manager/Cargo.toml; then
      echo "ERROR: Missing time_tools workspace crate - prefer over chrono"
      exit 1
    fi

- name: Check iron_cage crate reuse (must extend existing infrastructure)
  run: |
    if ! grep -q "iron_cost.*workspace.*true" module/iron_token_manager/Cargo.toml; then
      echo "ERROR: Missing iron_cost dependency - reuse for pricing calculations"
      exit 1
    fi
    if ! grep -q "iron_state.*workspace.*true" module/iron_token_manager/Cargo.toml; then
      echo "ERROR: Missing iron_state dependency - extend for token tables"
      exit 1
    fi

- name: Check forbidden external crates (when workspace alternatives exist)
  run: |
    # Forbid anyhow/thiserror when error_tools exists
    if grep -q "^anyhow = " module/iron_token_manager/Cargo.toml; then
      echo "ERROR: anyhow dependency found - use error_tools instead"
      exit 1
    fi
    if grep -q "^thiserror = " module/iron_token_manager/Cargo.toml; then
      echo "ERROR: thiserror dependency found - use error_tools instead"
      exit 1
    fi
    # Forbid typed-builder when former exists
    if grep -q "typed-builder" module/iron_token_manager/Cargo.toml; then
      echo "ERROR: typed-builder dependency found - use former instead"
      exit 1
    fi
```

---

## Appendix A: Decision Log

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|------------------------|
| 2025-12-02 | Use Vue 3 instead of React | User requirement | React 18, Svelte |
| 2025-12-02 | Use shadcn-vue for components | Consistent with user preference | Vuetify, PrimeVue |
| 2025-12-02 | DEFER to post-pilot (Q1 2026) | Timeline conflict (23 days vs 75 days) | Rush implementation for pilot |
| 2025-12-02 | Use workspace api_llm crates | Anti-duplication, existing tested code | Implement custom HTTP clients |
| 2025-12-02 | Use workspace wTools utility crates | Anti-duplication, consistent ecosystem | Use external crates (anyhow, chrono, etc.) |
| 2025-12-02 | Extend existing iron_cage crates | Reuse iron_cost/iron_state infrastructure | Create new crates from scratch |
| 2025-12-02 | No mocks policy (TDD with real APIs) | Production-quality code, user requirement | Mock-based testing |
| 2025-12-02 | Mandatory `w3 .test l::3` at each phase | Continuous green state enforcement | Test at end only |
| TBD | PostgreSQL vs MySQL | TBD | PostgreSQL (better JSON support) |
| TBD | SQLx vs Diesel | TBD | SQLx (async-first) |

---

## Appendix B: Key Contacts

| Role | Name | Responsibility | Contact |
|------|------|----------------|---------|
| Product Owner | TBD | Requirements, acceptance | TBD |
| Technical Lead | TBD | Architecture, code review | TBD |
| Backend Developer | TBD | Rust implementation | TBD |
| Frontend Developer | TBD | Vue.js dashboard | TBD |
| DevOps Engineer | TBD | Infrastructure, deployment | TBD |
| Security Reviewer | TBD | Security audit | TBD |

---

## Appendix C: Quick Reference

**Key Commands:**
```bash
# Run tests (level 3)
w3 .test l::3

# Run tests (all levels)
w3 .test l::5

# Security audit
cargo audit

# Code coverage
cargo tarpaulin --out Html

# Load testing
k6 run load_test.js

# Database migration
sqlx migrate run
```

**Key Directories:**
```
module/iron_token_manager/    # Backend business logic
module/iron_api/              # REST API
module/iron_state/            # Database layer
dashboard/                    # Vue.js frontend (separate repo)
```

**Key Files:**
```
business/docs/features/002_token_management.md              # Architecture
business/docs/features/005_token_management_implementation_plan.md  # This file
task/backlog/001_implement_llm_token_management_dashboard_and_backend.md  # Requirements
```

---

**Document Status:** ✅ Complete
**Version:** 3.2.0
**Last Updated:** 2025-12-02
**Next Review:** After pilot completion (Dec 18, 2025)

---

## Documentation Consistency Notes

**Version History:**
- v3.2.0: Added workspace utility crates (wTools), documentation consistency fixes
- v3.1.0: Added workspace LLM API crates integration
- v3.0.0: Added TDD enforcement, no-mocks policy, green state blocking gates
- v2.1.0: Added CLI/API parity requirements
- v2.0.0: Added CLI development to Phase 4
- v1.0.0: Initial implementation plan

**Terminology Standards:**
- **Provider names**: Use "OpenAI", "Anthropic Claude", "Google Gemini" (full formal names)
- **API env vars**: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GEMINI_API_KEY` (consistent with api_llm workspace)
- **Crate names**: Backticks for all crate references (e.g., `error_tools`, `api_openai`)
- **Workspace paths**: Use full absolute paths from project root (e.g., `/home/user1/pro/lib/wip_iron/wTools/dev`)
- **Code indentation**: 2-space indentation for all Rust code examples (per wTools codestyle)
- **Quality gates**: All use "BLOCKING - must pass to continue" format with ✅ checkmarks
- **Test commands**: `w3 .test l::3` (daily), `w3 .test l::5` (final release)

**Cross-References Verified:**
- ✅ Architecture document: `002_token_management.md`
- ✅ Validation framework: `006_token_management_validation_framework.md`
- ✅ CLI/API parity spec: `004_token_management_cli_api_parity.md`
- ✅ Task requirements: `task/backlog/001_implement_llm_token_management_dashboard_and_backend.md`
- ✅ Workspace crates: `api_llm`, `wTools`, `iron_cage` (28 total crates referenced)
