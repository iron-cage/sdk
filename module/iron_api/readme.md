# iron_api

REST API and WebSocket endpoints for programmatic control of Iron Cage runtime and real-time dashboard communication.

### Scope

**Responsibilities:**
Provides REST API and WebSocket endpoints for programmatic control of Iron Cage runtime (agent lifecycle, monitoring, metrics) and real-time dashboard communication. Enables external tools, CI/CD pipelines, and web dashboards to start agents, monitor execution, and receive live updates via standardized HTTP/WebSocket interfaces. Supports Pilot Mode (localhost, shared iron_state) and Production Mode (distributed, PostgreSQL + Redis). Requires Rust 1.75+, all platforms supported, integrates with iron_runtime for orchestration and iron_state for persistence.

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
- State persistence - See iron_state

## Installation

```toml
[dependencies]
iron_api = { version = "0.1", features = ["enabled"] }
```

## Features

- `default = ["enabled"]`: Standard configuration
- `enabled`: Full API functionality (depends on iron_types, iron_state, iron_token_manager, iron_telemetry, iron_cost)

## Architecture

**Pilot/Demo Mode (Current):**
- Single Rust process (localhost:8080)
- Shared iron_state (DashMap + SQLite) with iron_runtime
- WebSocket for real-time dashboard updates
- No authentication (localhost-only, trusted dashboard)

**Production Mode (Post-Pilot):**
- Distributed deployment (iron_control_store + PostgreSQL, Redis pub/sub)
- Telemetry ingestion endpoints (POST /v1/telemetry/events)
- Full authentication and rate limiting

## API Endpoints

**Token Management (FR-7):**
- `POST /v1/tokens` - Create new API token
- `POST /v1/tokens/:id/rotate` - Rotate token secret
- `POST /v1/tokens/:id/revoke` - Revoke token
- `GET /v1/tokens` - List all tokens
- `GET /v1/tokens/:id` - Get token details

**Usage Analytics (FR-8):**
- `GET /v1/usage/aggregate` - Get aggregate usage metrics
- `GET /v1/usage/by-project/:id` - Get usage by project
- `GET /v1/usage/by-provider/:name` - Get usage by provider

**Budget Limits (FR-9):**
- `POST /v1/limits` - Create budget limit
- `GET /v1/limits` - List all limits
- `GET /v1/limits/:id` - Get limit details
- `PUT /v1/limits/:id` - Update limit
- `DELETE /v1/limits/:id` - Delete limit

**Request Traces (FR-10):**
- `GET /v1/traces` - List request traces
- `GET /v1/traces/:id` - Get trace details

## Example

```rust
use iron_api::routes;
use axum::Router;

// Create API router with all endpoints
let app = Router::new()
  .nest("/v1/tokens", routes::tokens::router())
  .nest("/v1/usage", routes::usage::router())
  .nest("/v1/limits", routes::limits::router())
  .nest("/v1/traces", routes::traces::router());

// Start server
let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
axum::serve(listener, app).await?;
```

## Testing

**Test Coverage:** 353 tests (100% pass rate)
- Unit tests: Token validation, usage calculations, limit enforcement
- Integration tests: Full API contract validation across all endpoints
- Corner case tests: DoS protection, NULL byte injection, concurrency, malformed JSON
- Security tests: SQL injection, XSS, command injection, path traversal

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

**Version:** 0.4 (2025-12-07)
**Implementation:** ✅ COMPLETE (FR-7/8/9/10 + Phases 1-5 Security Fixes and Corner Case Coverage)
**Test Coverage:** 353 tests (+152 from baseline 201, +76% increase)
**Verification:** All 11 fixes complete (3 Phase 1 issues + 8 implementation bugs), 0 clippy warnings, 0 regressions

**Next Steps:** Post-pilot (API authentication, rate limiting, distributed deployment, telemetry ingestion)
