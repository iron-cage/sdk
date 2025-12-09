# iron_api - Specification

**Module:** iron_api
**Layer:** 5 (Integration)
**Status:** Active

---

## Responsibility

REST API and WebSocket server for Iron Cage Control Panel. Provides HTTP endpoints for token management, usage tracking, and limits. Coordinates real-time dashboard updates via WebSocket.

---

## Scope

**In Scope:**
- REST API endpoints (tokens, usage, limits, traces, auth handshake)
- WebSocket server for dashboard real-time updates
- Authentication and authorization (IC Token validation)
- Request routing and validation
- Integration with iron_token_manager, iron_state

**Out of Scope:**
- Dashboard UI components (see iron_dashboard)
- Token generation logic (see iron_token_manager)
- Budget calculation (see iron_cost)
- Agent execution (see iron_runtime)
- Database schema (see iron_control_store)

---

## Dependencies

**Required Modules:**
- iron_token_manager - Token authentication
- iron_state - State persistence
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
- iron_state - Persists and retrieves state data

---

*For detailed API specification, see spec/-archived_detailed_spec.md*
*For REST protocol, see docs/protocol/002_rest_api_protocol.md*
*For WebSocket protocol, see docs/protocol/003_websocket_protocol.md*
