# ADR-009: REST API Design Decisions

**Status:** Accepted
**Date:** 2025-12-11

---

## Context

The Iron Control Panel requires a production REST API to manage agents, providers, analytics, budgets, and authentication for the Pilot launch. This API serves three primary consumers:

1. **Web Dashboard** - User-facing interface for monitoring and management
2. **CLI Tools** (`iron` command) - Developer workflow automation
3. **Future Integrations** - Third-party tools and automation scripts

**Design Constraints:**
- **Pilot Timeline:** 3-week MVP requires proven, simple technologies
- **Scale:** Hundreds of agents, dozens of providers (not millions)
- **CLI Parity:** Every user-facing API endpoint must have corresponding CLI command
- **Consistency:** Uniform patterns across all endpoints (pagination, errors, auth)
- **Security:** Audit trails, role-based access, budget governance

**Scope:** This ADR documents 61 design decisions across 8 resource categories, totaling 46 user-facing endpoints distributed across 9 protocol files.

---

## Decision Summary

Made 61 design decisions across 8 categories:

1. **Core REST API Design (Q1-Q19)** - 19 decisions
   - Resource priorities (Projects, Providers, Analytics, API Tokens, Budget Limits, Settings)
   - CRUD operation scope per resource
   - Deletion policies and constraints

2. **Agent API Details (Q20-Q27)** - 8 decisions
   - Agent creation method and parameters
   - Provider assignment patterns
   - Zero-provider support

3. **Analytics API Details (Q28-Q30)** - 3 decisions
   - Pagination strategy
   - Filtering capabilities
   - Empty results handling

4. **API Tokens Details (Q31-Q32)** - 2 decisions
   - Token management operations
   - Token visibility and security

5. **Budget Limits API Details (Q33)** - 1 decision
   - Budget modification authorization (dual-path architecture)

6. **Cross-Cutting Concerns (Q34-Q36)** - 3 decisions
   - Universal pagination standard
   - Audit logging strategy
   - CLI-API parity enforcement

7. **Error Handling & Validation (Q37-Q42)** - 6 decisions
   - Error response format
   - HTTP status code strategy
   - Validation error details

8. **Data Formats & Constraints (Q43-Q61)** - 19 decisions
   - ID formats and prefixes
   - Timestamp formats
   - Currency handling
   - Soft delete patterns
   - POST-PILOT deferrals

**Implementation:** 9 protocol files (002, 008, 010-015, 017), 3 standards files, 46 user-facing endpoints, 100% CLI parity.

---

## Key Decisions

### 1. Universal Offset Pagination (Q28, Q34)

**Decision:** All list endpoints use offset pagination with `page` and `per_page` parameters.

**Reasoning:**
- **Developer Familiarity:** Standard REST API pattern, universally understood
- **Pilot Scale:** Sufficient for hundreds of agents, dozens of providers
- **UI Consistency:** Page-based navigation natural for dashboard pagination controls
- **Implementation Simplicity:** Straightforward SQL `LIMIT`/`OFFSET` queries

**Alternatives Considered:**
- **Cursor-based pagination:** More complex, unnecessary for Pilot scale (hundreds not millions)
- **GraphQL pagination:** Out of scope for REST API design
- **Mixed strategies:** Inconsistent developer experience, rejected for uniformity

**Implementation:**
```
Query Parameters:
  page (integer, default: 1, min: 1)
  per_page (integer, default: 50, min: 1, max: 100)

Response Format:
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

**Applies to:** All 7 list endpoint categories (agents, providers, analytics, tokens, projects, budget requests, users)

**Reference:** Protocol 002:101-210 (Universal Pagination Standard)

---

### 2. Budget Modification - Dual-Path Architecture (Q16, Q33)

**Decision:** Admin can modify budgets directly (increase AND decrease). Developers can request budget changes (requires admin approval).

**Reasoning:**
- **Emergency Scenarios:** Admin needs ability to reduce budgets immediately (runaway costs, quota violations)
- **Governance:** Budget changes require oversight and justification (audit trail)
- **Self-Service:** Developers can request increases without pre-approval (reduces admin bottleneck)
- **Safety:** Decreases require `force: true` flag to prevent accidental budget cuts

**Alternatives Considered:**
- **INCREASE-ONLY:** Too restrictive for admin emergency scenarios (can't reduce runaway budgets)
- **IMMUTABLE:** Forces agent recreation for budget changes (disruptive, loses history)
- **NO-WORKFLOW:** Direct developer modification lacks governance oversight

**Implementation:**

**Path 1: Admin Direct Modification**
```
PUT /api/v1/limits/agents/{id}/budget
Authorization: Bearer <admin-token>

{
  "new_budget": 150.00,
  "force": true  // Required for decreases
}
```

**Path 2: Developer Request/Approval Workflow**
```
POST /api/v1/budget-requests        (Developer creates request)
PUT /api/v1/budget-requests/{id}/approve  (Admin approves)
PUT /api/v1/budget-requests/{id}/reject   (Admin rejects)
```

**Justification:** Dual-path balances emergency flexibility (admin) with governance oversight (request/approval).

**Reference:** Protocol 013:48-200 (Budget Limits API), Protocol 017 (Budget Requests API)

---

### 3. Provider Assignment - Sub-Resource Pattern (Q25)

**Decision:** Provider assignment via sub-resource endpoints: `PUT /api/v1/agents/{id}/providers`

**Reasoning:**
- **RESTful Design:** Most RESTful pattern with clear separation of concerns
- **Explicit Intent:** Dedicated endpoints signal provider management is distinct from agent metadata
- **Granular Operations:** Supports replace-all (PUT), list (GET), and remove-single (DELETE)
- **User Preference:** User explicitly requested "Separate API to modify after creation"

**Alternatives Considered:**
- **Agent Update:** `PUT /api/v1/agents/{id}` with providers field - bundles concerns, less clear
- **Assignment Resource:** `POST /api/assignments` - unnecessary indirection, non-standard

**Implementation:**
```
PUT /api/v1/agents/{id}/providers          - Replace provider list
GET /api/v1/agents/{id}/providers          - List assigned providers
DELETE /api/v1/agents/{id}/providers/{pid} - Remove single provider
```

**Edge Cases Supported:**
- Zero providers allowed: `{"providers": []}` returns 200 OK with warning
- Duplicate providers: Automatically deduplicated
- Removing last provider: Allowed (agent continues with zero providers)

**Reference:** Protocol 010:501-878 (Provider Assignment Endpoints)

---

### 4. Zero-Provider Support (Q26)

**Decision:** Agents can exist with zero providers (minimum: 0, maximum: unlimited).

**Reasoning:**
- **Flexible Workflows:** Create agent first, configure providers later (separation of concerns)
- **Maintenance:** Temporarily remove providers during maintenance/migration without agent deletion
- **Gradual Migration:** Incrementally add/remove providers during provider transitions
- **User Confirmation:** User explicitly stated "agent can have zero IPs"

**Alternatives Considered:**
- **One Required:** Too restrictive, forces provider decision at agent creation time
- **Set at Creation:** Rigid workflow, prevents maintenance scenarios

**Implementation:**
```
POST /api/v1/agents
{
  "name": "Agent 1",
  "budget": 100.00,
  "providers": []  // Zero providers valid
}

Response: 201 Created
{
  "agent_id": "agent-abc123",
  "providers": [],
  "warning": "Agent has no providers assigned. Assign providers to enable LLM requests."
}
```

**Behavior:** Zero-provider agents return warning (not error). Agent can be used once providers assigned.

**Reference:** Protocol 010:114-162 (Agent Creation), Protocol 010:501-878 (Provider Assignment)

---

### 5. API Token Scope - SAME-AS-USER (Q15)

**Decision:** API Tokens inherit the user's role and permissions exactly (no custom scopes in Pilot).

**Reasoning:**
- **Simplicity:** Easiest model for Pilot - token = user identity, no complex permission management
- **Use Case:** Primary use case is dashboard access (long-lived sessions), not fine-grained service accounts
- **Consistency:** Token authorization identical to user session authorization (same code paths)
- **Deferral:** Fine-grained scopes (resource-specific, read-only) can be added POST-PILOT if needed

**Alternatives Considered:**
- **FINE-GRAINED:** Each token has custom permission set - complex, unnecessary for Pilot
- **READ-ONLY:** Tokens can only read - too restrictive for dashboard use case
- **RESOURCE-SPECIFIC:** Tokens scoped to specific resources - added complexity without clear benefit

**Implementation:**
```
Authorization Header:
  Bearer <api-token>

Permission Check:
  token.user.role === 'admin'  → Full access
  token.user.role === 'user'   → Own resources only
  token.user.role === 'viewer' → Read-only access
```

**Behavior:** Token permissions update if user role changes (no token regeneration required).

**Reference:** Protocol 014:1-187 (API Tokens API)

---

### 6. Token Visibility - Show Once (Q32)

**Decision:** API Token value shown ONCE at creation time, never retrievable again.

**Reasoning:**
- **Security Best Practice:** Industry standard (GitHub, GitLab, AWS, Stripe) - prevents token leakage
- **Forces Secure Storage:** Users must save token immediately (no "I'll get it later" risk)
- **No Retrieval Endpoint:** Cannot retrieve token via GET (prevents unauthorized access)
- **Lost Token Recovery:** User must revoke old token + create new token (forces proper hygiene)

**Alternatives Considered:**
- **Always Available:** `GET /api/tokens/{id}/value` endpoint - security risk, enables token theft
- **Encrypted Storage:** Decrypt on demand - still requires retrieval endpoint (same risk)

**Implementation:**

**Creation Response (Token Shown ONCE):**
```
POST /api/v1/api-tokens
{
  "name": "Dashboard Token"
}

Response: 201 Created
{
  "id": "at-abc123",
  "token": "apitok_xyz789...",  // ← Shown ONCE, never again
  "name": "Dashboard Token",
  "created_at": "2025-12-11T10:00:00Z",
  "message": "Save this token now. You won't be able to see it again."
}
```

**Get/List Responses (Token NOT Shown):**
```
GET /api/v1/api-tokens/{id}

Response: 200 OK
{
  "id": "at-abc123",
  "name": "Dashboard Token",
  "created_at": "2025-12-11T10:00:00Z",
  "last_used": "2025-12-11T14:30:00Z"
  // Note: "token" field NOT included
}
```

**Storage:** Token stored as bcrypt hash (irreversible) - cannot retrieve plaintext after creation.

**Reference:** Protocol 014:62-112 (Create Token), Protocol 014:189-228 (Get Token)

---

### 7. Analytics Use Cases - 8 Critical Questions (Q12)

**Decision:** Analytics API must answer 8 specific questions critical for budget monitoring.

**Reasoning:**
- **User-Driven:** Questions directly from user requirements (not speculative features)
- **Budget Visibility:** Pilot requires real-time budget monitoring (primary use case)
- **Actionable Insights:** Each question maps to specific operational decision
- **Minimal Scope:** 8 questions sufficient for Pilot, additional analytics deferred POST-PILOT

**8 Critical Questions:**
```
1. "What's the total spend across all agents?"
   → GET /api/v1/analytics/spending/total

2. "How much has each agent spent?"
   → GET /api/v1/analytics/spending/by-agent

3. "Which agents are near their budget limits?"
   → GET /api/v1/analytics/budget/status

4. "What's the cost breakdown by provider?"
   → GET /api/v1/analytics/spending/by-provider

5. "How many requests have been made today?"
   → GET /api/v1/analytics/usage/requests?period=today

6. "What's the token usage by agent?"
   → GET /api/v1/analytics/usage/tokens/by-agent

7. "Which models are being used most?"
   → GET /api/v1/analytics/usage/models

8. "What's the average cost per request?"
   → GET /api/v1/analytics/spending/avg-per-request
```

**Time Ranges:** Real-time + daily aggregations (period: today, yesterday, last-7-days, last-30-days, all-time)

**Deferred POST-PILOT:** Hourly, weekly, monthly aggregations, custom date ranges

**Reference:** Protocol 012:1-129 (Analytics API - 8 Endpoints)

---

### 8. Audit Logging - Mutation Only (Q35)

**Decision:** Log only state-changing operations (POST, PUT, DELETE), not read operations (GET).

**Reasoning:**
- **Compliance:** Mutation operations change system state, require audit trail for governance
- **Performance:** GET operations generate excessive log volume without compliance value
- **Storage:** 90-day retention meets standard compliance (mutation logs only)
- **Security:** Critical operations tracked (budget changes, token creation, agent deletion)

**Alternatives Considered:**
- **Full Audit:** Log all API calls including GET - excessive volume, unnecessary for compliance
- **Sensitive Only:** Log only budget/token operations - misses other important mutations (agent creation, provider updates)
- **No Audit:** Rely on application logs - insufficient for compliance and security investigations

**Logged Operations:**
- **POST** - Create (agent created, token generated, budget request submitted)
- **PUT** - Update (budget modified, request approved, provider credentials updated)
- **DELETE** - Delete (token revoked, agent deleted, provider removed)

**Not Logged:**
- **GET** - Read (too noisy, no compliance value)

**Standard Audit Log Fields:**
```
{
  "id": "audit_xyz",
  "timestamp": "2025-12-11T10:00:00Z",
  "operation": "AGENT_CREATED",
  "resource_type": "agent",
  "resource_id": "agent-abc123",
  "user_id": "user-def456",
  "user_role": "admin",
  "ip_address": "192.168.1.100",
  "request_id": "req-xyz789",
  "changes": {"budget": {"old": null, "new": 100.00}},
  "metadata": {"justification": "Emergency top-up for demo"}
}
```

**Retention:** 90 days rolling (Pilot), configurable POST-PILOT

**Reference:** Protocol 002:212-311 (Universal Audit Logging Standard)

---

### 9. CLI-API Parity - User-Facing Only (Q36)

**Decision:** 100% CLI parity for user-facing endpoints (46/46), NO CLI for agent-facing/system endpoints.

**Reasoning:**
- **Developer Experience:** CLI required for operational tasks (agent management, analytics, troubleshooting)
- **Automation:** CLI enables scripting and CI/CD integration
- **Agent-Facing Exclusion:** Budget Protocol used by iron_cage runtime (not humans), no CLI needed
- **Consistency:** All user resources have identical API + CLI interfaces

**CLI Required (46 User-Facing Endpoints):**

| Resource | API Endpoints | CLI Commands | Count |
|----------|---------------|--------------|-------|
| Agents | 8 | 8 | 8 |
| Providers | 8 | 8 | 8 |
| Analytics | 8 | 8 | 8 |
| Budget Limits | 2 | 2 | 2 |
| API Tokens | 4 | 4 | 4 |
| Projects | 2 | 2 | 2 |
| Budget Requests | 6 | 6 | 6 |
| Users | 8 | 8 | 8 |
| **Total** | **46** | **46** | **100%** |

**CLI Examples:**
```bash
iron agents list                  → GET /api/v1/agents
iron providers create             → POST /api/v1/providers
iron analytics spending by-agent  → GET /api/v1/analytics/spending/by-agent
iron api-tokens revoke {id}       → DELETE /api/v1/api-tokens/{id}
iron budget-requests approve {id} → PUT /api/v1/budget-requests/{id}/approve
```

**CLI NOT Required (Agent-Facing/System):**
- Budget Protocol (POST /api/v1/budget/*) - Agent runtime use only
- Health/Version endpoints - System monitoring
- Authentication (POST /api/auth/login) - Handled by `iron login`

**Reference:** Protocol 002:313-476 (CLI-API Parity Standard)

---

### 10. Error Format - Simple Custom (Q37)

**Decision:** Simple custom error format with `code`, `message`, and optional `field` for validation errors.

**Reasoning:**
- **Clarity:** Easy for clients to parse and handle
- **No External Dependencies:** No RFC 7807 schema requirements
- **Field-Level Errors:** Supports validation failures with specific field identification
- **Sufficient for Pilot:** Meets all error handling requirements without complexity

**Alternatives Considered:**
- **RFC 7807 Problem Details:** Industry standard but adds complexity, external schema dependency, excessive for Pilot scale
- **Nested Errors:** Overcomplicated structure, harder to parse

**Standard Error Response:**
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Budget must be at least 0.01",
    "field": "budget"
  }
}
```

**Multiple Validation Errors:**
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "budget": "Must be >= 0.01",
      "name": "Required field",
      "providers": "At least one required"
    }
  }
}
```

**Standard Error Codes:**
- `VALIDATION_ERROR` - Validation failures (400 Bad Request)
- `AUTHENTICATION_REQUIRED` - Missing/invalid auth (401 Unauthorized)
- `FORBIDDEN` - Insufficient permissions (403 Forbidden)
- `NOT_FOUND` - Resource doesn't exist (404 Not Found)
- `CONFLICT` - Resource conflict (409 Conflict)
- `RATE_LIMIT_EXCEEDED` - Rate limit hit (429 Too Many Requests)
- `INTERNAL_ERROR` - Server error (500 Internal Server Error)
- `SERVICE_UNAVAILABLE` - Service down (503 Service Unavailable)

**Reference:** Standards file `docs/standards/error_format_standards.md` (Q37-Q42 coverage)

---

### 11. HTTP Status Codes - 400 for Validation (Q38)

**Decision:** Use 400 Bad Request for validation errors (not 422 Unprocessable Entity).

**Reasoning:**
- **Simplicity:** 400 universally understood as "client error including validation"
- **Industry Standard:** GitHub, Stripe, Twilio use 400 for validation failures
- **Semantic Clarity:** 422 creates unnecessary distinction without practical benefit
- **Client Handling:** Simpler error handling (single status code for all client errors)

**Alternatives Considered:**
- **422 Unprocessable Entity:** Semantic distinction between malformed (400) vs invalid (422) - unnecessary complexity, not widely adopted

**Status Code Usage:**
```
200 OK                    - Successful GET, PUT (non-empty response)
201 Created               - Successful POST (resource created)
204 No Content            - Successful DELETE (optional, can use 200)
400 Bad Request           - Validation errors, malformed requests, invalid parameters
401 Unauthorized          - Missing or invalid authentication
403 Forbidden             - Authenticated but insufficient permissions
404 Not Found             - Resource doesn't exist
409 Conflict              - Resource conflict (duplicate, constraint violation)
429 Too Many Requests     - Rate limit exceeded (POST-PILOT)
500 Internal Server Error - Unexpected server error
503 Service Unavailable   - Service temporarily down
```

**Example - Validation Error:**
```
POST /api/v1/agents
{
  "name": "",
  "budget": -10.00
}

Response: 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "name": "Required field",
      "budget": "Must be >= 0.01"
    }
  }
}
```

**Reference:** Standards file `docs/standards/error_format_standards.md`

---

### 12. Validation - Batch All Errors (Q40)

**Decision:** Return all validation errors at once (batch validation), not fail-fast.

**Reasoning:**
- **User Experience:** Developer sees all issues in single request (no repeated trial-and-error)
- **Efficiency:** Reduces round-trips (fix all errors at once vs one-by-one)
- **Form Validation:** UI forms can highlight all invalid fields simultaneously
- **Industry Standard:** Most REST APIs use batch validation (GitHub, Stripe, AWS)

**Alternatives Considered:**
- **Fail-Fast:** Return first error only - poor developer experience, requires multiple requests to discover all issues

**Implementation:**

**Single Error:**
```
POST /api/v1/agents
{
  "name": "",
  "budget": 100.00
}

Response: 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "field": "name",
    "message": "Required field"
  }
}
```

**Multiple Errors:**
```
POST /api/v1/agents
{
  "name": "",
  "budget": -10.00,
  "providers": ["ip-invalid"]
}

Response: 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "fields": {
      "name": "Required field",
      "budget": "Must be >= 0.01",
      "providers[0]": "Provider 'ip-invalid' not found"
    }
  }
}
```

**Reference:** Standards file `docs/standards/error_format_standards.md`

---

### 13. Agent Creation - Minimal Required Fields (Q23)

**Decision:** Only `name` and `budget` required at agent creation. Project defaults to `proj-master`, providers optional (zero allowed).

**Reasoning:**
- **Simplicity:** Minimal friction for agent creation (2 required fields)
- **Single Project Pilot:** Project ID defaults to Master Project (no need to specify)
- **Flexible Workflows:** Providers can be assigned after creation (zero-provider support)
- **Owner Inference:** Owner ID inferred from auth token (authenticated user becomes owner)

**Alternatives Considered:**
- **Require Project ID:** Unnecessary in Pilot (single project only)
- **Require Providers:** Too rigid, prevents create-then-configure workflow
- **Require Owner ID:** Redundant, auth token already identifies user

**Required Fields:**
- `name` (string) - Agent identifier
- `budget` (number) - Agent budget in dollars (e.g., 100.00)

**Optional Fields:**
- `providers` (array) - Provider IDs (defaults to empty array)
- `description` (string) - Agent description
- `tags` (array) - Agent tags/labels

**Example:**
```
POST /api/v1/agents
{
  "name": "Production Agent 1",
  "budget": 100.00,
  "providers": ["ip_openai_001", "ip_anthropic_001"],
  "description": "Main production agent",
  "tags": ["production", "customer-facing"]
}

Response: 201 Created
{
  "agent_id": "agent-abc123",
  "ic_token": "ictoken_xyz789...",  // ← Shown ONCE
  "name": "Production Agent 1",
  "budget": 100.00,
  "budget_used": 0.00,
  "project_id": "proj-master",  // ← Auto-assigned
  "owner_id": "user-def456",     // ← Inferred from auth
  "providers": ["ip_openai_001", "ip_anthropic_001"],
  "created_at": "2025-12-11T10:00:00Z"
}
```

**Reference:** Protocol 010:62-112 (Agent Creation Endpoint)

---

### 14. Provider API - FULL-CRUD (Q10)

**Decision:** Providers API supports full CRUD operations (Create, List, Get, Update, Delete) in Pilot.

**Reasoning:**
- **User Requirement:** User explicitly confirmed "Both CLI and REST API should work" for provider management
- **Credentials Management:** Providers must be manageable via REST API (HTTPS encrypted) for dashboard access
- **Operational Flexibility:** Admins need ability to add/remove/update providers without database access
- **Self-Service:** Developers can manage provider credentials without admin intervention

**Alternatives Considered:**
- **READ-ONLY:** Providers configured via vault CLI only - too restrictive, prevents dashboard use case
- **PARTIAL:** Create/List only - insufficient for operational needs (can't update credentials, can't remove providers)

**Operations:**
```
POST   /api/v1/providers             - Create provider (name, URL, credentials, models)
GET    /api/v1/providers             - List providers (paginated)
GET    /api/v1/providers/{id}        - Get provider details
PUT    /api/v1/providers/{id}        - Update provider (credentials, models, URL)
DELETE /api/v1/providers/{id}        - Delete provider (soft delete with audit)
GET    /api/v1/providers/{id}/models - List provider models
PUT    /api/v1/providers/{id}/models - Update provider models
DELETE /api/v1/providers/{id}/models/{model_id} - Remove single model
```

**Authorization:**
- **Admin:** Full CRUD access (all providers)
- **User:** Read-only access (view provider list/details, no modification)

**Security:** Credentials transmitted over HTTPS, stored encrypted, never returned in GET responses (similar to API Token pattern).

**Reference:** Protocol 011 (Providers API)

---

### 15. Analytics Filtering - Agent + Provider Only (Q29)

**Decision:** Analytics endpoints support filtering by `agent_id` and `provider_id` only. Project filtering unnecessary (single project in Pilot).

**Reasoning:**
- **Common Use Cases:** "Show spending for specific agent" and "Show usage for specific provider" are primary drill-down queries
- **Pilot Simplicity:** Single project (proj-master) makes project filtering redundant
- **Deferred Complexity:** Advanced filters (threshold, date range, status) deferred POST-PILOT

**Alternatives Considered:**
- **Project Filtering:** Unnecessary (Pilot has single project only)
- **Threshold Filtering:** `?budget_percent_used_gt=80` - nice-to-have, deferred POST-PILOT
- **Date Range Filtering:** Custom ranges - use `period` parameter instead (simpler)

**Query Parameters:**
```
agent_id (string, optional)    - Filter by specific agent (e.g., agent-abc123)
provider_id (string, optional) - Filter by specific provider (e.g., ip_openai_001)
```

**Examples:**
```
GET /api/v1/analytics/spending/by-agent?agent_id=agent-abc
GET /api/v1/analytics/usage/requests?provider_id=ip-openai-001
GET /api/v1/analytics/spending/timeline?agent_id=agent-abc&provider_id=ip-openai-001
```

**Behavior:** Filters are additive (AND logic). Omitting both parameters returns data for all agents/providers.

**Reference:** Protocol 012:741-748 (Analytics Filtering)

---

### 16. Analytics Empty Results - 200 OK (Q30)

**Decision:** Analytics queries with no matching data return 200 OK with empty array (not 404 Not Found).

**Reasoning:**
- **Standard REST Practice:** Successful query with zero results is not an error
- **Client Simplicity:** No error handling required for common case (empty results)
- **Pagination Consistency:** Empty results still include pagination metadata (total: 0)
- **Semantic Correctness:** 404 implies endpoint doesn't exist (wrong), 204 loses pagination metadata (incorrect)

**Alternatives Considered:**
- **404 Not Found:** Incorrect semantics - endpoint exists, just no matching data
- **204 No Content:** Loses pagination metadata, breaks response format consistency

**Example:**
```
GET /api/v1/analytics/spending/by-agent?agent_id=nonexistent

Response: 200 OK
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

**Other Error Scenarios:**
- Invalid period: 400 Bad Request
- Invalid agent_id format: 400 Bad Request
- Query timeout (>30s): 504 Gateway Timeout
- Database unavailable: 503 Service Unavailable

**Reference:** Protocol 012:769-784 (Empty Results Handling)

---

### 17. Projects API - READ-ONLY (Q7)

**Decision:** Projects API is READ-ONLY in Pilot (List + Get only). Create/Update/Delete deferred POST-PILOT.

**Reasoning:**
- **Single Project Pilot:** Pilot uses single Master Project (proj-master), no project creation needed
- **Simplicity:** Read-only API sufficient for Pilot requirements (view project details, no management)
- **Deferred Complexity:** Full project CRUD deferred until multi-project requirements clear

**Alternatives Considered:**
- **FULL-CRUD:** Unnecessary complexity for Pilot (single project only)
- **NO-API:** Insufficient - CLI/Dashboard need to view project details

**Operations:**
```
GET /api/v1/projects        - List projects (paginated)
GET /api/v1/projects/{id}   - Get project details
```

**POST-PILOT Operations (Deferred):**
```
POST   /api/v1/projects       - Create project
PUT    /api/v1/projects/{id}  - Update project
DELETE /api/v1/projects/{id}  - Delete project
```

**Reference:** Protocol 015:1-108 (Projects API)

---

### 18. Settings API - POST-PILOT (Q6, Q18, Q19)

**Decision:** Settings API deferred to POST-PILOT. Pilot uses environment variables (.env files).

**Reasoning:**
- **Cost-Benefit:** Settings API has 48:1 cost-benefit ratio (48 days implementation vs 1 day benefit)
- **Pilot Sufficiency:** .env files sufficient for Pilot configuration
- **Hot-Reload:** Settings API requires hot-reload for value (not needed in Pilot, restart acceptable)
- **Complete Specification:** Settings API fully specified for future implementation

**Settings Requiring API (POST-PILOT):**
- Default Agent Budget
- Rate Limits (per-endpoint)
- Logging Level (debug, info, warn, error)
- Feature Flags (enable/disable features)
- User Token Lifetime (expiration duration)

**Pilot Approach:** Environment variables in .env file, service restart required for changes.

**POST-PILOT:** Settings API with hot-reload (changes take effect without restart).

**Reference:** Protocol 016 (Settings API - POST-PILOT)

---

### 19. Agent Deletion - POST-PILOT (Q22, Q24)

**Decision:** Agent deletion (DELETE /api/v1/agents/{id}) deferred to POST-PILOT.

**Reasoning:**
- **Risk:** High-risk operation that can cause data loss (budget history, request logs)
- **Unclear Requirements:** Deletion policy unclear (hard delete vs soft delete, cascade behavior)
- **Audit Trail:** Preserving audit trail critical for compliance (conflicts with hard delete)
- **Pilot Workaround:** Agents can be effectively "disabled" by removing all providers (zero-provider support)

**POST-PILOT Recommendation:** SOFT-DELETE (archive pattern)
- Preserves audit trail (compliance requirement)
- Prevents accidental data loss
- IC Token invalidated immediately (security)
- Agent marked inactive but data preserved
- Consistent with user deletion REASSIGN pattern

**Reference:** Protocol 010 (Agents API - DELETE deferred)

---

### 20. POST-PILOT Deferrals Summary

**Decision:** 12 questions deferred to POST-PILOT to meet 3-week MVP timeline.

**Deferred Features (~9 endpoints):**

**Projects API (3 endpoints):**
- POST /api/v1/projects - Create project
- PUT /api/v1/projects/{id} - Update project
- DELETE /api/v1/projects/{id} - Delete project

**Budget Limits API (3 endpoints):**
- POST /api/v1/limits/agents/{id}/daily - Set daily limits
- POST /api/v1/limits/agents/{id}/hourly - Set hourly limits
- POST /api/v1/limits/providers/{id}/budget - Set provider budgets

**API Tokens (1 endpoint):**
- POST /api/v1/api-tokens/{id}/rotate - Rotate token

**Agents API (1 endpoint):**
- DELETE /api/v1/agents/{id} - Delete agent

**Settings API (~1 endpoint):**
- Entire Settings API deferred (Protocol 016)

**Rationale:** Focus on core functionality (agent management, budget monitoring, provider configuration) for Pilot. Advanced features (project multi-tenancy, granular limits, token rotation) can wait until POST-PILOT based on user feedback.

**Reference:** `-rest_api_questions_complete.md` lines 2030-2132 (POST-PILOT Summary)

---

## Consequences

### Positive

**Developer Experience:**
- **Consistency:** Uniform pagination, error handling, and auth across all 46 endpoints
- **Familiarity:** Standard REST patterns (offset pagination, HTTP status codes) universally understood
- **CLI Parity:** 100% coverage for user-facing endpoints enables scripting and automation
- **Clear Errors:** Field-level validation errors with actionable messages

**Implementation:**
- **Simplicity:** Proven technologies (REST, offset pagination, bcrypt) reduce implementation risk
- **Pilot-Ready:** 46 endpoints cover all Pilot requirements without unnecessary complexity
- **Audit Trail:** Comprehensive logging for compliance and security investigations
- **Scalability:** Architecture supports POST-PILOT enhancements (cursor pagination, fine-grained tokens, hot-reload settings)

**Security:**
- **Token Security:** Show-once pattern prevents token leakage
- **Budget Governance:** Dual-path architecture balances emergency flexibility with oversight
- **Audit Logging:** Mutation-only logging captures all state changes with 90-day retention
- **Role-Based Access:** Admin/Owner/Viewer permissions enforced consistently

**Maintenance:**
- **Documentation:** 9 protocol files + 3 standards files provide complete API specification
- **Zero-Provider Support:** Flexible workflows reduce operational friction
- **Soft Delete Pattern:** Preserves data for compliance and recovery

### Negative

**Limitations:**
- **Offset Pagination:** May require optimization for large datasets POST-PILOT (cursor-based)
- **Token Scope:** SAME-AS-USER model lacks fine-grained permissions (acceptable for Pilot)
- **Agent Deletion:** Deferred to POST-PILOT (no deletion in Pilot)
- **Settings Management:** .env files require service restart (no hot-reload in Pilot)

**Complexity:**
- **Dual-Path Budget:** Admin direct modification + developer request/approval adds implementation complexity
- **Provider Assignment:** Sub-resource endpoints add 3 additional endpoints vs bundled agent update
- **Audit Logging:** Asynchronous writes require careful implementation to avoid blocking requests

### Mitigations

**Offset Pagination:**
- Sufficient for Pilot scale (hundreds of agents, not millions)
- Can add cursor-based pagination POST-PILOT without breaking existing clients

**Token Scope:**
- SAME-AS-USER covers 90% of Pilot use cases (dashboard, admin automation)
- Fine-grained scopes can be added POST-PILOT as optional feature

**Settings Management:**
- .env files acceptable for Pilot (infrequent configuration changes)
- Settings API fully specified (Protocol 016) for POST-PILOT implementation

---

## Related Documents

### Protocol Files (Implementation Specifications)

1. **Protocol 002:** Cross-Cutting Standards
   - Universal Pagination (Q28, Q34)
   - Audit Logging (Q35)
   - CLI-API Parity (Q36)

2. **Protocol 008:** User Management API
   - User CRUD operations
   - Role-based permissions

3. **Protocol 010:** Agents API
   - Agent creation, list, get, update, status (Q20-Q24)
   - Provider assignment endpoints (Q25-Q27)
   - Zero-provider support (Q26)

4. **Protocol 011:** Providers API
   - Full CRUD operations (Q10-Q11)
   - Credentials management

5. **Protocol 012:** Analytics API
   - 8 critical questions (Q12)
   - Pagination and filtering (Q28-Q30)

6. **Protocol 013:** Budget Limits API
   - Admin direct modification (Q16, Q33)
   - Full-mutable budget policy

7. **Protocol 014:** API Tokens API
   - Token operations (Q31)
   - Show-once pattern (Q32)
   - SAME-AS-USER scope (Q15)

8. **Protocol 015:** Projects API
   - READ-ONLY operations (Q7-Q9)

9. **Protocol 017:** Budget Requests API
   - Request/approval workflow (Q33)
   - Developer self-service

### Standards Files (Implementation Guidance)

1. **error_format_standards.md** - Error response format (Q37-Q42)
2. **data_format_standards.md** - Data formats and constraints (Q43-Q48)
3. **id_format_standards.md** - ID prefixes and validation (Q45)

### Validation Reports (Quality Assurance)

1. **-rest_api_completion_report.md** - Comprehensive completion tracking
2. **-cross_reference_validation_report.md** - Protocol reference accuracy
3. **-endpoint_count_analysis.md** - Endpoint inventory and categorization

---

**Document Version:** 1.0.0
**Authors:** System Design Team
**Review Status:** Accepted
**Implementation Status:** Documented (61/61 decisions), Protocols Complete (9/9 files)
