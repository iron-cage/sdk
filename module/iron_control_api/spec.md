# iron_control_api - Specification

**Module:** iron_control_api
**Layer:** 5 (Integration)
**Status:** Active

---

### Responsibility

REST API and WebSocket server for Iron Cage Control Panel. Provides HTTP endpoints for token management, usage tracking, budget control, and analytics. Coordinates real-time dashboard updates via WebSocket.

---

### Scope

**In Scope:**
- REST API endpoints (tokens, usage, limits, traces, auth handshake, user management)
- Budget control endpoints (Protocol 005: handshake, usage reporting, budget refresh)
- Analytics endpoints (Protocol 012: event ingestion, spending/usage queries)
- Agent token enforcement (blocking unauthorized credential access)
- WebSocket server for dashboard real-time updates
- Authentication and authorization (IC Token validation, JWT)
- RBAC enforcement (role-based access control)

**Out of Scope:**
- Dashboard UI components (see iron_dashboard)
- Token generation logic (see iron_token_manager)
- Budget calculation (see iron_cost)

---

### Dependencies

**Required:** iron_token_manager, iron_runtime_state, iron_telemetry, iron_cost
**External:** axum, tokio, tower-http

---

### Core Concepts

- **REST Router:** Handles HTTP endpoints for tokens, usage, limits
- **Budget Control Router (Protocol 005):** Manages budget handshake, usage reporting, budget refresh
- **Analytics Router (Protocol 012):** Event ingestion and spending/usage queries
- **Agent Token Enforcement:** Blocks agent tokens from unauthorized credential endpoints
- **WebSocket Server:** Broadcasts real-time agent events to dashboard
- **Auth Middleware:** Validates IC Tokens and JWT, enforces authorization

---

### API Contract Summary

**Budget Control (Protocol 005):**
- `POST /api/budget/handshake` - Exchange IC Token for IP Token (encrypted provider API key)
- `POST /api/budget/report` - Report LLM usage for a budget lease
- `POST /api/budget/return` - Return unused budget when a lease is closed
- `POST /api/budget/refresh` - Request additional budget allocation

**Budget Request Workflow (Protocol 012):**
- `POST /api/v1/budget/requests` - Create budget change request
- `GET /api/v1/budget/requests/:id` - Get request by ID
- `GET /api/v1/budget/requests` - List requests with filtering
- `PATCH /api/v1/budget/requests/:id/approve` - Approve request
- `PATCH /api/v1/budget/requests/:id/reject` - Reject request

**Analytics (Protocol 012):**
- `POST /api/v1/analytics/events` - Report LLM request events
- `GET /api/v1/analytics/spending/*` - Spending metrics endpoints
- `GET /api/v1/analytics/usage/*` - Usage metrics endpoints

**Agent Provider Keys (Feature 014):**
- `POST /api/v1/agents/provider-key` - Retrieve assigned provider API key for agent (IC Token auth)

---

### Integration Points

**Used by:** iron_dashboard, iron_runtime, Developers (CLI/SDK)
**Uses:** iron_token_manager, iron_runtime_state

---

*For detailed API specification, see spec/-archived_detailed_spec.md*
*For REST protocol, see docs/protocol/002_rest_api_protocol.md*
*For Budget Control Protocol, see docs/protocol/005_budget_control_protocol.md*
*For Analytics API, see docs/protocol/012_analytics_api.md*
