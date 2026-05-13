# Task 029: FreeForm component infrastructure

## Goal

Establish the shared FreeForm UX primitive that every classic form in the Control Panel can opt into: a toggle icon next to the classic form, a `FreeFormDialog` for paste input, an AI-backed parse adapter, and a `DetectedConfirmation` review step labeled "AI inferred — review and confirm". The result is observable through a single component pair that downstream features (Registration, Team Setup, Providers+Policy, Member Registration) consume to deliver the "Zero to protected in under 60 seconds" onboarding pattern. Testable by mounting the components against a stub parser and verifying the open → paste → parse → detected → apply loop, plus an explicit Cancel/Edit/Back path that loses no user input.

## Dependencies
- None — foundational task that unblocks 030, 031, 032, 033

## Codebase reality check (2026-05-13)
- `iron_llm_core` exposes only `forward_request()` for proxy traffic forwarding — there is **no internal LLM-call abstraction** the backend can use to originate its own inference calls. This task introduces that path.
- RBAC: only the `Admin` role exists today; `SuperUser` is not implemented. Auth on `POST /freeform/parse` is therefore "any authenticated workspace member" (not Admin-gated) — parse is a UX helper, not an admin operation.
- Design-system primitives `ShadowCard` and `TerminalWindow` already exist in `iron_site/` (consumed later by task 032's `/use-cases` page; not directly needed here).

## In Scope

- `FreeFormDialog.vue` — modal with paste textarea, "Paste anything — a sentence, a list, an email, or just describe yourself" placeholder (configurable per host form), Cancel and Parse buttons, top-right toggle icon to return to the classic form
- `DetectedConfirmation.vue` — review card showing parsed fields as a green-check list, "AI inferred — review and confirm" subtitle, Apply and Edit buttons (Edit returns to either the classic form prefilled with parsed values, or to the FreeForm dialog with original text — host decides)
- `FreeFormToggle.vue` — the small icon (top-right of classic form) that opens `FreeFormDialog`; renders consistently across all host forms
- `useFreeForm()` composable orchestrating the open → paste → parse → detected → apply state machine, with per-host adapter injection
- Parse adapter contract: `(rawText, schema) -> Promise<ParsedPayload | ParseError[]>` — host forms supply their own schema and apply callback; the dialog never knows the domain
- **Internal LLM client** in `iron_llm_core` (or a sibling crate): a new high-level abstraction `internal_complete(schema, prompt, examples) -> Result<StructuredJson>` that handles prompt formation, JSON-mode / tool-use response extraction, and cost accounting. Powered by a designated workspace-system IP key (a new concept: "system key" — a provider key marked as available for platform-internal calls, not exposed to users). Cost is attributed to the workspace's own ledger as `internal:freeform_parse` so admins see what FreeForm parsing actually costs.
- Default LLM-backed parse adapter calling a control-API endpoint `POST /freeform/parse` that takes `{ schema_id, raw_text }` and returns typed fields plus a confidence map; the endpoint uses the new internal LLM client. Authentication: any logged-in workspace member (not Admin-gated) — parse is a UX helper
- Telemetry hooks: emit `freeform.opened`, `freeform.parsed`, `freeform.applied`, `freeform.edited`, `freeform.cancelled` events with schema_id, duration, and outcome (no raw text)
- Loss-prevention: Cancel from `DetectedConfirmation` returns to `FreeFormDialog` with the original paste preserved; Cancel from `FreeFormDialog` warns if text is non-empty
- Accessibility: focus trap inside the dialog, Esc to cancel, keyboard navigation between fields in the confirmation card

## Out of Scope

- Any host-form integration (Registration, Team Setup, Providers, Policy, Member Registration) — those land in 030, 031, 032, 033
- The `/freeform/parse` endpoint's per-schema implementations beyond the contract — each consuming task ships its own schema and parse logic
- Offline/local parsing fallback (LLM unavailable) — return a clear error; recovery is the host's choice
- Streaming parse output — single-response model only for v1
- Custom themes; the dialog uses the standard `ShadowCard` styling from the existing design system

## Description

Across the Rusticon onboarding demo every classic form has a FreeForm twin — Registration, Team Setup, Providers, Usage Policy, Member Registration — each reachable via a toggle icon and following the same paste → AI-parsed "Detected" review → Apply pattern. Without a shared primitive, each surface would re-implement the dialog, the confirmation card, the AI-disclosure subtitle, the Edit-back-to-classic affordance, and the loss-prevention rules, drifting in copy and behavior. This task ships the primitive once.

The shape is intentionally minimal: a dialog component, a confirmation component, a toggle component, a composable that orchestrates state, and an adapter contract that hosts implement. The default LLM-backed parse adapter calls a single control-API endpoint that delegates to a per-schema parser registered server-side. Workspace credentials power the parse — end users never see provider keys for the inference call.

The "AI inferred — review and confirm" subtitle is a hard requirement on every confirmation render: this is the explicit AI-disclosure pattern the product depends on for trust. The Edit affordance must round-trip without loss: a user who pastes, parses, then edits one field on the classic form must not lose the rest of the inferred values; a user who clicks "Edit" back into the FreeForm dialog must see their original paste.

## Context

The Control Panel frontend (Vue 3) is the host. Critical areas:
- `iron_site/src/components/freeform/` (new) — `FreeFormDialog.vue`, `DetectedConfirmation.vue`, `FreeFormToggle.vue`, `useFreeForm.ts`
- `iron_site/src/api/freeform.ts` (new) — default parse adapter calling `POST /freeform/parse`
- `module/iron_llm_core/src/internal.rs` (new) — internal LLM client (`internal_complete()`) and "system key" concept
- `module/iron_token_manager` (extension) — flag on `provider_key` rows marking system-internal keys; reservation API for internal cost accounting
- `module/iron_control_api/src/routes/freeform.rs` (new) — `POST /freeform/parse` endpoint dispatching on `schema_id`
- `module/iron_control_api/src/freeform/` (new) — schema registry trait; per-schema implementations land in consuming tasks
- Existing `ShadowCard` and design tokens for visual consistency

## Work Procedure

1. Define the parse adapter TypeScript contract and the `POST /freeform/parse` JSON shape (request: `schema_id`, `raw_text`; response: `fields`, `confidence`, `notes`, or `errors[]`).
2. Implement `FreeFormDialog.vue` with paste textarea, configurable placeholder, top-right toggle icon, Cancel/Parse buttons, and Esc/focus-trap behavior.
3. Implement `DetectedConfirmation.vue` rendering the parsed payload as a green-check list with the mandatory "AI inferred — review and confirm" subtitle and Apply/Edit buttons.
4. Implement `FreeFormToggle.vue` for use in the top-right corner of any classic form.
5. Implement `useFreeForm()` composable wiring the state machine and emitting telemetry events.
6. Implement the default LLM-backed parse adapter and the server-side `POST /freeform/parse` endpoint with a `SchemaRegistry` trait (no schemas registered yet; that's per consuming task).
7. Write unit tests for the state machine: open → paste → parse success → detected → apply; open → paste → parse error → stay in dialog; detected → edit back to dialog (text preserved); detected → edit back to classic (fields prefilled).
8. Write Storybook stories for each component with a stub parse adapter (no backend dependency).
9. Document the integration recipe in `docs/freeform_integration.md` for the consuming tasks.

## Implementation plan

1. Components and composable in `src/components/freeform/`.
2. Default parse adapter in `src/api/freeform.ts`.
3. Server endpoint and registry trait in `module/iron_control_api`.
4. Telemetry events and integration docs.

## Test Matrix

| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Open FreeForm from classic form | Dialog appears, textarea focused | Focus on textarea, Esc closes |
| Parse success | Detected card shown with parsed fields | Green check list rendered, "AI inferred — review and confirm" visible |
| Parse failure | Stay in dialog, show error list | Error messages shown per line; paste preserved |
| Click Apply | Host apply callback invoked, dialog closes | Callback receives typed payload |
| Click Edit from detected card | Return to classic form, prefilled | Classic form fields populated; user can adjust |
| Cancel from detected | Return to dialog with original text | Textarea contains original paste |
| Cancel from dialog with text | Confirmation warning | User can confirm discard or keep editing |
| Esc in dialog | Same as Cancel | Loss-prevention warning when text non-empty |
| Telemetry events fired | All 5 events emitted at correct points | Event payloads contain schema_id and outcome, no raw text |
| Server endpoint with unknown schema_id | 400 returned | Clear error in adapter, surfaced in dialog |
| Toggle icon visible on every classic-form host | Consistent placement, top-right | Visual snapshot tests pass |

## Validation Checklist

- [ ] `FreeFormDialog.vue` implemented with Cancel/Parse, Esc, focus trap, top-right toggle icon
- [ ] `DetectedConfirmation.vue` implemented with mandatory "AI inferred — review and confirm" subtitle
- [ ] `FreeFormToggle.vue` implemented and visually consistent
- [ ] `useFreeForm()` composable orchestrates state machine
- [ ] Default LLM-backed parse adapter calls `POST /freeform/parse`
- [ ] Server endpoint and `SchemaRegistry` trait exist (no schemas yet)
- [ ] Loss-prevention: Cancel-from-detected preserves paste; Cancel-from-dialog warns
- [ ] Telemetry events fire without raw text
- [ ] Storybook stories cover all states
- [ ] Integration recipe documented at `docs/freeform_integration.md`

## Validation Procedure

1. Mount `FreeFormDialog` against a stub parser in Storybook; verify open/close/Esc/focus-trap.
2. Drive the state machine: paste → Parse → Detected → Apply; verify apply callback fires.
3. Paste invalid input; verify error list rendered and paste preserved.
4. From Detected, click Edit → verify return to classic form with fields prefilled.
5. From Detected, click Cancel → verify dialog reopens with original paste.
6. Inspect emitted telemetry events; verify no raw text in payloads.
7. Hit `POST /freeform/parse` with unknown `schema_id`; verify 400 and adapter surfaces error.
8. Run visual snapshot tests for the toggle icon across mounted host forms.

## Acceptance Criteria

- A single `FreeFormDialog` + `DetectedConfirmation` + `FreeFormToggle` trio is the only implementation any host form uses.
- The "AI inferred — review and confirm" subtitle is always rendered on the confirmation card.
- The state machine round-trips Edit back to either the classic form (prefilled) or the dialog (paste preserved) without loss.
- An internal LLM client exists in `iron_llm_core` with a "system key" concept and per-call cost attribution.
- The default LLM-backed parse adapter and `POST /freeform/parse` endpoint exist with a schema registry trait.
- Telemetry events emit at the five lifecycle points without leaking raw paste content.
- Integration documentation enables tasks 030, 031, 032, 033 to ship without re-implementing the primitive.
