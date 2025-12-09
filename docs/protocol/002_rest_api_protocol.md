# Protocol: REST API Protocol

### Scope

HTTP REST API endpoint schemas for Control Panel operations including budget management, token CRUD, and usage analytics.

**In Scope:**
- Budget handshake endpoint (POST /api/v1/auth/handshake)
- Budget reporting endpoint (POST /api/v1/budget/report)
- Budget refresh endpoint (POST /api/v1/budget/refresh)
- Token management endpoints (GET/POST /api/v1/tokens)
- HTTP status codes and error responses
- Request/response JSON schemas
- Authentication via IC Token (agent authentication)
- Authentication via User Token (Control Panel CLI/Dashboard access)

**Out of Scope:**
- WebSocket protocol (see [003_websocket_protocol.md](003_websocket_protocol.md))
- IronLang data protocol (see [001_ironlang_data_protocol.md](001_ironlang_data_protocol.md))
- Implementation details (see `module/iron_api/spec.md`)
- Authentication implementation (see `module/iron_token_manager/`)

---

### Purpose

Provide HTTP REST API for Control Panel operations enabling budget management, token lifecycle, and programmatic access.

**Problem:**

Control Panel needs programmatic access for:
- Runtime budget initialization (handshake with IC Token)
- Real-time cost tracking (report after each LLM call)
- Incremental budget allocation (refresh when low)
- Token management (create, list for developers)
- External tool integration (CI/CD, admin scripts)

**Solution:**

RESTful HTTP API provides:
- Standard HTTP semantics (GET, POST, conventional status codes)
- JSON request/response bodies
- IC Token authentication (agent authentication)
- User Token authentication (Control Panel CLI/Dashboard access)
- Budget protocol endpoints (handshake, report, refresh per protocol/005)
- Clear error responses

**Authentication Types:**
- **IC Token:** Used by agents to authenticate with Control Panel for budget operations. 1:1 with agent.
- **User Token:** Used by users (admin, super user, developer) for CLI and Dashboard access to Control Panel. Multiple tokens per user allowed.

---

### Protocol Definition

### Token Handshake

```
POST /api/v1/auth/handshake
Authorization: Bearer <IC_TOKEN>

Request:
{
  "requested_budget": 10.00
}

Response: 200 OK
{
  "ip_token": "sk-proj-...", (encrypted)
  "budget_granted": 10.00,
  "budget_remaining": 90.00,
  "lease_id": "lease-001"
}
```

### Budget Reporting

```
POST /api/v1/budget/report
Authorization: Bearer <IC_TOKEN>

Request:
{
  "lease_id": "lease-001",
  "tokens": 500,
  "cost_usd": 0.015,
  "timestamp": "2025-12-09T09:00:00Z"
}

Response: 204 No Content
```

**Implementation Variants:**
- **Pilot:** Runtime reports per-request (simpler implementation, 5ms overhead)
- **Production:** Runtime batches reports (every 10 requests, 0.5ms avg overhead, optimized for scale)

**See:** [protocol/005: Budget Control Protocol](005_budget_control_protocol.md#implementation-variants) for complete details.

### Budget Refresh

```
POST /api/v1/budget/refresh
Authorization: Bearer <IC_TOKEN>

Request:
{
  "lease_id": "lease-001",
  "requested_budget": 10.00
}

Response: 200 OK / 403 Forbidden
{
  "budget_granted": 10.00,
  "budget_remaining": 80.00,
  "lease_id": "lease-002"
}
```

### Authentication

**All endpoints require IC Token:**
```
Authorization: Bearer ic_abc123...
```

### Status Codes

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | Success | Handshake, refresh granted |
| 204 | Success (no content) | Reporting accepted |
| 401 | Unauthorized | Invalid IC Token |
| 403 | Forbidden | Budget exhausted |
| 429 | Too Many Requests | Rate limited |
| 500 | Server Error | Internal error |

---

### Cross-References

**Dependencies:**
- [protocol/005: Budget Control Protocol](005_budget_control_protocol.md) - Defines budget messages implemented in REST endpoints

**Used By:**
- [capabilities/002: LLM Access Control](../capabilities/002_llm_access_control.md) - Uses budget API for enforcement
- [architecture/004: Data Flow](../architecture/004_data_flow.md) - REST API in runtime initialization
- Runtime implementations - Call these endpoints for budget management

**Related:**
- [003: WebSocket Protocol](003_websocket_protocol.md) - Complementary real-time protocol
- [001: IronLang Data Protocol](001_ironlang_data_protocol.md) - Different protocol type

**Implementation:**
- Source: `module/iron_api/src/routes/` - Endpoint handlers
- Tests: `module/iron_api/tests/` - Endpoint integration tests
- Specification: `module/iron_api/spec.md` - Complete API specification (FR-7, FR-8, FR-9, FR-10)

---

**Last Updated:** 2025-12-09
**Protocol Version:** v1
**Status:** Specification complete (budget endpoints defined)
