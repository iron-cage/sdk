# iron_dashboard - Specification

**Module:** iron_dashboard
**Layer:** 6 (Application)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

Web control panel for Iron Cage administration. Vue 3 SPA providing real-time agent monitoring, token management UI, usage analytics, and budget control dashboard.

---

## Scope

**In Scope:**
- Dashboard with global analytics (spending, success rate, agent count)
- Agent management UI (create, list, edit, delete agents)
- Token management UI (create, list, revoke tokens)
- Usage analytics with per-agent filtering and recent logs
- Budget control for agents
- User management (admin only)

**Out of Scope:**
- REST API backend (see iron_control_api)
- WebSocket server (see iron_control_api)
- Token authentication logic (see iron_token_manager)
- Agent execution (see iron_runtime)

---

## Dependencies

**Required Modules:**
- iron_control_api - REST endpoints and WebSocket server

**Required External:**
- Vue 3 - Frontend framework
- TypeScript - Type safety
- Vite - Build tool
- TanStack Query (vue-query) - Data fetching and caching
- Tailwind CSS - Styling
- shadcn/vue - UI components

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **Dashboard View:** Global analytics overview (spending, success rate, agents count)
- **Agents View:** Agent CRUD with budget assignment
- **Usage Analytics View:** Per-agent analytics with logs table, provider/model breakdowns
- **Budgets View:** Agent budget management
- **Providers View:** AI provider configuration (admin)
- **Users View:** User management (admin)

**Shared Components:**
- **MainLayout:** Sidebar navigation with active page indicator
- **Card Components:** Reusable UI cards from shadcn/vue

---

## Integration Points

**Used by:**
- Administrators - Web browser access

**Uses:**
- iron_control_api - REST API for data, WebSocket for real-time updates

---

*For detailed UI specifications, see spec/-archived_detailed_spec.md*
*For deployment, see docs/deployment/001_package_model.md*
