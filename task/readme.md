<!-- task_system_metadata
type: root
version: 1.0
highest_id: 33
last_allocation:
  id: 33
  crate: workspace
  timestamp: 2026-05-13T00:00:00Z
-->

# Master Task Index - Workspace

**Last Updated**: 2026-05-13
**Purpose**: Comprehensive task tracking for this workspace task folder
**Total Tasks**: 32 tasks

---

## Quick Navigation

- [Global ID Registry](#global-id-registry) - Centralized ID allocation
- [Tasks Index](#tasks-index) - Workspace-level tasks
- [Summary Statistics](#summary-statistics)

---

## Global ID Registry

<!-- registry_data
highest_id: 33
-->

**Current State:**
- **Highest Allocated ID**: 33
- **Active Tasks**: 32
- **Completed Tasks**: 0
- **Backlog Tasks**: 0
- **Task Systems**: 1 (root workspace task system)

**ID Allocation Policy:**
- All task IDs are allocated from this registry
- IDs must be globally unique within this workspace task system
- Next available ID: 34

---

## Tasks Index

**Note:** Workspace-level tasks for current implementation wave. Advisability = V * E * P * S. Priority scale: 0=completed, 1=deferred, 2=default, 3=elevated, 4=high, 5=critical.

| Order | ID | Advisability | Value | Easiness | Safety | Priority | Status | Task | Description |
|-------|----|-------------:|------:|---------:|-------:|---------:|--------|------|-------------|
| 1 | [009](009_fix_rwlock_unwrap_panics.md) | 3150 | 9 | 7 | 10 | 5 | 🔄 (Planned) | Fix RwLock unwrap panics in CLI adapters | Replace 35 RwLock .unwrap() calls with proper error propagation in CLI adapters |
| 2 | [001](001_fix_ic_token_regeneration_invalidation.md) | 2400 | 8 | 6 | 10 | 5 | 🔄 (Planned) | Fix IC token invalidation after regeneration | Ensure old IC tokens are rejected after regenerate/revoke |
| 3 | [014](014_persistent_audit_trail_sqlite.md) | 2250 | 10 | 5 | 9 | 5 | 🔄 (Planned) | Persistent audit trail (SQLite) | Introduce AuditBackend trait with SQLite implementation and write-through cache |
| 4 | [028](028_budget_enforcement_benchmarks.md) | 2240 | 7 | 8 | 10 | 4 | 🔄 (Planned) | Budget enforcement benchmarks | Add criterion benchmarks proving sub-microsecond CostController reserve/commit/cancel |
| 5 | [016](016_expand_pii_detection_patterns.md) | 2160 | 9 | 6 | 10 | 4 | 🔄 (Planned) | Expand PII detection patterns | Add SSN, credit card, E.164 phone, IP address, and AWS key detection patterns |
| 6 | [002](002_support_multiple_provider_keys_per_provider.md) | 2016 | 8 | 7 | 9 | 4 | 🔄 (Planned) | Support multiple keys per provider | Remove single-key behavior and enforce owner-scoped key selection |
| 7 | [012](012_unify_error_handling_strategy.md) | 1960 | 7 | 7 | 10 | 4 | 🔄 (Planned) | Unify error handling strategy | Converge formatters and config to Result-based error handling, eliminating silent defaults |
| 8 | [029](029_freeform_workspace_setup.md) | 1800 | 9 | 5 | 10 | 4 | 🔄 (Planned) | FreeForm component infrastructure | Shared `FreeFormDialog` + `DetectedConfirmation` + toggle primitive and LLM-backed parse adapter; foundation for 030–033 |
| 9 | [004](004_per_ic_key_limits.md) | 1728 | 8 | 6 | 9 | 4 | 🔄 (Planned) | Add dedicated limits per IC key | Introduce independent IC-key limit controls per agent |
| 10 | [003](003_provider_key_spending_and_limits.md) | 1620 | 9 | 5 | 9 | 4 | 🔄 (Planned) | Add spending and limits per provider key | Introduce key-level budget accounting and enforcement |
| 11 | [007](007_migrate_to_api_llm_bindings.md) | 1620 | 9 | 5 | 9 | 4 | 🔄 (Planned) | Migrate to `api_llm` bindings | Establish binding layer, migrate all providers, remove legacy direct-HTTP code |
| 12 | [032](032_freeform_providers_and_policy.md) | 1620 | 9 | 5 | 9 | 4 | 🔄 (Planned) | FreeForm: Providers + Usage Policy + `/use-cases` page | Structured paste-block grammar, transactional apply, idempotency, CLI parity, and `/use-cases` showcase |
| 13 | [015](015_budget_lease_auto_refresh_worker.md) | 1536 | 8 | 6 | 8 | 4 | 🔄 (Planned) | Budget lease auto-refresh worker | Add background tokio task for threshold-based budget lease auto-refresh |
| 14 | [017](017_complete_python_sdk_async_bridge.md) | 1500 | 10 | 3 | 10 | 5 | 🔄 (Planned) | Complete Python SDK async bridge | Complete pyo3-async-runtimes bridge and implement start_agent() for Python SDK |
| 15 | [019](019_e2e_rusticon_demo_script.md) | 1500 | 10 | 3 | 10 | 5 | 🔄 (Planned) | End-to-end Rusticon demo script | Create reproducible end-to-end demo validating all Rusticon promises |
| 16 | [030](030_freeform_registration.md) | 1470 | 7 | 7 | 10 | 3 | 🔄 (Planned) | FreeForm: Registration twin | LLM-inferred first/last/birthday from one-sentence paste; classic-form prefill with AI-disclosure subtitle |
| 17 | [031](031_freeform_team_setup.md) | 1470 | 7 | 7 | 10 | 3 | 🔄 (Planned) | FreeForm: Team Setup twin | LLM-inferred company name/domain/account-type from free description; constrained to existing enum |
| 18 | [033](033_invite_link_and_member_access.md) | 1296 | 8 | 6 | 9 | 3 | 🔄 (Planned) | Magic invite link + Member Model Access | Policy-snapshot invite links with seat decrement; member-side IC token panel + Request gated models |
| 19 | [018](018_pypi_distribution_pipeline.md) | 1080 | 8 | 5 | 9 | 3 | 🔄 (Planned) | PyPI distribution pipeline | Build maturin-based PyPI package and distribution pipeline |
| 20 | [005](005_add_gemini_provider.md) | 1008 | 7 | 6 | 8 | 3 | 🔄 (Planned) | Add Gemini inference provider | End-to-end Gemini support in control, runtime, and analytics |
| 21 | [006](006_add_xai_provider.md) | 1008 | 7 | 6 | 8 | 3 | 🔄 (Planned) | Add xAI inference provider | End-to-end xAI support in control, runtime, and analytics |
| 22 | [013](013_increase_test_coverage_zero_modules.md) | 900 | 6 | 5 | 10 | 3 | 🔄 (Planned) | Increase test coverage for untested modules | Add tests for untested public functions in provider_adapter.rs and trace_storage.rs |
| 23 | [022](022_provider_failover.md) | 864 | 8 | 4 | 9 | 3 | 🔄 (Planned) | Provider failover | Implement automatic provider failover with health-based routing |
| 24 | [020](020_postgresql_audit_backend.md) | 840 | 7 | 5 | 8 | 3 | 🔄 (Planned) | PostgreSQL audit backend | Add PostgreSQL AuditBackend implementation for multi-user deployments |
| 25 | [011](011_upgrade_rust_2024_edition.md) | 840 | 5 | 7 | 8 | 3 | 🔄 (Planned) | Upgrade to Rust 2024 edition | Update workspace edition from 2021 to 2024 and fix any breaking changes |
| 26 | [023](023_hardening.md) | 810 | 9 | 3 | 10 | 3 | 🔄 (Planned) | Hardening | Add property tests, fuzz testing, and load testing for hardening |
| 27 | [021](021_token_revocation_vault_backend.md) | 768 | 8 | 4 | 8 | 3 | 🔄 (Planned) | Token revocation and vault backend | Add dedicated token revocation action and integrate vault backend for key storage |
| 28 | [024](024_api_surface_polish.md) | 700 | 7 | 5 | 10 | 2 | 🔄 (Planned) | API surface polish | Stabilize public API surface with consistent naming and versioning |
| 29 | [026](026_documentation_and_examples.md) | 700 | 7 | 5 | 10 | 2 | 🔄 (Planned) | Documentation and examples | Write user-facing documentation, API reference, and integration examples |
| 30 | [025](025_ci_cd_pipeline.md) | 576 | 8 | 4 | 9 | 2 | 🔄 (Planned) | CI/CD pipeline | Set up GitHub Actions CI/CD with automated testing, linting, and release workflows |
| 31 | [027](027_dashboard_integration.md) | 504 | 7 | 4 | 9 | 2 | 🔄 (Planned) | Dashboard integration | Connect Control Panel dashboard to backend API endpoints |
| 32 | [008](008_deploy_iron_cage_internal.md) | 360 | 10 | 4 | 9 | 1 | 🔄 (Planned) | Deploy Iron Cage as internal centralized token control platform | Centralized IP key sharing, IC token distribution, RBAC, and secure internal deployment |

---

## Summary Statistics

### Tasks by Status

| Status | Count | Percentage | Location |
|--------|-------|------------|----------|
| 🔄 Planned | 32 | 100.0% | `task/` |
| ✅ Completed | 0 | 0.0% | `task/completed/` |
| 📥 Backlog | 0 | 0.0% | `task/backlog/` |
| **TOTAL** | **32** | **100%** | |

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
| FreeForm Onboarding | 5 | 5 | 0 | 0 |
| **TOTAL** | **32** | **32** | **0** | **0** |

---

## Organization Structure

### Task File Locations

Tasks are organized in this workspace as:
- **Active**: `task/*.md`
- **Completed**: `task/completed/*.md`
- **Backlog**: `task/backlog/*.md`

### Status Indicators

- 🔄 **Planned**: Ready to start, dependencies met
- ⏳ **In Progress**: Currently being worked on
- ✅ **Completed**: Finished and validated
- ⛔️ **Blocked**: Waiting on dependency or external factor
- 📥 **Backlog**: Future work, not yet prioritized

---

## Navigation

### By Concern

**Deployment & Operations** ⚡
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

**FreeForm Onboarding (Zero-to-Protected-in-60s)**
- Task 029 (FreeForm component infrastructure — `FreeFormDialog`, `DetectedConfirmation`, toggle, parse adapter)
- Task 030 (Registration FreeForm twin)
- Task 031 (Team Setup FreeForm twin)
- Task 032 (Providers + Usage Policy FreeForm + `/use-cases` page)
- Task 033 (Magic invite link + Member Model Access)

---

## Documentation

- Workspace root: `.`
- Task folder: `task/`

---

## Recent Changes

**2026-05-13**:
- ✅ Split task 029 into the full FreeForm Onboarding pillar (Rusticon "Zero to protected in 60 seconds")
  - Task 029 rewritten as **FreeForm component infrastructure** (V=9, E=5, S=10, P=4, Adv=1800)
  - Task 030 added — **Registration FreeForm twin** (V=7, E=7, S=10, P=3, Adv=1470)
  - Task 031 added — **Team Setup FreeForm twin** (V=7, E=7, S=10, P=3, Adv=1470)
  - Task 032 added — **Providers + Usage Policy FreeForm + `/use-cases`** (V=9, E=5, S=9, P=4, Adv=1620)
  - Task 033 added — **Magic invite link + Member Model Access** (V=8, E=6, S=9, P=3, Adv=1296)
- ✅ Updated global registry to highest ID `033` and next available ID `034`
- ✅ Added "FreeForm Onboarding" domain to the by-domain breakdown and navigation
- ✅ Re-sorted index across all 32 tasks by advisability

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
