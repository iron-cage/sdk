# Protocol: Budget Limits API

### Scope

This protocol defines the HTTP API for modifying agent budgets after creation and viewing budget modification history.

**In scope**:
- Budget modification endpoint (PUT /api/v1/limits/agents/{agent_id}/budget) for admin-only direct budget changes (increase or decrease)
- Force flag requirement for budget decreases preventing accidental agent shutdowns (force: true confirmation with impact analysis)
- Budget history endpoint (GET /api/v1/limits/agents/{agent_id}/budget/history) returning modification records sorted newest first
- Authorization model (admin can modify any agent budget, agent owner can view own history, non-admin cannot modify directly)
- Emergency use case for long-running agents approaching budget limits (admin bypasses request workflow for immediate top-up)
- Audit logging for all budget modifications (previous/new budget, reason, modified_by, timestamp, 90-day retention)
- Safety validation (budget decreases rejected without force flag, budget unchanged validation, new budget >= 0.01)
- Developer request workflow alternative (see Protocol 017: Budget Change Requests API for planned budget increases)

**Out of scope**:
- Initial budget setting at agent creation (see Protocol 010: Agents API for POST /api/v1/agents endpoint)
- Budget enforcement and consumption tracking (see Protocol 005: Budget Control Protocol)
- Budget status monitoring and analytics queries (see Protocol 012: Analytics API)
- Automatic budget alerts or threshold notifications (Future Enhancement)
- Budget presets or quick increase templates (Future Enhancement)
- Budget decrease without force flag (safety policy, requires explicit confirmation)
- Non-admin budget modifications (developers use Protocol 017: Budget Change Requests API instead)
- Budget modification for deleted agents (404 error, no resurrection of deleted agents)

### Purpose

**User Need**: Admins need the ability to modify agent budgets after creation to handle emergency scenarios where long-running agents approach budget limits mid-task. Without budget modification capability, agents would fail critical tasks (95% budget used, 4-hour customer task running) forcing developers to abandon work and create new agents. Developers need transparency into budget modification history for compliance audits and operational review. Both need safety guardrails preventing accidental budget decreases that could immediately exhaust agents mid-task.

**Solution**: This API provides admin-only budget modification (PUT /api/v1/limits/agents/{agent_id}/budget) with full mutability (increase or decrease) protected by force flag requirement for decreases. Budget increases are immediate (emergency top-up bypassing request workflow) while decreases require explicit force: true confirmation with impact analysis showing current_spent and new_remaining. Budget history endpoint (GET /api/v1/limits/agents/{agent_id}/budget/history) returns all modifications with reasons, modified_by, timestamps, and summary statistics (initial budget, total increases, modification count). All modifications are audit logged (90-day retention) with previous/new budget, reason, and user identity. Developers use Protocol 017: Budget Change Requests API for planned increases (request/approval workflow) while admins bypass for emergencies.

**Key Insight**: Budget modification requires asymmetric safety policy. Budget increases are safe (more resources, no interruption risk) and should be frictionless for emergency scenarios. Budget decreases are dangerous (could immediately exhaust agent mid-task) and must require explicit confirmation with impact preview. The force flag design achieves this: first decrease attempt rejected with detailed impact analysis (current budget, requested budget, decrease amount, current spent, new remaining if applied), then admin retries with force: true after reviewing impact. This prevents accidental shutdowns while preserving admin control. The admin-only modification policy (developers use request workflow) provides centralized budget governance while emergency bypass enables immediate response to critical situations.

---

**Status**: Specification
**Version**: 1.0.0
**Last Updated**: 2025-12-14
**Priority**: MUST-HAVE

### Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- Budget Limits API uses short alphanumeric IDs for agent and user identifiers to optimize performance, readability, and operational clarity
- `agent_id`: `agent_<alphanumeric>` with regex `^agent_[a-z0-9]{6,32}$` (e.g., `agent_abc123`)
- `user_id`: `user_<alphanumeric>` with regex `^user_[a-z0-9_]{3,32}$` (e.g., `user_admin_001`, `user_xyz789`)

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Currency amounts: Decimal with exactly 2 decimal places (e.g., `100.00`)
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Booleans: JSON boolean `true`/`false` (not strings)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Consistent error response structure across all endpoints
- Machine-readable error codes: `VALIDATION_ERROR`, `UNAUTHORIZED`, `NOT_FOUND`, `BUDGET_DECREASED_WITHOUT_FORCE`
- HTTP status codes: 200, 400, 401, 403, 404

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- URL structure: `/api/v1/agents/{id}/budget`
- Standard HTTP methods: PATCH for budget modifications


### Endpoints

#### Modify Agent Budget

**Endpoint:** `PUT /api/v1/limits/agents/{agent_id}/budget`

**Description:** Modifies an agent's budget (increase or decrease). Decrease requests require `force: true` confirmation to prevent accidental shutdowns.

**Use Case:** Emergency budget top-up for long-running agents (admin-only direct path)
- **Scenario:** Agent at 95% budget usage, running multi-hour task
- **Problem:** Task will fail if budget exhausted
- **Solution:** Admin immediately increases budget to prevent interruption (bypasses request workflow)
- **Developer alternative:** Use [Budget Change Requests](017_budget_requests_api.md) for planned budget increases

**Request:**

```json
PUT /api/v1/limits/agents/agent_abc123/budget
Authorization: Bearer <user-token or api-token>
Content-Type: application/json

{
  "budget": 150.00,
  "reason": "Emergency top-up: agent running critical customer task"
}
```

**Request Parameters:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `budget` | number | Yes | >= 0.01 | New budget amount in USD (2 decimal places) |
| `force` | boolean | No | Default: false | Required for budget decreases (safety confirmation) |
| `reason` | string | No | Max 500 chars | Optional explanation for audit trail |

**Important:** Budget decreases require `force: true` to prevent accidental agent shutdowns. Increases dont require force flag.

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "agent_id": "agent_abc123",
  "previous_budget": 100.00,
  "new_budget": 150.00,
  "increase_amount": 50.00,
  "increase_percent": 50.00,
  "current_spent": 95.75,
  "new_remaining": 54.25,
  "reason": "Emergency top-up: agent running critical customer task",
  "modified_by": "user_admin_001",
  "modified_at": "2025-12-10T15:30:45Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `agent_id` | string | Agent identifier |
| `previous_budget` | number | Budget before modification (USD) |
| `new_budget` | number | Budget after modification (USD) |
| `increase_amount` | number | Absolute increase (USD) |
| `increase_percent` | number | Percentage increase (0-100, 2 decimal places) |
| `current_spent` | number | Amount already spent (USD) |
| `new_remaining` | number | Remaining budget after increase (USD) |
| `reason` | string | Modification reason (omitted if not provided) |
| `modified_by` | string | User ID who made the modification |
| `modified_at` | string | ISO 8601 timestamp of modification |

**Error Responses:**

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "BUDGET_DECREASE_REQUIRES_CONFIRMATION",
    "message": "Budget decrease requires 'force: true' confirmation. Current budget: $100.00, requested: $80.00. WARNING: Decreasing budget may immediately exhaust agent mid-task.",
    "current_budget": 100.00,
    "requested_budget": 80.00,
    "decrease_amount": 20.00,
    "current_spent": 45.00,
    "new_remaining_if_applied": 35.00
  }
}
```

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "BUDGET_UNCHANGED",
    "message": "New budget must be different from current budget. Current budget: $100.00",
    "current_budget": 100.00,
    "requested_budget": 100.00
  }
}
```

```json
HTTP 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "budget": "Must be >= 0.01",
      "reason": "Maximum 500 characters"
    }
  }
}
```

```json
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient permissions. Only admin or agent owner can modify budget.",
    "agent_owner": "user_xyz789",
    "requesting_user": "user_other_001"
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent 'agent_invalid' does not exist"
  }
}
```

**Authorization:**
- **Admin only:** Can modify any agent's budget (increase or decrease)
- **Non-admin:** 403 Forbidden (use [Budget Change Requests](017_budget_requests_api.md) instead)

**Audit Log:** Yes (mutation operation, includes previous/new budget and reason)


#### Get Budget Modification History

**Endpoint:** `GET /api/v1/limits/agents/{agent_id}/budget/history`

**Description:** Returns history of budget modifications for an agent, sorted by modification time descending (most recent first).

**Request:**

```
GET /api/v1/limits/agents/agent_abc123/budget/history?page=1&per_page=50
Authorization: Bearer <user-token or api-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number (1-indexed) |
| `per_page` | integer | 50 | Results per page (max 100) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "agent_id": "agent_abc123",
  "current_budget": 150.00,
  "modifications": [
    {
      "previous_budget": 100.00,
      "new_budget": 150.00,
      "increase_amount": 50.00,
      "increase_percent": 50.00,
      "reason": "Emergency top-up: agent running critical customer task",
      "modified_by": "user_admin_001",
      "modified_by_name": "Admin User",
      "modified_at": "2025-12-10T15:30:45Z"
    },
    {
      "previous_budget": 50.00,
      "new_budget": 100.00,
      "increase_amount": 50.00,
      "increase_percent": 100.00,
      "reason": "Initial budget adjustment after testing",
      "modified_by": "user_xyz789",
      "modified_by_name": "Agent Owner",
      "modified_at": "2025-12-09T10:15:20Z"
    }
  ],
  "summary": {
    "initial_budget": 50.00,
    "current_budget": 150.00,
    "total_increases": 100.00,
    "modification_count": 2
  },
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 2,
    "total_pages": 1
  }
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `agent_id` | string | Agent identifier |
| `current_budget` | number | Current budget (USD) |
| `modifications[]` | array | Budget modification records (sorted newest first) |
| `modifications[].previous_budget` | number | Budget before modification (USD) |
| `modifications[].new_budget` | number | Budget after modification (USD) |
| `modifications[].increase_amount` | number | Absolute increase (USD) |
| `modifications[].increase_percent` | number | Percentage increase (0-100) |
| `modifications[].reason` | string | Modification reason (omitted if not provided) |
| `modifications[].modified_by` | string | User ID who made modification |
| `modifications[].modified_by_name` | string | User name who made modification |
| `modifications[].modified_at` | string | ISO 8601 timestamp |
| `summary` | object | Aggregate statistics |
| `summary.initial_budget` | number | Budget at agent creation (USD) |
| `summary.current_budget` | number | Current budget (USD) |
| `summary.total_increases` | number | Sum of all increases (USD) |
| `summary.modification_count` | integer | Number of modifications |
| `pagination` | object | Pagination metadata |

**Note:** Initial budget (set at agent creation via `POST /api/v1/agents`) is NOT included in modifications array. Use summary.initial_budget to see original budget.

**Empty History:**

```json
HTTP 200 OK
{
  "agent_id": "agent_abc123",
  "current_budget": 100.00,
  "modifications": [],
  "summary": {
    "initial_budget": 100.00,
    "current_budget": 100.00,
    "total_increases": 0.00,
    "modification_count": 0
  },
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
HTTP 403 Forbidden
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient permissions"
  }
}
```

```json
HTTP 404 Not Found
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent 'agent_invalid' does not exist"
  }
}
```

**Authorization:**
- **Agent Owner:** Can view own agent's history
- **Admin:** Can view any agent's history
- **Other Users:** 403 Forbidden

**Audit Log:** No (read operation)


### Data Models

#### Budget Modification Object

```json
{
  "agent_id": "agent_abc123",
  "previous_budget": 100.00,
  "new_budget": 150.00,
  "increase_amount": 50.00,
  "increase_percent": 50.00,
  "current_spent": 95.75,
  "new_remaining": 54.25,
  "reason": "Emergency top-up: agent running critical customer task",
  "modified_by": "user_admin_001",
  "modified_at": "2025-12-10T15:30:45Z"
}
```

#### Budget History Object

```json
{
  "agent_id": "agent_abc123",
  "current_budget": 150.00,
  "modifications": [
    {
      "previous_budget": 100.00,
      "new_budget": 150.00,
      "increase_amount": 50.00,
      "increase_percent": 50.00,
      "reason": "Emergency top-up",
      "modified_by": "user_admin_001",
      "modified_by_name": "Admin User",
      "modified_at": "2025-12-10T15:30:45Z"
    }
  ],
  "summary": {
    "initial_budget": 50.00,
    "current_budget": 150.00,
    "total_increases": 100.00,
    "modification_count": 2
  },
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 2,
    "total_pages": 1
  }
}
```


### Security

#### Authorization Matrix

| Operation | Agent Owner | Admin | Other User |
|-----------|-------------|-------|------------|
| Modify budget (direct) | ❌ (use request workflow) | ✅ (all agents) | ❌ |
| View budget history | ✅ (own agents) | ✅ (all agents) | ❌ |

**Reasoning:**
- **Admin control:** Only admins can directly modify budgets (full governance)
- **Developer path:** Developers use [Budget Change Requests](017_budget_requests_api.md) (request/approval workflow)
- **Admin oversight:** Admins have system-wide visibility and control
- **Safety:** Budget modifications require admin authorization (prevents unauthorized spending)

#### Force Flag Policy

**Why require force flag for decreases?**
1. **Prevent accidental shutdowns:** Decreasing budget could immediately exhaust agent mid-task
2. **Admin control with safety:** Admin has full control but must explicitly confirm dangerous operations
3. **Informative errors:** Decrease attempt without force shows impact (current spent, remaining if applied)
4. **Audit trail:** All modifications logged with force flag status

**Budget decrease workflow:**
1. Admin attempts decrease without force flag
2. API returns `BUDGET_DECREASE_REQUIRES_CONFIRMATION` with impact analysis
3. Admin reviews impact (current spent, new remaining)
4. Admin retries with `force: true` to confirm
5. Budget decreased, modification logged

**Example decrease request:**
```json
PUT /api/v1/limits/agents/agent_abc123/budget
Authorization: Bearer <user-token>
Content-Type: application/json

{
  "budget": 80.00,
  "force": true,
  "reason": "Correcting budget misconfiguration"
}
```


### Error Handling

#### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Field validation failed |
| `BUDGET_DECREASE_REQUIRES_CONFIRMATION` | 400 | Budget decrease attempted without force flag |
| `BUDGET_UNCHANGED` | 400 | Requested budget = current budget |
| `UNAUTHORIZED` | 401 | Missing/invalid authentication |
| `TOKEN_EXPIRED` | 401 | Authentication token expired |
| `FORBIDDEN` | 403 | Not admin or agent owner |
| `AGENT_NOT_FOUND` | 404 | Agent does not exist |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Unexpected server error |


### Rate Limiting

#### Limits (per user)

| Endpoint | Limit | Window | Reasoning |
|----------|-------|--------|-----------|
| `PUT /api/v1/limits/agents/{id}/budget` | 10 | 1 minute | Budget increases rare, prevent abuse |
| `GET /api/v1/limits/agents/{id}/budget/history` | 60 | 1 minute | Standard read rate |


### Audit Logging

#### Logged Operations

| Endpoint | Method | Logged | Special Fields |
|----------|--------|--------|----------------|
| `PUT /api/v1/limits/agents/{id}/budget` | PUT | ✅ Yes | previous_budget, new_budget, reason |
| `GET /api/v1/limits/agents/{id}/budget/history` | GET | ❌ No | N/A |

#### Audit Log Entry

```json
{
  "timestamp": "2025-12-10T15:30:45Z",
  "user_id": "user_admin_001",
  "endpoint": "PUT /api/v1/limits/agents/agent_abc123/budget",
  "method": "PUT",
  "resource_type": "agent_budget",
  "resource_id": "agent_abc123",
  "action": "increase",
  "parameters": {
    "previous_budget": 100.00,
    "new_budget": 150.00,
    "increase_amount": 50.00,
    "reason": "Emergency top-up: agent running critical customer task"
  },
  "status": "success",
  "ip_address": "203.0.113.42",
  "user_agent": "iron-cli/1.0.0"
}
```

**Retention:** 90 days (compliance standard)


### CLI Integration

#### iron limits agent_budget increase

```bash
iron limits agent_budget increase agent_abc123 150.00
iron limits agent_budget increase agent_abc123 150.00 \
  --reason "Emergency top-up: agent running critical customer task"

# Output:
# Budget increased for agent_abc123
# Previous: $100.00 → New: $150.00 (+ $50.00, +50%)
# Current spent: $95.75
# New remaining: $54.25
# Modified by: user_admin_001
# Modified at: 2025-12-10 15:30:45
```

#### iron limits agent_budget history

```bash
iron limits agent_budget history agent_abc123

# Output:
# Budget Modification History for agent_abc123
# Current budget: $150.00
#
# DATE                 FROM      TO        INCREASE  REASON                         BY
# 2025-12-10 15:30:45  $100.00   $150.00   +$50.00   Emergency top-up: critical...  Admin User
# 2025-12-09 10:15:20  $50.00    $100.00   +$50.00   Initial adjustment             Agent Owner
#
# Summary:
#   Initial budget: $50.00
#   Current budget: $150.00
#   Total increases: $100.00
#   Modifications: 2
```

#### iron limits agent_budget get

```bash
iron limits agent_budget get agent_abc123

# Output:
# Agent: agent_abc123 (Production Agent 1)
# Budget: $150.00
# Spent: $95.75 (63.83%)
# Remaining: $54.25
# Status: active
#
# Initial budget: $50.00
# Total increases: $100.00
# Modifications: 2
```


### Use Case Examples

#### Example 1: Emergency Budget Increase

**Scenario:** Production agent at 95% budget, running critical 4-hour customer task

**Steps:**
1. **Monitoring:** Dashboard shows agent at 95% budget usage
2. **Alert:** Email/notification sent to agent owner and admin
3. **Decision:** Owner decides to increase budget (task critical, cannot fail)
4. **Action:** Owner increases budget via CLI or dashboard
5. **Result:** Agent continues task without interruption

**CLI:**
```bash
# Check current status
iron agents status agent_abc123
# Agent: agent_abc123 (Production Agent 1)
# Status: active
# Budget: $95.00 / $100.00 (95% used)
# ⚠️  WARNING: Agent near budget limit

# Increase budget
iron limits agent_budget increase agent_abc123 150.00 \
  --reason "Emergency top-up: running critical customer task"
# Budget increased: $100.00 → $150.00 (+$50.00)
# New remaining: $55.00

# Verify
iron agents status agent_abc123
# Agent: agent_abc123 (Production Agent 1)
# Status: active
# Budget: $95.00 / $150.00 (63% used)
# ✅ Budget healthy
```


#### Example 2: Budget Decrease Attempt (Rejected)

**Scenario:** Admin accidentally tries to decrease budget

**Steps:**
1. **Mistake:** Admin enters budget lower than current
2. **Validation:** API rejects request with clear error
3. **Resolution:** Admin realizes mistake, uses correct value

**CLI:**
```bash
# Attempt decrease
iron limits agent_budget increase agent_abc123 50.00

# Error:
# ❌ Budget decrease not allowed
# Current budget: $100.00
# Requested budget: $50.00
# Budget can only be increased to prevent accidental agent shutdowns.

# Correct usage
iron limits agent_budget increase agent_abc123 150.00
# ✅ Budget increased: $100.00 → $150.00
```


#### Example 3: Budget Audit Trail

**Scenario:** Compliance audit requires budget modification history

**Steps:**
1. **Query:** Admin retrieves budget history for all agents
2. **Review:** Examine modifications for suspicious patterns
3. **Export:** Generate report for compliance team

**CLI:**
```bash
# View history
iron limits agent_budget history agent_abc123

# Output shows:
# - All modifications with reasons
# - Who made each modification
# - When modifications occurred
# - Initial budget vs current budget
```


### Future Enhancements (Post-Pilot)

#### Budget Decrease Support

**Endpoint:** `PUT /api/v1/limits/agents/{agent_id}/budget/decrease`

**Requirements:**
- Separate endpoint (explicit action)
- Requires admin approval workflow
- Blocked if new budget < current_spent
- Comprehensive audit logging

**Use case:** Fix misconfiguration without creating new agent


#### Budget Alerts

**Endpoint:** `POST /api/v1/limits/agents/{agent_id}/budget/alerts`

**Features:**
- Configure budget threshold alerts (e.g., 80%, 90%, 95%)
- Email/webhook notifications
- Auto-suggest budget increase amount


#### Budget Presets

**Endpoint:** `GET /api/v1/limits/budget-presets`

**Features:**
- Pre-defined budget levels (small: $10, medium: $50, large: $100, etc.)
- Quick budget increases without entering amounts


### Cross-References

#### Related Principles Documents

None.

#### Related Architecture Documents

- Architecture 007: Entity Model - Agent Budget entity definition

#### Used By

None currently. Budget modification capability consumed by admin dashboard UI (not yet documented).

#### Dependencies

- Protocol 002: REST API Protocol - General REST API standards and conventions
- Protocol 005: Budget Control Protocol - Budget enforcement and consumption tracking referenced in budget modification context
- Protocol 010: Agents API - Agent entity for budget association, initial budget setting at agent creation
- Protocol 012: Analytics API - Budget status monitoring and alerting referenced in emergency use case
- Protocol 017: Budget Change Requests API - Developer request/approval workflow alternative to admin direct modification

#### Implementation

- `/home/user1/pro/lib/wip_iron/iron_runtime/dev/module/iron_control_api/src/routes/limits.rs` - Budget modification endpoint handlers

