# iron_dashboard - Specification

**Module:** iron_dashboard
**Layer:** 6 (Application)
**Status:** Active

---

## Responsibility

Web control panel for Iron Cage administration. Vue 3 SPA providing real-time agent monitoring, token management UI, usage analytics, and budget control dashboard.

---

## Scope

**In Scope:**
- Real-time agent monitoring (state, progress, costs)
- Token management UI (create, list, revoke tokens)
- Usage analytics with charts
- Budget control dashboard with alerts
- WebSocket integration for live updates

**Out of Scope:**
- REST API backend (see iron_api)
- WebSocket server (see iron_api)
- Token authentication logic (see iron_token_manager)
- Agent execution (see iron_runtime)

---

## Dependencies

**Required Modules:**
- iron_api - REST endpoints and WebSocket server

**Required External:**
- Vue 3 - Frontend framework
- TypeScript - Type safety
- Vite - Build tool
- Chart.js - Data visualization
- Tailwind CSS - Styling

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **Dashboard View:** Main monitoring interface with agent state
- **Token Manager:** CRUD UI for API tokens
- **Analytics Charts:** Cost and usage visualization
- **WebSocket Client:** Real-time event handling

---

## Integration Points

**Used by:**
- Administrators - Web browser access

**Uses:**
- iron_api - REST API for data, WebSocket for real-time updates

---

*For detailed UI specifications, see spec/-archived_detailed_spec.md*
*For deployment, see docs/deployment/001_package_model.md*
