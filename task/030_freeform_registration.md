# Task 030: Registration FreeForm twin

## Goal

Ship the FreeForm twin of the Registration form so a new admin can paste a single sentence ("Jane Smith, jane@acmecorp.com — born Feb 23 1982") and have first name, last name, and birthday inferred and prefilled. The result is observable through the Registration screen showing the FreeForm toggle in the top-right and the "AI inferred — review and confirm" path landing the user on the classic form with all three fields populated. Testable by feeding a corpus of paste variants (full sentence, comma list, freeform prose, partial info) and verifying expected field extraction plus the Edit-back-to-classic round-trip.

## Dependencies
- Task 029 (FreeForm component infrastructure) — required

## Codebase reality check (2026-05-13)
- `RegistrationPage.vue` **does not exist** in `iron_site/src/views/` — this task scaffolds the classic form as well as the FreeForm twin.
- No birthday-parsing utility exists in either repo; this task introduces a small `parse_birthday(text)` helper covering the formats in scope.
- Only the `Admin` role exists in the RBAC model (no `SuperUser`). Registration is pre-auth in any case — RBAC does not gate it.

## In Scope

- **Scaffold the classic Registration form** at `iron_site/src/views/RegistrationPage.vue` (first_name, last_name, birthday fields with Continue/Confirm button) — required because the page does not yet exist
- Mount `FreeFormToggle` on the classic Registration form
- Implement the `registration` schema in the server-side `SchemaRegistry` (fields: `first_name`, `last_name`, `birthday`)
- Wire the parse adapter to the registration form's apply callback (prefill classic form fields, mark as AI-inferred)
- Confidence-based field rendering: if confidence < threshold on any field, leave it empty for the user to complete
- Birthday parsing supports the formats seen in the deck: "born Feb 23 1982", "23/02/1982", "Feb 23, 1982", "1982-02-23"
- Empty-paste prevention and graceful "could not extract anything" error
- Telemetry: extends the base `freeform.*` events with `schema_id=registration`

## Out of Scope

- SSO sign-in path (GitHub, Google) — already handled by the classic form, unchanged
- Magic-link code recovery — already handled, unchanged
- Birthday timezone normalization beyond ISO date
- Multi-language input (English only for v1)

## Description

The Registration screen is the very first surface a new admin sees after the magic-link login. The classic form requests first name, last name, and birthday. The FreeForm twin accepts a single sentence — the deck shows "Jane Smith, jane@acmecorp.com — born Feb 23 1982" — and infers all three fields plus, implicitly, confirms the email already known from the magic-link session.

The user's path: arrive at Registration → tap the toggle icon in the top-right → paste sentence → Parse → land on the **same classic form** with fields prefilled and the "AI inferred — review and confirm" subtitle. The user reviews, optionally edits, and clicks Confirm. The Edit affordance is therefore implicit — the user is already on the classic form; they just edit a field.

The deck's birthday rendering on the Confirmed screen ("02/23/1982") establishes the display format. Internally fields are stored as ISO 8601 dates.

## Context

Critical areas:
- `src/views/RegistrationPage.vue` — host classic form
- `src/components/freeform/` (from 029) — primitives consumed here
- `module/iron_control_api/src/freeform/registration.rs` (new) — server-side schema and parser
- Date parsing — `chrono` with permissive formats

## Work Procedure

1. Scaffold `RegistrationPage.vue` with the three classic fields (first name, last name, birthday) and the Continue/Confirm button per the deck (timestamps 0:06 → 0:08); add the route to the router.
2. Add `FreeFormToggle` to the Registration page header.
3. Register the `registration` schema in the server-side `SchemaRegistry` with the three target fields and confidence map.
4. Implement the LLM prompt template for the registration schema, with few-shot examples covering the variants in scope.
5. Wire the apply callback: prefill the classic form, render the "AI inferred — review and confirm" subtitle on it, and replace the Continue button with Confirm.
6. Build a paste-variant test corpus and run it through the parser.
7. Verify the Edit affordance: a user can change any prefilled field before confirming.

## Implementation plan

1. Mount toggle, wire schema registration, implement prompt and parser.
2. Apply callback updates the classic form's reactive state.
3. Add the test corpus and run the integration tests.

## Test Matrix

| Paste input | Expected extraction | Pass criteria |
|---|---|---|
| `Jane Smith, jane@acmecorp.com — born Feb 23 1982` | first=Jane, last=Smith, bday=1982-02-23 | All three fields prefilled |
| `Jane Smith, born 23/02/1982` | first=Jane, last=Smith, bday=1982-02-23 | Same |
| `Jane Smith` | first=Jane, last=Smith, bday empty | Bday remains empty, low confidence |
| `Hi I'm Jane and I was born in 1982` | first=Jane, bday empty (year-only insufficient) | Last name and bday empty |
| Empty paste | Error in dialog | "Could not extract" message |
| Paste cancelled | Dialog closes, classic form untouched | No prefill |
| Edit after parse | User changes Last to "Doe" | Confirm sends Doe, not Smith |
| Confidence below threshold | Field empty, others prefilled | User completes manually |

## Validation Checklist

- [ ] `FreeFormToggle` mounted on Registration page
- [ ] `registration` schema registered server-side
- [ ] LLM prompt handles all variants in the test corpus
- [ ] Classic form prefilled with parsed values
- [ ] "AI inferred — review and confirm" subtitle shown on classic form post-parse
- [ ] Confirm button replaces Continue when AI-inferred
- [ ] Low-confidence fields left empty
- [ ] Empty paste shows error
- [ ] Telemetry events emit with `schema_id=registration`

## Validation Procedure

1. Open Registration, click the FreeForm toggle.
2. Paste the canonical sentence; verify Parse → classic form prefilled with all three fields and the AI-inferred subtitle.
3. Edit Last Name; click Confirm; verify the edited value is persisted.
4. Run the full test corpus through the parser and verify the expected extractions.
5. Try an empty paste; verify error.

## Acceptance Criteria

- The Registration form has a FreeForm twin reachable via the toggle.
- Parsing the deck's canonical sentence prefills all three classic-form fields with the AI-inferred subtitle visible.
- Low-confidence fields are left empty; the user can complete them by hand.
- The Edit affordance works by directly editing the classic form before Confirm.
- The 60-second clock from the deck (Registration 0:06 → 0:08) is achievable on a real browser run.
