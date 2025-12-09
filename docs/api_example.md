# Iron Cage API Reference

## Overview

| Location | Caller | Auth |
|----------|--------|------|
| Server | Dashboard/Admin | JWT |
| Server | Client (Python) | IC Token |
| Client | Developer code | - |

---

## Server API

### Health Check

| Method | Endpoint | Caller | Purpose |
|--------|----------|--------|---------|
| GET | `/api/health` | Any | Server health status |

---

### Authentication (Dashboard)

| Method | Endpoint | Caller | Purpose |
|--------|----------|--------|---------|
| POST | `/api/auth/login` | Dashboard | Get JWT tokens |
| POST | `/api/auth/refresh` | Dashboard | Refresh access token |
| POST | `/api/auth/logout` | Dashboard | Invalidate session |

---

### Token Management (Dashboard)

| Method | Endpoint | Caller | Purpose |
|--------|----------|--------|---------|
| GET | `/api/tokens` | Admin | List all tokens |
| POST | `/api/tokens` | Admin | Create new token |
| GET | `/api/tokens/:id` | Admin | Get token details |
| POST | `/api/tokens/:id/rotate` | Admin | Rotate token |
| DELETE | `/api/tokens/:id` | Admin | Revoke token |

---

### Usage Analytics (Dashboard)

| Method | Endpoint | Caller | Purpose |
|--------|----------|--------|---------|
| GET | `/api/usage/aggregate` | Dashboard | Aggregate usage statistics |
| GET | `/api/usage/by-project/:project_id` | Dashboard | Usage by project |
| GET | `/api/usage/by-provider/:provider` | Dashboard | Usage by provider |

---

### Limits Management (Dashboard)

| Method | Endpoint | Caller | Purpose |
|--------|----------|--------|---------|
| GET | `/api/limits` | Dashboard | List all limits |
| POST | `/api/limits` | Admin | Create new limit |
| GET | `/api/limits/:id` | Dashboard | Get specific limit |
| PUT | `/api/limits/:id` | Admin | Update limit |
| DELETE | `/api/limits/:id` | Admin | Delete limit |

---

### Traces (Dashboard)

| Method | Endpoint | Caller | Purpose |
|--------|----------|--------|---------|
| GET | `/api/traces` | Dashboard | List all traces |
| GET | `/api/traces/:id` | Dashboard | Get specific trace |

---

### Provider Key Management (Dashboard)

| Method | Endpoint | Caller | Purpose |
|--------|----------|--------|---------|
| GET | `/api/providers` | Admin | List provider keys |
| POST | `/api/providers` | Admin | Create provider key |
| GET | `/api/providers/:id` | Admin | Get provider key |
| PUT | `/api/providers/:id` | Admin | Update provider key |
| DELETE | `/api/providers/:id` | Admin | Delete provider key |
| POST | `/api/projects/:project_id/provider` | Admin | Assign provider to project |
| DELETE | `/api/projects/:project_id/provider` | Admin | Unassign provider from project |

---

### Key Fetch (Client)

| Method | Endpoint | Caller | Purpose |
|--------|----------|--------|---------|
| GET | `/api/keys` | Client | Fetch API key (rate limited: 10/min) |

---

### WebSocket (PLANNED)

> **Note:** WebSocket endpoint exists but events are not yet implemented.

| Method | Endpoint | Caller | Purpose |
|--------|----------|--------|---------|
| GET | `/ws` | Dashboard | Real-time events (PLANNED) |

---

## Client API (Python)

| Method | Caller | Purpose |
|--------|--------|---------|
| `Runtime(budget, verbose)` | Developer | Create runtime |
| `start_agent(path)` | Developer | Start agent locally |
| `stop_agent(agent_id)` | Developer | Stop agent |
| `get_metrics(agent_id)` | Developer | Get agent metrics |

---

## Data Flow

```
Developer Code          Client (iron_runtime)         Server (iron_api)
      │                        │                            │
      │  Runtime(budget=50)    │                            │
      │───────────────────────▶│                            │
      │                        │                            │
      │  start_agent(path)     │                            │
      │───────────────────────▶│                            │
      │                        │  POST agent state          │
      │                        │───────────────────────────▶│
      │                        │                            │
      │     [agent runs]       │                            │
      │                        │  POST usage updates        │
      │                        │───────────────────────────▶│
      │                        │                            │
      │                        │         Dashboard ◀────────│
      │                        │    GET /api/usage/aggregate│
      │                        │         WS events (PLANNED)│
```

---

## Auth

| Type | Header | Used by | Expiry |
|------|--------|---------|--------|
| JWT Access | `Authorization: Bearer <token>` | Dashboard | 1 hour |
| JWT Refresh | Body parameter | Dashboard | 7 days |
| IC Token | `X-IC-Key: <token>` | Client | Configurable |

---

## WebSocket Events (PLANNED)

> **Note:** These events are planned but not yet implemented.

| Event | Source | Purpose |
|-------|--------|---------|
| `AgentStarted` | Client → Server | Agent began execution |
| `CostUpdate` | Client → Server | Cost recorded |
| `PiiAlert` | Client → Server | PII detected |
| `BudgetWarning` | Server | Near budget limit |

---

## Errors

| Code | HTTP | Meaning |
|------|------|---------|
| `UNAUTHORIZED` | 401 | Invalid token |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `BUDGET_EXCEEDED` | 402 | Budget limit reached |
| `RATE_LIMITED` | 429 | Too many requests |
| `CIRCUIT_OPEN` | 503 | Service unavailable |

---

## Summary

| Location | Endpoint | Auth | Caller |
|----------|----------|------|--------|
| Server | `GET /api/health` | - | Any |
| Server | `POST /api/auth/login` | - | Dashboard |
| Server | `POST /api/auth/refresh` | - | Dashboard |
| Server | `POST /api/auth/logout` | JWT | Dashboard |
| Server | `GET /api/tokens` | JWT | Admin |
| Server | `POST /api/tokens` | JWT | Admin |
| Server | `GET /api/tokens/:id` | JWT | Admin |
| Server | `POST /api/tokens/:id/rotate` | JWT | Admin |
| Server | `DELETE /api/tokens/:id` | JWT | Admin |
| Server | `GET /api/usage/aggregate` | JWT | Dashboard |
| Server | `GET /api/usage/by-project/:id` | JWT | Dashboard |
| Server | `GET /api/usage/by-provider/:name` | JWT | Dashboard |
| Server | `GET /api/limits` | JWT | Dashboard |
| Server | `POST /api/limits` | JWT | Admin |
| Server | `GET /api/limits/:id` | JWT | Dashboard |
| Server | `PUT /api/limits/:id` | JWT | Admin |
| Server | `DELETE /api/limits/:id` | JWT | Admin |
| Server | `GET /api/traces` | JWT | Dashboard |
| Server | `GET /api/traces/:id` | JWT | Dashboard |
| Server | `GET /api/providers` | JWT | Admin |
| Server | `POST /api/providers` | JWT | Admin |
| Server | `GET /api/providers/:id` | JWT | Admin |
| Server | `PUT /api/providers/:id` | JWT | Admin |
| Server | `DELETE /api/providers/:id` | JWT | Admin |
| Server | `POST /api/projects/:id/provider` | JWT | Admin |
| Server | `DELETE /api/projects/:id/provider` | JWT | Admin |
| Server | `GET /api/keys` | IC Token | Client |
| Server | `GET /ws` | JWT | Dashboard (PLANNED) |
| Client | `Runtime()` | - | Developer |
| Client | `start_agent()` | - | Developer |
| Client | `stop_agent()` | - | Developer |
| Client | `get_metrics()` | - | Developer |
