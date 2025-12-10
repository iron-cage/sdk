# iron_control_api - Specification

**Module:** iron_control_api
**Layer:** 5 (Integration)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

REST API and WebSocket server for Iron Cage Control Panel. Provides HTTP endpoints for token management, usage tracking, and limits. Coordinates real-time dashboard updates via WebSocket.

---

## Scope

**In Scope:**
- REST API endpoints (tokens, usage, limits, traces, auth handshake, user management)
- WebSocket server for dashboard real-time updates
- Authentication and authorization (IC Token validation)
- Request routing and validation
- RBAC enforcement (role-based access control)
- Integration with iron_token_manager, iron_runtime_state

**Out of Scope:**
- Dashboard UI components (see iron_dashboard)
- Token generation logic (see iron_token_manager)
- Budget calculation (see iron_cost)
- Agent execution (see iron_runtime)
- Database schema (see iron_control_schema)

---

## Dependencies

**Required Modules:**
- iron_token_manager - Token authentication
- iron_runtime_state - State persistence
- iron_telemetry - Logging
- iron_cost - Cost types

**Required External:**
- axum - HTTP server framework
- tokio - Async runtime
- tower-http - HTTP middleware

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **REST Router:** Handles HTTP endpoints for tokens, usage, limits
- **WebSocket Server:** Broadcasts real-time agent events to dashboard
- **Auth Middleware:** Validates IC Tokens, enforces authorization
- **Request Handler:** Routes requests to appropriate modules

---

## Integration Points

**Used by:**
- iron_dashboard - Vue app consumes REST API and WebSocket
- iron_runtime - Sends telemetry and state updates
- Developers - CLI and SDK interact with API

**Uses:**
- iron_token_manager - Token authentication and management
- iron_runtime_state - Persists and retrieves state data

---

*For detailed API specification, see spec/-archived_detailed_spec.md*
*For REST protocol, see docs/protocol/002_rest_api_protocol.md*
*For WebSocket protocol, see docs/protocol/003_websocket_protocol.md*
*For user management API, see docs/protocol/008_user_management_api.md*
