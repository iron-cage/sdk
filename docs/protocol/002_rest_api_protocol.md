# Protocol: REST API Protocol

### Scope

HTTP REST API endpoint schemas for all Control Panel operations organized by resource type.

**In Scope:**
- **Entity Resources:** IC Token CRUD, Project management, IP management
- **Operation Resources:** Authentication (login/logout), Budget protocol (handshake/report/refresh)
- **Analytics Resources:** Analytics (usage metrics, spending analysis, performance data)
- **Configuration Resources:** Budget limits, system settings
- **System Resources:** Health check, API version
- HTTP status codes and error responses
- Request/response JSON schemas
- Authentication types: IC Token (agents), User Token (users), None (public endpoints)
- Common patterns: Pagination, filtering, sorting, search
- API versioning strategy

**Out of Scope:**
- WebSocket protocol (see [003_websocket_protocol.md](003_websocket_protocol.md))
- IronLang data protocol (archived, not in use for Control Panel API)
- Implementation details (see `module/iron_control_api/spec.md`)
- Individual resource-specific protocols (see [006_token_management_api.md](006_token_management_api.md), [007_authentication_api.md](007_authentication_api.md))

---

### Purpose

Provide comprehensive HTTP REST API for all Control Panel operations including entity management, agent budget protocol, authentication, analytics, and system configuration.

**Problem:**

Control Panel requires programmatic access for:
- **Agent Operations:** Budget protocol (handshake/report/refresh) using IC Token
- **User Operations:** IC Token management, authentication (login/logout) using User Token
- **Admin Operations:** Project/IP management, budget limits configuration using User Token (admin role)
- **Analytics:** Usage metrics, spending analysis, performance monitoring using User Token
- **Integration:** CLI tool access, external tool integration (CI/CD, admin scripts)
- **Discovery:** System health, API version (no authentication)

**Solution:**

RESTful HTTP API organized by resource type:
- **Entity Resources:** Standard CRUD for IC Tokens, Projects, IPs (User Token auth)
- **Operation Resources:** Budget protocol (IC Token auth), authentication (User Token auth)
- **Analytics Resources:** Usage, spending, metrics (User Token auth)
- **Configuration Resources:** Limits, settings (User Token auth, admin-only)
- **System Resources:** Health, version (no auth)
- Standard HTTP semantics (GET, POST, PUT, DELETE)
- JSON request/response bodies
- Consistent error responses
- CLI-API parity for all user-facing resources

**Resource Organization:** See [Resource Organization](#resource-organization) section below for complete taxonomy.

---

### Resource Organization

**Four Resource Categories:**

| Category | Definition | Auth Type | Examples | Certainty |
|----------|-----------|-----------|----------|-----------|
| **Entity Resources** | CRUD operations on domain entities | User Token | `/api/tokens`, `/api/projects`, `/api/providers` | Tokens: ✅ Certain, Projects/IPs: ⚠️ Uncertain |
| **Operation Resources** | RPC-style operations spanning entities | IC Token or User Token | `/api/auth`, `/api/budget/*` | ✅ Certain (required for Pilot) |
| **Analytics Resources** | Read-only derived/aggregated data | User Token | `/api/usage`, `/api/spending`, `/api/metrics` | ⚠️ Uncertain (not Pilot-critical) |
| **Configuration Resources** | System-level configuration | User Token (Admin) | `/api/limits`, `/api/settings` | ⚠️ Uncertain (admin tooling) |
| **System Resources** | Health/version endpoints | None | `/api/health`, `/api/version` | ✅ Certain (standard endpoints) |

**Certainty Classification:**
- ✅ **Certain:** Required for Pilot, design complete, documented in permanent protocol docs
- ⚠️ **Uncertain:** Not Pilot-critical or design pending, specifications deferred

**Complete Resource Inventory:** See [architecture/009_resource_catalog.md](../architecture/009_resource_catalog.md)

**Resource-Specific Protocols (Certain Resources Only):**
- [006_token_management_api.md](006_token_management_api.md) - IC Token CRUD (✅ Certain)
- [007_authentication_api.md](007_authentication_api.md) - User authentication (✅ Certain)

**Note:** Uncertain resources (projects, providers, analytics, API tokens, limits, settings) are documented separately and not included in permanent protocol index.

---

### Complete Protocol Reference

| Resource Category | Resources | Protocol Document | Status | Pilot |
|-------------------|-----------|-------------------|--------|-------|
| **Entity Resources** | | | | |
| IC Tokens | `/api/tokens/*` | [006_token_management_api.md](006_token_management_api.md) | ✅ Certain | Yes |
| Projects | `/api/projects/*` | (deferred) | ⚠️ Uncertain | No |
| Providers | `/api/providers/*` | (deferred) | ⚠️ Uncertain | No |
| **Operation Resources** | | | | |
| Authentication | `/api/auth/*` | [007_authentication_api.md](007_authentication_api.md) | ✅ Certain | Yes |
| Budget Protocol | `/api/budget/*` | [005_budget_control_protocol.md](005_budget_control_protocol.md) | ✅ Certain | Yes |
| API Tokens | `/api/api-tokens/*` | (deferred) | ⚠️ Uncertain | TBD |
| **Analytics Resources** | | | | |
| Analytics | `/api/analytics/*` | (deferred) | ⚠️ Uncertain | No |
| **Configuration Resources** | | | | |
| Budget Limits | `/api/limits/*` | (deferred) | ⚠️ Uncertain | No |
| System Settings | `/api/settings/*` | (deferred) | ⚠️ Uncertain | No |
| **System Resources** | | | | |
| Health & Version | `/api/health`, `/api/version` | [002_rest_api_protocol.md](#system-resources) | ✅ Certain | Yes |

**Legend:**
- ✅ **Certain:** Required for Pilot, specification complete
- ⚠️ **Uncertain:** Post-Pilot or design pending, specifications not yet finalized

---

### Authentication Architecture

**Three Authentication Types:**

#### 1. IC Token (Agent Authentication)

**Format:** JWT with `ic_` prefix

**Characteristics:**
- 1:1 relationship with agent (one agent = one IC Token)
- Lifetime: Until agent deleted (long-lived, no auto-expiration)
- Regeneration: Developer can regenerate own, admin can regenerate any
- Used by: iron_runtime for agent operations

**Resources:**
- `POST /api/budget/handshake` - Negotiate budget
- `POST /api/budget/report` - Report usage
- `POST /api/budget/refresh` - Refresh budget

**Header Format:**
```http
Authorization: Bearer ic_abc123def456...
```

#### 2. User Token (Control Panel Access)

**Format:** JWT

**Characteristics:**
- Multiple tokens per user allowed
- Lifetime: 30 days (configurable, refreshable)
- Scope: User + accessible projects
- Used by: iron_cli, web dashboard

**Resources:**
- All entity resources (`/api/tokens`, `/api/projects`, `/api/providers`)
- All analytics resources (`/api/usage`, `/api/spending`, `/api/metrics`)
- All configuration resources (`/api/limits`, `/api/settings`)
- Authentication operations (`POST /api/auth/login`, `POST /api/auth/refresh`)

**Header Format:**
```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### 3. No Authentication (Public Endpoints)

**Resources:**
- `GET /api/health` - Health check
- `GET /api/version` - API version

**No Header Required**

**Authentication Protocol:** See [005_budget_control_protocol.md](005_budget_control_protocol.md) for IC Token handshake details.

---

### Common Patterns

**Pagination (List Endpoints):**

```http
GET /api/tokens?page=1&per_page=50

Response:
{
  "data": [...],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 250,
    "total_pages": 5
  }
}
```

**Filtering:**

```http
GET /api/tokens?project_id=proj-123&status=active
```

**Sorting:**

```http
GET /api/tokens?sort_by=created_at&order=desc
```

**Search:**

```http
GET /api/tokens?search=agent-name
```

**Field Selection (Future):**

```http
GET /api/tokens?fields=id,name,status
```

---

### Example Data Standards

**Use these standard values in all protocol documentation examples for consistency:**

**IDs:**
- Agent ID: `agent-abc123`
- IC Token ID: `tok-def456`
- Project ID: `proj-ghi789`
- Provider ID: `ip-openai-001`, `ip-anthropic-001`
- User ID: `user-jkl012`
- User Token ID: `ut-mno345`
- API Token ID: `at-pqr678`

**Tokens:**
- IC Token: `ic_abc123def456ghi789...`
- User Token: `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`
- API Token: `apitok_abc123def456ghi789...`

**Names:**
- Agent: `Production Agent 1`, `Development Agent 2`
- Project: `My Project`, `Production Project`
- User: `john.doe@example.com`, `admin@example.com`

**Timestamps:**
- Created: `2025-12-01T09:00:00Z`
- Updated: `2025-12-09T14:00:00Z`
- Last Used: `2025-12-09T12:30:00Z`

**Costs:**
- Budget Allocated: `$100.00`
- Budget Spent: `$42.35`
- Budget Remaining: `$57.65`
- Budget Portion: `$10.00`

**Request IDs:**
- Request ID: `req-xyz789`

**Purpose:** Consistent examples help readers recognize patterns across documentation and make cross-references clearer.

---

### System Resources

System resources provide operational endpoints for health checks and API discovery. These endpoints require no authentication and are publicly accessible.

#### Health Check

```http
GET /api/health

Response: 200 OK
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": "2025-12-09T14:00:00Z",
  "services": {
    "database": "healthy",
    "redis": "healthy",
    "vault": "healthy"
  },
  "uptime_seconds": 86400
}

Response: 503 Service Unavailable
{
  "status": "unhealthy",
  "version": "1.0.0",
  "timestamp": "2025-12-09T14:00:00Z",
  "services": {
    "database": "healthy",
    "redis": "unhealthy",
    "vault": "healthy"
  },
  "errors": [
    {
      "service": "redis",
      "message": "Connection timeout"
    }
  ]
}
```

**Purpose:**
- Load balancer health checks
- Monitoring system integration
- Operational readiness verification

**Response Codes:**
- `200 OK`: All services healthy
- `503 Service Unavailable`: One or more services unhealthy

#### API Version

```http
GET /api/version

Response: 200 OK
{
  "current_version": "v1",
  "supported_versions": ["v1"],
  "deprecated_versions": [],
  "latest_endpoint": "/api/v1",
  "build": {
    "commit": "76dfb54",
    "timestamp": "2025-12-09T12:00:00Z",
    "environment": "production"
  }
}
```

**Purpose:**
- API version discovery
- Client compatibility checks
- Build information for debugging

**Response Codes:**
- `200 OK`: Always succeeds

---

### Error Response Format

**Standard Error Structure:**

```json
{
  "error": {
    "code": "BUDGET_EXHAUSTED",
    "message": "Agent budget exhausted (allocated: $100, spent: $100)",
    "details": {
      "agent_id": "agent-abc123",
      "budget_allocated": 100.00,
      "budget_spent": 100.00,
      "budget_remaining": 0.00
    },
    "timestamp": "2025-12-09T09:00:00Z",
    "request_id": "req-xyz789"
  }
}
```

**Error Code Categories:**

| Code Prefix | Category | Examples |
|-------------|----------|----------|
| `AUTH_*` | Authentication errors | `AUTH_INVALID_TOKEN`, `AUTH_TOKEN_EXPIRED` |
| `BUDGET_*` | Budget errors | `BUDGET_EXHAUSTED`, `BUDGET_INSUFFICIENT` |
| `RESOURCE_*` | Resource errors | `RESOURCE_NOT_FOUND`, `RESOURCE_CONFLICT` |
| `VALIDATION_*` | Validation errors | `VALIDATION_REQUIRED_FIELD`, `VALIDATION_INVALID_FORMAT` |
| `PERMISSION_*` | Authorization errors | `PERMISSION_DENIED`, `PERMISSION_INSUFFICIENT_ROLE` |
| `RATE_LIMIT_*` | Rate limiting | `RATE_LIMIT_EXCEEDED` |
| `SERVER_*` | Server errors | `SERVER_INTERNAL_ERROR`, `SERVER_SERVICE_UNAVAILABLE` |

---

### Rate Limiting

**Standard Rate Limits:**

| Endpoint Category | Limit | Window | Scope |
|-------------------|-------|--------|-------|
| Authentication | 5 attempts | 5 minutes | Per IP address |
| Token Create/Delete | 10 operations | 1 hour | Per user |
| Token Rotate | 5 operations | 1 hour | Per token |
| List/Get Operations | 100 requests | 1 minute | Per user |
| Analytics Queries | 20 requests | 1 minute | Per user |
| Settings Updates | 30 operations | 1 hour | Per user (admin) |

**Rate Limit Response:**

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 300
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1733754300

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded (max 100 req/min)",
    "details": {
      "limit": 100,
      "window_seconds": 60,
      "retry_after_seconds": 300,
      "reset_at": "2025-12-09T14:05:00Z"
    },
    "timestamp": "2025-12-09T14:00:00Z",
    "request_id": "req-xyz789"
  }
}
```

**Headers:**
- `X-RateLimit-Limit`: Maximum requests in window
- `X-RateLimit-Remaining`: Requests remaining in current window
- `X-RateLimit-Reset`: Unix timestamp when limit resets

**Implementation:**
- Token bucket algorithm with distributed tracking (Redis)
- Per-user tracking for authenticated endpoints
- Per-IP tracking for public endpoints (health, version, auth)

---

### Budget Protocol Summary

Budget protocol endpoints enable agent runtime to negotiate and report LLM usage. These are agent-facing endpoints (not exposed via CLI).

**Endpoints:**

| Endpoint | Purpose | Request | Response |
|----------|---------|---------|----------|
| `POST /api/v1/budget/handshake` | Negotiate budget and get IP Token | `{requested_budget: float}` | `{ip_token, budget_granted, lease_id}` |
| `POST /api/v1/budget/report` | Report usage (tokens, cost) | `{lease_id, tokens, cost_usd}` | 204 No Content |
| `POST /api/v1/budget/refresh` | Refresh budget during execution | `{lease_id, requested_budget}` | `{budget_granted, lease_id}` |

**Authentication:** IC Token (agent authentication)

**Complete Specification:** See [protocol/005: Budget Control Protocol](005_budget_control_protocol.md) for:
- Full request/response schemas
- Error responses and retry logic
- Implementation variants (per-request vs batched reporting)
- Sequence diagrams and state transitions
- Security considerations

---

### HTTP Status Codes

**Success Codes:**

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | OK | Successful GET, POST (with response body), PUT |
| 201 | Created | Successful POST (resource created) |
| 204 | No Content | Successful DELETE, POST (no response body) |

**Client Error Codes:**

| Code | Meaning | Usage |
|------|---------|-------|
| 400 | Bad Request | Invalid request body, malformed JSON |
| 401 | Unauthorized | Missing or invalid authentication token |
| 403 | Forbidden | Valid token but insufficient permissions, budget exhausted |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Resource already exists, state conflict |
| 422 | Unprocessable Entity | Validation errors |
| 429 | Too Many Requests | Rate limit exceeded |

**Server Error Codes:**

| Code | Meaning | Usage |
|------|---------|-------|
| 500 | Internal Server Error | Unexpected server error |
| 503 | Service Unavailable | Service temporarily unavailable |

**Example Response Bodies:**

```json
// 200 OK (GET /api/tokens/tok-123)
{
  "id": "tok-123",
  "agent_id": "agent-abc",
  "status": "active",
  "created_at": "2025-12-09T09:00:00Z"
}

// 201 Created (POST /api/tokens)
{
  "id": "tok-456",
  "token": "ic_abc123...",
  "agent_id": "agent-xyz",
  "created_at": "2025-12-09T10:00:00Z"
}

// 204 No Content (DELETE /api/tokens/tok-123)
(empty response body)

// 400 Bad Request
{
  "error": {
    "code": "VALIDATION_REQUIRED_FIELD",
    "message": "Required field 'agent_id' missing",
    "details": {"field": "agent_id"}
  }
}

// 401 Unauthorized
{
  "error": {
    "code": "AUTH_INVALID_TOKEN",
    "message": "Invalid or expired authentication token"
  }
}

// 403 Forbidden (Budget)
{
  "error": {
    "code": "BUDGET_EXHAUSTED",
    "message": "Agent budget exhausted",
    "details": {
      "agent_id": "agent-abc",
      "budget_allocated": 100.00,
      "budget_remaining": 0.00
    }
  }
}

// 404 Not Found
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "IC Token not found",
    "details": {"token_id": "tok-nonexistent"}
  }
}

// 409 Conflict
{
  "error": {
    "code": "RESOURCE_CONFLICT",
    "message": "IC Token already exists for agent",
    "details": {"agent_id": "agent-abc", "existing_token_id": "tok-123"}
  }
}

// 429 Too Many Requests
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded (max 100 req/min)",
    "details": {
      "limit": 100,
      "window": "60s",
      "retry_after": 30
    }
  }
}
```

---

### API Versioning

**Current Version:** v1

**Versioning Strategy:**
- URL-based versioning: `/api/v1/`, `/api/v2/`
- Version in URL path (not header)
- Each version independently documented

**Breaking Changes (Require New Version):**
- Removing endpoints or fields
- Changing field types
- Changing endpoint semantics
- Removing query parameters

**Non-Breaking Changes (Same Version):**
- Adding new endpoints
- Adding optional fields
- Adding optional query parameters
- Expanding enum values (with graceful degradation)

**Version Lifecycle:**
- Latest version: Always available
- Previous version: Supported for 6 months after new version release
- Deprecated version: 3-month sunset notice before removal

**Version Discovery:**
```http
GET /api/version

Response:
{
  "current_version": "v1",
  "supported_versions": ["v1"],
  "deprecated_versions": [],
  "latest_endpoint": "/api/v1"
}
```

---

### CLI-API Parity

**Principle:** Every user-facing API resource has corresponding CLI command.

**Exceptions:**
- Agent-facing resources (budget protocol) - No CLI (used by iron_runtime)
- System resources (health, version) - No CLI (operational endpoints)

**Mapping Pattern:**

| API Pattern | HTTP Method | CLI Pattern | Example |
|-------------|-------------|-------------|---------|
| `GET /api/{resource}` | GET (list) | `iron {resource} list` | `GET /api/tokens` → `iron tokens list` |
| `GET /api/{resource}/{id}` | GET (get) | `iron {resource} get <id>` | `GET /api/tokens/tok-123` → `iron tokens get tok-123` |
| `POST /api/{resource}` | POST | `iron {resource} create` | `POST /api/tokens` → `iron tokens create` |
| `PUT /api/{resource}/{id}` | PUT | `iron {resource} update <id>` | `PUT /api/tokens/tok-123` → `iron tokens update tok-123` |
| `DELETE /api/{resource}/{id}` | DELETE | `iron {resource} delete <id>` | `DELETE /api/tokens/tok-123` → `iron tokens delete tok-123` |
| `POST /api/{resource}/{id}/{action}` | POST | `iron {resource} {action} <id>` | `POST /api/tokens/tok-123/rotate` → `iron tokens rotate tok-123` |
| `POST /api/{operation}` | POST | `iron {operation}` | `POST /api/auth/login` → `iron login` |

**Entity Resources:**

| API Endpoint | CLI Command | Auth | Status |
|--------------|-------------|------|--------|
| `GET /api/tokens` | `iron tokens list` | User Token | ✅ Certain |
| `POST /api/tokens` | `iron tokens create` | User Token | ✅ Certain |
| `DELETE /api/tokens/{id}` | `iron tokens delete <id>` | User Token | ✅ Certain |
| `PUT /api/tokens/{id}/rotate` | `iron tokens rotate <id>` | User Token | ✅ Certain |
| `GET /api/api-tokens` | `iron api-tokens list` | User Token | ⚠️ Uncertain |
| `POST /api/api-tokens` | `iron api-tokens create` | User Token | ⚠️ Uncertain |
| `DELETE /api/api-tokens/{id}` | `iron api-tokens revoke <id>` | User Token | ⚠️ Uncertain |
| `GET /api/projects` | `iron projects list` | User Token | ⚠️ Uncertain |
| `GET /api/providers` | `iron providers list` | User Token | ⚠️ Uncertain |

**Operation Resources:**

| API Endpoint | CLI Command | Auth | Status |
|--------------|-------------|------|--------|
| `POST /api/auth/login` | `iron login` | None → User Token | ✅ Certain |
| `POST /api/auth/logout` | `iron logout` | User Token | ✅ Certain |
| `POST /api/budget/handshake` | (no CLI) | IC Token | ✅ Certain |
| `POST /api/budget/report` | (no CLI) | IC Token | ✅ Certain |
| `POST /api/budget/refresh` | (no CLI) | IC Token | ✅ Certain |

**Analytics Resources:**

| API Endpoint | CLI Command | Auth | Status |
|--------------|-------------|------|--------|
| `GET /api/analytics/usage` | `iron usage report` or `iron analytics usage` | User Token | ⚠️ Uncertain |
| `GET /api/analytics/spending` | `iron spending show` or `iron analytics spending` | User Token | ⚠️ Uncertain |
| `GET /api/analytics/metrics` | `iron metrics view` or `iron analytics metrics` | User Token | ⚠️ Uncertain |

**Parity Details:** See [features/004_token_management_cli_api_parity.md](../features/004_token_management_cli_api_parity.md) for complete 24-operation mapping.

---

### Version Terminology

**API Version (`/api/v1/`):**
- URL path segment indicating API iteration
- Breaking changes require new API version (e.g., `/api/v2/`)
- Current API version: **v1**
- Appears in all endpoint URLs

**Document Version (1.0, 1.1, 2.0):**
- Protocol documentation iteration
- Major version: Breaking documentation changes or restructuring
- Minor version: Clarifications, additions, non-breaking updates
- Current document version: **1.0**

**Independence:** API version and document version evolve independently. API v1 docs may be at document version 1.2 after clarifications and additions.

---

### Cross-References

**Resource Organization:**
- [architecture/009: Resource Catalog](../architecture/009_resource_catalog.md) - Complete resource inventory and entity mapping

**Resource-Specific Protocols (Certain Resources Only):**
- [006: Token Management API](006_token_management_api.md) - IC Token CRUD endpoints (✅ Certain)
- [007: Authentication API](007_authentication_api.md) - User login/logout endpoints (✅ Certain)

**Note:** Uncertain resources (analytics, API tokens, limits, projects, providers, settings) have deferred specifications.

**Dependencies:**
- [protocol/005: Budget Control Protocol](005_budget_control_protocol.md) - Budget handshake/report/refresh protocol
- [architecture/007: Entity Model](../architecture/007_entity_model.md) - Domain entities (Agent, IC Token, Project, IP)
- [architecture/006: Roles and Permissions](../architecture/006_roles_and_permissions.md) - Admin, Super User, Developer roles

**Used By:**
- [capabilities/002: LLM Access Control](../capabilities/002_llm_access_control.md) - Uses budget API for enforcement
- [architecture/004: Data Flow](../architecture/004_data_flow.md) - REST API in runtime initialization
- `iron_runtime` - Calls budget protocol endpoints (handshake/report/refresh)
- `iron_cli` - Calls user-facing endpoints (tokens, auth, usage)
- `iron_dashboard` - Calls user-facing endpoints (tokens, analytics, config)

**Related:**
- [003: WebSocket Protocol](003_websocket_protocol.md) - Real-time dashboard protocol
- [features/004: Token Management CLI-API Parity](../features/004_token_management_cli_api_parity.md) - Complete CLI ↔ API mapping

**Implementation:**
- Module: `module/iron_control_api/` - REST API server (Rust/axum)
- Source: `module/iron_control_api/src/routes/` - Endpoint handlers
- Tests: `module/iron_control_api/tests/` - Endpoint integration tests
- Specification: `module/iron_control_api/spec.md` - Implementation spec

---

**Last Updated:** 2025-12-09
**Document Version:** 1.0
**API Version:** v1 (`/api/v1/`)
**Status:** Overview complete, resource-specific protocols in progress
