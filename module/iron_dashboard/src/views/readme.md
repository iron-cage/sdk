# src/views

Top-level route components mounted by the Vue Router. Each view owns a single page and is responsible for its own data fetching via `useApi` and TanStack Vue Query.

| File | Responsibility |
|------|----------------|
| `AgentsView.vue` | Agent CRUD and IC token generate/regenerate/revoke flow |
| `BudgetsView.vue` | Per-agent budget status and allocation management |
| `DashboardView.vue` | Overview page with summary stats and quick actions |
| `LoginView.vue` | Unauthenticated login page |
| `ProvidersView.vue` | AI provider key management, quick-add and full-form create |
| `UsageView.vue` | Analytics and usage breakdown by period, provider and agent |
| `UsersView.vue` | Admin-only user management with role and status controls |
