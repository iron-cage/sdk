# Iron Cage API Reference

## Overview

| Location | Caller | Auth |
|----------|--------|------|
| Server | Dashboard/Admin | JWT |
| Server | Client (Python) | IC Token |
| Client | Developer code | - |

---

## Server API

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

### Monitoring (Dashboard + Client)

| Method | Endpoint | Caller | Purpose |
|--------|----------|--------|---------|
| GET | `/api/agents/:id/status` | Dashboard | Get agent state |
| POST | `/api/agents/:id/stop` | Dashboard | Request agent stop |
| GET | `/api/usage` | Dashboard | Budget status |
| GET | `/api/limits` | Dashboard | Rate limits |

---

### System

| Method | Endpoint | Caller | Purpose |
|--------|----------|--------|---------|
| GET | `/health` | Any | Health check |
| GET | `/ws` | Dashboard | WebSocket events |

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
      │                        │         GET /api/usage     │
      │                        │         WS events          │
```

---

## Auth

| Type | Header | Used by | Expiry |
|------|--------|---------|--------|
| JWT Access | `Authorization: Bearer <token>` | Dashboard | 1 hour |
| JWT Refresh | Body parameter | Dashboard | 7 days |
| IC Token | `X-IC-Key: <token>` | Client | Configurable |

---

## WebSocket Events

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
| Server | `POST /api/auth/login` | - | Dashboard |
| Server | `POST /api/auth/refresh` | - | Dashboard |
| Server | `POST /api/auth/logout` | JWT | Dashboard |
| Server | `GET /api/tokens` | JWT | Admin |
| Server | `POST /api/tokens` | JWT | Admin |
| Server | `GET /api/tokens/:id` | JWT | Admin |
| Server | `POST /api/tokens/:id/rotate` | JWT | Admin |
| Server | `DELETE /api/tokens/:id` | JWT | Admin |
| Server | `GET /api/agents/:id/status` | JWT | Dashboard |
| Server | `POST /api/agents/:id/stop` | JWT | Dashboard |
| Server | `GET /api/usage` | JWT | Dashboard |
| Server | `GET /api/limits` | JWT | Dashboard |
| Server | `GET /health` | - | Any |
| Server | `GET /ws` | JWT | Dashboard |
| Client | `Runtime()` | - | Developer |
| Client | `start_agent()` | - | Developer |
| Client | `stop_agent()` | - | Developer |
| Client | `get_metrics()` | - | Developer |