# iron_dashboard/src - File Responsibility Table

| File | Responsibility |
|------|----------------|
| main.ts | Vue application entry point with router setup |
| App.vue | Root component with authentication routing logic |
| router/index.ts | Vue Router configuration with protected routes |
| components/MainLayout.vue | Authenticated layout wrapper component |
| views/LoginView.vue | Unauthenticated login page |
| views/DashboardView.vue | Overview dashboard with stats and quick actions |
| views/AgentsView.vue | Agent CRUD and IC token management |
| views/UsageView.vue | Analytics and usage breakdown by period/provider/agent |
| views/BudgetsView.vue | Agent budget status and allocation management |
| views/ProvidersView.vue | AI provider key management (admin only) |
| views/UsersView.vue | User management with role and status controls (admin only) |
| stores/auth.ts | Pinia auth store — tokens, role, login/logout/refresh |
| composables/useApi.ts | API client composable wrapping all backend endpoints |
| composables/useConfirm.ts | Reusable confirmation modal state composable |
| components/icons/ | SVG icon components (IconX, IconCheck, IconPlus, …) |
| components/cards/ | Stat and widget card components (StatCard) |
| components/ui/ | shadcn-vue primitive components (button, input, dialog, …) |
| lib/formatters.ts | Display formatting helpers (currency, timestamp, numbers) |
| lib/utils.ts | Tailwind class merge utility (cn) |
| lib/providers.ts | Provider label/icon helpers |
