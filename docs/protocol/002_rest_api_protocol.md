# REST API Protocol

**Purpose:** HTTP endpoint schemas and contracts for Control Panel API.

---

### User Need

Understand REST API message formats for tokens, usage, limits, and traces.

### Core Idea

**RESTful API with standard HTTP semantics:**

```
Developer/Runtime → Control Panel API
                    - POST /api/v1/auth/handshake (get IP token + budget)
                    - POST /api/v1/budget/report (report usage)
                    - POST /api/v1/budget/refresh (request more budget)
                    - GET /api/v1/tokens (list tokens)
                    - POST /api/v1/tokens (create token)
```

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

*Related: [003_websocket_protocol.md](003_websocket_protocol.md) | [../architecture/006_budget_control_protocol.md](../architecture/006_budget_control_protocol.md)*
