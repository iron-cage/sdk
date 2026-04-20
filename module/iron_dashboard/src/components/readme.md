# src/components

Shared Vue components used across views. Low-level UI primitives live under `ui/`; this directory holds project-specific compositions built on top of them.

| File / Directory | Responsibility |
|------------------|----------------|
| `AvatarInitial.vue` | Circular avatar that renders the user's initial |
| `ConfirmDialog.vue` | Reusable confirmation dialog wired to `useConfirm` state |
| `DataTable.vue` | Table wrapper with loading, error, empty and retry states |
| `MainLayout.vue` | Authenticated layout wrapper with sidebar and navigation |
| `PageLayout.vue` | Standard page frame with title and `actions` slot |
| `PercentBar.vue` | Horizontal progress bar showing a percentage value |
| `ProviderBadge.vue` | Coloured badge displaying an AI provider name |
| `StatusBadge.vue` | Badge indicating active/inactive state |
| `TrendBadge.vue` | Badge showing positive or negative trend delta |
| `cards/` | Dashboard stat and widget cards (e.g., `StatCard.vue`) |
| `icons/` | SVG icon components (see `icons/readme.md`) |
| `ui/` | Low-level UI primitives wrapping Reka UI (see `ui/readme.md`) |
