# iron_control_api - Specification

**Module:** iron_control_api
**Layer:** 5 (Integration)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

REST API and WebSocket server for Iron Cage Control Panel. Provides HTTP endpoints for token management, usage tracking, and limits. Coordinates real-time dashboard updates via WebSocket.

---

## Scope

**In Scope:**
- REST API endpoints (tokens, usage, limits, traces, auth handshake, user management)
- Budget control endpoints (Protocol 005: handshake, usage reporting, budget refresh)
- Analytics endpoints (Protocol 012: event ingestion, spending/usage queries)
- Agent token enforcement (blocking unauthorized credential access)
- WebSocket server for dashboard real-time updates
- Authentication and authorization (IC Token validation, JWT)
- Request routing and validation
- RBAC enforcement (role-based access control)
- Integration with iron_token_manager, iron_runtime_state

**Out of Scope:**
- Dashboard UI components (see iron_dashboard)
- Token generation logic (see iron_token_manager)
- Budget calculation (see iron_cost)
- Agent execution (see iron_runtime)
- Database schema (see iron_control_schema)

---

## Dependencies

**Required Modules:**
- iron_token_manager - Token authentication
- iron_runtime_state - State persistence
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
- **Budget Control Router (Protocol 005):** Manages budget handshake, usage reporting, budget refresh
- **Analytics Router (Protocol 012):** Event ingestion and spending/usage queries
- **Agent Token Enforcement:** Blocks agent tokens from unauthorized credential endpoints
- **WebSocket Server:** Broadcasts real-time agent events to dashboard
- **Auth Middleware:** Validates IC Tokens and JWT, enforces authorization
- **Request Handler:** Routes requests to appropriate modules

---

## API Contract

### Budget Control Endpoints (Protocol 005)

Protocol 005 provides budget-controlled LLM access for agents through a three-step workflow: handshake, usage reporting, and budget refresh.

#### POST /api/budget/handshake

Exchange IC Token for IP Token (encrypted provider API key).

**Request:**
```json
{
  "ic_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "provider": "openai",
  "provider_key_id": 123
}
```

**Request Fields:**
- `ic_token` (string, required): JWT containing agent_id, budget_id, permissions. HMAC-SHA256 signed.
- `provider` (string, required): Provider type: "openai" | "anthropic" | "google"
- `provider_key_id` (integer, optional): Specific provider key ID. If omitted, uses first available key for provider.

**Response (200 OK):**
```json
{
  "ip_token": "ip_v1:AQIDBAUGBwgJ...",
  "lease_id": "lease_abc123-def4-5678-90ab-cdef12345678",
  "budget_granted": 10000000,
  "budget_remaining": 0,
  "expires_at": 1735689600000
}
```

**Response Fields:**
- `ip_token` (string): AES-256-GCM encrypted provider API key. Format: `ip_v1:<base64_ciphertext>`
- `lease_id` (string): Budget lease identifier. Format: `lease_<uuid>`
- `budget_granted` (integer): Microdollars allocated for this session (1 USD = 1,000,000 microdollars)
- `budget_remaining` (integer): Remaining agent budget in microdollars after lease allocation
- `expires_at` (integer, nullable): Expiration timestamp in milliseconds since epoch. NULL = no expiration.

**Error Responses:**
- `400 Bad Request`: Invalid IC Token, missing required fields, or malformed request
- `403 Forbidden`: Insufficient budget, expired IC Token, or unauthorized agent
- `404 Not Found`: Provider key not found
- `500 Internal Server Error`: Database error, encryption failure, or server malfunction

**Side Effects:**
- Creates budget lease in database (budget_leases table)
- Allocates budget from agent's total budget (agent_budgets table)
- Encrypts provider API key into IP Token (memory-only, not persisted)

---

#### POST /api/budget/report

Report LLM usage for a budget lease.

**Request:**
```json
{
  "lease_id": "lease_abc123-def4-5678-90ab-cdef12345678",
  "request_id": "req_abc123",
  "tokens": 10000,
  "cost_microdollars": 2500000,
  "model": "gpt-4",
  "provider": "openai"
}
```

**Request Fields:**
- `lease_id` (string, required): Budget lease identifier from handshake response
- `request_id` (string, required): Unique request identifier
- `tokens` (integer, required): Total tokens consumed (prompt + completion). Must be > 0.
- `cost_microdollars` (integer, required): Cost in microdollars (1 USD = 1,000,000). Must be >= 0.
- `model` (string, required): Model identifier used for this request
- `provider` (string, required): Provider name (e.g., "openai", "anthropic")

**Response (200 OK):**
```json
{
  "success": true,
  "budget_remaining": 7500000
}
```

**Response Fields:**
- `success` (boolean): Whether usage was recorded successfully
- `budget_remaining` (integer): Remaining agent budget in microdollars

**Error Responses:**
- `400 Bad Request`: Invalid lease_id format, missing fields, or validation errors
- `403 Forbidden`: Lease expired, revoked, or insufficient budget
- `404 Not Found`: Lease not found in database
- `500 Internal Server Error`: Database error during usage recording

**Side Effects:**
- Updates budget_spent in budget_leases table
- Updates total_spent and budget_remaining in agent_budgets table
- Maintains budget invariant: total_allocated = total_spent + budget_remaining

---

#### POST /api/budget/return

Return unused budget when a lease is closed.

**Request:**
```json
{
  "lease_id": "lease_abc123-def4-5678-90ab-cdef12345678",
  "spent_microdollars": 2500000
}
```

**Request Fields:**
- `lease_id` (string, required): Budget lease identifier from handshake response
- `spent_microdollars` (integer, optional): Actual amount spent in microdollars. Default: 0.

**Response (200 OK):**
```json
{
  "success": true,
  "returned": 7500000
}
```

**Response Fields:**
- `success` (boolean): Whether budget was returned successfully
- `returned` (integer): Microdollars returned to agent budget (granted - spent)

**Error Responses:**
- `400 Bad Request`: Invalid lease_id format, lease not active, or validation errors
- `404 Not Found`: Lease not found in database
- `500 Internal Server Error`: Database error during budget return

**Side Effects:**
- Closes the lease (sets status to "closed")
- Returns unused budget to agent_budgets (budget_remaining += returned)
- Credits usage_limits (current_cost_microdollars_this_month -= returned)

---

#### POST /api/budget/refresh

Request additional budget allocation for an agent.

**Request:**
```json
{
  "agent_id": 42,
  "additional_budget": 20000000,
  "reason": "Extended task execution"
}
```

**Request Fields:**
- `agent_id` (integer, required): Agent database ID
- `additional_budget` (integer, required): Additional microdollars to allocate. Must be > 0.
- `reason` (string, optional): Human-readable justification for budget increase

**Response (200 OK):**
```json
{
  "total_allocated": 30000000,
  "budget_remaining": 27500000,
  "updated_at": 1735689700000
}
```

**Response Fields:**
- `total_allocated` (integer): New total allocated budget in microdollars (old + additional)
- `budget_remaining` (integer): Updated remaining budget in microdollars
- `updated_at` (integer): Timestamp of budget update in milliseconds since epoch

**Error Responses:**
- `400 Bad Request`: Invalid agent_id or negative additional_budget
- `404 Not Found`: Agent budget not found (agent_id doesn't exist)
- `500 Internal Server Error`: Database error during budget update

**Side Effects:**
- Updates total_allocated and budget_remaining in agent_budgets table
- Updates updated_at timestamp to current time

---

### Budget Request Workflow Endpoints (Protocol 012)

Protocol 012 provides a request-approval workflow for budget modifications, enabling agents to request budget increases through an approval process.

#### POST /api/v1/budget/requests

Create a new budget change request.

**Request:**
```json
{
  "agent_id": 1,
  "requester_id": "user-123",
  "requested_budget_micros": 250000000,
  "justification": "Need increased budget for expanded testing and model experimentation"
}
```

**Request Fields:**
- `agent_id` (integer, required): Agent database ID
- `requester_id` (string, required): ID of user creating the request
- `requested_budget_micros` (integer, required): Requested budget amount in microdollars (1 USD = 1,000,000). Must be > 0.
- `justification` (string, required): Reason for budget request. Length: 20-500 characters.

**Response (201 Created):**
```json
{
  "request_id": "breq_550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "created_at": 1735689600000
}
```

**Response Fields:**
- `request_id` (string): Unique identifier for this request
- `status` (string): Request status, always "pending" on creation
- `created_at` (integer): Timestamp in milliseconds since epoch

**Error Responses:**
- `400 Bad Request`: Invalid parameters, justification too short/long, or negative budget
- `404 Not Found`: Agent doesn't exist in database
- `500 Internal Server Error`: Database error during request creation

---

#### GET /api/v1/budget/requests/:id

Get a budget change request by ID.

**Path Parameters:**
- `id` (string): Budget request ID (e.g., "breq_550e8400-e29b-41d4-a716-446655440000")

**Response (200 OK):**
```json
{
  "id": "breq_550e8400-e29b-41d4-a716-446655440000",
  "agent_id": 1,
  "requester_id": "user-123",
  "current_budget_micros": 100000000,
  "requested_budget_micros": 250000000,
  "justification": "Need increased budget for expanded testing",
  "status": "pending",
  "created_at": 1735689600000,
  "updated_at": 1735689600000
}
```

**Response Fields:**
- `id` (string): Request identifier
- `agent_id` (integer): Agent database ID
- `requester_id` (string): User who created the request
- `current_budget_micros` (integer): Budget in microdollars at time of request creation
- `requested_budget_micros` (integer): Requested budget amount in microdollars
- `justification` (string): Request justification
- `status` (string): Current status: "pending" | "approved" | "rejected" | "cancelled"
- `created_at` (integer): Creation timestamp
- `updated_at` (integer): Last update timestamp

**Error Responses:**
- `404 Not Found`: Request ID doesn't exist
- `500 Internal Server Error`: Database error

---

#### GET /api/v1/budget/requests

List budget change requests with optional filtering.

**Query Parameters:**
- `agent_id` (integer, optional): Filter by agent ID
- `status` (string, optional): Filter by status ("pending" | "approved" | "rejected" | "cancelled")

**Response (200 OK):**
```json
{
  "requests": [
    {
      "id": "breq_550e8400-e29b-41d4-a716-446655440000",
      "agent_id": 1,
      "requester_id": "user-123",
      "current_budget_micros": 100000000,
      "requested_budget_micros": 250000000,
      "justification": "Need increased budget for expanded testing",
      "status": "pending",
      "created_at": 1735689600000,
      "updated_at": 1735689600000
    }
  ]
}
```

**Response Fields:**
- `requests` (array): Array of budget request objects (empty array if no matches)

**Error Responses:**
- `400 Bad Request`: Invalid status parameter
- `500 Internal Server Error`: Database error

---

#### PATCH /api/v1/budget/requests/:id/approve

Approve a budget change request and apply the budget change.

**Path Parameters:**
- `id` (string): Budget request ID

**Side Effects:**
- Updates request status to "approved"
- Updates agent budget to requested amount
- Records change in budget_modification_history
- All operations are atomic (uses database transaction)

**Response (200 OK):**
```json
{
  "request_id": "breq_550e8400-e29b-41d4-a716-446655440000",
  "status": "approved",
  "updated_at": 1735689700000
}
```

**Response Fields:**
- `request_id` (string): Request identifier
- `status` (string): New status, always "approved"
- `updated_at` (integer): Approval timestamp

**Error Responses:**
- `404 Not Found`: Request ID doesn't exist
- `409 Conflict`: Request is not in pending status (already approved/rejected/cancelled)
- `500 Internal Server Error`: Database error or transaction failure

---

#### PATCH /api/v1/budget/requests/:id/reject

Reject a budget change request.

**Path Parameters:**
- `id` (string): Budget request ID

**Side Effects:**
- Updates request status to "rejected"
- Does NOT modify agent budget

**Response (200 OK):**
```json
{
  "request_id": "breq_550e8400-e29b-41d4-a716-446655440000",
  "status": "rejected",
  "updated_at": 1735689700000
}
```

**Response Fields:**
- `request_id` (string): Request identifier
- `status` (string): New status, always "rejected"
- `updated_at` (integer): Rejection timestamp

**Error Responses:**
- `404 Not Found`: Request ID doesn't exist
- `409 Conflict`: Request is not in pending status
- `500 Internal Server Error`: Database error

---

### Analytics API (Protocol 012)

Protocol 012 provides analytics for LLM usage tracking and dashboard display.

#### POST /api/v1/analytics/events

Report LLM request events from LlmRouter.

**Request:**
```json
{
  "ic_token": "<ic_token>",
  "event_id": "evt_7c9e6679-7425-40de-944b",
  "timestamp_ms": 1733830245123,
  "event_type": "llm_request_completed",
  "model": "gpt-4o-mini",
  "provider": "openai",
  "input_tokens": 150,
  "output_tokens": 50,
  "cost_micros": 1250,
  "provider_id": "ip_openai-001"
}
```

**Request Fields:**
- `ic_token` (string, required): IC Token for authentication - agent_id derived from token claims
- `event_id` (string, required): UUID for deduplication, unique per agent
- `timestamp_ms` (integer, required): Unix timestamp in milliseconds
- `event_type` (string, required): `llm_request_completed` or `llm_request_failed`
- `model` (string, required): Model name (e.g., `gpt-4o-mini`)
- `provider` (string, required): Provider: `openai`, `anthropic`, `unknown`
- `input_tokens` (integer, required*): Input token count (*required for completed)
- `output_tokens` (integer, required*): Output token count (*required for completed)
- `cost_micros` (integer, required*): Cost in microdollars (1 USD = 1,000,000)
- `provider_id` (string, optional): Provider key identifier
- `error_code` (string, optional): Error code (for failed events)
- `error_message` (string, optional): Error message (for failed events)

**Note:** `agent_id` is automatically extracted from IC Token claims (format: `agent_<id>`).

**Response (202 Accepted):**
```json
{
  "event_id": "evt_7c9e6679-7425-40de-944b",
  "status": "accepted"
}
```

**Error Responses:**
- `400 Bad Request`: Invalid event data or validation error
- `401 Unauthorized`: Invalid or expired IC Token

---

#### GET Endpoints (JWT Protected)

All GET endpoints require JWT authentication via `Authorization: Bearer <jwt_token>` header.

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/analytics/spending/total` | Total spending across all agents |
| `GET /api/v1/analytics/spending/by-agent` | Spending breakdown by agent |
| `GET /api/v1/analytics/spending/by-provider` | Spending breakdown by provider |
| `GET /api/v1/analytics/spending/avg-per-request` | Average cost per request |
| `GET /api/v1/analytics/budget/status` | Budget status for agents |
| `GET /api/v1/analytics/usage/requests` | Request count statistics |
| `GET /api/v1/analytics/usage/tokens/by-agent` | Token usage by agent |
| `GET /api/v1/analytics/usage/models` | Model usage statistics |

**Common Query Parameters:**
- `period` (string): Time range: `today`, `yesterday`, `last-7-days`, `last-30-days`, `all-time`
- `agent_id` (integer): Filter by specific agent
- `provider_id` (string): Filter by specific provider
- `page` (integer): Page number (default: 1)
- `per_page` (integer): Results per page (default: 50, max: 100)

---

### Agent Token Enforcement

**Enforcement Rule:** Agent tokens (api_tokens table rows where agent_id IS NOT NULL) CANNOT access credential endpoints that bypass Protocol 005.

**Affected Endpoints:**
- `GET /api/keys` - Blocked for agent tokens (returns 403 Forbidden)

**Enforcement Response (403 Forbidden):**
```json
{
  "error": "Agent tokens cannot use this endpoint",
  "details": "Agent credentials must be obtained through Protocol 005 (Budget Control). Use POST /api/budget/handshake with your IC Token.",
  "protocol": "005"
}
```

**Implementation:**
- Middleware checks api_tokens.agent_id for all credential requests
- If agent_id IS NOT NULL, request is rejected before reaching handler
- Ensures Protocol 005 is the EXCLUSIVE path for agent credential access

---

## Integration Points

**Used by:**
- iron_dashboard - Vue app consumes REST API and WebSocket
- iron_runtime - Sends telemetry and state updates
- Developers - CLI and SDK interact with API

**Uses:**
- iron_token_manager - Token authentication and management
- iron_runtime_state - Persists and retrieves state data

---

*For detailed API specification, see spec/-archived_detailed_spec.md*
*For REST protocol, see docs/protocol/002_rest_api_protocol.md*
*For WebSocket protocol, see docs/protocol/003_websocket_protocol.md*
*For Budget Control Protocol (Protocol 005), see docs/protocol/005_budget_control_protocol.md*
*For user management API, see docs/protocol/008_user_management_api.md*
*For Analytics API (Protocol 012), see docs/protocol/012_analytics_api.md*
