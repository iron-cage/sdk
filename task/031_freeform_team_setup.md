# Task 031: Team Setup FreeForm twin

## Goal

Ship the FreeForm twin of the Team Account Setup form so an admin can paste a single description ("Acme Corp — acme.com — B2B SaaS, 40 engineers, Series B") and have company name, domain, and account type inferred and prefilled. The result is observable through the Team Account Setup screen showing the FreeForm toggle and the parsed payload landing on the classic form with the "AI inferred — review and confirm" subtitle visible. Testable by feeding paste variants (full description, minimal pair, name+domain only, prose) and verifying expected field extraction, account-type classification, and the Edit-back round-trip.

## Dependencies
- Task 029 (FreeForm component infrastructure) — required

## Codebase reality check (2026-05-13)
- `TeamSetupPage.vue` **does not exist** in `iron_site/src/views/` — this task scaffolds the classic form first.
- No `account_type` enum exists yet — this task defines the enum (initial values: `Client Account`, `Internal`; extensible) in `iron_types` and exposes it through the workspace policy model.
- No RFC-1035 hostname validation helper exists — introduce a small utility for it.
- Only the `Admin` role exists. Team Setup is per-workspace and runs immediately after Registration; it should be restricted to the workspace's first user (the registering admin) — there's no SuperUser role to consider.

## In Scope

- **Scaffold the classic Team Account Setup form** at `iron_site/src/views/TeamSetupPage.vue` (company name, company domain, account type fields with Continue/Confirm button) — required because the page does not yet exist
- **Define the `account_type` enum** in `iron_types` with the initial values seen in the deck (Client Account, Internal); persist on the workspace record
- **Domain validation utility** (RFC-1035 hostname shape) in `iron_types` or a shared validation crate
- Mount `FreeFormToggle` on the Team Account Setup form
- Implement the `team_setup` schema in `SchemaRegistry` (fields: `company_name`, `company_domain`, `account_type`)
- LLM prompt that classifies the company description into the existing `account_type` enum (Client Account, Internal, etc. — match current `accountType` options)
- Domain validation (RFC-1035 hostname shape); reject obvious garbage
- Wire apply callback to prefill the classic form
- Telemetry: extends with `schema_id=team_setup`

## Out of Scope

- New `account_type` values beyond what the classic form already supports
- Company logo extraction or favicon scraping
- Multi-tenant workspaces beyond what the platform currently models
- Address / billing fields (not in the deck's flow)

## Description

The Team Account Setup screen appears immediately after Registration in the demo flow (deck timestamps 0:11 → 0:13). The classic form has three fields: company name, company domain, account type. The FreeForm twin accepts a free-text description — the deck shows "Acme Corp — acme.com — B2B SaaS, 40 engineers, Series B" — and infers all three.

The interesting part is account-type classification: "B2B SaaS, 40 engineers, Series B" maps to "Client Account" in the deck. The LLM prompt must classify against the current enum, not invent new categories. If the description is ambiguous, the parser returns low confidence and the field is left empty for the user to pick from the existing dropdown.

The user flow mirrors Registration: paste → Parse → classic form prefilled with the AI-inferred subtitle → Confirm.

## Context

Critical areas:
- `src/views/TeamSetupPage.vue` (or equivalent) — host classic form
- `module/iron_control_api/src/freeform/team_setup.rs` (new) — server-side schema and parser
- Existing `account_type` enum definition — single source of truth for classification

## Work Procedure

1. Define the `account_type` enum in `iron_types` and the matching workspace-record column (or extend the workspace policy table from task 032 if it lands first; otherwise add a small migration here).
2. Implement the RFC-1035 domain validation utility.
3. Scaffold `TeamSetupPage.vue` with the three classic fields and the Continue/Confirm button (deck 0:11 → 0:13); add the route to the router.
4. Add `FreeFormToggle` to the Team Account Setup page header.
5. Register the `team_setup` schema with three target fields and the `account_type` enum injected into the prompt.
6. Implement the LLM prompt with few-shot examples covering the deck's canonical description and several variants.
7. Wire the apply callback to prefill the classic form and render the "AI inferred — review and confirm" subtitle.
8. Add the paste-variant test corpus.
9. Verify that the account_type dropdown still works for low-confidence cases.

## Implementation plan

1. Mount toggle, register schema, implement prompt.
2. Apply callback prefills classic form.
3. Add tests and verify against the deck flow.

## Test Matrix

| Paste input | Expected extraction | Pass criteria |
|---|---|---|
| `Acme Corp — acme.com — B2B SaaS, 40 engineers, Series B` | name=Acme Corp, domain=acme.com, type=Client Account | All three prefilled |
| `Acme Corp, acme.com` | name=Acme Corp, domain=acme.com, type empty | First two filled, type left for user |
| `Internal AI lab at MegaCorp` | name=MegaCorp, type=Internal (if enum has it), domain empty | Name + type filled, domain empty |
| `acme.com` (domain only) | domain=acme.com, name empty | Only domain filled |
| `cats` (garbage) | All empty, low confidence | Dialog error: nothing extracted |
| Edit after parse | User changes domain | Confirm persists edited value |
| Invalid domain shape | Domain field empty, classification continues | No invalid hostname accepted |

## Validation Checklist

- [ ] `FreeFormToggle` mounted on Team Account Setup
- [ ] `team_setup` schema registered server-side
- [ ] Account-type classification uses existing enum only
- [ ] Domain validation rejects invalid hostnames
- [ ] Classic form prefilled with parsed values
- [ ] "AI inferred — review and confirm" subtitle shown
- [ ] Low-confidence fields left empty
- [ ] Telemetry events emit with `schema_id=team_setup`

## Validation Procedure

1. Open Team Account Setup, click the FreeForm toggle.
2. Paste the canonical description; verify Parse → all three fields prefilled with the AI-inferred subtitle.
3. Run the full test corpus and verify expected extractions.
4. Try an invalid domain in the paste; verify the domain field is left empty rather than populated with garbage.
5. Verify that the account_type dropdown still functions for cases where the parser returned low confidence.

## Acceptance Criteria

- The Team Account Setup form has a FreeForm twin reachable via the toggle.
- Parsing the deck's canonical description prefills company name, domain, and account type.
- Account-type classification is constrained to the existing enum.
- Domain validation prevents invalid hostnames from being prefilled.
- The 60-second clock from the deck (Team Setup 0:11 → 0:13) is achievable on a real browser run.
