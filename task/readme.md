<!-- task_system_metadata
type: root
version: 1.0
highest_id: 11
last_allocation:
  id: 11
  crate: workspace
  timestamp: 2026-03-06T00:00:00Z
-->

# Master Task Index - Workspace

**Last Updated**: 2026-03-06
**Purpose**: Comprehensive task tracking for this workspace task folder
**Total Tasks**: 11 tasks

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
highest_id: 8
-->

**Current State:**
- **Highest Allocated ID**: 11
- **Active Tasks**: 11
- **Completed Tasks**: 0
- **Backlog Tasks**: 0
- **Task Systems**: 1 (root workspace task system)

**ID Allocation Policy:**
- All task IDs are allocated from this registry
- IDs must be globally unique within this workspace task system
- Next available ID: 12

---

## Tasks Index (Local)

**Note:** Workspace-level tasks for current implementation wave.

| Order | ID | Advisability | Value | Easiness | Safety | Priority | Status | Task | Description |
|-------|----|--------------:|------:|---------:|-------:|---------:|--------|------|-------------|
| 1 | [008](backend/008-deploy-iron-cage-internal.md) | 1800 | 10 | 4 | 9 | 5 | 🔄 (Planned) | Deploy Iron Cage as internal centralized token control platform | Centralized IP key sharing, IC token distribution, RBAC, and secure internal deployment by 2026-02-28 |
| 2 | [001](backend/001-fix-ic-token-regeneration-invalidation.md) | 240 | 8 | 6 | 10 | 1 | 🔄 (Planned) | Fix IC token invalidation after regeneration | Ensure old IC tokens are rejected after regenerate/revoke |
| 3 | [007](backend/007-migrate-to-api-llm-bindings.md) | 235 | 9 | 5 | 9 | 1 | 🔄 (Planned) | Migrate to `api_llm` bindings | Establish binding layer, migrate all providers, remove legacy direct-HTTP code |
| 4 | [003](backend/003-provider-key-spending-and-limits.md) | 230 | 9 | 5 | 9 | 1 | 🔄 (Planned) | Add spending and limits per provider key | Introduce key-level budget accounting and enforcement |
| 5 | [002](backend/002-support-multiple-provider-keys-per-provider.md) | 220 | 8 | 7 | 9 | 1 | 🔄 (Planned) | Support multiple keys per provider | Remove single-key behavior and enforce owner-scoped key selection |
| 6 | [004](backend/004-per-ic-key-limits.md) | 210 | 8 | 6 | 9 | 1 | 🔄 (Planned) | Add dedicated limits per IC key | Introduce independent IC-key limit controls per agent |
| 7 | [005](backend/005-add-gemini-provider.md) | 200 | 7 | 6 | 8 | 2 | 🔄 (Planned) | Add Gemini inference provider | End-to-end Gemini support in control, runtime, and analytics |
| 8 | [006](backend/006-add-xai-provider.md) | 200 | 7 | 6 | 8 | 2 | 🔄 (Planned) | Add xAI inference provider | End-to-end xAI support in control, runtime, and analytics |
| 9 | [010](backend/010-analytics-trend-comparison.md) | 180 | 7 | 6 | 10 | 2 | 🔄 (Planned) | Analytics trend comparison | Backend analytics improvements and trend data |
| 10 | [009](frontend/009-linear-style-ui.md) | 150 | 6 | 7 | 10 | 3 | 🔄 (Planned) | Linear-style UI | Restyle dashboard to match Linear's light mode aesthetic |
| 11 | [011](frontend/011-analytics-ui-refactor.md) | 150 | 6 | 7 | 10 | 3 | 🔄 (Planned) | Analytics UI refactor | Surface unused backend data, add spending by agent and cost efficiency views |

---

## Tasks Index (Aggregated - Active & Backlog Only)

**Note:** Same as local index in this repository (single root task system).

| Order | ID | Crate | Advisability | Value | Easiness | Safety | Priority | Status | Task | Description |
|-------|----|-------|-------------:|------:|---------:|-------:|---------:|--------|------|-------------|
| 1 | [008](backend/008-deploy-iron-cage-internal.md) | backend | 1800 | 10 | 4 | 9 | 5 | 🔄 (Planned) | Internal deployment | IP key sharing, IC distribution, RBAC, TLS — by 2026-02-28 |
| 2 | [001](backend/001-fix-ic-token-regeneration-invalidation.md) | backend | 240 | 8 | 6 | 10 | 1 | 🔄 (Planned) | Fix IC token invalidation | Reject stale tokens after rotate/revoke |
| 3 | [007](backend/007-migrate-to-api-llm-bindings.md) | backend | 235 | 9 | 5 | 9 | 1 | 🔄 (Planned) | Migrate to `api_llm` bindings | Establish binding layer, migrate all providers, remove legacy direct-HTTP code |
| 4 | [003](backend/003-provider-key-spending-and-limits.md) | backend | 230 | 9 | 5 | 9 | 1 | 🔄 (Planned) | Provider-key spending/limits | Enforce and expose key-level budgeting |
| 5 | [002](backend/002-support-multiple-provider-keys-per-provider.md) | backend | 220 | 8 | 7 | 9 | 1 | 🔄 (Planned) | Multi-key provider support | Create and manage multiple keys per provider safely |
| 6 | [004](backend/004-per-ic-key-limits.md) | backend | 210 | 8 | 6 | 9 | 1 | 🔄 (Planned) | Per-IC-key limits | Independent IC policy limits per agent |
| 7 | [005](backend/005-add-gemini-provider.md) | backend | 200 | 7 | 6 | 8 | 2 | 🔄 (Planned) | Gemini provider integration | Add Gemini across API, runtime, analytics |
| 8 | [006](backend/006-add-xai-provider.md) | backend | 200 | 7 | 6 | 8 | 2 | 🔄 (Planned) | xAI provider integration | Add xAI across API, runtime, analytics |
| 9 | [010](backend/010-analytics-trend-comparison.md) | backend | 180 | 7 | 6 | 10 | 2 | 🔄 (Planned) | Analytics trend comparison | Backend analytics improvements and trend data |
| 10 | [009](frontend/009-linear-style-ui.md) | frontend | 150 | 6 | 7 | 10 | 3 | 🔄 (Planned) | Linear-style UI | Restyle dashboard to match Linear's light mode aesthetic |
| 11 | [011](frontend/011-analytics-ui-refactor.md) | frontend | 150 | 6 | 7 | 10 | 3 | 🔄 (Planned) | Analytics UI refactor | Surface unused backend data, spending by agent, cost efficiency |

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
| 🔄 Planned | 11 | 100.0% | `task/` |
| ✅ Completed | 0 | 0.0% | `task/completed/` |
| 📥 Backlog | 0 | 0.0% | `task/backlog/` |
| **TOTAL** | **11** | **100%** | |

### Tasks by Domain

| Domain | Total | Planned | Completed | Backlog |
|--------|-------|---------|-----------|---------|
| Security & Auth | 1 | 1 | 0 | 0 |
| Provider Key Management | 1 | 1 | 0 | 0 |
| Budget & Limits | 2 | 2 | 0 | 0 |
| Provider Integrations | 2 | 2 | 0 | 0 |
| LLM Binding & Migration | 1 | 1 | 0 | 0 |
| Analytics | 1 | 1 | 0 | 0 |
| Frontend & UI | 2 | 2 | 0 | 0 |
| **TOTAL** | **11** | **11** | **0** | **0** |

---

## Organization Structure

### Task File Locations

Tasks are organized in this workspace as:
- **Backend**: `task/backend/*.md`
- **Frontend**: `task/frontend/*.md`
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

**Deployment & Operations**
- Task 008 (internal deployment — IP key sharing, IC distribution, RBAC, TLS)

**Security & Token Lifecycle**
- Task 001 (IC token invalidation)

**Provider Key Model and Multi-tenancy**
- Task 002 (multi-key support)
- Task 003 (spending/limits per provider key)

**Budget and Policy Enforcement**
- Task 003 (provider-key enforcement)
- Task 004 (per-IC-key limits)

**Provider Integrations**
- Task 005 (Gemini)
- Task 006 (xAI)

**LLM Binding Standardization and Migration**
- Task 007 (migrate to `api_llm` bindings)

**Analytics**
- Task 010 (analytics trend comparison — backend)

**Frontend & UI**
- Task 009 (Linear-style UI)
- Task 011 (analytics UI refactor)

---

## Documentation

- Workspace root: `/home/tihilya/obox/iron/sdk`
- Task folder: `/home/tihilya/obox/iron/sdk/task`

---

## Recent Changes

**2026-03-06**:
- ✅ Registered task 009 (Linear-style UI) — frontend
- ✅ Registered task 010 (Analytics trend comparison) — backend
- ✅ Registered task 011 (Analytics UI refactor) — frontend
- ✅ Restructured task folder into backend/, frontend/, devops/ subfolders
- ✅ Updated global registry to highest ID `011` and next available ID `012`

**2026-02-22**:
- ✅ Registered task 008 (internal deployment) — Value=10, Easiness=4, Safety=9, Priority=5, Advisability=1800
- ✅ Updated global registry to highest ID `008` and next available ID `009`
- ✅ Re-sorted all indexes by advisability (task 008 at top)

**2026-02-06**:
- ✅ Created root task index for this workspace task system
- ✅ Registered tasks 001-007 with implementation plans and acceptance criteria
- ✅ Added `api_llm` binding migration task
- ✅ Updated global registry to highest ID `007` and next available ID `008`
