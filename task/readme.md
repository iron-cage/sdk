<!-- task_system_metadata
type: root
version: 1.0
highest_id: 28
last_allocation:
  id: 28
  crate: workspace
  timestamp: 2026-03-20T00:00:00Z
-->

# Master Task Index - Workspace

**Last Updated**: 2026-03-20
**Purpose**: Comprehensive task tracking for this workspace task folder
**Total Tasks**: 27 tasks

---

## Quick Navigation

- [Global ID Registry](#global-id-registry) - Centralized ID allocation
- [Tasks Index (Local)](#tasks-index-local) - Workspace-level tasks
- [Tasks Index (Aggregated)](#tasks-index-aggregated---active--backlog-only) - Active and backlog tasks
- [Issues Index](#issues-index) - Issue tracking
- [Issues](#issues) - Issue details
- [Summary Statistics](#summary-statistics)

---

## Global ID Registry

<!-- registry_data
highest_id: 28
-->

**Current State:**
- **Highest Allocated ID**: 28
- **Active Tasks**: 27
- **Completed Tasks**: 0
- **Backlog Tasks**: 0
- **Task Systems**: 1 (root workspace task system)

**ID Allocation Policy:**
- All task IDs are allocated from this registry
- IDs must be globally unique within this workspace task system
- Next available ID: 29

---

## Tasks Index (Local)

**Note:** Workspace-level tasks for current implementation wave.

| Order | ID | Advisability | Value | Easiness | Safety | Priority | Status | Task | Description |
|-------|----|-------------:|------:|---------:|-------:|---------:|--------|------|-------------|
| 1 | [009](009-fix-rwlock-unwrap-panics.md) | 630 | 9 | 7 | 10 | 1 | 🔄 (Planned) | Fix RwLock unwrap panics in CLI adapters | Replace 35 RwLock .unwrap() calls with proper error propagation in CLI adapters |
| 2 | [028](028-budget-enforcement-benchmarks.md) | 560 | 7 | 8 | 10 | 1 | 🔄 (Planned) | Budget enforcement benchmarks | Add criterion benchmarks proving sub-microsecond CostController reserve/commit/cancel |
| 3 | [016](016-expand-pii-detection-patterns.md) | 540 | 9 | 6 | 10 | 1 | 🔄 (Planned) | Expand PII detection patterns | Add SSN, credit card, E.164 phone, IP address, and AWS key detection patterns |
| 4 | [002](002-support-multiple-provider-keys-per-provider.md) | 504 | 8 | 7 | 9 | 1 | 🔄 (Planned) | Support multiple keys per provider | Remove single-key behavior and enforce owner-scoped key selection |
| 5 | [012](012-unify-error-handling-strategy.md) | 490 | 7 | 7 | 10 | 1 | 🔄 (Planned) | Unify error handling strategy | Converge formatters and config to Result-based error handling, eliminating silent defaults |
| 6 | [001](001-fix-ic-token-regeneration-invalidation.md) | 480 | 8 | 6 | 10 | 1 | 🔄 (Planned) | Fix IC token invalidation after regeneration | Ensure old IC tokens are rejected after regenerate/revoke |
| 7 | [014](014-persistent-audit-trail-sqlite.md) | 450 | 10 | 5 | 9 | 1 | 🔄 (Planned) | Persistent audit trail (SQLite) | Introduce AuditBackend trait with SQLite implementation and write-through cache |
| 8 | [004](004-per-ic-key-limits.md) | 432 | 8 | 6 | 9 | 1 | 🔄 (Planned) | Add dedicated limits per IC key | Introduce independent IC-key limit controls per agent |
| 9 | [003](003-provider-key-spending-and-limits.md) | 405 | 9 | 5 | 9 | 1 | 🔄 (Planned) | Add spending and limits per provider key | Introduce key-level budget accounting and enforcement |
| 10 | [007](007-migrate-to-api-llm-bindings.md) | 405 | 9 | 5 | 9 | 1 | 🔄 (Planned) | Migrate to `api_llm` bindings | Establish binding layer, migrate all providers, remove legacy direct-HTTP code |
| 11 | [015](015-budget-lease-auto-refresh-worker.md) | 384 | 8 | 6 | 8 | 1 | 🔄 (Planned) | Budget lease auto-refresh worker | Add background tokio task for threshold-based budget lease auto-refresh |
| 12 | [008](008-deploy-iron-cage-internal.md) | 360 | 10 | 4 | 9 | 1 | 🔄 (Planned) | Deploy Iron Cage as internal centralized token control platform | Centralized IP key sharing, IC token distribution, RBAC, and secure internal deployment by 2026-02-28 |
| 13 | [017](017-complete-python-sdk-async-bridge.md) | 300 | 10 | 3 | 10 | 1 | 🔄 (Planned) | Complete Python SDK async bridge | Complete pyo3-asyncio bridge and implement start_agent() for Python SDK |
| 14 | [019](019-e2e-rusticon-demo-script.md) | 300 | 10 | 3 | 10 | 1 | 🔄 (Planned) | End-to-end Rusticon demo script | Create reproducible end-to-end demo validating all Rusticon promises |
| 15 | [018](018-pypi-distribution-pipeline.md) | 180 | 8 | 5 | 9 | 2 | 🔄 (Planned) | PyPI distribution pipeline | Build maturin-based PyPI package and distribution pipeline |
| 16 | [005](005-add-gemini-provider.md) | 168 | 7 | 6 | 8 | 2 | 🔄 (Planned) | Add Gemini inference provider | End-to-end Gemini support in control, runtime, and analytics |
| 17 | [006](006-add-xai-provider.md) | 168 | 7 | 6 | 8 | 2 | 🔄 (Planned) | Add xAI inference provider | End-to-end xAI support in control, runtime, and analytics |
| 18 | [013](013-increase-test-coverage-zero-modules.md) | 150 | 6 | 5 | 10 | 2 | 🔄 (Planned) | Increase test coverage for untested modules | Add tests for untested public functions in provider_adapter.rs and trace_storage.rs |
| 19 | [022](022-provider-failover.md) | 144 | 8 | 4 | 9 | 2 | 🔄 (Planned) | Provider failover | Implement automatic provider failover with health-based routing |
| 20 | [020](020-postgresql-audit-backend.md) | 140 | 7 | 5 | 8 | 2 | 🔄 (Planned) | PostgreSQL audit backend | Add PostgreSQL AuditBackend implementation for multi-user deployments |
| 21 | [011](011-upgrade-rust-2024-edition.md) | 140 | 5 | 7 | 8 | 2 | 🔄 (Planned) | Upgrade to Rust 2024 edition | Update workspace edition from 2021 to 2024 and fix any breaking changes |
| 22 | [023](023-hardening.md) | 135 | 9 | 3 | 10 | 2 | 🔄 (Planned) | Hardening | Add property tests, fuzz testing, and load testing for hardening |
| 23 | [021](021-token-revocation-vault-backend.md) | 128 | 8 | 4 | 8 | 2 | 🔄 (Planned) | Token revocation and vault backend | Add dedicated token revocation action and integrate vault backend for key storage |
| 24 | [024](024-api-surface-polish.md) | 117 | 7 | 5 | 10 | 3 | 🔄 (Planned) | API surface polish | Stabilize public API surface with consistent naming and versioning |
| 25 | [026](026-documentation-and-examples.md) | 117 | 7 | 5 | 10 | 3 | 🔄 (Planned) | Documentation and examples | Write user-facing documentation, API reference, and integration examples |
| 26 | [025](025-ci-cd-pipeline.md) | 96 | 8 | 4 | 9 | 3 | 🔄 (Planned) | CI/CD pipeline | Set up GitHub Actions CI/CD with automated testing, linting, and release workflows |
| 27 | [027](027-dashboard-integration.md) | 84 | 7 | 4 | 9 | 3 | 🔄 (Planned) | Dashboard integration | Connect Control Panel dashboard to backend API endpoints |

---

## Tasks Index (Aggregated - Active & Backlog Only)

**Note:** Same as local index in this repository (single root task system).

| Order | ID | Crate | Advisability | Value | Easiness | Safety | Priority | Status | Task | Description |
|-------|----|-------|-------------:|------:|---------:|-------:|---------:|--------|------|-------------|
| 1 | [009](009-fix-rwlock-unwrap-panics.md) | workspace | 630 | 9 | 7 | 10 | 1 | 🔄 (Planned) | Fix RwLock unwrap panics | Replace .unwrap() with error propagation in adapters |
| 2 | [028](028-budget-enforcement-benchmarks.md) | workspace | 560 | 7 | 8 | 10 | 1 | 🔄 (Planned) | Budget enforcement benchmarks | Criterion benchmarks for CostController operations |
| 3 | [016](016-expand-pii-detection-patterns.md) | workspace | 540 | 9 | 6 | 10 | 1 | 🔄 (Planned) | Expand PII detection | SSN, credit cards, E.164, IP, AWS keys |
| 4 | [002](002-support-multiple-provider-keys-per-provider.md) | workspace | 504 | 8 | 7 | 9 | 1 | 🔄 (Planned) | Multi-key provider support | Create and manage multiple keys per provider safely |
| 5 | [012](012-unify-error-handling-strategy.md) | workspace | 490 | 7 | 7 | 10 | 1 | 🔄 (Planned) | Unify error handling | Converge formatters/config to Result-based errors |
| 6 | [001](001-fix-ic-token-regeneration-invalidation.md) | workspace | 480 | 8 | 6 | 10 | 1 | 🔄 (Planned) | Fix IC token invalidation | Reject stale tokens after rotate/revoke |
| 7 | [014](014-persistent-audit-trail-sqlite.md) | workspace | 450 | 10 | 5 | 9 | 1 | 🔄 (Planned) | Persistent audit trail (SQLite) | AuditBackend trait, SQLite, write-through cache |
| 8 | [004](004-per-ic-key-limits.md) | workspace | 432 | 8 | 6 | 9 | 1 | 🔄 (Planned) | Per-IC-key limits | Independent IC policy limits per agent |
| 9 | [003](003-provider-key-spending-and-limits.md) | workspace | 405 | 9 | 5 | 9 | 1 | 🔄 (Planned) | Provider-key spending/limits | Enforce and expose key-level budgeting |
| 10 | [007](007-migrate-to-api-llm-bindings.md) | workspace | 405 | 9 | 5 | 9 | 1 | 🔄 (Planned) | Migrate to `api_llm` bindings | Establish binding layer, migrate all providers, remove legacy direct-HTTP code |
| 11 | [015](015-budget-lease-auto-refresh-worker.md) | workspace | 384 | 8 | 6 | 8 | 1 | 🔄 (Planned) | Budget auto-refresh worker | Background tokio task, threshold-based refresh |
| 12 | [008](008-deploy-iron-cage-internal.md) | workspace | 360 | 10 | 4 | 9 | 1 | 🔄 (Planned) | Internal deployment | IP key sharing, IC distribution, RBAC, TLS - by 2026-02-28 |
| 13 | [017](017-complete-python-sdk-async-bridge.md) | workspace | 300 | 10 | 3 | 10 | 1 | 🔄 (Planned) | Python SDK async bridge | pyo3-asyncio bridge, start_agent() impl |
| 14 | [019](019-e2e-rusticon-demo-script.md) | workspace | 300 | 10 | 3 | 10 | 1 | 🔄 (Planned) | E2E Rusticon demo | Reproducible demo validating all promises |
| 15 | [018](018-pypi-distribution-pipeline.md) | workspace | 180 | 8 | 5 | 9 | 2 | 🔄 (Planned) | PyPI distribution | maturin PyPI package and CI pipeline |
| 16 | [005](005-add-gemini-provider.md) | workspace | 168 | 7 | 6 | 8 | 2 | 🔄 (Planned) | Gemini provider integration | Add Gemini across API, runtime, analytics |
| 17 | [006](006-add-xai-provider.md) | workspace | 168 | 7 | 6 | 8 | 2 | 🔄 (Planned) | xAI provider integration | Add xAI across API, runtime, analytics |
| 18 | [013](013-increase-test-coverage-zero-modules.md) | workspace | 150 | 6 | 5 | 10 | 2 | 🔄 (Planned) | Test coverage for untested modules | Tests for untested public functions |
| 19 | [022](022-provider-failover.md) | workspace | 144 | 8 | 4 | 9 | 2 | 🔄 (Planned) | Provider failover | Auto failover with health-based routing |
| 20 | [020](020-postgresql-audit-backend.md) | workspace | 140 | 7 | 5 | 8 | 2 | 🔄 (Planned) | PostgreSQL audit backend | PostgreSQL AuditBackend for production |
| 21 | [011](011-upgrade-rust-2024-edition.md) | workspace | 140 | 5 | 7 | 8 | 2 | 🔄 (Planned) | Rust 2024 edition | Edition 2021 to 2024 with breaking change fixes |
| 22 | [023](023-hardening.md) | workspace | 135 | 9 | 3 | 10 | 2 | 🔄 (Planned) | Hardening | Property tests, fuzz testing, load testing |
| 23 | [021](021-token-revocation-vault-backend.md) | workspace | 128 | 8 | 4 | 8 | 2 | 🔄 (Planned) | Token revocation/vault | Revocation action, vault backend integration |
| 24 | [024](024-api-surface-polish.md) | workspace | 117 | 7 | 5 | 10 | 3 | 🔄 (Planned) | API surface polish | Consistent naming, versioning, stability |
| 25 | [026](026-documentation-and-examples.md) | workspace | 117 | 7 | 5 | 10 | 3 | 🔄 (Planned) | Documentation/examples | User docs, API reference, examples |
| 26 | [025](025-ci-cd-pipeline.md) | workspace | 96 | 8 | 4 | 9 | 3 | 🔄 (Planned) | CI/CD pipeline | GitHub Actions, automated test/release |
| 27 | [027](027-dashboard-integration.md) | workspace | 84 | 7 | 4 | 9 | 3 | 🔄 (Planned) | Dashboard integration | Connect dashboard UI to backend API |

---

## Issues Index

| ID | Title | Related Task | Status |
|----|-------|--------------|--------|

*No task-level issues are currently registered in this workspace task system.*

---

## Issues

*No open issues recorded in `task/issue/` for this workspace.*

---

## Summary Statistics

### Tasks by Status

| Status | Count | Percentage | Location |
|--------|-------|------------|----------|
| 🔄 Planned | 27 | 100.0% | `task/` |
| ✅ Completed | 0 | 0.0% | `task/completed/` |
| 📥 Backlog | 0 | 0.0% | `task/backlog/` |
| **TOTAL** | **27** | **100%** | |

### Tasks by Domain

| Domain | Total | Planned | Completed | Backlog |
|--------|-------|---------|-----------|---------|
| Deployment & Operations | 2 | 2 | 0 | 0 |
| Security & Auth | 2 | 2 | 0 | 0 |
| Provider Key Management | 2 | 2 | 0 | 0 |
| Budget & Limits | 3 | 3 | 0 | 0 |
| Provider Integrations | 3 | 3 | 0 | 0 |
| LLM Binding & Migration | 1 | 1 | 0 | 0 |
| Code Quality & Hardening | 5 | 5 | 0 | 0 |
| Compliance & Safety | 3 | 3 | 0 | 0 |
| SDK, Integration & Documentation | 6 | 6 | 0 | 0 |
| **TOTAL** | **27** | **27** | **0** | **0** |

---

## Organization Structure

### Task File Locations

Tasks are organized in this workspace as:
- **Active**: `task/*.md`
- **Completed**: `task/completed/*.md`
- **Backlog**: `task/backlog/*.md`
- **Obsolete**: `task/obsolete/*.md`

### Status Indicators

- 🔄 **Planned**: Ready to start, dependencies met
- ⏳ **In Progress**: Currently being worked on
- ✅ **Completed**: Finished and validated
- ⛔️ **Blocked**: Waiting on dependency or external factor
- 📥 **Backlog**: Future work, not yet prioritized
- 📦 **Obsolete**: Archived as no longer needed

---

## Navigation

### By Concern

**Deployment & Operations** ⚡ (deadline 2026-02-28)
- Task 008 (internal deployment — IP key sharing, IC distribution, RBAC, TLS)
- Task 025 (CI/CD pipeline)

**Security & Token Lifecycle**
- Task 001 (IC token invalidation)
- Task 021 (token revocation and vault backend)

**Provider Key Model and Multi-tenancy**
- Task 002 (multi-key support)
- Task 003 (spending/limits per provider key)

**Budget and Policy Enforcement**
- Task 003 (provider-key enforcement)
- Task 004 (per-IC-key limits)
- Task 015 (budget auto-refresh worker)
- Task 028 (budget enforcement benchmarks)

**Provider Integrations**
- Task 005 (Gemini)
- Task 006 (xAI)
- Task 022 (provider failover)

**LLM Binding Standardization and Migration**
- Task 007 (migrate to `api_llm` bindings)

**Code Quality & Hardening**
- Task 009 (fix RwLock unwrap panics)
- Task 011 (Rust 2024 edition upgrade)
- Task 012 (unify error handling strategy)
- Task 013 (increase test coverage)
- Task 023 (hardening - property tests, fuzz, load)

**Compliance & Safety**
- Task 014 (persistent audit trail - SQLite)
- Task 016 (expand PII detection patterns)
- Task 020 (PostgreSQL audit backend)

**SDK, Integration & Documentation**
- Task 017 (complete Python SDK async bridge)
- Task 018 (PyPI distribution pipeline)
- Task 019 (E2E Rusticon demo script)
- Task 024 (API surface polish)
- Task 026 (documentation and examples)
- Task 027 (dashboard integration)

---

## Documentation

- Workspace root: `sdk/`
- Task folder: `sdk/task/`

---

## Recent Changes

**2026-03-20**:
- ✅ Registered tasks 009-028 (Architecture Review & Roadmap - 3-stage plan)
- ✅ Stage 1: Close Rusticon Gap (tasks 009-019, 028)
- ✅ Stage 2: Pilot Customer Readiness (tasks 020-023)
- ✅ Stage 3: Open Source Launch (tasks 024-027)
- ✅ Deleted task 010 (compiler warnings/dead code) - invalid, zero warnings exist
- ✅ Added task 028 (budget enforcement benchmarks) - proves sub-microsecond Rusticon claim
- ✅ Corrected advisability scores and re-prioritized all tasks
- ✅ Updated global registry to highest ID `028` and next available ID `029`
- ✅ Re-sorted all indexes by advisability
- ✅ Updated task 008 (internal deployment) - Value=10, Easiness=4, Safety=9, Priority=1, Advisability=360

**2026-02-22**:
- ✅ Registered task 008 (internal deployment) — Value=10, Easiness=4, Safety=9, Priority=5, Advisability=1800
- ✅ Updated global registry to highest ID `008` and next available ID `009`
- ✅ Re-sorted all indexes by advisability (task 008 at top)

**2026-02-06**:
- ✅ Created root task index for this workspace task system
- ✅ Registered tasks 001-007 with implementation plans and acceptance criteria
- ✅ Added `api_llm` binding migration task
- ✅ Updated global registry to highest ID `007` and next available ID `008`
