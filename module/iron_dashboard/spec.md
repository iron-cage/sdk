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
- Role-based access control (RBAC) - users see only their agents, admins see all
- Agent ownership - admins can assign/reassign agents to users
- IC token generation for agent API access
- Usage analytics with per-agent filtering and paginated logs
- Budget control for agents (admin only)
- Provider key management (admin only)
- User management (admin only)
- Budget request approval workflow (admin only)

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
- **Agents View:** Agent CRUD with owner assignment, IC token generation
- **Usage Analytics View:** Per-agent analytics with paginated logs, provider/model breakdowns
- **Limits View:** Agent budget status and modification (admin only)
- **Providers View:** AI provider key configuration (admin only)
- **Users View:** User management (admin only)
- **Budget Requests View:** Budget increase request approval (admin only)

**Role-Based Access Control (RBAC):**
- **Admin role:** Full access to all agents, budgets, analytics, and management features
- **User role:** Access limited to owned agents and their data only
- **Data filtering:** Backend filters all queries by owner_id for non-admin users
- **UI restrictions:** Edit/delete/budget buttons hidden for non-admin users

**Agent Ownership:**
- Each agent has an `owner_id` linking to a user
- Admins can assign agents to any user during creation
- Admins can reassign existing agents to different users
- Users can only view/interact with agents they own

**Shared Components:**
- **MainLayout:** Sidebar navigation with active page indicator
- **Card Components:** Reusable UI cards from shadcn/vue

---

## Integration Points

**Used by:**
- Administrators - Full access via web browser
- Regular users - Limited access to owned agents via web browser

**Uses:**
- iron_control_api - REST API for all data operations
- JWT authentication - Bearer tokens for API authorization

---

## Security Model

| Resource | Admin | User |
|----------|-------|------|
| View all agents | Yes | Only owned |
| Create agent | Yes (assign to any user) | Yes (assigned to self) |
| Edit/Delete agent | Yes | No |
| View analytics | All data | Owned agents only |
| Modify budget | Yes | No |
| Manage providers | Yes | No |
| Manage users | Yes | No |
| Approve budget requests | Yes | No |

---

*For detailed changelog, see CHANGELOG_AGENT_OWNERSHIP.md*
*For deployment, see docs/deployment/001_package_model.md*
