# TKT-01 — iron_sdk: Commit and push completed feat/server-proxy session fixes

- **Priority:** 1 · important
- **Status:** open
- **Client:** iron_sdk
- **Assigned:** Wandalen
- **Opened:** 2026-03-21
- **Deadline:** —
- **Closed:** —

## Goal

The completed PR #53 session fixes are committed to `feat/server-proxy`, pushed to `origin`, and PR #53 description is updated to reflect the new changes.

## Context

On 2026-03-21, two continuation sessions addressed four of the 44 findings from the PR #53 review. The following changes are complete and verified (3033 tests pass, 5 skipped, 0 failures, clippy clean) but remain uncommitted on the `feat/server-proxy` branch:

**S7 — Restored deleted security tests:**
- `module/iron_control_api/tests/auth/authorization_bypass_comprehensive.rs` — 5 `#[ignore]`d RBAC privilege-escalation tests restored from commit `98e9326` with role names updated for post-migration-025 vocabulary (`"manager"` / `"developer"` replacing `"user"` / `"viewer"`).
- `module/iron_control_api/tests/auth.rs` — module declaration added.
- `module/iron_control_api/tests/auth/readme.md` — Responsibility Table row added.

**S2 partial — Migration 023 randomblob fix:**
- `module/iron_token_manager/migrations/023_seed_agent1_ic_token.sql` — Replaced hardcoded predictable hash (`897b52e2...`) with `lower(hex(randomblob(32)))`. Added `BEGIN`/`COMMIT` transaction wrapper.

**D3 — Rate limiter O(n) sweep removed:**
- `module/iron_server_proxy/src/rate_limiter.rs` — Removed `lock_and_sweep()` method that swept all LRU cache entries on every request (O(n × k)). Replaced with lazy per-IP expiry in `check()` and `record_failure()` (O(k), k ≤ 20 max failures per IP).

**D11 — Migrations 026 and 027 registered:**
- `module/iron_token_manager/src/migrations.rs` — Added `migration!("026_add_spending_constraints")` and `migration!("027_add_agent_token_index")` to the static `MIGRATIONS` list so they run on fresh databases.

**Q5 — qqq: marker removed:**
- `module/iron_llm_core/src/provider.rs` — Removed stray `// qqq: gemini and xai providers also need to be added to the Python client library.` comment.

**Additional changes from prior sessions** (also uncommitted) appear across the remaining ~18 files visible in `git status`, including proxy logic, provider routes, handshake routes, init_admin CLI, and spending cap tests.

The user's explicit policy forbids git operations without direct authorization. The work is complete and verified.

## Done When

- [ ] All modified and untracked files reviewed by Wandalen
- [ ] Changes committed to `feat/server-proxy` with appropriate commit message(s)
- [ ] Branch pushed to `origin/feat/server-proxy`
- [ ] PR #53 on GitHub updated (description and/or comments) noting the session-23 fixes

## Links

- [module/iron_server_proxy/src/rate_limiter.rs](../module/iron_server_proxy/src/rate_limiter.rs)
- [module/iron_token_manager/migrations/023_seed_agent1_ic_token.sql](../module/iron_token_manager/migrations/023_seed_agent1_ic_token.sql)
- [module/iron_token_manager/src/migrations.rs](../module/iron_token_manager/src/migrations.rs)
- [module/iron_control_api/tests/auth/authorization_bypass_comprehensive.rs](../module/iron_control_api/tests/auth/authorization_bypass_comprehensive.rs)
- [-default_topic/-pr53_fix_plan.md](-pr53_fix_plan.md)
