# Task 032: Providers + Usage Policy FreeForm and `/use-cases` page

## Goal

Ship the FreeForm twin for the Providers screen and the Usage Policy dialog so an admin can, in a single paste, register multiple provider keys, queue team-member invites, set a workspace spending cap, set a default model, and gate models behind a `requestable:` allowlist. The result is observable through the Providers screen showing the toggle, the paste block being parsed into a "Detected" confirmation card listing providers + queued invites + policy items with green checkmarks, and an Apply step that performs the work transactionally. Testable by feeding the five canonical paste blocks from the `/use-cases` page and verifying providers registered, invites queued, cap applied, default and requestable models set, plus idempotency on re-apply. Also ships the `/use-cases` documentation page hosting the five canonical examples.

## Dependencies
- Task 029 (FreeForm component infrastructure) — required
- Task 002 (multi-key per provider) — provider key registration path
- Task 003 (provider key spending and limits) — cap enforcement targets

## Codebase reality check (2026-05-13)
- **No workspace policy model exists.** There is no table or struct holding `default_model`, `requestable_models`, or workspace-wide spending cap. This task owns the design and migration.
- **CLI uses dot-prefix `unilang` convention.** The actual binary is `iron-control` (not `iron_cli`); the form `iron-control .workspace.setup config::"..."` is correct. Example 4 on the `/use-cases` page updated accordingly.
- **No SuperUser role.** Only `Admin` exists. `POST /workspace/setup` is gated to Admin only.
- **"Sitemap v2" is not a real concept** in the repo — routes are inline in the router. This task simply registers `/use-cases` in the existing router file.
- Multi-key provider APIs from PR #56 are in `iron_token_manager` and ready to be called by the apply pipeline.
- `ShadowCard` and `TerminalWindow` design-system primitives exist in `iron_site/` and are the right components for the `/use-cases` page.

## In Scope

- **Workspace policy model**: new table `workspace_policy` with columns `workspace_id`, `default_model`, `requestable_models` (JSON array of model IDs), `spending_cap_amount`, `spending_cap_period` (`day` | `week` | `month`), `updated_at`. Migration script and `iron_token_manager` accessor APIs (`get_policy`, `update_policy`).
- Mount `FreeFormToggle` on the Providers screen and on the Usage Policy "Set Policy" dialog
- Implement two schemas in `SchemaRegistry`:
  - `providers_and_invites` — provider keys (`provider: <key>`) + email invites; mixed paste supported
  - `usage_policy` — `limit all users $N/<period>`, `default: <model>` (or `default model: <model>`), `requestable: <model>[, <model>...]`
- Structured grammar (not LLM) for both schemas — these blocks have a deterministic shape and must round-trip predictably; the AI adapter is reserved for free-prose schemas (Registration, Team Setup)
- Validation: provider key prefix sanity (`sk-`, `sk-ant-`, `AIzaSy`, etc.), RFC-5322 email, model-id against provider catalog, period vocabulary (`day`, `week`, `month`)
- `DetectedConfirmation` card lists per-line outcomes with green checkmarks matching the deck (e.g., "✓ 3 Inference Providers — Gemini · OpenAI · Anthropic", "✓ 10 team members queued, not yet invited")
- Apply pipeline: transactional (all-or-nothing) over existing iron_token_manager APIs for provider key creation, invite issuance, workspace policy update, spending cap configuration
- Idempotency: re-applying the same paste reconciles existing providers/users/policy without duplicate creation
- `POST /workspace/setup` HTTP endpoint (Admin RBAC) sharing the parser/apply core
- `iron-control .workspace.setup config::"..."` subcommand (unilang dot-prefix convention) reaching feature parity with the HTTP path
- `/use-cases` page at `iron_site/src/views/UseCasesPage.vue` showing the five canonical examples as `ShadowCard`s with H2 + `TerminalWindow` + single-line `→` outcome; examples 4 and 5 use a second `TerminalWindow`. No intro, no per-example prose. Route registered in the existing router file (no separate "sitemap" registry exists).

## Out of Scope

- Free-form natural language for these schemas — only the structured grammar
- Per-user (vs workspace-wide) spending caps in the paste — v1 supports `limit all users` only
- Edit/delete operations from the paste — apply is additive + reconciling, never destructive
- Multi-workspace setup in a single paste

## Description

This is the headline FreeForm surface from the demo deck (timestamps 0:18 → 0:26). The Providers screen toggle opens a paste block that mixes provider keys and team-member emails; the parsed result is shown in a Detected card matching the deck exactly: "✓ 3 Inference Providers — Gemini · OpenAI · Anthropic / ✓ 10 team members queued, not yet invited". A separate FreeForm dialog on the Usage Policy "Set Policy" button accepts the cap/default/requestable line and produces a similar Detected card.

Unlike Registration and Team Setup, the input here has a deterministic shape: `provider: key`, `email@domain`, `limit all users $N/period`, `default: model`, `requestable: model, model`. A structured parser (not the LLM) is the right choice — predictable, fast, and the source of truth for the `/use-cases` documentation.

The Apply pipeline runs in one transaction over existing `iron_token_manager` APIs. Re-running the same paste produces a clean no-op report ("already present" per line). The HTTP endpoint and CLI subcommand share the parser/apply core.

The five canonical examples on the `/use-cases` page:

1. **Workspace bootstrap** — 3 provider keys, 3 emails, `limit all users $100/week`, `default model: gpt-5.4-mini` → 3 providers, 3 invites queued, $100/week cap, default set.
2. **Usage policy** — `limit all users $100/week`, `default: gpt-5.4-mini`, `requestable: claude-4-6-sonnet, gemini-3.1-pro-preview` → cap, default, two requestable models.
3. **More invites** — 3 emails alone → 3 invites inheriting workspace policy.
4. **CLI equivalent** — `iron-control .workspace.setup config::"gemini: AIzaSy...xxxx, openai: sk-proj-...xxxx, alice@company.com, bob@company.com"` → 2 providers, 2 invites.
5. **Existing OpenAI code** — env vars `IRON_CAGE_TOKEN`, `OPENAI_BASE_URL` + standard OpenAI SDK call → existing code, governed.

## Context

Critical areas:
- `iron_site/src/views/ProvidersPage.vue`, Usage Policy "Set Policy" dialog — host surfaces
- `iron_site/src/views/UseCasesPage.vue` (new) — documentation page
- `iron_site/src/components/freeform/` (from 029) — primitives consumed here
- `iron_site/src/router/` — register `/use-cases` route here
- `module/iron_control_api/src/freeform/providers_and_invites.rs`, `usage_policy.rs` (new) — schemas + parsers (structured, not LLM)
- `module/iron_control_api/src/routes/workspace_setup.rs` (new) — `POST /workspace/setup`
- `module/iron_token_manager` — existing provider-key APIs (multi-key from PR #56); new `workspace_policy` table + accessors; existing invite APIs (or new if absent — check before implementation)
- `module/iron_cli` — `.workspace.setup` subcommand under `iron-control` binary (unilang dot-prefix convention; YAML command definition under `commands/control/`)

## Work Procedure

1. Design the `workspace_policy` table, write the migration, and add accessor APIs in `iron_token_manager` (`get_policy`, `update_policy`). Verify whether invite APIs already exist; if not, add them as part of this task.
2. Define the exact grammar for both schemas; pin the period vocabulary and model-id resolution rules.
3. Implement the structured parser with span-aware errors as a pure function.
4. Implement the apply pipeline with transactional semantics over existing iron_token_manager APIs.
5. Implement reconciliation: skip existing providers by `(provider, key-prefix)`, skip already-invited emails, update policy fields in place.
6. Mount `FreeFormToggle` on the Providers screen and on the Usage Policy dialog; wire the Detected card to show per-line outcomes with green checkmarks matching the deck.
7. Override the default LLM adapter with the structured adapter for these schemas.
8. Add `POST /workspace/setup` endpoint with Admin RBAC.
9. Add the `.workspace.setup` subcommand under the `iron-control` binary (YAML command definition + handler) sharing the parser/apply core.
10. Build `/use-cases` page with the five examples and register the route in the existing router file.
11. Write integration tests for all five examples plus an idempotency re-apply test.
12. Document the grammar at `docs/freeform_setup.md`.

## Implementation plan

1. Grammar + structured parser with span-aware errors.
2. Transactional apply pipeline + reconciliation.
3. HTTP endpoint and CLI subcommand on top of the shared core.
4. UI integration on Providers and Usage Policy surfaces.
5. `/use-cases` page + sitemap v2 entry.
6. Integration tests + grammar docs.

## Test Matrix

| Input/Scenario | Expected behavior | Pass criteria |
|---|---|---|
| Example 1 paste on empty workspace | 3 providers, 3 invites queued, cap, default set | Detected card matches deck; Apply succeeds; DB reflects each |
| Example 2 paste | Cap, default, two requestable models | Policy row updated; allowlist contains both |
| Example 3 paste (emails only) | 3 invites inheriting workspace policy | 3 invite rows with policy reference |
| Example 4 CLI form | 2 providers, 2 invites | Outcome identical to HTTP path |
| Re-apply Example 1 unchanged | No-op | Report shows all items "already present"; no DB writes |
| Invalid provider key prefix | Parse error, transaction aborted | 400; no partial state |
| Unknown model in `default:` | Parse error referencing catalog | 400 with model-not-found |
| Mixed valid + invalid lines | Whole block rejected | 400 with per-line errors |
| Non-Admin caller | 403 returned | RBAC enforced |
| Unknown period (`/fortnight`) | Parse error | Error lists supported periods |
| Duplicate email in same paste | Deduplicated, one invite issued | Single row; report notes dedup |
| `/use-cases` page renders 5 examples | Five `ShadowCard`s, correct structure | Snapshot tests pass |

## Validation Checklist

- [ ] Structured parser handles all five canonical paste blocks deterministically
- [ ] Apply pipeline is transactional (all-or-nothing)
- [ ] Idempotent re-application is a clean no-op
- [ ] RBAC enforced on `POST /workspace/setup` (Admin only — SuperUser does not exist in the current model)
- [ ] `workspace_policy` table created via migration; `get_policy` / `update_policy` accessors in `iron_token_manager`
- [ ] CLI `iron-control .workspace.setup config::"..."` reaches feature parity with HTTP endpoint
- [ ] Provider key prefix validation enforced per provider
- [ ] Email shape validated
- [ ] Model-id resolved against provider catalog
- [ ] Detected card matches the deck's visual structure (green checkmarks per line)
- [ ] `FreeFormToggle` mounted on Providers and Usage Policy
- [ ] `/use-cases` page exists at `iron_site/src/views/UseCasesPage.vue`
- [ ] Route registered in the existing router file
- [ ] Each example: H2 + `TerminalWindow` + `→` outcome; examples 4 and 5 use a second `TerminalWindow`
- [ ] Grammar documented at `docs/freeform_setup.md`

## Validation Procedure

1. Start the control API and dashboard with a fresh database.
2. Open Providers, click the FreeForm toggle, paste Example 1; verify Detected card shows "✓ 3 Inference Providers — Gemini · OpenAI · Anthropic" and "✓ 10 team members queued, not yet invited" (adjusting for the example's 3 emails).
3. Click Apply; verify providers, invites, cap, and default model appear in the dashboard.
4. Click Set Policy, open the FreeForm toggle, paste Example 2; verify Detected card lists spend limit, default model, and requestable models, then Apply.
5. Paste Example 3; verify three additional invites queued.
6. Run the CLI Example 4 against a fresh workspace; verify identical outcome.
7. Re-paste Example 1; verify the report marks every item "already present" and no DB rows were written.
8. Paste a block with an invalid provider key prefix; verify 400 and no partial state.
9. Attempt the endpoint as a Developer; verify 403.
10. Navigate to `/use-cases`; verify five `ShadowCard`s render with the specified structure.

## Acceptance Criteria

- A single paste block of mixed provider keys, emails, spending caps, default model, and requestable models is parsed and applied in one transaction.
- The Detected card matches the deck's visual structure with green checkmarks per inferred item.
- The five canonical example blocks each produce their documented outcomes.
- Re-applying the same block is a clean no-op (idempotent).
- The `/use-cases` documentation page renders all five examples in the specified `ShadowCard` + `TerminalWindow` layout and is reachable at `/use-cases`.
- The CLI `.workspace.setup` subcommand reaches feature parity with the HTTP endpoint.
- Invalid input rejects the entire block with per-line diagnostics; no partial state is ever persisted.
- RBAC restricts the endpoint to the Admin role (the only role in the current model).
