# Protocol 017: Budget Change Requests API

**Status:** ✅ MUST-HAVE
**Version:** 1.0.0
**Last Updated:** 2025-12-10
**Priority:** MUST-HAVE for Pilot

---

## Overview

The Budget Change Requests API provides a workflow for developers to request budget increases for their agents, requiring admin approval. This enables governance and audit trails for budget modifications while allowing developer self-service.

**Key characteristics:**
- **Request/Approval workflow:** Developers create requests, admins approve/reject
- **Admin-only approval:** Only admins can approve budget changes
- **Required justification:** All requests must include business justification (20-500 chars)
- **Complete audit trail:** Request history + approval notes + budget modification history
- **State machine:** pending → approved/rejected/cancelled (terminal states)
- **Integration:** Approved requests automatically update agent budgets and create budget history entries

**Use cases:**
- Emergency budget top-up for agents approaching limits
- Planned budget increase for upcoming workload
- Budget governance (admin oversight of all budget changes)

---

## Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- All entity IDs use `prefix_uuid` format with underscore separator
- `request_id`: `budgetreq_<uuid>` (e.g., `budgetreq_550e8400-e29b-41d4-a716-446655440000`)
- `agent_id`: `agent_<uuid>`
- `user_id`: `user_<uuid>`

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Currency amounts: Decimal with exactly 2 decimal places (e.g., `100.00`)
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Booleans: JSON boolean `true`/`false` (not strings)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Consistent error response structure across all endpoints
- Machine-readable error codes: `VALIDATION_ERROR`, `UNAUTHORIZED`, `NOT_FOUND`, `INVALID_STATE_TRANSITION`, `INSUFFICIENT_PERMISSIONS`
- HTTP status codes: 200, 201, 400, 401, 403, 404, 409

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- Pagination: Offset-based with `?page=N&per_page=M` (default 50 items/page)
- Filtering: Query parameters for `status`, `agent_id`, `requester_id`
- Sorting: Optional `?sort=-created_at` (newest first, default)
- URL structure: `/api/v1/budget-requests`, `/api/v1/budget-requests/{id}`

---

## Relationship to Direct Budget Modification

**Two paths for budget modification:**

| Path | Endpoint | Who | When | Audit Trail |
|------|----------|-----|------|-------------|
| **Direct** | `PUT /api/v1/limits/agents/{id}/budget` | Admin only | Emergency changes, admin-initiated | Budget history only |
| **Request** | `POST /api/v1/budget-requests` | Developer | Normal workflow, requires approval | Request + justification + budget history |

**Integration:**
- Direct modifications bypass request workflow (admin convenience)
- Direct modifications do NOT auto-cancel pending requests
- Pending requests show snapshot of budget at creation time (may become stale)
- See [Protocol 013: Budget Limits API](013_budget_limits_api.md) for direct modification

---

## State Machine

```
┌─────────┐
│ pending │  (initial state: created by developer)
└────┬────┘
     │
     ├──────────────┬──────────────┬────────────────┐
     │              │              │                │
     ▼              ▼              ▼                ▼
┌──────────┐  ┌──────────┐  ┌───────────┐   ┌──────────┐
│ approved │  │ rejected │  │ cancelled │   │ (deleted)│
└──────────┘  └──────────┘  └───────────┘   └──────────┘
(terminal)    (terminal)    (terminal)       (agent gone)

Transitions:
- pending → approved (admin action, budget updated, history created)
- pending → rejected (admin action, review_notes required)
- pending → cancelled (requester or admin action)
- pending → (deleted) (agent deleted, request auto-cancelled)

No transitions FROM terminal states (approved/rejected/cancelled immutable)
```

---

## Endpoints

### Create Budget Request

**Endpoint:** `POST /api/v1/budget-requests`

**Description:** Creates a new budget change request for an agent. Requester must own the agent (or be admin). Request includes current budget snapshot, requested budget, and required justification.

**Request:**

```json
POST /api/v1/budget-requests
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "agent_id": "agent_abc123",
  "requested_budget": 150.00,
  "justification": "Agent approaching 95% budget utilization (94.50/100). Expecting 500 additional customer demo requests next week (estimated $45-55 cost). Request increase to 150 to ensure uninterrupted service."
}
```

**Request Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `agent_id` | string | Yes | Must exist, requester must own | Agent to increase budget for |
| `requested_budget` | decimal | Yes | > 0, > current_budget, 2 decimal places | Desired budget amount (absolute, not delta) |
| `justification` | string | Yes | Min 20 chars, max 500 chars | Business justification for increase |

**Success Response:**

```json
HTTP 201 Created
Content-Type: application/json

{
  "id": "breq_xyz789",
  "agent_id": "agent_abc123",
  "agent_name": "Production Agent 1",
  "requester_id": "user_dev123",
  "requester_name": "John Developer",
  "current_budget": 100.00,
  "requested_budget": 150.00,
  "justification": "Agent approaching 95% budget utilization...",
  "status": "pending",
  "created_at": "2025-12-10T15:30:00Z",
  "reviewed_at": null,
  "reviewed_by": null,
  "review_notes": null,
  "approved_budget": null
}
```

**Response Fields:**
- **`id`:** Unique request identifier (breq_ prefix)
- **`current_budget`:** Agent budget at time of request (snapshot, may become stale)
- **`status`:** Always "pending" on creation
- **All review fields null:** reviewed_at, reviewed_by, review_notes, approved_budget

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "requested_budget": "Must be greater than current budget (100.00)",
      "justification": "Must be at least 20 characters"
    }
  }
}
```

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "BUDGET_DECREASE_REQUEST",
    "message": "Budget requests must be for increases only. Current budget: 100.00, Requested: 80.00. Use direct budget modification with force flag for decreases."
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Cannot create budget request for agents you don't own"
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent does not exist"
  }
}
```

**Authorization:**
- **Agent Owner:** Can create request for own agents
- **Admin:** Can create request for any agent (typically uses direct modification instead)
- **Other users:** 403 Forbidden

**Audit Log:** Yes (request creation tracked)

**Rate Limiting:** 10 requests per hour per user

---

### List Budget Requests

**Endpoint:** `GET /api/v1/budget-requests`

**Description:** Returns paginated list of budget requests. Users see only their own requests; admins see all requests.

**Request:**

```
GET /api/v1/budget-requests?status=pending&page=1&per_page=50&sort=-created_at
Authorization: Bearer <user-token or api-token>

# Admin filtering by specific agent:
GET /api/v1/budget-requests?agent_id=agent_abc123&status=pending
Authorization: Bearer <admin-user-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number (1-indexed) |
| `per_page` | integer | 50 | Results per page (max 100) |
| `status` | string | - | Filter by status: "pending", "approved", "rejected", "cancelled" |
| `agent_id` | string | - | Filter by agent ID (admin sees all, users see own) |
| `sort` | string | `-created_at` | Sort field: `created_at`, `requested_budget` (prefix `-` for desc) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "data": [
    {
      "id": "breq_xyz789",
      "agent_id": "agent_abc123",
      "agent_name": "Production Agent 1",
      "requester_id": "user_dev123",
      "requester_name": "John Developer",
      "current_budget": 100.00,
      "requested_budget": 150.00,
      "justification": "Agent approaching 95% budget utilization...",
      "status": "pending",
      "created_at": "2025-12-10T15:30:00Z",
      "reviewed_at": null,
      "reviewed_by": null,
      "reviewed_by_name": null,
      "review_notes": null,
      "approved_budget": null
    },
    {
      "id": "breq_abc456",
      "agent_id": "agent_def456",
      "agent_name": "Test Agent",
      "requester_id": "user_dev123",
      "requester_name": "John Developer",
      "current_budget": 50.00,
      "requested_budget": 75.00,
      "justification": "Extended testing period requires additional budget...",
      "status": "approved",
      "created_at": "2025-12-09T14:00:00Z",
      "reviewed_at": "2025-12-09T16:30:00Z",
      "reviewed_by": "user_admin001",
      "reviewed_by_name": "Admin User",
      "review_notes": "Approved as requested",
      "approved_budget": 75.00
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 2,
    "total_pages": 1
  }
}
```

**Response Fields:**
- **`data[]`:** Array of budget request objects
- **Scoping:** Users see only requests they created; admins see all requests
- **Agent filtering:** Respects user permissions (users can only filter by agents they own)

**Empty Results:**

```json
HTTP 200 OK
{
  "data": [],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 0,
    "total_pages": 0
  }
}
```

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "page": "Must be >= 1",
      "per_page": "Must be between 1 and 100",
      "status": "Invalid status (allowed: pending, approved, rejected, cancelled)"
    }
  }
}
```

**Authorization:**
- **User:** Can list own budget requests only
- **Admin:** Can list all budget requests

**Audit Log:** No (read operation)

**Rate Limiting:** 60 requests per minute per user

---

### Get Budget Request Details

**Endpoint:** `GET /api/v1/budget-requests/{id}`

**Description:** Returns detailed information about a specific budget request, including current agent budget status.

**Request:**

```
GET /api/v1/budget-requests/breq_xyz789
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "breq_xyz789",
  "agent_id": "agent_abc123",
  "agent_name": "Production Agent 1",
  "agent_current_budget": 100.00,
  "agent_spent": 94.50,
  "agent_remaining": 5.50,
  "agent_status": "active",
  "requester_id": "user_dev123",
  "requester_name": "John Developer",
  "current_budget": 100.00,
  "requested_budget": 150.00,
  "justification": "Agent approaching 95% budget utilization...",
  "status": "pending",
  "created_at": "2025-12-10T15:30:00Z",
  "reviewed_at": null,
  "reviewed_by": null,
  "reviewed_by_name": null,
  "review_notes": null,
  "approved_budget": null
}
```

**Response Fields:**
- **`agent_current_budget`:** Current agent budget (may differ from `current_budget` snapshot)
- **`agent_spent`:** Current agent spending
- **`agent_remaining`:** Current agent remaining budget
- **`agent_status`:** Current agent status
- **`current_budget`:** Budget at time of request creation (snapshot, may be stale)

**Note:** If agent budget was modified after request creation, `agent_current_budget` will differ from `current_budget` snapshot.

**Error Responses:**

```json
HTTP 404 Not Found
{
  "error": {
    "code": "REQUEST_NOT_FOUND",
    "message": "Budget request does not exist"
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Cannot access other users' budget requests"
  }
}
```

**Authorization:**
- **Requester:** Can view own requests
- **Admin:** Can view any request
- **Other users:** 403 Forbidden

**Audit Log:** No (read operation)

**Rate Limiting:** 60 requests per minute per user

---

### Approve Budget Request

**Endpoint:** `PUT /api/v1/budget-requests/{id}/approve`

**Description:** Approves a budget request. Updates agent budget automatically and creates budget modification history entry. Admin-only operation.

**Request:**

```json
PUT /api/v1/budget-requests/breq_xyz789/approve
Authorization: Bearer <admin-user-token>
Content-Type: application/json

{
  "approved_budget": 140.00,
  "review_notes": "Approved with 10% reduction due to budget constraints. 140 should be sufficient for demo period."
}
```

**Request Fields (all optional):**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `approved_budget` | decimal | No | > current agent budget, 2 decimal places | Final approved amount (defaults to requested_budget if omitted) |
| `review_notes` | string | No | Max 1000 chars | Admin comments (optional for approval) |

**Note:** In Pilot, `approved_budget` parameter is accepted but typically equals `requested_budget`. POST-PILOT will support flexible approval (admin can modify amount).

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "breq_xyz789",
  "status": "approved",
  "approved_budget": 140.00,
  "reviewed_at": "2025-12-10T16:00:00Z",
  "reviewed_by": "user_admin001",
  "reviewed_by_name": "Admin User",
  "review_notes": "Approved with 10% reduction...",
  "budget_updated": true,
  "agent": {
    "id": "agent_abc123",
    "name": "Production Agent 1",
    "old_budget": 100.00,
    "new_budget": 140.00
  },
  "history_entry_id": "bh-uvw012"
}
```

**Response Fields:**
- **`budget_updated`:** Always true (budget automatically updated)
- **`agent.old_budget`:** Budget before approval
- **`agent.new_budget`:** Budget after approval
- **`history_entry_id`:** Created budget modification history entry ID

**Automatic Actions:**
1. Update request: status=approved, reviewed_at=now, reviewed_by=admin_id
2. Update agent budget to approved_budget
3. Create budget modification history entry with request_id link
4. Optional: Send email notification to requester

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "APPROVAL_DECREASES_BUDGET",
    "message": "Approved budget (80.00) is less than current agent budget (100.00). Use direct budget modification with force flag for decreases.",
    "current_budget": 100.00,
    "approved_budget": 80.00
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "REQUEST_NOT_FOUND",
    "message": "Budget request does not exist"
  }
}
```

```json
HTTP 409 Conflict
{
  "error": {
    "code": "REQUEST_ALREADY_REVIEWED",
    "message": "Request has already been approved",
    "current_status": "approved",
    "reviewed_by": "user_admin_xyz",
    "reviewed_by_name": "Other Admin",
    "reviewed_at": "2025-12-10T15:00:00Z"
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Only admins can approve budget requests"
  }
}
```

**Authorization:**
- **Admin only:** Must have admin role
- **Non-admin:** 403 Forbidden

**Validation:**
- Request must exist
- Request status must be "pending"
- approved_budget (if provided) must be > current agent budget
- approved_budget (if provided) must have max 2 decimal places

**Audit Log:** Yes (approval + budget modification both logged)

**Rate Limiting:** 30 requests per minute per user

---

### Reject Budget Request

**Endpoint:** `PUT /api/v1/budget-requests/{id}/reject`

**Description:** Rejects a budget request with required review notes explaining why. Agent budget NOT changed. Admin-only operation.

**Request:**

```json
PUT /api/v1/budget-requests/breq_xyz789/reject
Authorization: Bearer <admin-user-token>
Content-Type: application/json

{
  "review_notes": "Cannot approve at this time due to budget constraints. Current project budget is fully allocated for Q1. Please reduce agent workload or wait until Q2 for budget refresh. Contact me if this is critical for customer commitments."
}
```

**Request Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `review_notes` | string | Yes | Min 20 chars, max 1000 chars | Admin explanation for rejection (REQUIRED) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "breq_xyz789",
  "status": "rejected",
  "reviewed_at": "2025-12-10T16:00:00Z",
  "reviewed_by": "user_admin001",
  "reviewed_by_name": "Admin User",
  "review_notes": "Cannot approve at this time due to budget constraints...",
  "agent": {
    "id": "agent_abc123",
    "name": "Production Agent 1",
    "budget": 100.00
  }
}
```

**Response Fields:**
- **`agent.budget`:** Unchanged (rejection does NOT modify budget)

**Automatic Actions:**
1. Update request: status=rejected, reviewed_at=now, reviewed_by=admin_id, review_notes
2. Optional: Send email notification to requester with review_notes

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "review_notes": "review_notes is required for rejection and must be at least 20 characters"
    }
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "REQUEST_NOT_FOUND",
    "message": "Budget request does not exist"
  }
}
```

```json
HTTP 409 Conflict
{
  "error": {
    "code": "REQUEST_ALREADY_REVIEWED",
    "message": "Request has already been rejected",
    "current_status": "rejected",
    "reviewed_by": "user_admin_xyz",
    "reviewed_by_name": "Other Admin",
    "reviewed_at": "2025-12-10T15:00:00Z"
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Only admins can reject budget requests"
  }
}
```

**Authorization:**
- **Admin only:** Must have admin role
- **Non-admin:** 403 Forbidden

**Validation:**
- Request must exist
- Request status must be "pending"
- review_notes required, min 20 chars, max 1000 chars

**Audit Log:** Yes (rejection logged)

**Rate Limiting:** 30 requests per minute per user

---

### Cancel Budget Request

**Endpoint:** `DELETE /api/v1/budget-requests/{id}`

**Description:** Cancels a pending budget request. Only requester or admin can cancel. Cannot cancel already-reviewed requests.

**Request:**

```
DELETE /api/v1/budget-requests/breq_xyz789
Authorization: Bearer <user-token or api-token>
```

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "id": "breq_xyz789",
  "status": "cancelled",
  "cancelled_at": "2025-12-10T16:00:00Z",
  "cancelled_by": "user_dev123",
  "cancelled_by_name": "John Developer"
}
```

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "CANNOT_CANCEL_REVIEWED",
    "message": "Cannot cancel request that has been reviewed",
    "current_status": "approved",
    "reviewed_by": "user_admin001",
    "reviewed_at": "2025-12-10T15:00:00Z"
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "REQUEST_NOT_FOUND",
    "message": "Budget request does not exist"
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Can only cancel your own budget requests"
  }
}
```

**Authorization:**
- **Requester:** Can cancel own pending requests
- **Admin:** Can cancel any pending request
- **Other users:** 403 Forbidden

**Validation:**
- Request must exist
- Request status must be "pending" (cannot cancel approved/rejected)

**Audit Log:** Yes (cancellation logged)

**Rate Limiting:** 30 requests per minute per user

---

## Data Models

### Budget Request Object

```json
{
  "id": "breq_xyz789",
  "agent_id": "agent_abc123",
  "agent_name": "Production Agent 1",
  "requester_id": "user_dev123",
  "requester_name": "John Developer",
  "current_budget": 100.00,
  "requested_budget": 150.00,
  "justification": "Agent approaching 95% budget utilization...",
  "status": "pending",
  "created_at": "2025-12-10T15:30:00Z",
  "reviewed_at": null,
  "reviewed_by": null,
  "reviewed_by_name": null,
  "review_notes": null,
  "approved_budget": null
}
```

**Fields:**

| Field | Type | Always Present | Description |
|-------|------|----------------|-------------|
| `id` | string | Yes | Unique request ID (breq_ prefix) |
| `agent_id` | string | Yes | Agent being requested budget for |
| `agent_name` | string | Yes | Agent display name (denormalized) |
| `requester_id` | string | Yes | User who created request |
| `requester_name` | string | Yes | Requester display name (denormalized) |
| `current_budget` | decimal | Yes | Agent budget at request time (snapshot) |
| `requested_budget` | decimal | Yes | Developer's requested amount |
| `justification` | string | Yes | Business justification (20-500 chars) |
| `status` | string | Yes | "pending", "approved", "rejected", "cancelled" |
| `created_at` | string | Yes | ISO 8601 timestamp with Z |
| `reviewed_at` | string | No | When admin acted (null if pending) |
| `reviewed_by` | string | No | Admin user ID (null if pending) |
| `reviewed_by_name` | string | No | Admin display name (null if pending) |
| `review_notes` | string | No | Admin comments (null if pending/approved without notes) |
| `approved_budget` | decimal | No | Final approved amount (null if pending/rejected/cancelled) |

---

## Security

### Authorization Matrix

| Operation | Requester | Admin | Other User |
|-----------|-----------|-------|------------|
| Create request | ✅ (own agents) | ✅ (any agent) | ❌ |
| List requests | ✅ (own) | ✅ (all) | ❌ |
| Get request details | ✅ (own) | ✅ (all) | ❌ |
| Approve request | ❌ | ✅ | ❌ |
| Reject request | ❌ | ✅ | ❌ |
| Cancel request | ✅ (own, pending) | ✅ (any, pending) | ❌ |

**Key principles:**
- **Admin oversight:** Only admins can approve/reject budget changes
- **Developer self-service:** Developers can create requests and cancel pending ones
- **Privacy:** Users cannot see other users' requests
- **Audit trail:** All actions logged with user_id and timestamp

### Justification Requirements

**Purpose:** Ensure all budget changes have business rationale (governance + audit)

**Rules:**
- Justification REQUIRED for all requests (min 20 chars)
- Justification should explain WHY increase is needed (not just WHAT)
- Review notes REQUIRED for rejection (min 20 chars)
- Review notes OPTIONAL for approval

**Good justification examples:**
- ✅ "Agent approaching 95% budget (94.50/100). Customer demo next week expects 500 requests ($45-55 cost). Need $150 total to ensure no interruption."
- ✅ "Production workload increased 3x due to new feature launch. Current $50 budget exhausted in 2 days instead of expected 7 days. Need $200 for sustainable operation."

**Bad justification examples:**
- ❌ "Need more budget" (too vague, no context)
- ❌ "Increase to 150" (states WHAT, not WHY)
- ❌ "asdfghjkl" (meaningless)

### Sensitive Data

**NOT exposed via API:**
- Token values (IC tokens, API tokens)
- Provider credentials (IP tokens)
- User passwords

**Exposed:**
- Agent budgets (requester can already see via agent API)
- Spending amounts (requester can already see via analytics)
- Request justifications (audit trail requirement)

---

## Error Handling

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Field validation failed |
| `BUDGET_DECREASE_REQUEST` | 400 | Request must be for increase (requested <= current) |
| `APPROVAL_DECREASES_BUDGET` | 400 | Approval would decrease budget (approved < current) |
| `CANNOT_CANCEL_REVIEWED` | 400 | Cannot cancel already-reviewed request |
| `UNAUTHORIZED` | 401 | Missing or invalid authentication |
| `TOKEN_EXPIRED` | 401 | User token expired |
| `FORBIDDEN` | 403 | Insufficient permissions (non-admin approving, non-owner creating) |
| `AGENT_NOT_FOUND` | 404 | Agent does not exist |
| `REQUEST_NOT_FOUND` | 404 | Budget request does not exist |
| `REQUEST_ALREADY_REVIEWED` | 409 | Request already approved/rejected (concurrency conflict) |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

### Common Error Scenarios

**Scenario 1: Request for non-existent agent**
```json
POST /api/v1/budget-requests
{"agent_id": "agent_invalid", "requested_budget": 100, "justification": "..."}

→ 404 Not Found
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent does not exist"
  }
}
```

**Scenario 2: Non-admin tries to approve**
```json
PUT /api/v1/budget-requests/breq_xyz/approve

→ 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Only admins can approve budget requests"
  }
}
```

**Scenario 3: Concurrent approval**
```json
Admin A: PUT /api/v1/budget-requests/breq_xyz/approve (10:00:00.000)
Admin B: PUT /api/v1/budget-requests/breq_xyz/approve (10:00:00.100)

Admin B → 409 Conflict
{
  "error": {
    "code": "REQUEST_ALREADY_REVIEWED",
    "message": "Request has already been approved by Admin A"
  }
}
```

---

## Rate Limiting

### Limits (per user)

| Endpoint | Limit | Window | Scope |
|----------|-------|--------|-------|
| Create request | 10 | 1 hour | Per user |
| List requests | 60 | 1 minute | Per user |
| Get request details | 60 | 1 minute | Per user |
| Approve request | 30 | 1 minute | Per admin |
| Reject request | 30 | 1 minute | Per admin |
| Cancel request | 30 | 1 minute | Per user |

**Exceeded limit response:**
```json
HTTP 429 Too Many Requests
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Try again in 45 seconds.",
    "retry_after": 45
  }
}
```

---

## Integration with Budget Modification History

### Linking Requests to Budget History

When request is approved, system creates budget modification history entry with `request_id` field:

**Budget history entry:**
```json
{
  "id": "bh-uvw012",
  "agent_id": "agent_abc123",
  "previous_budget": 100.00,
  "new_budget": 150.00,
  "modified_by": "user_admin001",
  "modified_by_name": "Admin User",
  "modified_at": "2025-12-10T16:00:00Z",
  "change_type": "increase",
  "reason": "Budget request approved",
  "request_id": "breq_xyz789",
  "force_flag": false
}
```

**Querying:**
```
GET /api/v1/limits/agents/agent_abc123/history

Returns history with request_id links. Client can follow link:
GET /api/v1/budget-requests/breq_xyz789

To see full justification and approval notes.
```

---

## Notifications (Optional for Pilot)

### Email Notifications

**Trigger: Request Created**
- To: All admin users
- Subject: "Budget Request Pending: {agent_name}"
- Body: Includes requester, agent, amounts, justification, review link

**Trigger: Request Approved**
- To: Requester
- Subject: "Budget Request Approved: {agent_name}"
- Body: Includes approved amount, reviewer, review notes

**Trigger: Request Rejected**
- To: Requester
- Subject: "Budget Request Rejected: {agent_name}"
- Body: Includes reviewer, review notes (explains why)

**Configuration:**
- Email notifications are OPTIONAL in Pilot (nice-to-have)
- Can be enabled/disabled per user in settings
- Default: Enabled for admins, disabled for users

---

## Edge Cases

### Agent Deleted While Request Pending

**Behavior:** Request auto-cancelled with review_notes="Auto-cancelled: Agent was deleted"

**Database:**
```sql
-- When agent deleted:
UPDATE budget_change_requests
SET status = 'cancelled',
    review_notes = 'Auto-cancelled: Agent was deleted'
WHERE agent_id = ? AND status = 'pending';
```

### Multiple Pending Requests for Same Agent

**Allowed:** Yes, no restriction on number of pending requests per agent

**Rationale:**
- Developer might refine request based on changing needs
- Admin can see evolution of budget requirements
- Developer can cancel old request after creating new one

### Budget Modified While Request Pending

**Behavior:** Request shows stale `current_budget` snapshot, acceptable

**Example:**
1. Developer creates request: current=100, requested=150
2. Admin directly sets budget to 120
3. Request still shows: current=100 (stale)
4. Admin approves: budget goes 120 → 150 (not 100 → 150)

**Note:** `current_budget` is historical snapshot (budget AT TIME OF REQUEST), not live value

### Approval Amount Less Than Current Budget

**Validation:** Error 400 APPROVAL_DECREASES_BUDGET

**Rationale:** Requests are for INCREASES only. Use direct modification with force flag for decreases.

---

## Pilot vs POST-PILOT Features

### Pilot (MUST-HAVE)

**Included:**
- All 6 endpoints (Create, List, Get, Approve, Reject, Cancel)
- Required justification (20-500 chars)
- Admin-only approval/rejection
- Request scoping (users see own, admins see all)
- State machine (pending → approved/rejected/cancelled)
- Integration with budget history (request_id link)
- Validation rules
- Error handling
- Rate limiting (basic)
- Email notifications (OPTIONAL, nice-to-have)

**Simple approval:**
- `approved_budget` defaults to `requested_budget`
- Admin can specify different amount, but typically doesn't

### POST-PILOT Enhancements

**Deferred features:**
1. **Flexible approval amount** - Admin routinely modifies amount during approval
2. **Request expiration** - Auto-reject requests older than N days
3. **Bulk operations** - Approve/reject multiple requests at once
4. **Advanced filters** - Date range, amount range, multiple statuses
5. **Request templates** - Pre-filled justifications for common scenarios
6. **Approval delegation** - Admin assigns approval authority to senior developers
7. **Rejection cooldown** - Cannot re-request same agent for 24h after rejection
8. **Webhook events** - budget_request.created, approved, rejected, cancelled
9. **Request comments** - Discussion thread on requests (requester ↔ admin conversation)

---

## Related Documentation

- **[Protocol 010: Agents API](010_agents_api.md)** - Agent management (request references agents)
- **[Protocol 013: Budget Limits API](013_budget_limits_api.md)** - Direct budget modification (admin path)
- **[Protocol 008: User Management API](008_user_management_api.md)** - User roles (admin authorization)
- **[Protocol 002: REST API Protocol](002_rest_api_protocol.md)** - Standard patterns (pagination, errors)
- **[Resource Catalog](009_resource_catalog.md)** - Complete resource inventory
- **[Vocabulary](../vocabulary.md)** - Terminology definitions
