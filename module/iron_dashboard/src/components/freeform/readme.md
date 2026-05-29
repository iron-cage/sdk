# freeform/ - FreeForm Onboarding UI Components

## Responsibility Table

| File                  | Responsibility                                                                                    |
| --------------------- | ------------------------------------------------------------------------------------------------- |
| `AutoSetupWizard.vue` | Orchestrate the 4-step onboarding flow (company -> providers -> budget -> invite) against the API |
| `FreeFormDialog.vue`  | Reusable controlled paste dialog: a textarea that emits trimmed text for the parent to parse      |
| `FreeFormToggle.vue`  | Presentational icon button that signals a request to switch into FreeForm mode                    |
| `index.ts`            | Barrel re-export of the three FreeForm components                                                 |

## Directory Purpose

UI layer for the FreeForm onboarding flow. The components are split by
responsibility so the parsing/API logic lives in exactly one place
(`AutoSetupWizard.vue`) while the reusable dialog and toggle stay free of
business logic and can be reused elsewhere.

Data flow: a host view shows `FreeFormToggle`; clicking it opens
`AutoSetupWizard`, which drives `FreeFormDialog` once per step, sends each
trimmed paste to the API (`useApi`), and accumulates the per-step results.
`FreeFormDialog` itself never calls the API - it only collects text and emits
it, so the same dialog backs every step and any future freeform entry point.

## Component Contracts

### `AutoSetupWizard.vue`

- Props: `open: boolean`
- Emits: `update:open(value: boolean)`, `complete(results)` - where `results`
  holds the company / providers / budget / invite API responses.
- Owns: the `step` state machine (`company -> providers -> budget -> invite ->
done`), `parsing`/`error` flags, and all `useApi` calls
  (`freeformCompany`, `freeformProviders`, `setWorkspaceBudget`,
  `generateInvite`). Reuses `FreeFormDialog` for steps 1-3 and an inline
  `Dialog` for the invite form + generated-link result (with copy-to-clipboard).

### `FreeFormDialog.vue`

- Props: `open` (required), `title`, `description`, `placeholder`,
  `initialText`, `parseLabel`, `cancelLabel`, `parsing`, `error` (all optional
  with defaults).
- Emits: `update:open(value)`, `parse(text)` (text is trimmed; not emitted
  while `parsing` or when empty), `cancel`.
- Controlled component: the parent owns `open`, `parsing`, and `error`; the
  dialog resets its textarea to `initialText` whenever it (re)opens.

### `FreeFormToggle.vue`

- Props: `active?: boolean`, `title?: string`
- Emits: `click`
- Pure presentation: a `Sparkles` icon button with `aria-pressed`/`aria-label`
  for accessibility. Holds no state and performs no logic.

### `index.ts`

- Re-exports `AutoSetupWizard`, `FreeFormDialog`, and `FreeFormToggle` so
  consumers import from `@/components/freeform` rather than individual files.
