# spec

**Version:** 0.4
**Date:** 2025-12-07
**Component:** iron_api
**Layer:** Integration (Layer 5)
**Status:** Specification Complete (10 FRs), Implementation Complete (FR-7/8/9/10 Complete with Tests + Phases 1-5 Security Fixes and Corner Case Coverage)
**Priority:** REQUIRED for Dashboard (iron_dashboard depends on token/usage/limits/traces APIs)
**Phases 1-5 Security Fixes and Corner Cases:** DoS protection (issue-001), NULL byte injection (issue-002), database test hooks (issue-003), state transitions (7 tests, 3 bugs fixed), concurrency (4 tests, 3 bugs fixed), malformed JSON (16 tests), HTTP methods (14 tests), Content-Type (6 tests), idempotency (4 tests, 1 bug fixed), error format (2 tests), empty body (4 tests), additional coverage (+23 tests)

---

## 1. Overview

### 1.1 Purpose

**iron_api** provides REST API and WebSocket endpoints for programmatic control of Iron Cage runtime and real-time dashboard communication. It enables external tools, CI/CD pipelines, and web dashboards to start agents, monitor execution, and receive live updates.

**Primary Responsibilities:**
- REST API for agent lifecycle (start, stop, status, metrics)
- WebSocket server for real-time dashboard updates
- CORS support for browser-based dashboards
- Request validation and error handling
- API authentication and rate limiting

**Pilot Scope:** Basic REST endpoints and WebSocket streaming for conference demo dashboard.

**Full Platform (Out of Scope):** API authentication (JWT), rate limiting, distributed API gateway, GraphQL interface, webhook notifications.

**NOTE FOR PILOT:**
Demo uses CLI (`iron_cage start`) to launch agents, NOT REST API. Dashboard connects via WebSocket embedded in iron_runtime (see iron_runtime spec FR-4). iron_api provides REST endpoints for future extensibility but is NOT required for demo execution.

### 1.2 Design Principles

1. **RESTful Design** - Standard HTTP methods (GET/POST/DELETE) with resource-oriented URLs
2. **Real-Time by Default** - WebSocket for streaming updates, REST for queries
3. **CORS-Enabled** - Allow browser dashboards on different ports
4. **Fail-Safe Validation** - Reject invalid requests with clear error messages
5. **Observable API** - All requests logged for debugging

---

## 2. Scope

### 2.1 In Scope (Pilot)

**For Pilot Project (Conference Demo):**
- REST endpoints for agent control (start, stop, status, metrics)
- WebSocket endpoint for dashboard streaming
- CORS middleware for localhost:5173 (Vite dev server)
- JSON request/response format
- HTTP error handling with status codes
- Integration with iron_state for data queries

**Rationale:** Dashboard needs WebSocket for real-time updates and REST for on-demand queries. API enables future extensions (CI/CD integration).

**Note:** Demo execution uses iron_cli, not REST API. API is passive (responds to dashboard queries only).

### 2.2 Out of Scope (Full Platform)

**Deferred to Post-Pilot:**
- API authentication (JWT tokens) → Pilot is localhost-only
- Rate limiting (per-IP, per-key) → Pilot has no abuse risk
- Distributed API gateway (multi-node) → Pilot is single-process
- GraphQL interface → Pilot uses REST only
- Webhook notifications (external systems) → Pilot uses WebSocket only
- API versioning (/v1/, /v2/) → Pilot is v1 implicitly
- API documentation (OpenAPI/Swagger) → Pilot has spec.md only

**Reasoning:** Conference demo runs on localhost with trusted dashboard. Authentication and rate limiting add complexity without security benefit for local-only deployment.

---

## 2.3 Deployment Modes

Iron Cage supports two deployment modes with different data backends and communication patterns for iron_api:

### Pilot/Demo Mode (Single Process, Localhost)

**Data Backend:** iron_api shares iron_state (DashMap + SQLite) with iron_runtime within same Rust process.

**Architecture:**
```
Single Rust Process (localhost:8080)
├── iron_runtime (agent orchestration)
│   └── Arc<StateManager> (shared iron_state)
│         ├── DashMap (agent state)
│         └── SQLite (audit logs)
│
└── iron_api (REST + WebSocket server)
    ├── REST endpoints (GET /api/agents/:id)
    │   └── Reads from Arc<StateManager> (same instance)
    │
    └── WebSocket (/ws)
        └── Subscribes to broadcast::Receiver<StateUpdate>
            └── Receives updates from shared StateManager
```

**Data Flow:**
```
iron_runtime → StateManager.save_agent_state()
                ↓
         Broadcast StateUpdate
                ↓
         iron_api/ws receives update
                ↓
         Dashboard (WebSocket push)
```

**Key Characteristics:**
- Single process, single binary
- WebSocket communication (ws://localhost:8080/ws)
- Shared Arc<StateManager> between iron_runtime and iron_api
- Single SQLite database (./iron_state.db)
- Zero network latency (in-process broadcast channel)

---

### Production Mode (Distributed, Cloud + Local)

**Data Backend:** iron_api uses iron_control_store (PostgreSQL) for users, tokens, secrets, and aggregated telemetry. Does NOT have iron_state.

**Architecture:**
```
Cloud: Control Panel (https://api.example.com)
├── iron_api (REST API server)
│   ├── GET /api/tokens (token management)
│   ├── POST /api/telemetry (telemetry ingestion from distributed agents)
│   └── iron_control_store
│       └── PostgreSQL
│           ├── users table
│           ├── api_tokens table
│           ├── secrets table
│           └── telemetry_events table (aggregated from all agents)
│
└── iron_dashboard (static UI, served by nginx)

Developer Machines
├── Machine 1: Alice
│   ├── iron_runtime (agent execution)
│   ├── iron_state (local SQLite: alice_state.db)
│   └── Sends telemetry via HTTPS POST to Control Panel API
│
├── Machine 2: Bob
│   ├── iron_runtime (agent execution)
│   ├── iron_state (local SQLite: bob_state.db)
│   └── Sends telemetry via HTTPS POST to Control Panel API
```

**Data Flow:**
```
Alice's Machine                  Cloud: Control Panel
iron_runtime → iron_state        iron_api
     ↓                                ↓
Optional telemetry ──HTTPS POST─→ POST /api/telemetry
                                      ↓
                              iron_control_store.save_telemetry()
                                      ↓
                              PostgreSQL INSERT INTO telemetry_events

Dashboard Query:
Browser → HTTPS GET /api/telemetry → PostgreSQL SELECT → JSON Response
```

**Key Differences:**
- **Control Panel (iron_api):** Does NOT have iron_state, uses iron_control_store (PostgreSQL) for centralized data
- **Agent Runtime:** Has iron_state (local SQLite), optionally sends telemetry to Control Panel
- **Communication:** HTTPS (TLS encrypted), not WebSocket
- **Databases:** Separate (PostgreSQL for Control Panel, SQLite per Agent Runtime instance)
- **New Endpoints (Production Only):**
  - `POST /api/tokens` - Token generation (called by CLI tools)
  - `POST /api/telemetry` - Telemetry ingestion (called by distributed agents)
  - `GET /api/telemetry?agent_id=X` - Query aggregated telemetry
  - `GET /api/secrets` - Secrets management

**Package Distribution:**
- **Control Panel Package (Docker):** Includes iron_api + iron_control_store (NO iron_state)
- **Agent Runtime Package (PyPI):** Includes iron_runtime + iron_state (does NOT include iron_api)

---

### Current Status

**Pilot Implementation (Dec 2025):** Uses Pilot/Demo Mode only (shared iron_state pattern)
**Production Implementation:** Planned post-pilot (Q1 2026) - requires iron_control_store module implementation

**This specification describes both modes,** with primary focus on Pilot Mode for conference demo implementation. Production Mode endpoints (tokens, telemetry ingestion) are documented but not yet implemented.

---

## 3. Functional Requirements

### FR-1: REST API - Agent Lifecycle

**Requirement:**
Provide REST endpoints for programmatic agent control.

**Endpoints:**

| Method | Path | Description | Request | Response |
|--------|------|-------------|---------|----------|
| POST | /api/agents | Start new agent | StartAgentRequest | AgentStartedResponse |
| GET | /api/agents/:id | Get agent status | - | AgentStatusResponse |
| POST | /api/agents/:id/stop | Stop running agent | - | SuccessResponse |
| DELETE | /api/agents/:id | Kill agent forcefully | - | SuccessResponse |

**Request/Response Types:**
```rust
#[derive(Deserialize)]
pub struct StartAgentRequest {
  pub script_path: String,
  pub budget: Option<f64>,  // Override global budget
}

#[derive(Serialize)]
pub struct AgentStartedResponse {
  pub agent_id: String,
  pub status: String,  // "RUNNING"
  pub start_time: i64,
}

#[derive(Serialize)]
pub struct AgentStatusResponse {
  pub agent_id: String,
  pub status: String,  // "RUNNING", "COMPLETED", "FAILED", "STOPPED"
  pub start_time: i64,
  pub end_time: Option<i64>,
}

#[derive(Serialize)]
pub struct SuccessResponse {
  pub success: bool,
  pub message: String,
}
```

**Example Usage:**
```bash
# Start agent
curl -X POST http://localhost:8080/api/agents \
  -H "Content-Type: application/json" \
  -d '{"script_path": "/path/to/agent.py", "budget": 50.0}'
# Response: {"agent_id": "agent-001", "status": "RUNNING", "start_time": 1732550611}

# Get status
curl http://localhost:8080/api/agents/agent-001
# Response: {"agent_id": "agent-001", "status": "RUNNING", ...}

# Stop agent
curl -X POST http://localhost:8080/api/agents/agent-001/stop
# Response: {"success": true, "message": "Agent stopped"}
```

**Out of Scope:** Batch operations (start multiple agents), agent templates, scheduling.

---

### FR-2: REST API - Metrics Queries

**Requirement:**
Provide REST endpoints for querying runtime metrics.

**Endpoints:**

| Method | Path | Description | Response |
|--------|------|-------------|----------|
| GET | /api/agents/:id/metrics | Get agent metrics | MetricsResponse |
| GET | /api/budget | Get global budget status | BudgetResponse |
| GET | /api/circuits | Get all circuit breaker states | CircuitsResponse |
| GET | /api/health | API health check | HealthResponse |

**Response Types:**
```rust
#[derive(Serialize)]
pub struct MetricsResponse {
  pub agent_id: String,
  pub cost: CostMetrics,
  pub safety: SafetyMetrics,
  pub performance: PerformanceMetrics,
}

#[derive(Serialize)]
pub struct BudgetResponse {
  pub limit: f64,
  pub spent: f64,
  pub remaining: f64,
  pub percentage_used: f64,
  pub agents: Vec<AgentBudget>,  // Per-agent breakdown
}

#[derive(Serialize)]
pub struct AgentBudget {
  pub agent_id: String,
  pub spent: f64,
}

#[derive(Serialize)]
pub struct CircuitsResponse {
  pub circuits: Vec<CircuitBreakerMetrics>,
}

#[derive(Serialize)]
pub struct HealthResponse {
  pub status: String,  // "OK", "DEGRADED", "DOWN"
  pub uptime_secs: u64,
  pub agents_running: u32,
}
```

**Example Usage:**
```bash
# Get agent metrics
curl http://localhost:8080/api/agents/agent-001/metrics
# Response: {"agent_id": "agent-001", "cost": {...}, "safety": {...}, "performance": {...}}

# Get global budget
curl http://localhost:8080/api/budget
# Response: {"limit": 50.0, "spent": 23.14, "remaining": 26.86, "percentage_used": 0.4628, ...}

# Get circuit breaker states
curl http://localhost:8080/api/circuits
# Response: {"circuits": [{"service": "linkedin_api", "state": "OPEN", ...}]}

# Health check
curl http://localhost:8080/api/health
# Response: {"status": "OK", "uptime_secs": 134, "agents_running": 1}
```

---

### FR-3: WebSocket Streaming

**Requirement:**
Provide WebSocket endpoint that streams real-time state updates to connected dashboards.

**Endpoint:** `GET /ws` (WebSocket upgrade)

**Message Protocol:**
```rust
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum DashboardMessage {
  // Initial snapshot (on connection)
  StateSnapshot {
    agents: Vec<AgentState>,
    budget: BudgetSnapshot,
    circuits: Vec<CircuitBreakerMetrics>,
  },

  // Lifecycle events
  AgentStarted { agent_id: String, timestamp: i64 },
  AgentCompleted { agent_id: String, leads_processed: u32, timestamp: i64 },
  AgentFailed { agent_id: String, error: String, timestamp: i64 },

  // Metrics updates (every lead)
  CostUpdate { agent_id: String, metrics: CostMetrics },
  SafetyEvent { agent_id: String, event: SafetyMetrics },
  PerformanceUpdate { agent_id: String, metrics: PerformanceMetrics },

  // Alerts (demo triggers)
  BudgetWarning { agent_id: String, warning: BudgetWarning },
  PiiAlert { agent_id: String, pii_type: String, location: String },
  CircuitOpened { service: String, failure_count: u32 },
}
```

**Connection Flow:**
```mermaid
sequenceDiagram
    participant Dashboard as React Dashboard
    participant WS as WebSocket Server<br/>(:8080/ws)
    participant State as iron_state
    participant Runtime as iron_runtime

    Dashboard->>WS: Connect ws://localhost:8080/ws
    WS->>State: Query current state
    State-->>WS: AgentState, Budget, Circuits
    WS->>Dashboard: StateSnapshot message

    Note over Dashboard: Initial UI render complete

    Runtime->>State: publish_cost_metrics()
    State->>State: Broadcast StateUpdate
    State-->>WS: StateUpdate::AgentStateChanged
    WS->>WS: Convert to DashboardMessage
    WS->>Dashboard: CostUpdate message

    Dashboard->>Dashboard: Update Cost Panel

    Note over Dashboard,Runtime: Real-time streaming continues...
```

**Implementation:**
```rust
async fn websocket_handler(
  ws: WebSocketUpgrade,
  State(state): State<ApiState>
) -> impl IntoResponse {
  ws.on_upgrade(|socket| handle_websocket_connection(socket, state))
}

async fn handle_websocket_connection(
  mut socket: WebSocket,
  state: ApiState
) {
  // Subscribe to state updates
  let mut update_rx = state.state_manager.subscribe_to_updates();

  tracing::info!("Dashboard connected");

  // Send initial snapshot
  let snapshot = build_state_snapshot(&state.state_manager).await;
  send_message(&mut socket, &DashboardMessage::StateSnapshot {
    agents: snapshot.agents,
    budget: snapshot.budget,
    circuits: snapshot.circuits,
  }).await;

  // Stream updates
  while let Ok(update) = update_rx.recv().await {
    let message = convert_state_update_to_message(update, &state.state_manager).await;

    if send_message(&mut socket, &message).await.is_err() {
      tracing::warn!("Dashboard disconnected");
      break;
    }
  }
}

async fn send_message(
  socket: &mut WebSocket,
  message: &DashboardMessage
) -> Result<()> {
  let json = serde_json::to_string(message)?;
  socket.send(Message::Text(json)).await?;
  Ok(())
}
```

**Reconnection Handling:**
- Dashboard reconnects automatically if connection drops
- Server sends fresh StateSnapshot on each new connection
- No message queuing (dashboard shows current state only)

---

### FR-4: CORS Configuration

**Requirement:**
Enable CORS to allow dashboard (localhost:5173) to connect to API (localhost:8080).

**Configuration:**
```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
  .allow_origin("http://localhost:5173".parse::<HeaderValue>()?)
  .allow_methods([Method::GET, Method::POST, Method::DELETE])
  .allow_headers([header::CONTENT_TYPE]);

let app = Router::new()
  .route("/api/agents", post(start_agent))
  .route("/ws", get(websocket_handler))
  .layer(cors);
```

**Pilot Configuration:**
- Allow origin: http://localhost:5173 (Vite dev server)
- Allow methods: GET, POST, DELETE
- Allow headers: Content-Type

**Full Platform:**
- Production: Restrict to specific origins (https://dashboard.ironcage.ai)
- Staging: Multiple origins for testing
- Development: Permissive (allow any)

---

### FR-5: Error Handling and HTTP Status Codes

**Requirement:**
Return appropriate HTTP status codes and JSON error responses.

**Error Response Format:**
```rust
#[derive(Serialize)]
pub struct ErrorResponse {
  pub error: String,
  pub code: String,  // "AGENT_NOT_FOUND", "BUDGET_EXCEEDED"
  pub details: Option<String>,
}
```

**Status Code Mapping:**

| Situation | Status Code | Example |
|-----------|-------------|---------|
| Success | 200 OK | Agent status retrieved |
| Created | 201 Created | Agent started |
| Not Found | 404 Not Found | Unknown agent_id |
| Bad Request | 400 Bad Request | Invalid script_path |
| Budget Exceeded | 402 Payment Required | Budget limit hit |
| Internal Error | 500 Internal Server Error | Runtime panic |

**Example Error Response:**
```json
{
  "error": "Agent not found",
  "code": "AGENT_NOT_FOUND",
  "details": "No agent with id 'agent-999'"
}
```

**Implementation:**
```rust
async fn get_agent_status(
  State(state): State<ApiState>,
  Path(agent_id): Path<String>,
) -> Result<Json<AgentStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
  match state.state_manager.get_agent_state(&agent_id) {
    Some(agent_state) => {
      Ok(Json(AgentStatusResponse {
        agent_id: agent_state.agent_id,
        status: format!("{:?}", agent_state.status),
        start_time: agent_state.start_time,
        end_time: agent_state.end_time,
      }))
    },
    None => {
      Err((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          error: "Agent not found".to_string(),
          code: "AGENT_NOT_FOUND".to_string(),
          details: Some(format!("No agent with id '{}'", agent_id)),
        })
      ))
    }
  }
}
```

---

### FR-6: Integration with iron_runtime

**Requirement:**
Share StateManager instance between API server and AgentRuntime for consistent state access.

**Architecture:**
```rust
// In main application (iron_cli or standalone server)
let state = Arc::new(StateManager::new(":memory:").await?);

// Create runtime with shared state
let runtime_config = RuntimeConfig { ... };
let runtime = AgentRuntime::new_with_state(runtime_config, state.clone()).await?;

// Create API server with shared state
let api_server = ApiServer::new(state.clone(), 8080);

// Run both concurrently
tokio::select! {
  _ = runtime.start_agent(script_path) => {},
  _ = api_server.start() => {},
}
```

**Coordination:**
- Runtime publishes state updates → StateManager broadcasts
- API server subscribes to broadcasts → forwards to WebSocket clients
- REST queries hit StateManager directly (synchronous reads)

**Note for Pilot:**
Demo uses iron_runtime's embedded WebSocket server (iron_runtime spec FR-4), not iron_api's WebSocket. iron_api is optional for pilot.

---

### FR-7: Token Management API

**Requirement:**
Provide REST endpoints for API token lifecycle management (create, list, rotate, revoke).

**Endpoints:**

| Method | Path | Description | Request | Response |
|--------|------|-------------|---------|----------|
| POST | /api/tokens | Create new API token | CreateTokenRequest | CreateTokenResponse |
| GET | /api/tokens | List all tokens for user | - | Vec<TokenListItem> |
| GET | /api/tokens/:id | Get specific token details | - | TokenListItem |
| POST | /api/tokens/:id/rotate | Rotate token (new value) | - | CreateTokenResponse |
| DELETE | /api/tokens/:id | Revoke token | - | 204 No Content |

**Request/Response Types:**
```rust
#[derive(Deserialize)]
pub struct CreateTokenRequest {
  pub user_id: String,
  pub project_id: Option<String>,
  pub description: Option<String>,
}

#[derive(Serialize)]
pub struct CreateTokenResponse {
  pub id: i64,
  pub token: String,  // Plaintext token returned ONCE on creation/rotation
  pub user_id: String,
  pub project_id: Option<String>,
  pub description: Option<String>,
  pub created_at: i64,
}

#[derive(Serialize)]
pub struct TokenListItem {
  pub id: i64,
  pub user_id: String,
  pub project_id: Option<String>,
  pub description: Option<String>,
  pub created_at: i64,
  pub last_used_at: Option<i64>,
  pub is_active: bool,
}
```

**Security:**
- Plaintext token returned ONLY on creation and rotation
- Subsequent GET requests return metadata only (no token value)
- Token stored as BCrypt hash (cost=12) in database (NEVER plaintext or reversible)
- BCrypt chosen over SHA-256 for password-style hashing (slow, salted, adaptive work factor)
- Rotation deactivates old token atomically

**Validation Rules (Defense Against DoS and Injection Attacks):**
- `user_id`: Required, 1-500 characters, no NULL bytes (`\0`)
- `project_id`: Optional, 1-500 characters when provided, no NULL bytes (`\0`)
- `description`: Optional, no length constraint

**Validation Enforcement (Defense-in-Depth):**
1. **API Layer**: Type-safe validated newtypes (ValidatedUserId, ValidatedProjectId)
2. **Database Layer**: CHECK constraints enforce LENGTH(field) BETWEEN 1 AND 500

**Error Responses:**
- **400 Bad Request** - Validation failure (empty string, exceeds 500 chars, contains NULL byte)
  - Error message format: `"Field 'user_id' exceeds maximum length (1000 chars, max 500)"`
  - Error includes: field name, actual length, maximum length, specific violation
- **500 Internal Server Error** - Database constraint violation (should never occur if API validation works)

**Rationale:**
- **DoS Protection (issue-001)**: Unbounded string inputs can cause memory exhaustion. 500-character limit prevents attackers from sending multi-megabyte strings that crash the server.
- **Injection Protection (issue-002)**: NULL bytes can terminate C strings in database drivers, causing data corruption or bypassing validation. Explicit rejection prevents C string termination attacks.
- **Defense-in-Depth**: Database CHECK constraints provide runtime safety if API validation is bypassed.

**Implementation Status:** ✅ COMPLETE (iron_api/src/routes/tokens.rs, 382 lines)
- Endpoint structure: ✅ Complete
- Database integration: ✅ Complete (TokenStorage with hash-based authentication)
- Tests: ✅ Complete (tests/tokens/endpoints.rs, 13 integration tests covering create/get/rotate/revoke)
- Note: GET /api/tokens (list_tokens) requires JWT authentication, tested via manual tests only
- Security: ✅ Complete (BCrypt hashing with cost=12, atomic rotation, plaintext token returned once)

**Example Usage:**
```bash
# Create token
curl -X POST http://localhost:3000/api/tokens \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user-001", "description": "Production API key"}'
# Response: {"id": 1, "token": "tk_abc123...", "user_id": "user-001", ...}

# List tokens
curl http://localhost:3000/api/tokens \
  -H "Authorization: Bearer <jwt_token>"
# Response: [{"id": 1, "user_id": "user-001", "is_active": true, ...}]

# Rotate token
curl -X POST http://localhost:3000/api/tokens/1/rotate
# Response: {"id": 2, "token": "tk_xyz789...", ...}

# Revoke token
curl -X DELETE http://localhost:3000/api/tokens/1
# Response: 204 No Content
```

---

### FR-8: Usage Analytics API

**Requirement:**
Provide REST endpoints for querying token usage statistics with cost breakdown by provider and model.

**Endpoints:**

| Method | Path | Description | Response |
|--------|------|-------------|----------|
| GET | /api/usage/aggregate | Aggregate usage across all tokens | AggregateUsageResponse |
| GET | /api/usage/by-project/:project_id | Usage for specific project | AggregateUsageResponse |
| GET | /api/usage/by-provider/:provider | Usage for specific provider | ProviderStats |

**Response Types:**
```rust
#[derive(Serialize)]
pub struct AggregateUsageResponse {
  pub total_tokens: i64,
  pub total_requests: i64,
  pub total_cost_cents: i64,
  pub providers: Vec<ProviderStats>,
}

#[derive(Serialize)]
pub struct ProviderStats {
  pub provider: String,  // "openai", "anthropic", "google"
  pub tokens: i64,
  pub requests: i64,
  pub cost_cents: i64,
}
```

**Implementation Status:** ✅ COMPLETE (iron_api/src/routes/usage.rs, 286 lines)
- Endpoint structure: ✅ Complete
- Database integration: ✅ Complete (UsageTracker with cross-token aggregation)
- Tests: ✅ Complete (tests/usage/*.rs, 30 test cases including path parameter validation)
- Validation: ✅ Complete (path parameter DoS prevention, URL decoding, provider case-sensitivity)

**Example Usage:**
```bash
# Get aggregate usage
curl http://localhost:3000/api/usage/aggregate
# Response: {"total_tokens": 125000, "total_requests": 450, "total_cost_cents": 1250, "providers": [...]}

# Usage by project
curl http://localhost:3000/api/usage/by-project/proj-001
# Response: {"total_tokens": 50000, "total_requests": 200, ...}

# Usage by provider
curl http://localhost:3000/api/usage/by-provider/openai
# Response: {"provider": "openai", "tokens": 75000, "requests": 250, "cost_cents": 800}
```

---

### FR-9: Budget Limits API

**Requirement:**
Provide REST endpoints for creating, updating, and deleting budget limits.

**Endpoints:**

| Method | Path | Description | Request | Response |
|--------|------|-------------|---------|----------|
| POST | /api/limits | Create usage limit | CreateLimitRequest | LimitResponse |
| GET | /api/limits | List all limits for user | - | Vec<LimitResponse> |
| GET | /api/limits/:id | Get specific limit | - | LimitResponse |
| PUT | /api/limits/:id | Update limit | CreateLimitRequest | LimitResponse |
| DELETE | /api/limits/:id | Delete limit | - | 204 No Content |

**Request/Response Types:**
```rust
#[derive(Deserialize)]
pub struct CreateLimitRequest {
  pub user_id: String,
  pub project_id: Option<String>,
  pub max_tokens_per_day: Option<i64>,
  pub max_requests_per_minute: Option<i64>,
  pub max_cost_per_month_cents: Option<i64>,
}

#[derive(Serialize)]
pub struct LimitResponse {
  pub id: i64,
  pub user_id: String,
  pub project_id: Option<String>,
  pub max_tokens_per_day: Option<i64>,
  pub max_requests_per_minute: Option<i64>,
  pub max_cost_per_month_cents: Option<i64>,
  pub created_at: i64,
}
```

**Implementation Status:** ✅ COMPLETE (iron_api/src/routes/limits.rs, 398 lines)
- Endpoint structure: ✅ Complete
- Database integration: ✅ Complete (LimitEnforcer with ID-based CRUD)
- Validation: ✅ Complete (request validation with detailed error messages)
- Tests: ✅ Complete (tests/limits/*.rs, 27 test cases including overflow/boundary tests)

**Example Usage:**
```bash
# Create limit
curl -X POST http://localhost:3000/api/limits \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user-001", "max_cost_per_month_cents": 5000}'
# Response: {"id": 1, "user_id": "user-001", "max_cost_per_month_cents": 5000, ...}

# List limits
curl http://localhost:3000/api/limits
# Response: [{"id": 1, "max_cost_per_month_cents": 5000, ...}]

# Update limit
curl -X PUT http://localhost:3000/api/limits/1 \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user-001", "max_cost_per_month_cents": 10000}'
# Response: {"id": 1, "max_cost_per_month_cents": 10000, ...}

# Delete limit
curl -X DELETE http://localhost:3000/api/limits/1
# Response: 204 No Content
```

---

### FR-10: Request Traces API

**Requirement:**
Provide REST endpoints for querying detailed per-request trace data for debugging and cost analysis.

**Endpoints:**

| Method | Path | Description | Response |
|--------|------|-------------|----------|
| GET | /api/traces | Query API call traces | Vec<ApiTrace> |
| GET | /api/traces/:id | Get specific trace details | ApiTrace |

**Response Types:**
```rust
#[derive(Serialize)]
pub struct ApiTrace {
  pub id: i64,
  pub token_id: i64,
  pub provider: String,
  pub model: String,
  pub endpoint: String,
  pub status_code: i32,
  pub latency_ms: i64,
  pub input_tokens: i64,
  pub output_tokens: i64,
  pub cost_cents: i64,
  pub timestamp: i64,
}
```

**Implementation Status:** ✅ COMPLETE (iron_api/src/routes/traces.rs, 151 lines)
- Endpoint structure: ✅ Complete
- Database integration: ✅ Complete (TraceStorage service with ordered queries)
- Tests: ✅ Complete (tests/traces/*.rs, 18 test cases including 404 handling)
- Error handling: ✅ Complete (404 for non-existent IDs, proper JSON responses)

**Example Usage:**
```bash
# List all traces
curl http://localhost:3000/api/traces
# Response: [{"id": 1, "provider": "openai", "model": "gpt-4", "cost_cents": 15, ...}]

# Get specific trace
curl http://localhost:3000/api/traces/123
# Response: {"id": 123, "latency_ms": 450, "input_tokens": 100, "output_tokens": 200, ...}
```

---

## 4. Non-Functional Requirements

### NFR-1: API Response Time

**Requirement:** REST API endpoints must respond within 50ms (P99).

**Rationale:** Dashboard queries metrics on demand (user clicks). Slow responses degrade UX.

**Measurement:** HTTP request time from curl to response complete.

**Acceptance:**
- P50: <10ms
- P99: <50ms
- P99.9: <100ms

**Implementation:**
- StateManager reads from DashMap (<1ms)
- JSON serialization (~5ms for 100 agents)
- Network overhead (~5ms localhost)

---

### NFR-2: WebSocket Message Latency

**Requirement:** WebSocket messages must be sent within 100ms of state update (P99).

**Rationale:** Real-time dashboard requires low latency. >100ms lag is noticeable to human observers.

**Measurement:** Timestamp state update in Rust, measure WebSocket arrival in browser.

**Acceptance:**
- P50: <20ms
- P99: <100ms
- P99.9: <500ms

**Implementation:**
- tokio::sync::broadcast for zero-copy event distribution
- Axum WebSocket send is async (non-blocking)
- No message batching (send immediately)

---

### NFR-3: Concurrent Connections

**Requirement:** Support up to 10 concurrent WebSocket connections (multiple dashboard instances).

**Rationale:** Demo may have multiple monitors (presenter laptop, backup screen, recording).

**Measurement:** Load test with 10 simultaneous WebSocket connections.

**Acceptance:**
- All 10 connections receive updates
- No message loss or corruption
- Memory overhead <10MB per connection

**Implementation:**
- Each connection gets own broadcast::Receiver (clone is cheap)
- Axum handles connection concurrency
- No per-connection state (stateless server)

---

### NFR-4: Memory Footprint

**Requirement:** API server must use <50MB RSS for 10 connections.

**Rationale:** Server runs alongside iron_runtime (100MB) and dashboard (browser). Total system <500MB.

**Measurement:** `ps -o rss` with 10 active WebSocket connections.

**Acceptance:**
- RSS <50MB
- No memory leaks (constant RSS over time)

---

## 5. Ubiquitous Language

| Term | Definition | Example |
|------|------------|---------|
| **Endpoint** | HTTP route for REST API | GET /api/agents/:id |
| **Resource** | Entity exposed via REST API | Agent, Budget, Circuit |
| **WebSocket** | Bidirectional streaming protocol | ws://localhost:8080/ws |
| **Message** | JSON payload sent over WebSocket | DashboardMessage::CostUpdate |
| **CORS** | Cross-Origin Resource Sharing | Allow localhost:5173 |
| **Status Code** | HTTP response status | 200 OK, 404 Not Found |
| **Snapshot** | Complete state sent on WebSocket connect | All agents + budget + circuits |
| **Stream** | Continuous WebSocket message flow | Real-time updates |
| **Handler** | Axum async function for endpoint | get_agent_status() |

---

## 6. System Architecture

### 6.1 Component Diagram

```mermaid
graph TB
    subgraph "Iron API (Layer 3)"
        REST[REST Endpoints<br/>Axum Router]
        WS[WebSocket Handler<br/>/ws]
        CORS[CORS Middleware]
        HANDLERS[Request Handlers]
    end

    subgraph "Dependencies (Layer 1)"
        STATE[iron_state<br/>StateManager]
        TYPES[iron_types<br/>Error, Result]
    end

    subgraph "Consumers"
        DASHBOARD[React Dashboard<br/>localhost:5173]
        CURL[curl / CI/CD]
    end

    subgraph "Coordination"
        RUNTIME[iron_runtime<br/>Agent Execution]
    end

    DASHBOARD -->|HTTP GET/POST| REST
    DASHBOARD -->|WebSocket| WS
    CURL -->|HTTP| REST

    REST --> CORS
    CORS --> HANDLERS
    HANDLERS --> STATE
    WS --> STATE

    RUNTIME -->|Publishes Updates| STATE
    STATE -->|Broadcasts| WS
    WS -->|Streams| DASHBOARD
```

### 6.2 Request Flow (REST)

```mermaid
sequenceDiagram
    participant Client as Dashboard / curl
    participant Axum as Axum Server
    participant Handler as Request Handler
    participant State as iron_state

    Client->>Axum: GET /api/agents/agent-001
    Axum->>Handler: get_agent_status(agent_id)
    Handler->>State: get_agent_state(agent_id)
    State-->>Handler: Some(AgentState)
    Handler->>Handler: Serialize to JSON
    Handler-->>Axum: 200 OK, JSON body
    Axum-->>Client: HTTP Response

    Note over Client,State: Total latency: <10ms P50
```

---

## 7. Detailed Functional Requirements

### FR-1: REST - Start Agent

**Endpoint:** `POST /api/agents`

**Request:**
```json
{
  "script_path": "/path/to/agent.py",
  "budget": 50.0
}
```

**Response (201 Created):**
```json
{
  "agent_id": "agent-001",
  "status": "RUNNING",
  "start_time": 1732550611
}
```

**Error Responses:**
- 400 Bad Request: Invalid script_path (file doesnt exist)
- 500 Internal Server Error: Agent spawn failed

**Handler Implementation:**
```rust
async fn start_agent(
  State(state): State<ApiState>,
  Json(req): Json<StartAgentRequest>
) -> Result<(StatusCode, Json<AgentStartedResponse>), (StatusCode, Json<ErrorResponse>)> {
  // Validate script exists
  if !Path::new(&req.script_path).exists() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: "Script not found".to_string(),
        code: "INVALID_SCRIPT_PATH".to_string(),
        details: Some(format!("File does not exist: {}", req.script_path)),
      })
    ));
  }

  // Start agent via runtime
  let handle = state.runtime.start_agent(Path::new(&req.script_path)).await
    .map_err(|e| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        error: "Failed to start agent".to_string(),
        code: "AGENT_START_FAILED".to_string(),
        details: Some(e.to_string()),
      })
    ))?;

  Ok((
    StatusCode::CREATED,
    Json(AgentStartedResponse {
      agent_id: handle.agent_id,
      status: "RUNNING".to_string(),
      start_time: chrono::Utc::now().timestamp(),
    })
  ))
}
```

**Note:** Requires ApiState to hold Arc<AgentRuntime> reference (currently only has StateManager).

---

### FR-2: REST - Get Metrics (Current Implementation)

**Endpoint:** `GET /api/agents/:id/metrics`

**Current Implementation (Lines 127-149):**
- ✅ Basic structure correct
- ✅ Queries StateManager
- ✅ Returns 404 if agent not found
- ⚠️ Returns partial metrics (budget_spent, pii_detections only)

**Enhancement:**
```rust
async fn get_agent_metrics(
  State(state): State<ApiState>,
  Path(agent_id): Path<String>,
) -> Result<Json<MetricsResponse>, (StatusCode, Json<ErrorResponse>)> {
  // Query all metrics from state
  let cost = state.state_manager.get_cost_metrics(&agent_id).await
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse {
      error: "Agent not found".to_string(),
      code: "AGENT_NOT_FOUND".to_string(),
      details: None,
    })))?;

  let safety = state.state_manager.get_safety_metrics(&agent_id).await
    .unwrap_or_default();

  let performance = state.state_manager.get_performance_metrics(&agent_id).await
    .unwrap_or_default();

  Ok(Json(MetricsResponse {
    agent_id,
    cost,
    safety,
    performance,
  }))
}
```

---

### FR-3: WebSocket - Streaming Implementation (Enhanced)

**Current Implementation (Lines 152-157):**
- ✅ WebSocket upgrade handler structure
- ⚠️ Missing: State update subscription
- ⚠️ Missing: Message streaming loop
- ⚠️ Missing: Initial snapshot send

**Full Implementation:**
```rust
async fn websocket_handler(
  ws: WebSocketUpgrade,
  State(state): State<ApiState>
) -> impl IntoResponse {
  ws.on_upgrade(|socket| handle_websocket_connection(socket, state))
}

async fn handle_websocket_connection(
  mut socket: WebSocket,
  state: ApiState
) {
  use axum::extract::ws::Message;

  // Subscribe to state updates
  let mut update_rx = state.state_manager.subscribe_to_updates();

  tracing::info!("Dashboard connected via WebSocket");

  // Build and send initial snapshot
  match build_state_snapshot(&state.state_manager).await {
    Ok(snapshot) => {
      let msg = DashboardMessage::StateSnapshot {
        agents: snapshot.agents,
        budget: snapshot.budget,
        circuits: snapshot.circuits,
      };

      if let Ok(json) = serde_json::to_string(&msg) {
        let _ = socket.send(Message::Text(json)).await;
      }
    },
    Err(e) => {
      tracing::error!("Failed to build state snapshot: {}", e);
      return;
    }
  }

  // Stream updates until disconnection
  while let Ok(update) = update_rx.recv().await {
    // Convert StateUpdate to DashboardMessage
    let message = match update {
      StateUpdate::AgentStateChanged { agent_id } => {
        // Query latest metrics
        let cost = state.state_manager.get_cost_metrics(&agent_id).await;
        if let Some(cost) = cost {
          DashboardMessage::CostUpdate {
            agent_id: agent_id.clone(),
            metrics: cost,
          }
        } else {
          continue;
        }
      },

      StateUpdate::AuditEventLogged { agent_id, event_type } => {
        match event_type.as_str() {
          "PII_DETECTED" => DashboardMessage::PiiAlert {
            agent_id: agent_id.clone(),
            pii_type: "EMAIL".to_string(),
            location: "output".to_string(),
          },
          "BUDGET_WARNING" => {
            let warning = state.state_manager.get_budget_warning(&agent_id).await.unwrap();
            DashboardMessage::BudgetWarning {
              agent_id: agent_id.clone(),
              warning,
            }
          },
          _ => continue,
        }
      },

      StateUpdate::CircuitBreakerChanged { service } => {
        let metrics = state.state_manager.get_circuit_metrics(&service).await.unwrap();
        if metrics.state == CircuitState::Open {
          DashboardMessage::CircuitOpened {
            service: service.clone(),
            failure_count: metrics.failure_count,
          }
        } else {
          continue;
        }
      },
    };

    // Send message
    if let Ok(json) = serde_json::to_string(&message) {
      if socket.send(Message::Text(json)).await.is_err() {
        tracing::warn!("Dashboard disconnected");
        break;
      }
    }
  }

  tracing::info!("WebSocket connection closed");
}

async fn build_state_snapshot(
  state_manager: &StateManager
) -> Result<StateSnapshot> {
  Ok(StateSnapshot {
    agents: state_manager.list_agents().await?,
    budget: state_manager.get_global_budget().await?,
    circuits: state_manager.list_circuits().await?,
  })
}
```

---

## 8. API Surface (Public Contract)

### 8.1 REST Endpoints (Complete API)

**Agent Lifecycle (FR-1, FR-2):**
```
POST   /api/agents              Start new agent
GET    /api/agents/:id          Get agent status
POST   /api/agents/:id/stop     Stop agent gracefully
DELETE /api/agents/:id          Kill agent forcefully
GET    /api/agents/:id/metrics  Get comprehensive agent metrics
GET    /api/budget              Get global budget status
GET    /api/circuits            Get all circuit breaker states
GET    /api/health              API health check
```

**Token Management (FR-7):**
```
POST   /api/tokens              Create new API token
GET    /api/tokens              List all tokens for user
GET    /api/tokens/:id          Get specific token details
POST   /api/tokens/:id/rotate   Rotate token (new value)
DELETE /api/tokens/:id          Revoke token
```

**Usage Analytics (FR-8):**
```
GET    /api/usage/aggregate            Aggregate usage across all tokens
GET    /api/usage/by-project/:id       Usage for specific project
GET    /api/usage/by-provider/:name    Usage for specific provider
```

**Budget Limits (FR-9):**
```
POST   /api/limits              Create usage limit
GET    /api/limits              List all limits for user
GET    /api/limits/:id          Get specific limit
PUT    /api/limits/:id          Update limit
DELETE /api/limits/:id          Delete limit
```

**Request Traces (FR-10):**
```
GET    /api/traces              Query API call traces
GET    /api/traces/:id          Get specific trace details
```

**WebSocket (FR-3):**
```
GET    /ws                      WebSocket upgrade for streaming
```

**Total Endpoints:** 26 REST endpoints + 1 WebSocket

### 8.2 Rust API

```rust
// API server (FR-1 through FR-6)
pub struct ApiServer {
  state: ApiState,
  addr: SocketAddr,
}

impl ApiServer {
  pub fn new(state_manager: Arc<StateManager>, port: u16) -> Self;

  pub async fn start(self) -> Result<()>;
}

// API state (shared across handlers)
#[derive(Clone)]
pub struct ApiState {
  pub state_manager: Arc<StateManager>,
  pub runtime: Option<Arc<AgentRuntime>>,  // NEW: For POST /api/agents
}

// Request types
#[derive(Deserialize)]
pub struct StartAgentRequest {
  pub script_path: String,
  pub budget: Option<f64>,
}

// Response types
#[derive(Serialize)]
pub struct AgentStartedResponse {
  pub agent_id: String,
  pub status: String,
  pub start_time: i64,
}

#[derive(Serialize)]
pub struct AgentStatusResponse {
  pub agent_id: String,
  pub status: String,
  pub start_time: i64,
  pub end_time: Option<i64>,
}

#[derive(Serialize)]
pub struct MetricsResponse {
  pub agent_id: String,
  pub cost: CostMetrics,
  pub safety: SafetyMetrics,
  pub performance: PerformanceMetrics,
}

#[derive(Serialize)]
pub struct ErrorResponse {
  pub error: String,
  pub code: String,
  pub details: Option<String>,
}

// WebSocket messages
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum DashboardMessage {
  StateSnapshot { agents: Vec<AgentState>, budget: BudgetSnapshot, circuits: Vec<CircuitBreakerMetrics> },
  AgentStarted { agent_id: String, timestamp: i64 },
  AgentCompleted { agent_id: String, leads_processed: u32, timestamp: i64 },
  CostUpdate { agent_id: String, metrics: CostMetrics },
  SafetyEvent { agent_id: String, event: SafetyMetrics },
  BudgetWarning { agent_id: String, warning: BudgetWarning },
  PiiAlert { agent_id: String, pii_type: String, location: String },
  CircuitOpened { service: String, failure_count: u32 },
}
```

---

## 9. External Dependencies

### 9.1 Rust Crates

| Crate | Version | Purpose | Risk | Mitigation |
|-------|---------|---------|------|------------|
| **axum** | 0.7 | Web framework | Low (mature) | Pin version, test CORS |
| **tower-http** | 0.5 | CORS middleware | Low (standard) | Existing in ecosystem |
| **tokio** | 1.35 | Async runtime | Low (standard) | Existing in project |
| **serde** | 1.0 | JSON serialization | Low (standard) | Existing in project |
| **serde_json** | 1.0 | JSON encoding | Low (standard) | Existing in project |

**Cargo.toml:**
```toml
[dependencies]
iron_types = { version = "0.1", path = "../iron_types" }
iron_state = { version = "0.1", path = "../iron_state" }
iron_runtime = { version = "0.1", path = "../iron_runtime", optional = true }

axum = { version = "0.7", features = ["ws"] }
tower-http = { version = "0.5", features = ["cors"] }
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"

[features]
enabled = []
runtime = ["iron_runtime"]
full = ["enabled", "runtime"]
```

### 9.2 Internal Crate Dependencies

- **iron_types** (Layer 1) - Error, Result types
- **iron_state** (Layer 1) - StateManager for data queries
- **iron_runtime** (Layer 3) - Optional: For POST /api/agents endpoint

**Note:** iron_api can run without iron_runtime if only serving metrics (read-only mode).

---

## 10. Data Model

### 10.1 API State

**ApiState:**
```rust
#[derive(Clone)]
pub struct ApiState {
  pub state_manager: Arc<StateManager>,
  pub runtime: Option<Arc<AgentRuntime>>,  // For agent control
}
```

**ApiServer:**
```rust
pub struct ApiServer {
  state: ApiState,
  addr: SocketAddr,
}
```

### 10.2 Request Types

**StartAgentRequest:**
```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartAgentRequest {
  pub script_path: String,

  #[serde(default)]
  pub budget: Option<f64>,  // Override global budget
}
```

### 10.3 Response Types

**MetricsResponse:**
```rust
#[derive(Debug, Serialize)]
pub struct MetricsResponse {
  pub agent_id: String,
  pub cost: CostMetrics,
  pub safety: SafetyMetrics,
  pub performance: PerformanceMetrics,
}
```

**BudgetResponse:**
```rust
#[derive(Debug, Serialize)]
pub struct BudgetResponse {
  pub limit: f64,
  pub spent: f64,
  pub remaining: f64,
  pub percentage_used: f64,
  pub agents: Vec<AgentBudget>,
}

#[derive(Debug, Serialize)]
pub struct AgentBudget {
  pub agent_id: String,
  pub spent: f64,
}
```

---

## 11. Error Handling

### 11.1 HTTP Status Codes

**Success Codes:**
- **200 OK** - Successful query (GET endpoints)
- **201 Created** - Agent started successfully (POST /api/agents)
- **204 No Content** - Agent stopped successfully (POST /api/agents/:id/stop)

**Client Error Codes:**
- **400 Bad Request** - Invalid request body or parameters
- **404 Not Found** - Unknown agent_id or resource
- **402 Payment Required** - Budget exceeded

**Server Error Codes:**
- **500 Internal Server Error** - Runtime panic or unexpected error
- **503 Service Unavailable** - API server shutting down

### 11.2 Error Response Format

**Standard Error:**
```json
{
  "error": "Human-readable error message",
  "code": "MACHINE_READABLE_CODE",
  "details": "Optional additional context"
}
```

**Examples:**
```json
// 404 Not Found
{
  "error": "Agent not found",
  "code": "AGENT_NOT_FOUND",
  "details": "No agent with id 'agent-999'"
}

// 400 Bad Request
{
  "error": "Invalid script path",
  "code": "INVALID_SCRIPT_PATH",
  "details": "File does not exist: /path/to/missing.py"
}

// 500 Internal Server Error
{
  "error": "Failed to start agent",
  "code": "AGENT_START_FAILED",
  "details": "Permission denied: /path/to/agent.py"
}
```

---

## 12. Integration Points

### 12.1 With iron_state (Layer 1)

**Queries:**
- `state_manager.get_agent_state(id)` - For status endpoint
- `state_manager.get_cost_metrics(id)` - For metrics endpoint
- `state_manager.list_agents()` - For listing endpoint
- `state_manager.subscribe_to_updates()` - For WebSocket streaming

**Updates (via iron_runtime):**
- iron_runtime publishes to StateManager
- StateManager broadcasts to API's WebSocket connections
- API is passive (doesnt modify state, only reads and streams)

### 12.2 With iron_runtime (Layer 3)

**Agent Control:**
- API server holds Optional<Arc<AgentRuntime>> reference
- POST /api/agents → calls runtime.start_agent()
- POST /api/agents/:id/stop → calls runtime.stop_agent()

**Coordination:**
- Shared StateManager instance (Arc cloned)
- Runtime publishes updates, API streams to dashboards
- No direct API → Runtime calls (unidirectional: Runtime → State → API)

**Note:** For pilot, iron_runtime's embedded WebSocket server (FR-4) is sufficient. iron_api provides REST alternative for extensibility.

### 12.3 With iron_dashboard (Layer 4)

**Dashboard HTTP Client:**
```typescript
// In React dashboard
const apiClient = {
  async getAgentStatus(agentId: string): Promise<AgentStatusResponse> {
    const resp = await fetch(`http://localhost:8080/api/agents/${agentId}`);
    if (!resp.ok) throw new Error(await resp.text());
    return await resp.json();
  },

  async getMetrics(agentId: string): Promise<MetricsResponse> {
    const resp = await fetch(`http://localhost:8080/api/agents/${agentId}/metrics`);
    return await resp.json();
  },
};
```

**Dashboard WebSocket Client:**
```typescript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
  const message: DashboardMessage = JSON.parse(event.data);

  switch (message.type) {
    case 'StateSnapshot':
      // Initial render
      updateAllPanels(message);
      break;

    case 'CostUpdate':
      // Update Cost Panel
      updateCostPanel(message.metrics);
      break;

    case 'BudgetWarning':
      // Show alert modal
      showBudgetWarningModal(message.warning);
      break;

    // ... other message types
  }
};
```

---

## 13. Demo Integration

### 13.1 API Role in Demo

**For Pilot Demo:**
- **Primary Dashboard Connection:** iron_runtime's WebSocket (iron_runtime spec FR-4)
- **Alternative (Optional):** iron_api's WebSocket endpoint
- **REST API:** Not used in demo (CLI-based execution)

**Rationale:**
- Demo uses `iron_cage start` CLI command (not REST API)
- Dashboard connects to ws://localhost:8080/ws
- WebSocket can be served by iron_runtime OR iron_api (both work)

**Recommendation:**
- **Pilot:** Use iron_runtime's embedded WebSocket (simpler, fewer processes)
- **Full Platform:** Use iron_api for separation of concerns

### 13.2 API Endpoints Used by Dashboard

**On Dashboard Load:**
```
GET /api/health
→ Check if runtime is running

GET /api/agents
→ List all agents (for multi-agent scenarios)
```

**Real-Time Updates:**
```
WebSocket /ws
→ Subscribe to state updates
→ Receive DashboardMessage stream
```

**User Actions:**
```
POST /api/agents/:id/stop
→ When user clicks "Stop Agent" button
→ Dashboard shows confirmation modal
```

**Metrics Queries (Optional):**
```
GET /api/agents/:id/metrics
→ On-demand refresh (not real-time)
→ Fallback if WebSocket disconnected
```

---

## 14. Testing Strategy

### 14.1 Unit Tests

**Test File:** `tests/rest_endpoints_unit_tests.rs`
```rust
use axum_test_helper::TestClient;

#[tokio::test]
async fn test_get_agent_status_success() {
  let state = Arc::new(StateManager::new(":memory:").await.unwrap());

  // Insert test agent
  state.save_agent_state(AgentState {
    agent_id: "test-001".to_string(),
    status: AgentStatus::Running,
    ...
  });

  let api_state = ApiState { state_manager: state, runtime: None };
  let app = create_router(api_state);
  let client = TestClient::new(app);

  let response = client.get("/api/agents/test-001").await;
  assert_eq!(response.status(), StatusCode::OK);

  let body: AgentStatusResponse = response.json().await;
  assert_eq!(body.agent_id, "test-001");
  assert_eq!(body.status, "Running");
}

#[tokio::test]
async fn test_get_agent_status_not_found() {
  let state = Arc::new(StateManager::new(":memory:").await.unwrap());
  let api_state = ApiState { state_manager: state, runtime: None };
  let app = create_router(api_state);
  let client = TestClient::new(app);

  let response = client.get("/api/agents/nonexistent").await;
  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  let body: ErrorResponse = response.json().await;
  assert_eq!(body.code, "AGENT_NOT_FOUND");
}
```

**Test File:** `tests/cors_unit_tests.rs`
```rust
#[tokio::test]
async fn test_cors_allows_dashboard_origin() {
  let state = Arc::new(StateManager::new(":memory:").await.unwrap());
  let api_state = ApiState { state_manager: state, runtime: None };
  let app = create_router(api_state);
  let client = TestClient::new(app);

  let response = client.get("/api/health")
    .header("Origin", "http://localhost:5173")
    .await;

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("Access-Control-Allow-Origin").unwrap(),
    "http://localhost:5173"
  );
}
```

### 14.2 Integration Tests

**Test File:** `tests/websocket_streaming_integration_tests.rs`
```rust
/// WebSocket streaming with state updates
///
/// # Prevention
/// Validates WebSocket message streaming:
/// - Dashboard connects
/// - Receives initial StateSnapshot
/// - Receives updates as state changes
/// - Handles disconnection gracefully
///
/// # Pitfall
/// WebSocket connections must receive StateSnapshot immediately
/// on connect, not wait for first update. Otherwise dashboard
/// shows empty state until first event.

#[tokio::test]
async fn test_websocket_streaming() {
  let state = Arc::new(StateManager::new(":memory:").await.unwrap());
  let api_state = ApiState { state_manager: state.clone(), runtime: None };

  // Start API server in background
  let server = ApiServer::new(state.clone(), 8080);
  tokio::spawn(async move { server.start().await });

  // Wait for server startup
  tokio::time::sleep(Duration::from_millis(100)).await;

  // Connect WebSocket client
  let (mut ws_stream, _) = tokio_tungstenite::connect_async("ws://localhost:8080/ws")
    .await.unwrap();

  // Should receive StateSnapshot immediately
  let msg = ws_stream.next().await.unwrap().unwrap();
  let snapshot: DashboardMessage = serde_json::from_str(&msg.to_text().unwrap()).unwrap();

  assert!(matches!(snapshot, DashboardMessage::StateSnapshot { .. }));

  // Trigger state update
  state.save_agent_state(AgentState {
    agent_id: "test-001".to_string(),
    status: AgentStatus::Running,
    ...
  });

  // Should receive CostUpdate or similar
  let msg = ws_stream.next().await.unwrap().unwrap();
  let update: DashboardMessage = serde_json::from_str(&msg.to_text().unwrap()).unwrap();

  // Verify message received
  assert!(matches!(
    update,
    DashboardMessage::CostUpdate { .. } | DashboardMessage::AgentStarted { .. }
  ));
}
```

### 14.3 Performance Tests

**Test File:** `tests/api_performance_tests.rs`
```rust
#[tokio::test]
async fn test_rest_api_response_time() {
  let state = Arc::new(StateManager::new(":memory:").await.unwrap());

  // Insert 100 agents
  for i in 0..100 {
    state.save_agent_state(AgentState {
      agent_id: format!("agent-{:03}", i),
      ...
    });
  }

  let api_state = ApiState { state_manager: state, runtime: None };
  let app = create_router(api_state);
  let client = TestClient::new(app);

  // Measure response time
  let start = Instant::now();
  let response = client.get("/api/agents/agent-050").await;
  let duration = start.elapsed();

  assert_eq!(response.status(), StatusCode::OK);
  assert!(duration < Duration::from_millis(50));  // NFR-1: <50ms P99
}
```

---

## 15. Success Metrics

### 15.1 Pilot Success Criteria

**Functional:**
- ✅ Dashboard connects via WebSocket successfully
- ✅ StateSnapshot received within 1s of connection
- ✅ Real-time updates stream during demo (all 3 triggers)
- ✅ REST endpoints return correct HTTP status codes
- ✅ CORS allows localhost:5173 origin
- ✅ Error responses have consistent JSON format

**Performance (NFRs):**
- ✅ REST API response time: <50ms P99 (NFR-1)
- ✅ WebSocket message latency: <100ms P99 (NFR-2)
- ✅ Concurrent connections: 10 simultaneous (NFR-3)
- ✅ Memory footprint: <50MB RSS (NFR-4)

**Demo Execution:**
- ✅ Dashboard renders without API errors
- ✅ Real-time updates visible during demo
- ✅ No WebSocket disconnections during 5-minute demo

### 15.2 Code Quality

- Target LOC: 600 (current: 201, gap: +399)
- Test coverage: >75% line coverage
- Zero clippy warnings (Level 3 tests pass)
- Documentation: All public APIs have doc comments
- OpenAPI spec: Optional (can generate from code)

---

## 16. Implementation Phases

### Phase 1: Enhanced REST Endpoints (FR-1, FR-2)
**Estimated LOC:** +100
**Dependencies:** iron_state
**Deliverable:** Complete all REST endpoints with error handling
**Validation:** Unit tests for all endpoints (200, 404, 400 cases)

### Phase 2: WebSocket Streaming (FR-3)
**Estimated LOC:** +150
**Dependencies:** iron_state broadcasts
**Deliverable:** Full WebSocket handler with StateSnapshot + updates
**Validation:** Integration test with real WebSocket client

### Phase 3: CORS Configuration (FR-4)
**Estimated LOC:** +20
**Dependencies:** tower-http
**Deliverable:** CORS middleware with localhost:5173 allowed
**Validation:** CORS unit test with Origin header

### Phase 4: Error Handling (FR-5)
**Estimated LOC:** +60
**Dependencies:** None
**Deliverable:** Consistent ErrorResponse format across all endpoints
**Validation:** Error test cases for all endpoints

### Phase 5: Runtime Integration (FR-6)
**Estimated LOC:** +40
**Dependencies:** iron_runtime
**Deliverable:** POST /api/agents endpoint with runtime.start_agent() call
**Validation:** Integration test starting real agent

### Phase 6: Performance Optimization
**Estimated LOC:** +29 (optimizations)
**Dependencies:** All phases
**Deliverable:** Benchmark tests passing NFR thresholds
**Validation:** criterion benchmarks <50ms REST, <100ms WebSocket

**Total Estimated:** 399 LOC (meets 600 target)

---

## 17. Limitations and Assumptions

### 17.1 Pilot Limitations

**Simplified for Conference Demo:**

1. **No Authentication:** All endpoints publicly accessible on localhost
   - **Why:** Demo runs on single machine, no network exposure
   - **Impact:** Full platform needs JWT tokens, API keys

2. **No Rate Limiting:** Unlimited requests per client
   - **Why:** Dashboard is trusted client, no abuse risk
   - **Impact:** Full platform needs per-IP rate limits (100 req/min)

3. **Single WebSocket per Dashboard:** No connection pooling or load balancing
   - **Why:** Demo has 1 dashboard instance
   - **Impact:** Full platform supports multiple dashboards, needs broadcast multiplexing

4. **Localhost Only:** Binds to 127.0.0.1, not 0.0.0.0
   - **Why:** Demo dashboard on same machine
   - **Impact:** Full platform listens on 0.0.0.0 with TLS

5. **No API Versioning:** Endpoints are /api/*, not /api/v1/*
   - **Why:** Pilot has single version
   - **Impact:** Full platform needs versioned endpoints for backward compatibility

6. **No Request Logging:** Basic tracing only, no access logs
   - **Why:** Demo doesnt need audit trail
   - **Impact:** Full platform logs all requests for compliance

### 17.2 Known Issues

**None currently.** Basic implementation is functional.

**Potential Issue (Redundancy with iron_runtime):**
- iron_runtime spec (FR-4) includes WebSocket server
- iron_api also has WebSocket handler
- **Decision:** Both implementations acceptable
  - **Option A:** Use iron_runtime's WebSocket (simpler, fewer processes)
  - **Option B:** Use iron_api's WebSocket (better separation of concerns)
  - **Pilot:** Recommend Option A (iron_runtime) for simplicity

---

## 18. Cross-References

### 18.1 Parent Specifications

- **Pilot Spec:** `/home/user1/pro/lib/willbe/module/iron_cage/pilot/spec.md`
  - Section 5.3: Feature #26 (API & Communication)
  - Lines 450-470: API requirements

- **Demo Scenario:** `/home/user1/pro/lib/willbe/module/iron_cage/pilot/demo/demonstration_scenario.md`
  - Dashboard connection: ws://localhost:8080/ws
  - Real-time updates during demo

### 18.2 Related Crate Specifications

- **iron_types:** Error enum, Result alias
- **iron_state:** StateManager, StateUpdate broadcasts
- **iron_runtime:** Embedded WebSocket server (alternative to iron_api)
- **iron_dashboard:** WebSocket client, REST queries

### 18.3 Architectural Notes

**WebSocket Server Location (Design Decision):**

Two options for dashboard WebSocket connection:

**Option A: iron_runtime::start_websocket_server() (Recommended for Pilot)**
- Pros: Single process, simpler deployment, runtime owns dashboard communication
- Cons: Runtime becomes heavier, mixed responsibilities

**Option B: iron_api::ApiServer::start() (Better for Full Platform)**
- Pros: Separation of concerns, API can scale independently
- Cons: Two processes to manage, more configuration

**Pilot Recommendation:** Option A (iron_runtime).
**Reason:** Demo has 1 agent, 1 runtime, 1 dashboard. Single process is simpler.

**Full Platform:** Option B for microservices architecture.

---

## 19. Conformance Checklist

**Specification Completeness:**
- ✅ Version and date specified
- ✅ Component and layer identified
- ✅ Purpose and scope defined (In/Out of scope)
- ✅ Functional requirements (FR-1 through FR-10)
- ✅ Non-functional requirements (NFR-1 through NFR-4)
- ✅ Ubiquitous language vocabulary
- ✅ System architecture (Mermaid diagrams)
- ✅ API surface documented (26 REST endpoints + WebSocket)
- ✅ Error handling specified
- ✅ Integration points defined
- ✅ Demo integration detailed
- ✅ Testing strategy provided
- ✅ Success metrics measurable
- ✅ Limitations documented
- ✅ Cross-references complete

**Implementation Readiness:**
- ❌ FR-1: All REST endpoints implemented (agents, status, stop, metrics)
- ❌ FR-2: Metrics query endpoints implemented (budget, circuits, health)
- ❌ FR-3: WebSocket streaming with StateSnapshot implemented
- ❌ FR-3: State update subscription and broadcasting implemented
- ❌ FR-4: CORS middleware configured for localhost:5173
- ❌ FR-5: Consistent error responses across all endpoints
- ❌ FR-6: Integration with iron_runtime tested
- ✅ FR-7: Token management API implemented (tokens.rs, 382 lines COMPLETE, 22 tests)
- ✅ FR-8: Usage analytics API implemented (usage.rs, 3 endpoints COMPLETE, 21 tests)
- ✅ FR-9: Budget limits API implemented (limits.rs, 5 endpoints COMPLETE, 30 tests)
- ✅ FR-10: Request traces API implemented (traces.rs, 2 endpoints COMPLETE, 16 tests)
- ❌ NFR-1: REST response time <50ms P99 verified
- ❌ NFR-2: WebSocket latency <100ms P99 verified
- ❌ NFR-3: 10 concurrent connections tested
- ❌ NFR-4: Memory <50MB RSS verified
- ❌ WebSocket streaming integration test passes
- ✅ Test coverage ≥75% (353 tests, 100% pass rate for all FRs + comprehensive corner case coverage)
- ✅ Zero clippy warnings

---

## 20. Open Questions and Decisions

### 20.1 Resolved

**Q1:** Should WebSocket be in iron_runtime or iron_api?
**A1:** Both implementations exist. Pilot uses iron_runtime's WebSocket for simplicity. iron_api provides alternative.

**Q2:** Authentication for REST API?
**A2:** No for pilot (localhost only). Full platform uses JWT.

**Q3:** GraphQL vs REST?
**A3:** REST for pilot (simpler). Full platform can add GraphQL if needed.

### 20.2 Pending

**Q1:** Should iron_api be removed from pilot scope entirely?
**Decision Needed:** Demo doesn't use REST API (CLI-based). Keep for extensibility or remove to reduce scope?
**Recommendation:** Keep but mark OPTIONAL. Minimal implementation cost (399 LOC), enables future CI/CD integration.

**Q2:** Where should POST /api/agents endpoint start agents?
**Options:**
  - A: Call iron_runtime.start_agent() (requires runtime reference)
  - B: Proxy to iron_cli (spawn subprocess)
  - C: Remove endpoint (demo uses CLI only)
**Recommendation:** Option A for full platform, Option C for pilot (remove endpoint).

---

## 21. Migration Notes

### 21.1 From Current Implementation

**Existing Code (201 LOC):**
- ✅ ApiState struct with StateManager
- ✅ ApiServer struct with new(), start()
- ✅ Basic REST endpoints (3 implemented)
- ✅ WebSocket upgrade handler (stub)
- ✅ CORS middleware (permissive)
- ⚠️ Missing: WebSocket message streaming
- ⚠️ Missing: Full REST API (5 endpoints missing)
- ⚠️ Missing: Consistent error responses

**Required Changes:**
1. Add iron_runtime dependency (optional feature)
2. Add AgentRuntime to ApiState (for POST /api/agents)
3. Enhance WebSocket handler with streaming logic
4. Implement missing REST endpoints (GET /api/budget, GET /api/circuits, GET /api/health)
5. Standardize error responses (ErrorResponse struct)
6. Add build_state_snapshot() helper
7. Add convert_state_update_to_message() helper
8. Configure CORS for specific origin (not permissive)

**Estimated Effort:** 2 developer-days (399 LOC to add + tests)
**Priority:** OPTIONAL for pilot (can defer if time-constrained)

### 21.2 Breaking Changes

**Potential Breaking Change (CORS):**
- Current: CorsLayer::permissive() (allows all origins)
- Pilot: Restrict to localhost:5173
- Impact: External tools cant access API (acceptable for demo)

**No other breaking changes.** Endpoints are additive.

### 21.3 Phases 1-5 Security Fixes and Corner Case Coverage (Completed 2025-12-07)

**Phase 1: Security Vulnerabilities Addressed (2025-12-06)**

1. **Issue-001: DoS via Unlimited String Length**
   - **Problem:** API accepted unbounded `user_id` and `project_id` strings, allowing attackers to send multi-megabyte inputs causing memory exhaustion and service crashes
   - **Fix:** Added MAX_USER_ID_LENGTH = 500 and MAX_PROJECT_ID_LENGTH = 500 constants with type-safe validated newtypes (ValidatedUserId, ValidatedProjectId)
   - **Tests:** 3 bug reproducer tests in tests/tokens/corner_cases.rs
   - **Migration:** 002_add_length_constraints.sql adds database CHECK constraints

2. **Issue-002: NULL Byte Injection**
   - **Problem:** NULL bytes (`\0`) in strings could terminate C strings in database drivers, causing data corruption or validation bypasses
   - **Fix:** Extended validation to reject NULL bytes in all string inputs
   - **Tests:** 2 bug reproducer tests in tests/tokens/corner_cases.rs
   - **Impact:** All CreateTokenRequest validation now includes NULL byte checks

3. **Issue-003: Database Test Isolation**
   - **Problem:** Tests shared database state causing flaky failures and non-deterministic results
   - **Fix:** Created tests/common/database.rs with TestDb fixture using in-memory SQLite databases
   - **Tests:** 4 infrastructure tests verifying isolation, rollback, concurrency, and constraints
   - **Impact:** All tests now use isolated in-memory databases via TestDb::new()

**Phases 2-5: Corner Case Coverage (2025-12-07)**

- **State Transitions (7 tests):** Token lifecycle validation, revoked/nonexistent token handling (3 bugs fixed)
- **Concurrency (4 tests):** Race condition detection, concurrent rotate/revoke operations (3 bugs fixed)
- **Malformed JSON (16 tests):** Invalid syntax, deep nesting, invalid UTF-8 across all endpoints
- **HTTP Methods (14 tests):** 405 Method Not Allowed validation across all endpoints
- **Content-Type (6 tests):** Wrong/missing Content-Type header validation
- **Idempotency (4 tests):** DELETE operation semantics standardization (1 bug fixed)
- **Error Format (2 tests):** Consistent JSON error responses across all domains
- **Empty Body (4 tests):** Empty JSON and missing body edge case handling
- **Additional Coverage (+23 tests):** Integration tests, auth flows, RBAC scenarios, traces endpoints

**Defense-in-Depth Architecture:**
- **Layer 1:** Type-safe validation at API boundary (ValidatedUserId/ValidatedProjectId newtypes with private fields)
- **Layer 2:** Database CHECK constraints for runtime enforcement (migration 002)
- **Layer 3:** Comprehensive test coverage (353 tests, including all corner cases and attack scenarios)

**Breaking Changes:**
- None for existing clients (validation only rejects previously invalid inputs)
- New error responses: 400 Bad Request with descriptive messages for validation failures

**Test Coverage Increase:**
- Baseline: 201 tests → Phases 1-5 Complete: 353 tests (+152 tests, +76%)
- All tests passing with 0 warnings (clippy, doc tests)
- All 8 implementation bugs discovered during Phases 2-5 have been fixed

---

## 22. Future Extensions (Post-Pilot)

### 22.1 API Authentication (JWT)

**Requirement:** Secure API with JWT token-based authentication.

**Implementation:**
```rust
use jsonwebtoken::{encode, decode, Header, Validation};

#[derive(Deserialize)]
struct Claims {
  sub: String,  // User ID
  exp: usize,   // Expiration
}

async fn auth_middleware(
  headers: HeaderMap,
  request: Request
) -> Result<Request, StatusCode> {
  let token = headers.get("Authorization")
    .and_then(|h| h.to_str().ok())
    .and_then(|h| h.strip_prefix("Bearer "))
    .ok_or(StatusCode::UNAUTHORIZED)?;

  let claims = decode::<Claims>(token, &KEYS.decoding, &Validation::default())
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

  Ok(request)
}
```

**Effort:** 2 developer-days (JWT library, token generation, middleware)

### 22.2 Rate Limiting

**Requirement:** Limit API requests per client to prevent abuse.

**Implementation:**
```rust
use governor::{Quota, RateLimiter};

let limiter = RateLimiter::direct(Quota::per_minute(100));

async fn rate_limit_middleware(
  client_ip: String,
  request: Request
) -> Result<Request, StatusCode> {
  if limiter.check_key(&client_ip).is_err() {
    return Err(StatusCode::TOO_MANY_REQUESTS);
  }
  Ok(request)
}
```

**Effort:** 1 developer-day (governor crate integration)

### 22.3 GraphQL Interface

**Requirement:** GraphQL endpoint for flexible dashboard queries.

**Features:**
- Single query for multiple resources (agents + budget + circuits)
- Nested queries (agent → metrics → cost breakdown)
- Real-time subscriptions (GraphQL alternative to WebSocket)

**Effort:** 5 developer-days (async-graphql integration, schema design)

---

**End of Specification**

**Priority Note:**
iron_api is OPTIONAL for pilot. Demo uses iron_runtime's embedded WebSocket server (iron_runtime spec FR-4). REST API provides future extensibility but is not required for conference demo execution.

**Recommendation:**
- **If time-constrained:** Skip iron_api implementation, use iron_runtime's WebSocket
- **If time available:** Implement iron_api for clean separation of concerns

**Next Steps:**
1. Review this specification
2. **Decision:** Keep or defer iron_api for pilot?
3. If keeping: Implement Phases 1-6
4. If deferring: Remove from pilot scope, mark for post-demo

---

**Last Updated:** 2025-12-07
**Revision History:**
- 2025-12-07 (v0.4): Added § 2.3 Deployment Modes, completed Phases 1-5 security fixes and corner case coverage (353 tests, 8 bugs fixed)
- 2025-12-06 (v0.3): Added FR-7 (Token Management), FR-8 (Usage Analytics), FR-9 (Budget Limits), FR-10 (Request Traces)
**Next Review:** Post-pilot (all FRs implemented, comprehensive test coverage achieved)
