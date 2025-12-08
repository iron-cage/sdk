# Pilot Development References

**Master reference for implementing Iron Cage pilot (Warsaw conference demo)**

**Last Updated:** 2025-11-24
**Status:** Active development (23 days to conference)

---

## Quick Navigation

| Section | Purpose |
|---------|---------|
| [What to Build](#what-to-build) | Features and technologies |
| [How to Build](#how-to-build) | Implementation guides and dependencies |
| [Where to Build](#where-to-build) | Code locations and structure |
| [Start Here](#start-here) | Quick start workflow |

---

## What to Build

| File | Purpose |
|------|---------|
| `/pilot/spec.md` | 35+ pilot features (with secrets) with acceptance criteria |
| `/pilot/tech_stack.md` | All technologies (Rust 1.75+, Python 3.11+, Vue 3.4+) |

---

## How to Build

| File | Purpose |
|------|---------|
| `-crates_workspace_migration_plan.md` | Step-by-step migration from runtime to workspace |
| `-crates_implementation_list.md` | Complete workspace structure (6 crates) |
| `/pilot/crates.md` | WHY each dependency crate needed (external: tokio, pyo3, axum) |
| `rulebook.md` | Mandatory workspace organization rules |

**Dependency distinction:**
- **External** (`/pilot/crates.md`): Dependencies we USE (tokio, pyo3, axum, etc.)
- **Internal** (`-crates_implementation_list.md`): Crates WE build (types, safety, cost, reliability, cli)
- **Migration** (`-crates_workspace_migration_plan.md`): How to migrate from runtime to workspace

---

## Where to Build

| Path | Purpose |
|------|---------|
| `/types/` | Rust library: `iron_cage_types` (shared types, traits) |
| `/safety/` | Rust library: `iron_cage_safety` (privacy protection, validation) |
| `/cost/` | Rust library: `iron_cage_cost` (budget tracking, token counting) |
| `/reliability/` | Rust library: `iron_cage_reliability` (circuit breakers, fallbacks) |
| `/cli/` | Rust binary: `iron_cage_cli` (CLI interface, orchestration) |
| `/pilot/demo/agent/` | Python lead generation agent (LangChain) |
| `/pilot/demo/control panel/` | Vue 3 control panel (real-time metrics) |

**Current status:**
- Workspace migration: Not started (see `-crates_workspace_migration_plan.md`)
- Legacy `runtime/` crate: Exists (will be deprecated after migration)
- Demo agent: Not implemented
- Control Panel: Not implemented

---

## Start Here

**Recommended workflow for developers:**

1. **Understand requirements**
   - Read `/pilot/spec.md` (35+ features with secrets)
   - Check `/pilot/tech_stack.md` (technology constraints)
   - Read `rulebook.md` (mandatory workspace architecture rules)

2. **Execute migration** (if not done yet)
   - Follow `-crates_workspace_migration_plan.md` (16-hour plan)
   - Review `-crates_implementation_list.md` (6-crate structure)

3. **Write code** (parallel development)
   - Developer 1: Implement in `/safety/src/` (privacy protection)
   - Developer 2: Implement in `/cost/src/` (budget tracking)
   - Developer 3: Implement in `/reliability/src/` (circuit breakers)
   - All: Add tests in respective `tests/` directories

4. **Verify**
   - Run tests: `cargo test -p iron_cage_safety` (isolated)
   - Integration: `cargo test -p iron_cage_cli`
   - Check against acceptance criteria in `/pilot/spec.md`

---

## Related Files

| Category | Files |
|----------|-------|
| **Specifications** | `/spec/` (full platform), `/pilot/spec.md` (pilot features) |
| **Business** | `/business/strategy/` (WHY), `/business/presentations/` (decks) |
| **Documentation** | `/docs/architecture.md`, `/docs/deployment_guide.md` |
| **Conference** | `/conferences/warsaw_2025/` (Dec 16-17, 2025) |

---

## Key Decisions

| Decision | Rationale | File |
|----------|-----------|------|
| Rust 1.75+ | Memory safety, performance | `/pilot/tech_stack.md` |
| Vue 3 (not React) | Smaller bundle, composition API | `/pilot/tech_stack.md` |
| **uv (not pip)** | **10-100x faster, better dependency resolution** | **`/pilot/tech_stack.md`** |
| **Workspace with 6 crates** | **Enables parallel development for 3 developers** | **`rulebook.md`** |
| Slides-only approach | Best for 23 days remaining | `/conferences/warsaw_2025/` |

---

**File Location:** `/pilot/development_references.md`
**Type:** Persistent master reference (not temporary)
