<!-- task_system_metadata
type: root
version: 1.0
highest_id: 6
last_allocation:
  id: 6
  crate: workspace
  timestamp: 2026-02-06T09:45:00Z
-->

# Master Task Index - Workspace

**Last Updated**: 2026-02-06
**Purpose**: Comprehensive task tracking for this workspace task folder
**Total Tasks**: 6 tasks

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
highest_id: 6
-->

**Current State:**
- **Highest Allocated ID**: 6
- **Active Tasks**: 6
- **Completed Tasks**: 0
- **Backlog Tasks**: 0
- **Task Systems**: 1 (root workspace task system)

**ID Allocation Policy:**
- All task IDs are allocated from this registry
- IDs must be globally unique within this workspace task system
- Next available ID: 7

---

## Tasks Index (Local)

**Note:** Workspace-level tasks for current implementation wave.

| Order | ID | Advisability | Value | Easiness | Safety | Priority | Status | Agent | Task | Description |
|-------|----|--------------:|------:|---------:|-------:|---------:|--------|-------|------|-------------|
| 1 | [001](001-fix-ic-token-regeneration-invalidation.md) | 240 | 8 | 6 | 10 | 1 | 🔄 (Planned) | `control-api-security-agent` | Fix IC token invalidation after regeneration | Ensure old IC tokens are rejected after regenerate/revoke |
| 2 | [002](002-support-multiple-provider-keys-per-provider.md) | 220 | 8 | 7 | 9 | 1 | 🔄 (Planned) | `provider-key-management-agent` | Support multiple keys per provider | Remove single-key behavior and enforce owner-scoped key selection |
| 3 | [003](003-provider-key-spending-and-limits.md) | 230 | 9 | 5 | 9 | 1 | 🔄 (Planned) | `budget-accounting-agent` | Add spending and limits per provider key | Introduce key-level budget accounting and enforcement |
| 4 | [004](004-per-ic-key-limits.md) | 210 | 8 | 6 | 9 | 1 | 🔄 (Planned) | `ic-policy-limits-agent` | Add dedicated limits per IC key | Introduce independent IC-key limit controls per agent |
| 5 | [005](005-add-gemini-provider.md) | 200 | 7 | 6 | 8 | 2 | 🔄 (Planned) | `gemini-integration-agent` | Add Gemini inference provider | End-to-end Gemini support in control, runtime, and analytics |
| 6 | [006](006-add-xai-provider.md) | 200 | 7 | 6 | 8 | 2 | 🔄 (Planned) | `xai-integration-agent` | Add xAI inference provider | End-to-end xAI support in control, runtime, and analytics |

---

## Tasks Index (Aggregated - Active & Backlog Only)

**Note:** Same as local index in this repository (single root task system).

| Order | ID | Crate | Advisability | Value | Easiness | Safety | Priority | Status | Agent | Task | Description |
|-------|----|-------|-------------:|------:|---------:|-------:|---------:|--------|-------|------|-------------|
| 1 | [001](001-fix-ic-token-regeneration-invalidation.md) | workspace | 240 | 8 | 6 | 10 | 1 | 🔄 (Planned) | `control-api-security-agent` | Fix IC token invalidation | Reject stale tokens after rotate/revoke |
| 2 | [002](002-support-multiple-provider-keys-per-provider.md) | workspace | 220 | 8 | 7 | 9 | 1 | 🔄 (Planned) | `provider-key-management-agent` | Multi-key provider support | Create and manage multiple keys per provider safely |
| 3 | [003](003-provider-key-spending-and-limits.md) | workspace | 230 | 9 | 5 | 9 | 1 | 🔄 (Planned) | `budget-accounting-agent` | Provider-key spending/limits | Enforce and expose key-level budgeting |
| 4 | [004](004-per-ic-key-limits.md) | workspace | 210 | 8 | 6 | 9 | 1 | 🔄 (Planned) | `ic-policy-limits-agent` | Per-IC-key limits | Independent IC policy limits per agent |
| 5 | [005](005-add-gemini-provider.md) | workspace | 200 | 7 | 6 | 8 | 2 | 🔄 (Planned) | `gemini-integration-agent` | Gemini provider integration | Add Gemini across API, runtime, analytics |
| 6 | [006](006-add-xai-provider.md) | workspace | 200 | 7 | 6 | 8 | 2 | 🔄 (Planned) | `xai-integration-agent` | xAI provider integration | Add xAI across API, runtime, analytics |

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
| 🔄 Planned | 6 | 100.0% | `task/` |
| ✅ Completed | 0 | 0.0% | `task/completed/` |
| 📥 Backlog | 0 | 0.0% | `task/backlog/` |
| **TOTAL** | **6** | **100%** | |

### Tasks by Domain

| Domain | Total | Planned | Completed | Backlog |
|--------|-------|---------|-----------|---------|
| Security & Auth | 1 | 1 | 0 | 0 |
| Provider Key Management | 2 | 2 | 0 | 0 |
| Budget & Limits | 2 | 2 | 0 | 0 |
| Provider Integrations | 2 | 2 | 0 | 0 |
| **TOTAL** | **6** | **6** | **0** | **0** |

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

---

## Documentation

- Workspace root: `/home/tihilya/obox/iron/sdk`
- Task folder: `/home/tihilya/obox/iron/sdk/task`

---

## Recent Changes

**2026-02-06**:
- ✅ Created root task index for this workspace task system
- ✅ Registered tasks 001-006 with explicit assigned agents and dependency mapping
- ✅ Initialized global registry with highest ID `006` and next available ID `007`

