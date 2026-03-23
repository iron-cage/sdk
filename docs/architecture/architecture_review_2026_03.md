# Iron Cage - Architecture Review & Roadmap

**Date**: 2026-03-20
**Scope**: Full workspace audit, Rusticon promise gap analysis, 3-stage roadmap

---

### 1. Current Metrics

- 15 Rust crates, 390 .rs files
- 707 tests, all passing
- 43.76% line coverage (llvm-cov, from previous measurement - no coverage config in repo)
- Clean build, zero errors
- 10 open PRs (all NOT READY except #55 APPROVED)
- 27 planned tasks in `task/` (none completed)

---

### 2. Strengths

- **Type-safe foundations** - Entity IDs with compile-time validation, `#[repr(transparent)]` zero-cost wrappers, injection prevention
- **Modular crate architecture** - Clean dependency graph: foundation -> infrastructure -> features -> integration -> application
- **CLI design** - Unilang three-layer pattern (routing -> adapters -> handlers) with pure logic separation
- **Documentation** - 90+ documents covering architecture, protocols, capabilities, constraints, security, ADRs, standards
- **Control API** - 17 protocol specifications, JWT auth, RBAC, structured error responses
- **LLM Router (Rust)** - Transparent proxy, provider auto-detection, OpenAI-compatible API, cross-provider request translation
- **Circuit breaker** - Production-ready, configurable, three-state machine
- **Budget primitives** - AtomicU64, microdollar precision, reserve-commit-cancel pattern

---

### 3. Weaknesses

| Area | Issue | Severity |
|------|-------|----------|
| Python SDK | `start_agent()` is placeholder, async bridge not implemented | CRITICAL |
| Audit trail | In-memory only (ArrayQueue from crossbeam), lost on restart | CRITICAL |
| PII detection | Email + US phone regex only | HIGH |
| Budget auto-refresh | Lease CRUD exists, no client-side refresh cycle | HIGH |
| Vault integration | AES-256-GCM primitives only, no vault backend | HIGH |
| Error handling | 35 RwLock `.unwrap()` calls (32 in_memory.rs, 3 http.rs) - panics on poisoned lock | MEDIUM |
| Test coverage | 43.76% overall; provider_adapter.rs and trace_storage.rs at 0% | MEDIUM |
| Formatter errors | 13 locations silently replace errors with defaults | MEDIUM |
| Rust edition | 2021, should be 2024 | LOW |

---

### 4. Coverage Highlights (llvm-cov)

| Module | Coverage | Status |
|--------|----------|--------|
| cost_calculator.rs | 96.88% | Good |
| seed.rs | 91.47% | Good |
| usage_tracker.rs | 91.05% | Good |
| storage.rs | 86.81% | Good |
| provider_adapter.rs | 0.00% | Missing |
| trace_storage.rs | 0.00% | Missing |
| budget_request.rs | 38.97% | Low |
| lease_manager.rs | 41.71% | Low |
| **TOTAL** | **43.76%** | Needs improvement (unverifiable - no coverage config in repo) |

---

### 5. Rusticon Promise vs Reality

Source: Rusticon presentation transcript

| # | Rusticon Promise | Implemented? | Works E2E? | Gap |
|---|-----------------|-------------|------------|-----|
| 1 | Sub-microsecond atomic budget enforcement | AtomicU64 exists | Single-proxy only | No benchmarks, no distributed CAS |
| 2 | PII detected and redacted (beta) | Regex exists | Email + US phone | No SSN, credit cards, international, ML |
| 3 | Circuit breakers | Yes, configurable | Yes | None - fully configurable, no hardcoded defaults |
| 4 | Every call logged to audit trail | In-memory events (ArrayQueue) | Lost on restart | No persistence, no compliance format |
| 5 | Three lines of code - protected | LlmRouter exists | Requires maturin build | Not on PyPI, not drop-in |
| 6 | Budget lease - $10 tranches, auto-refresh | Lease CRUD | No auto-refresh | Client-side refresh logic missing |
| 7 | IC Token revocation in one click | Token generation | Delete only | No revoke action, no audit event |
| 8 | IP Tokens in vault | AES-GCM crypto | No vault backend | Primitives only, no lifecycle |
| 9 | Zero to protected team in 60 seconds | CLI commands exist | Dashboard not connected | Control Panel UI is separate |
| 10 | Memory safety non-optional | Strict linting | 35 unwrap() panics | RwLock poison can crash process |

---

### 6. Proposed Architectural Changes

### A1. Eliminate panic-prone error handling

All 35 RwLock `.unwrap()` calls in adapters should use proper error propagation. Documenting a panic mode in `# Panics` is not a fix - it's acknowledging a crash path.

**Affected:** `iron_cli/src/adapters/implementations/http.rs`, `iron_cli/src/adapters/implementations/in_memory.rs`

### A2. Pluggable persistent audit trail

Introduce `AuditBackend` trait with two implementations:
- **SQLite** - Default, local-first, consistent with existing `iron_runtime_state` patterns
- **PostgreSQL** - Optional, for multi-user production deployments

Current in-memory `ArrayQueue<AnalyticsEvent>` (crossbeam) becomes a real-time cache with write-through to persistent storage.

**Affected:** `iron_runtime_analytics/src/event_storage.rs`

### A3. Unified error handling strategy

Currently three different strategies: structured `CliError` in handlers, `unwrap_or_else(default)` in formatters (13 locations), dual `build()`/`build_result()` in config. Should converge to one approach: all fallible operations return `Result`, no silent defaults.

**Affected:** `iron_cli/src/config.rs`, `iron_cli/src/formatting/tree_formatter.rs`

### A4. Budget auto-refresh worker

A background task monitoring `CostController::remaining()` that requests new leases when the local wallet depletes below a configurable threshold. Includes budget return on shutdown.

**Affected:** `iron_cost/`, `iron_runtime/src/llm_router/router.rs`

### A5. PII detection expansion

Add patterns for SSN, credit card numbers (Luhn), E.164 international phones, IP addresses, AWS access keys. Add configurable pattern registry for user-defined rules. Emit audit events on detection.

**Affected:** `iron_safety/src/`

### A6. Rust 2024 edition upgrade

Update workspace edition from 2021 to 2024.

---

### 7. WIP Awareness (Open PRs & Tasks)

### Open PRs

| PR | Title | Relates to |
|----|-------|-----------|
| #53 | Server proxy, iron_llm_core extraction, per-IP-key spending caps | Budget enforcement, multi-provider, spending limits |
| #55 | Dashboard UI/UX overhaul (APPROVED) | Control Panel frontend |
| #56 | Multiple IP keys per provider | Task 002, multi-tenancy |
| #57 | Per-key and per-agent spending limits | Task 003/004, budget enforcement |
| #59 | Server proxy, analytics, multi-provider (Gemini, xAI) | Task 005/006, analytics |
| #60 | Provider key filtering, median cost | Analytics improvements |
| #61 | Mobile adaptation | Dashboard mobile |
| #62 | Presentation branch | Rusticon demo |
| #63 | Quick Add provider key auto-detection | UX improvement |

### Planned Tasks (task/)

| ID | Task | Overlaps with |
|----|------|--------------|
| 001 | Fix IC token invalidation after regeneration | Promise #7 (revocation) |
| 002 | Multiple keys per provider | PR #56 |
| 003 | Provider key spending and limits | PR #57, #53 |
| 004 | Per-IC-key limits | PR #57 |
| 005 | Add Gemini provider | PR #59 |
| 006 | Add xAI provider | PR #59 |
| 007 | Migrate to api_llm bindings | LLM Router refactoring |
| 008 | Internal deployment | E2E validation |
| 009 | Fix RwLock unwrap panics | Promise #10 |
| 011 | Upgrade to Rust 2024 edition | |
| 012 | Unify error handling strategy | |
| 013 | Increase test coverage | |
| 014 | Persistent audit trail (SQLite) | Promise #4 |
| 015 | Budget lease auto-refresh worker | Promise #6 |
| 016 | Expand PII detection patterns | Promise #2 |
| 017 | Complete Python SDK async bridge | Promise #5 |
| 018 | PyPI distribution pipeline | Promise #5 |
| 019 | E2E Rusticon demo script | All promises |
| 020 | PostgreSQL audit backend | Promise #4 |
| 021 | Token revocation and vault backend | Promise #7, #8 |
| 022 | Provider failover | |
| 023 | Hardening | |
| 024 | API surface polish | |
| 025 | CI/CD pipeline | |
| 026 | Documentation and examples | |
| 027 | Dashboard integration | Promise #9, PR #55 |
| 028 | Budget enforcement benchmarks | Promise #1 |

---

### 8. Roadmap Summary

| Stage | Goal | Tasks | Key Deliverable |
|-------|------|-------|-----------------|
| 1 | Close Rusticon Gap | 009-019, 028 | Reproducible demo, all claims backed by code |
| 2 | Pilot Customer Readiness | 020-023 | Postgres, vault, multi-provider, failover, hardening |
| 3 | Open Source Launch | 024-027 | CI/CD, PyPI, docs, examples, 1.0.0 |

Maximum parallelism: 9 concurrent tasks in Stage 1 Group A (009, 011-017, 028 - all in different crates).

Full task breakdown and dependency graph: see `task/readme.md`.
