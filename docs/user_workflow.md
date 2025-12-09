# Iron Cage — How It Works

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  CLIENT (Developer Machine)                                     │
│                                                                 │
│  Python Script                                                  │
│       │                                                         │
│       ▼                                                         │
│  iron_runtime (PyO3)                                            │
│       │                                                         │
│       ├──▶ Start/stop agents locally                            │
│       ├──▶ Track budget (iron_cost)                             │
│       ├──▶ Detect PII (iron_safety)                             │
│       └──▶ Report state to server ─────────────────────┐        │
│                                                        │        │
└────────────────────────────────────────────────────────│────────┘
                                                         │
                                                         ▼
┌─────────────────────────────────────────────────────────────────┐
│  SERVER                                                         │
│                                                                 │
│  iron_api (REST)                                                │
│       │                                                         │
│       ├──▶ Authentication (JWT)                                 │
│       ├──▶ Token management                                     │
│       ├──▶ Store agent state (iron_state)                       │
│       ├──▶ Usage tracking                                       │
│       └──▶ WebSocket events (PLANNED)                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key point:** Agents run on CLIENT. Server only tracks state and provides monitoring.

---

## Client Side (Python)

Developer uses `iron_runtime` to run agents with protection:

```python
from iron_runtime import Runtime

runtime = Runtime(budget=50.0)
agent_id = runtime.start_agent("/path/to/agent.py")
metrics = runtime.get_metrics(agent_id)
runtime.stop_agent(agent_id)
```

**What happens locally:**
- Agent script executes on client machine
- Budget tracked per agent
- PII detected and redacted
- State synced to server

---

## Server Side (REST API)

Server provides monitoring and configuration:

### Authentication
| Endpoint | Purpose |
|----------|---------|
| `POST /api/auth/login` | Get JWT tokens |
| `POST /api/auth/refresh` | Refresh access token |
| `POST /api/auth/logout` | Invalidate session |

### Token Management
| Endpoint | Purpose |
|----------|---------|
| `POST /api/tokens` | Create token |
| `GET /api/tokens` | List tokens |
| `DELETE /api/tokens/:id` | Revoke token |

### Usage Analytics
| Endpoint | Purpose |
|----------|---------|
| `GET /api/usage/aggregate` | Aggregate usage statistics |
| `GET /api/usage/by-project/:id` | Usage by project |
| `GET /api/usage/by-provider/:name` | Usage by provider |

### Limits Management
| Endpoint | Purpose |
|----------|---------|
| `GET /api/limits` | List all limits |
| `POST /api/limits` | Create limit |
| `GET /api/limits/:id` | Get specific limit |
| `PUT /api/limits/:id` | Update limit |
| `DELETE /api/limits/:id` | Delete limit |

### Traces
| Endpoint | Purpose |
|----------|---------|
| `GET /api/traces` | List all traces |
| `GET /api/traces/:id` | Get specific trace |

### WebSocket (PLANNED)

> **Note:** WebSocket endpoint exists but events are not yet implemented.

```
ws://localhost:3000/ws
```
Events (PLANNED): `AgentStarted`, `CostUpdate`, `PiiAlert`, `BudgetWarning`

---

## Data Flow

```
1. Client starts agent
   iron_runtime.start_agent()
       │
       ├──▶ Execute script locally
       ├──▶ Generate agent_id (UUID)
       └──▶ POST state to server

2. During execution
   Agent makes LLM calls
       │
       ├──▶ iron_cost.record_cost()
       ├──▶ iron_safety.check() for PII
       └──▶ Sync updates to server

3. Server receives updates
       │
       ├──▶ Store in iron_state (DashMap + SQLite)
       ├──▶ Broadcast via WebSocket (PLANNED)
       └──▶ Available via REST API

4. Dashboard/Admin queries server
   GET /api/usage/aggregate
   GET /api/traces
```

---

## Components

| Component | Location | Purpose |
|-----------|----------|---------|
| `iron_runtime` | Client | Agent lifecycle, PyO3 bridge |
| `iron_cost` | Client | Budget tracking |
| `iron_safety` | Client | PII detection |
| `iron_reliability` | Client | Circuit breaker |
| `iron_api` | Server | REST API |
| `iron_state` | Server | State storage |
| `iron_token_manager` | Server | Auth tokens |
| `iron_telemetry` | Both | Logging |

---
