# iron_control_api

REST API and WebSocket endpoints for programmatic control of Iron Cage runtime and real-time dashboard communication.

### Scope

**Responsibilities:**
Provides REST API and WebSocket endpoints for programmatic control of Iron Cage runtime (agent lifecycle, monitoring, metrics) and real-time dashboard communication. Enables external tools, CI/CD pipelines, and web dashboards to start agents, monitor execution, and receive live updates via standardized HTTP/WebSocket interfaces. Supports Pilot Mode (localhost, shared iron_runtime_state) and Production Mode (distributed, PostgreSQL + Redis). Requires Rust 1.75+, all platforms supported, integrates with iron_runtime for orchestration and iron_runtime_state for persistence.

**In Scope:**
- REST API for token management (FR-7): Create, rotate, revoke, list API tokens
- REST API for usage analytics (FR-8): Query usage by project, provider, aggregates
- REST API for budget limits (FR-9): CRUD operations for budget constraints
- REST API for request traces (FR-10): List and retrieve execution traces
- WebSocket server for real-time dashboard updates
- CORS support for browser-based dashboards
- Request validation and error handling (DoS protection, NULL byte injection prevention)
- Comprehensive test coverage (353 tests, 100% pass rate)

**Out of Scope:**
- API authentication (JWT tokens) - Deferred to Post-Pilot (spec.md § 2.2)
- Rate limiting (per-IP, per-key) - Deferred to Post-Pilot (spec.md § 2.2)
- Distributed API gateway (multi-node) - Deferred to Post-Pilot (spec.md § 2.2)
- GraphQL interface - Pilot uses REST only (spec.md § 2.2)
- Webhook notifications (external systems) - Pilot uses WebSocket only (spec.md § 2.2)
- Agent lifecycle REST endpoints - See iron_cli
- Runtime orchestration - See iron_runtime
- State persistence - See iron_runtime_state

## Installation

```toml
[dependencies]
iron_control_api = { version = "0.1", features = ["enabled"] }
```

## Features

- `default = ["enabled"]`: Standard configuration
- `enabled`: Full API functionality (depends on iron_types, iron_runtime_state, iron_token_manager, iron_telemetry, iron_cost)

## Architecture

**Pilot/Demo Mode (Current):**
- Single Rust process (localhost:8080)
- Shared iron_runtime_state (DashMap + SQLite) with iron_runtime
- WebSocket for real-time dashboard updates
- No authentication (localhost-only, trusted dashboard)

**Production Mode (Post-Pilot):**
- Distributed deployment (iron_control_schema + PostgreSQL, Redis pub/sub)
- Telemetry ingestion endpoints (POST /v1/telemetry/events)
- Full authentication and rate limiting

## API Endpoints

**Health Check:**
- `GET /api/health` - Server health status

**Authentication (Protocol 014):**
- `POST /api/v1/auth/login` - User authentication
- `POST /api/v1/auth/refresh` - Refresh JWT token
- `POST /api/v1/auth/logout` - User logout
- `POST /api/v1/auth/validate` - Validate JWT token

**User Management:**
- `POST /api/v1/users` - Create user
- `GET /api/v1/users` - List users
- `GET /api/v1/users/:id` - Get user details
- `DELETE /api/v1/users/:id` - Delete user
- `PUT /api/v1/users/:id/suspend` - Suspend user
- `PUT /api/v1/users/:id/activate` - Activate user
- `PUT /api/v1/users/:id/role` - Change user role
- `POST /api/v1/users/:id/reset-password` - Reset password

**API Token Management (Protocol 014):**
- `POST /api/v1/api-tokens` - Create new API token
- `POST /api/v1/api-tokens/validate` - Validate API token (public endpoint)
- `GET /api/v1/api-tokens` - List all tokens (requires auth)
- `GET /api/v1/api-tokens/:id` - Get token details
- `POST /api/v1/api-tokens/:id/rotate` - Rotate token secret
- `DELETE /api/v1/api-tokens/:id` - Revoke token
- `PUT /api/v1/api-tokens/:id` - Update token metadata

**Agent Management (Protocol 010):**
- `POST /api/v1/agents` - Create agent
- `GET /api/v1/agents` - List agents
- `GET /api/v1/agents/:id` - Get agent details
- `PUT /api/v1/agents/:id` - Update agent
- `DELETE /api/v1/agents/:id` - Delete agent
- `GET /api/v1/agents/:id/tokens` - Get agent tokens

**Provider Key Management:**
- `POST /api/v1/providers` - Create provider key
- `GET /api/v1/providers` - List provider keys
- `GET /api/v1/providers/:id` - Get provider key
- `PUT /api/v1/providers/:id` - Update provider key
- `DELETE /api/v1/providers/:id` - Delete provider key
- `POST /api/v1/projects/:project_id/provider` - Assign provider to project
- `DELETE /api/v1/projects/:project_id/provider` - Unassign provider from project

**Key Fetch (API Token Auth):**
- `GET /api/v1/keys` - Fetch provider key by API token

**Usage Analytics (FR-8):**
- `GET /api/v1/usage/aggregate` - Get aggregate usage metrics
- `GET /api/v1/usage/by-project/:project_id` - Get usage by project
- `GET /api/v1/usage/by-provider/:provider` - Get usage by provider

**Budget Limits (FR-9):**
- `POST /api/v1/limits` - Create budget limit
- `GET /api/v1/limits` - List all limits
- `GET /api/v1/limits/:id` - Get limit details
- `PUT /api/v1/limits/:id` - Update limit
- `DELETE /api/v1/limits/:id` - Delete limit

**Request Traces (FR-10):**
- `GET /api/v1/traces` - List request traces
- `GET /api/v1/traces/:id` - Get trace details

**Budget Control (Protocol 005 & 012):**
- `POST /api/v1/budget/handshake` - Agent budget handshake
- `POST /api/v1/budget/report` - Report usage
- `POST /api/v1/budget/refresh` - Refresh budget
- `POST /api/v1/budget/requests` - Create budget request
- `GET /api/v1/budget/requests` - List budget requests
- `GET /api/v1/budget/requests/:id` - Get budget request
- `PATCH /api/v1/budget/requests/:id/approve` - Approve request
- `PATCH /api/v1/budget/requests/:id/reject` - Reject request

**Analytics (Protocol 012):**
- `POST /api/v1/analytics/events` - Post analytics event
- `GET /api/v1/analytics/spending/total` - Total spending
- `GET /api/v1/analytics/spending/by-agent` - Spending by agent
- `GET /api/v1/analytics/spending/by-provider` - Spending by provider
- `GET /api/v1/analytics/spending/avg-per-request` - Average spending
- `GET /api/v1/analytics/budget/status` - Budget status
- `GET /api/v1/analytics/usage/requests` - Usage requests
- `GET /api/v1/analytics/usage/tokens/by-agent` - Token usage by agent
- `GET /api/v1/analytics/usage/models` - Model usage

## Example

```rust
use iron_control_api::routes;
use axum::{Router, routing::{get, post}};

// Create API router with endpoints
let app = Router::new()
  // Token management
  .route( "/api/v1/api-tokens", post( routes::tokens::create_token ) )
  .route( "/api/v1/api-tokens/validate", post( routes::tokens::validate_token ) )
  .route( "/api/v1/api-tokens", get( routes::tokens::list_tokens ) )
  // Agent management
  .route( "/api/v1/agents", post( routes::agents::create_agent ) )
  .route( "/api/v1/agents", get( routes::agents::list_agents ) )
  // Analytics
  .route( "/api/v1/analytics/spending/total", get( routes::analytics::get_spending_total ) )
  // Health check
  .route( "/api/health", get( routes::health::health_check ) );

// Start server
let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
axum::serve(listener, app).await?;
```

## Testing

**Test Coverage:** 379 tests (100% pass rate)
- Unit tests: Token validation, usage calculations, limit enforcement, agent management
- Integration tests: Full API contract validation across all endpoints (tokens, agents, analytics, auth, users)
- Corner case tests: DoS protection, NULL byte injection, concurrency, malformed JSON
- Security tests: SQL injection, XSS, command injection, path traversal
- Protocol tests: Protocol 005 budget control (26 tests), Protocol 010 agents (39 tests), Protocol 012 analytics (30 tests), Protocol 014 tokens (111 tests)

**Run tests:**
```bash
# Level 1: Unit + integration tests
w3 .test l::1

# Level 3: Tests + doc tests + clippy
w3 .test l::3
```

**Manual testing:** See `tests/manual/readme.md` for comprehensive test procedures

## Security

**Defense-in-Depth Architecture:**
- **Layer 1:** Type-safe validation at API boundary (ValidatedUserId/ValidatedProjectId newtypes)
- **Layer 2:** Database CHECK constraints for runtime enforcement
- **Layer 3:** Comprehensive test coverage (353 tests, attack scenarios included)

**Protections:**
- DoS prevention: 1-500 character limits on all string inputs (issue-001)
- NULL byte injection prevention: Validation rejects embedded null characters (issue-002)
- Database isolation: In-memory SQLite per test for deterministic execution (issue-003)
- Atomic operations: SQLite IMMEDIATE transactions prevent race conditions
- Input validation: Malformed JSON, wrong Content-Type, invalid HTTP methods rejected

## Documentation

- **Specification:** `spec.md` - Complete API specification with all 10 Functional Requirements
- **Test Documentation:** `tests/readme.md` - Test organization and coverage summary
- **Manual Tests:** `tests/manual/readme.md` - Manual testing procedures for FR-7/8/9/10

## Status

**Version:** 0.5 (2025-12-12)
**Implementation:** ✅ COMPLETE (Protocol 014 Tokens + Protocol 010 Agents + Protocol 012 Analytics + Phases 1-5 QA)
**Test Coverage:** 379 tests (100% pass rate, 0 clippy warnings, 0 regressions)
  - Protocol 014 (API Tokens): 111 tests including validate endpoint
  - Protocol 010 (Agents): 39 tests (Phase 2 complete)
  - Protocol 012 (Analytics): 30 tests (Phase 4 complete)
  - Protocol 005 (Budget): 26 tests
  - Security & corner cases: 173 tests
**Verification:** All critical deliverables complete, Phase 1-4 at 100%, Phase 5 QA ongoing

**Completed Phases:**
- Phase 1: Protocol 014 API Tokens (10/11 deliverables, 91% - CLI stub remaining)
- Phase 2: Protocol 010 Agents API Foundation (100% complete)
- Phase 4: Protocol 012 Analytics API (100% complete)

**Next Steps:** Complete Phase 3 enhancements (templates, batch ops, search/filtering), Phase 5 QA completion (performance testing, documentation)

## Docker Deployment

### Build Backend Docker Image

```bash
# From workspace root
docker build -f module/iron_control_api/Dockerfile -t iron-backend:latest .

# Check image size
docker images iron-backend:latest
# Expected: ~50-60MB compressed
```

### Run Backend Container (Standalone)

```bash
docker run -d \
  -p 3000:3000 \
  -e DATABASE_URL="postgresql://iron_user:password@postgres:5432/iron_tokens" \
  -e JWT_SECRET="your-secret-min-32-chars" \
  -e IRON_DEPLOYMENT_MODE="production" \
  -e RUST_LOG=info \
  --name iron-backend \
  iron-backend:latest
```

### Run with Docker Compose

See `../../docker-compose.yml` for full stack deployment (PostgreSQL + Backend + Frontend).

```bash
# Start all services
docker compose up -d

# View backend logs
docker compose logs -f backend

# Check backend status
docker compose ps backend
```

### Health Check

```bash
curl http://localhost:3000/api/health

# Expected response:
# {"status":"healthy","database":"connected"}
```

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL or SQLite connection string |
| `JWT_SECRET` | Yes | `dev-secret-change-in-production` | JWT signing key (32+ chars) |
| `IRON_DEPLOYMENT_MODE` | No | auto-detect | `production`, `development`, or `pilot` |
| `RUST_LOG` | No | `info` | Log level (`error`, `warn`, `info`, `debug`, `trace`) |
| `IRON_SECRETS_MASTER_KEY` | No | - | AES-256-GCM key for provider key encryption (base64, 32 bytes) |

### Troubleshooting

**Issue:** Container exits immediately
**Solution:** Check `DATABASE_URL` is valid and database is accessible. Run `docker logs iron-backend` to see startup errors.

**Issue:** Health check fails
**Solution:** Ensure database migrations have run. Check database connectivity with `docker exec iron-backend curl localhost:3000/api/health`.

**Issue:** Permission denied errors
**Solution:** Container runs as non-root user (UID 1000). Ensure any mounted volumes have correct permissions.

### Multi-Stage Build Details

The Dockerfile uses multi-stage builds for security and efficiency:

**Stage 1 (Builder):** `rust:1.75-slim-bookworm`
- Installs build dependencies (pkg-config, libssl-dev)
- Copies workspace and builds release binary
- Output: `/app/target/release/iron_control_api_server`

**Stage 2 (Runtime):** `debian:bookworm-slim`
- Installs only runtime dependencies (ca-certificates, libssl3, curl)
- Copies binary from builder stage
- Creates non-root user (UID 1000)
- Final image: ~50MB (vs ~2GB with build tools)

### Related Documentation

- [Docker Compose Architecture](../../docs/deployment/006_docker_compose_deployment.md) - Design details
- [Getting Started Guide](../../docs/getting_started.md) § Deploy Control Panel - Quickstart
- [Deployment Guide](../../docs/deployment_guide.md) - Production procedures

## Directory Structure

### Source Files

| File | Responsibility |
|------|----------------|
| lib.rs | REST API and WebSocket server for Iron Runtime dashboard. |
| error.rs | Custom error types and JSON error responses for API |
| ic_token.rs | ic token claims implementation |
| ip_token.rs | IP Token (Iron Provider Token) encryption |
| jwt_auth.rs | JWT authentication middleware |
| rbac.rs | RBAC (Role-Based Access Control) module |
| token_auth.rs | API Token authentication middleware |
| user_auth.rs | User authentication and password verification |
| bin/ | REST API server binary for Iron Control API |
| middleware/ | Middleware modules for Iron Control API |
| routes/ | REST API route handlers |

**Notes:**
- Entries marked 'TBD' require manual documentation
- Entries marked '⚠️ ANTI-PATTERN' should be renamed to specific responsibilities

