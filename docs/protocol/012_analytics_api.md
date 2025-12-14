# Protocol: Analytics API

### Scope

This protocol defines the HTTP API for analytics event ingestion and query endpoints providing insights into agent spending, provider usage, budget status, and request patterns.

**In scope**:
- Event ingestion endpoint (POST /api/v1/analytics/events) for LlmRouter reporting with IC token authentication
- 8 query endpoints answering critical questions (total spend, spend by agent, budget status, spend by provider, request count, token usage, model usage, average cost)
- Period-based filtering (today, yesterday, last-7-days, last-30-days, all-time) across all query endpoints
- Authorization model (users see own data, admins see all data) with role-based access control
- Event deduplication using per-agent event_id (UNIQUE(agent_id, event_id)) preventing duplicate analytics
- Async event processing (202 Accepted) with non-blocking ingestion for minimal latency impact
- Cost tracking in microdollars (1 USD = 1,000,000 microdollars) for precision
- Pagination (offset-based, default 50 items/page, max 100) and filtering (agent_id, provider_id)

**Out of scope**:
- Event storage implementation details (see Database Schema reference in Event Ingestion section)
- Real-time streaming analytics (current implementation query-based)
- Custom aggregation periods beyond 5 standard periods (Future Enhancement)
- Event ingestion from sources other than LlmRouter (Future Enhancement)
- Historical data export/archival (Future Enhancement)
- Budget enforcement logic (see Protocol 005: Budget Control Protocol)
- Agent management (see Protocol 010: Agents API)
- Provider management (see Protocol 011: Providers API)

### Purpose

**User Need**: Developers need real-time visibility into agent LLM spending, token usage, and budget status to optimize costs and prevent budget exhaustion. Admins need aggregated analytics across all agents to understand provider cost breakdown, identify high-spend agents, and forecast capacity needs. Both need historical trend analysis (today, yesterday, last-7-days, last-30-days, all-time) and risk alerts when agents approach budget limits.

**Solution**: This API provides event-based analytics with dual functionality: async event ingestion (POST /api/v1/analytics/events) for LlmRouter to report completed/failed LLM requests without blocking request latency, and 8 query endpoints delivering answers to critical questions about spending, usage, and budget status. Events use per-agent deduplication (UNIQUE(agent_id, event_id)) enabling idempotent retries while cost tracking uses microdollars (1 USD = 1,000,000) for precision. Period-based queries (today through all-time) with pagination and filtering (agent_id, provider_id) enable both real-time monitoring and historical analysis. Authorization separates user data (own agents only) from admin data (all agents) while rate limiting (20 req/min queries, 1000 events/min) protects database performance.

**Key Insight**: Analytics requires event ingestion decoupled from query performance. By accepting events asynchronously (202 Accepted) and processing them in background, we eliminate analytics overhead from critical LLM request path. Per-agent event_id deduplication (not global) enables safe retries without duplicate charges while maintaining event namespace separation. The 8 query endpoints directly answer the questions developers ask ("How much have I spent?", "Which agents are near budget limits?", "What's the most expensive model?") rather than exposing raw events, optimizing for human decision-making over system flexibility.

---

**Status**: Specification
**Version**: 1.7.0
**Last Updated**: 2025-12-14
**Priority**: MUST-HAVE

### Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- Analytics API uses short alphanumeric IDs for provider (IP Token) and agent identifiers to optimize performance, readability, and operational clarity
- `provider_id`: `ip_<name>_<numeric>` for IP Token identifiers with regex `^ip_[a-z0-9-]+_[0-9]{3}$` (e.g., `ip_openai_001`, `ip_anthropic_001`)
- `agent_id`: `agent_<alphanumeric>` with regex `^agent_[a-z0-9]{6,32}$` (e.g., `agent_abc123`)
- `user_id`: `user_<uuid>` for cross-system compatibility

**Data Format Standards** ([data_format_standards.md](../standards/data_format_standards.md))
- Currency amounts: Decimal with exactly 2 decimal places (e.g., `100.00`)
- Timestamps: ISO 8601 with Z suffix (e.g., `2025-12-10T10:30:45.123Z`)
- Date ranges: `today`, `yesterday`, `last-7-days`, `last-30-days`, `all-time`
- Counts: Integer (e.g., `1500`)

**Error Format Standards** ([error_format_standards.md](../standards/error_format_standards.md))
- Consistent error response structure across all endpoints
- Machine-readable error codes: `VALIDATION_ERROR`, `UNAUTHORIZED`, `NOT_FOUND`
- HTTP status codes: 200, 400, 401, 403, 404

**API Design Standards** ([api_design_standards.md](../standards/api_design_standards.md))
- Pagination: Offset-based with `?page=N&per_page=M` (default 50 items/page)
- Filtering: Query parameters for `agent_id`, `provider_id`, `period`
- URL structure: `/api/v1/analytics/*`


### Use Cases

The Analytics API answers 8 critical questions:

1. **Total Spend:** "What's the total spend across all agents?"
2. **Spend by Agent:** "How much has each agent spent?"
3. **Budget Status:** "Which agents are near their budget limits?"
4. **Spend by Provider:** "What's the cost breakdown by provider?"
5. **Request Count:** "How many requests have been made today?"
6. **Token Usage:** "What's the token usage by agent?"
7. **Model Usage:** "Which models are being used most?"
8. **Average Cost:** "What's the average cost per request?"


### Event Ingestion

LlmRouter reports analytics events after each LLM request. Events are sent asynchronously (non-blocking) to avoid impacting request latency.

#### POST /api/v1/analytics/events

**Description:** Report LLM request events from LlmRouter to server.

**Use Case:** Record successful/failed LLM requests for analytics and dashboard.

**Request:**

```http
POST /api/v1/analytics/events
Content-Type: application/json
```

**Request Body:**

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

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `ic_token` | string | YES | IC Token for authentication - agent_id derived from token claims |
| `event_id` | string | YES | UUID for deduplication, unique per agent (`evt_<uuid>`) |
| `timestamp_ms` | integer | YES | Unix timestamp in milliseconds |
| `event_type` | string | YES | `llm_request_completed` or `llm_request_failed` |
| `model` | string | YES | Model name (e.g., `gpt-4o-mini`, `claude-3-opus`) |
| `provider` | string | YES | Provider: `openai`, `anthropic`, `unknown` |
| `input_tokens` | integer | YES* | Input token count (*required for completed) |
| `output_tokens` | integer | YES* | Output token count (*required for completed) |
| `cost_micros` | integer | YES* | Cost in microdollars (1 USD = 1,000,000) |
| `provider_id` | string | NO | Provider key identifier (optional) |
| `error_code` | string | NO | Error code (for failed events) |
| `error_message` | string | NO | Error message (for failed events) |

**Note:** `agent_id` is automatically extracted from the IC Token claims (format: `agent_<id>`).

**Event Types:**

| Type | Description | Required Fields |
|------|-------------|-----------------|
| `llm_request_completed` | Successful LLM request | `input_tokens`, `output_tokens`, `cost_micros` |
| `llm_request_failed` | Failed LLM request | `error_code`, `error_message` |

**Success Response:**

```json
HTTP 202 Accepted
Content-Type: application/json

{
  "event_id": "evt_7c9e6679-7425-40de-944b",
  "status": "accepted"
}
```

**Duplicate Response (idempotent):**

```json
HTTP 200 OK
Content-Type: application/json

{
  "event_id": "evt_7c9e6679-7425-40de-944b",
  "status": "duplicate"
}
```

**Error Response:**

```json
HTTP 400 Bad Request
Content-Type: application/json

{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid event_type",
    "details": {
      "field": "event_type",
      "allowed": ["llm_request_completed", "llm_request_failed"]
    }
  }
}
```

**Behavior:**

- **Async processing:** Server accepts immediately (202), processes asynchronously
- **Idempotent:** Duplicate `(agent_id, event_id)` returns 200 (not error)
- **Per-agent deduplication:** Same `event_id` from different agents are distinct events
- **Non-blocking:** LlmRouter sends fire-and-forget, doesn't wait for response
- **Retry:** On network failure, LlmRouter retries with exponential backoff

**Storage:**

All events stored in single shared `analytics_events` table:

```
┌─────────────────────────────────────────────────────────────┐
│                     analytics_events                         │
├─────────────────────────────────────────────────────────────┤
│ id │ event_id │ agent_id │ model     │ cost_micros │ ...   │
├────┼──────────┼──────────┼───────────┼─────────────┼───────┤
│ 1  │ evt_001  │ 42       │ gpt-4o    │ 1250        │       │
│ 2  │ evt_002  │ 42       │ gpt-4o    │ 800         │       │
│ 3  │ evt_001  │ 99       │ claude    │ 2100        │ ✓     │ ← same event_id, different agent
│ 4  │ evt_003  │ 42       │ gpt-4o    │ 500         │       │
└─────────────────────────────────────────────────────────────┘
        ↑
   UNIQUE(agent_id, event_id)  ← per-agent deduplication
```

**Deduplication examples:**

```
Agent 42 sends: { ic_token: "<token_for_agent_42>", event_id: "evt_001" } → 202 Accepted
Agent 99 sends: { ic_token: "<token_for_agent_99>", event_id: "evt_001" } → 202 Accepted (different agent)
Agent 42 sends: { ic_token: "<token_for_agent_42>", event_id: "evt_001" } → 200 Duplicate (same agent+event)
```

Note: `agent_id` is extracted from IC token claims, not from request body.

**Cost Units:**

Cost is reported in **microdollars** (1 USD = 1,000,000 microdollars) for precision:

```
$0.001250 USD = 1250 microdollars
$1.00 USD = 1,000,000 microdollars
```

**Example: Failed Request**

```json
{
  "ic_token": "<ic_token>",
  "event_id": "evt_8d0f7780-8536-51ef-955c",
  "timestamp_ms": 1733830246456,
  "event_type": "llm_request_failed",
  "model": "gpt-4o-mini",
  "provider": "openai",
  "provider_id": "ip_openai-001",
  "error_code": "rate_limit_exceeded",
  "error_message": "Rate limit exceeded. Please retry after 60 seconds."
}
```

**Authorization:**

- Requires valid IC Token in request body
- `agent_id` extracted from token claims (format: `agent_<id>`)
- Events are associated with token's agent

**Rate Limiting:**

| Limit | Value | Window |
|-------|-------|--------|
| Events per token | 1000 | 1 minute |


### Query Endpoints

#### 1. Total Spending

**Endpoint:** `GET /api/v1/analytics/spending/total`

**Description:** Returns total spending across all agents (or filtered subset).

**Use Case:** "What's the total spend across all agents?"

**Request:**

```
GET /api/v1/analytics/spending/total?period=today&agent_id=agent-abc123
Authorization: Bearer <user-token or api-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `period` | string | `all-time` | Time range: `today`, `yesterday`, `last-7-days`, `last-30-days`, `all-time` |
| `agent_id` | integer | - | Filter by specific agent (optional) |
| `provider_id` | string | - | Filter by specific provider (optional) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "total_spend": 1245.67,
  "currency": "USD",
  "period": "today",
  "filters": {
    "agent_id": "agent_abc123",
    "provider_id": null
  },
  "calculated_at": "2025-12-10T15:30:00Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `total_spend` | number | Total spending in USD (2 decimal places) |
| `currency` | string | Currency code (always "USD" in Pilot) |
| `period` | string | Time range used |
| `filters` | object | Applied filters (null if not used) |
| `calculated_at` | string | ISO 8601 timestamp of calculation |

**Authorization:**
- **User:** See own agents' spending
- **Admin:** See all agents' spending


#### 2. Spending by Agent

**Endpoint:** `GET /api/v1/analytics/spending/by-agent`

**Description:** Returns spending breakdown by agent, sorted by spend descending.

**Use Case:** "How much has each agent spent?"

**Request:**

```
GET /api/v1/analytics/spending/by-agent?period=last-7-days&page=1&per_page=50
Authorization: Bearer <user-token or api-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `period` | string | `all-time` | Time range |
| `agent_id` | integer | - | Filter by specific agent (returns single result) |
| `provider_id` | string | - | Filter by specific provider |
| `page` | integer | 1 | Page number |
| `per_page` | integer | 50 | Results per page (max 100) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "data": [
    {
      "agent_id": "agent_abc123",
      "agent_name": "Production Agent 1",
      "spending": 456.78,
      "budget": 1000.00,
      "percent_used": 45.68,
      "request_count": 2341
    },
    {
      "agent_id": "agent_def456",
      "agent_name": "Test Agent",
      "spending": 234.56,
      "budget": 500.00,
      "percent_used": 46.91,
      "request_count": 1205
    }
  ],
  "summary": {
    "total_spend": 691.34,
    "total_budget": 1500.00,
    "average_percent_used": 46.09
  },
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 2,
    "total_pages": 1
  },
  "period": "last-7-days",
  "calculated_at": "2025-12-10T15:30:00Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `data[]` | array | Agent spending records (sorted by spending desc) |
| `data[].agent_id` | integer | Agent identifier |
| `data[].agent_name` | string | Agent name |
| `data[].spending` | number | Amount spent (USD) |
| `data[].budget` | number | Total budget (USD) |
| `data[].percent_used` | number | Budget utilization (0-100) |
| `data[].request_count` | integer | Number of requests |
| `summary` | object | Aggregate statistics |
| `summary.total_spend` | number | Sum of all spending |
| `summary.total_budget` | number | Sum of all budgets |
| `summary.average_percent_used` | number | Average budget utilization |
| `pagination` | object | Pagination metadata |

**Authorization:**
- **User:** See own agents only
- **Admin:** See all agents


#### 3. Budget Status

**Endpoint:** `GET /api/v1/analytics/budget/status`

**Description:** Returns budget status for all agents, highlighting those near limits. Useful for monitoring and alerts.

**Use Case:** "Which agents are near their budget limits?"

**Request:**

```
GET /api/v1/analytics/budget/status?threshold=80&page=1&per_page=50
Authorization: Bearer <user-token or api-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `threshold` | integer | - | Filter agents above threshold (e.g., 80 = show agents with >80% budget used) |
| `agent_id` | integer | - | Filter by specific agent |
| `status` | string | - | Filter by status: `active`, `exhausted`, `inactive` |
| `page` | integer | 1 | Page number |
| `per_page` | integer | 50 | Results per page (max 100) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "data": [
    {
      "agent_id": "agent_abc123",
      "agent_name": "Production Agent 1",
      "budget": 1000.00,
      "spent": 956.78,
      "remaining": 43.22,
      "percent_used": 95.68,
      "status": "active",
      "risk_level": "critical"
    },
    {
      "agent_id": "agent_def456",
      "agent_name": "Test Agent",
      "budget": 500.00,
      "spent": 434.56,
      "remaining": 65.44,
      "percent_used": 86.91,
      "status": "active",
      "risk_level": "high"
    },
    {
      "agent_id": "agent_ghi789",
      "agent_name": "Dev Agent",
      "budget": 100.00,
      "spent": 100.00,
      "remaining": 0.00,
      "percent_used": 100.00,
      "status": "exhausted",
      "risk_level": "exhausted"
    }
  ],
  "summary": {
    "total_agents": 3,
    "active": 2,
    "exhausted": 1,
    "critical": 1,
    "high": 1,
    "medium": 0,
    "low": 0
  },
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 3,
    "total_pages": 1
  },
  "calculated_at": "2025-12-10T15:30:00Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `data[].risk_level` | string | Risk category: `low` (<50%), `medium` (50-79%), `high` (80-94%), `critical` (95-99%), `exhausted` (100%) |
| `summary` | object | Risk distribution summary |
| `summary.total_agents` | integer | Total agents in result set |
| `summary.active` | integer | Agents with remaining budget |
| `summary.exhausted` | integer | Agents with $0.00 remaining |
| `summary.critical` | integer | Agents at 95-99% budget usage |
| `summary.high` | integer | Agents at 80-94% budget usage |
| `summary.medium` | integer | Agents at 50-79% budget usage |
| `summary.low` | integer | Agents at <50% budget usage |

**Authorization:**
- **User:** See own agents only
- **Admin:** See all agents


#### 4. Spending by Provider

**Endpoint:** `GET /api/v1/analytics/spending/by-provider`

**Description:** Returns spending breakdown by provider, showing which providers cost most.

**Use Case:** "What's the cost breakdown by provider?"

**Request:**

```
GET /api/v1/analytics/spending/by-provider?period=last-30-days&page=1&per_page=50
Authorization: Bearer <user-token or api-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `period` | string | `all-time` | Time range |
| `agent_id` | integer | - | Filter by specific agent |
| `provider_id` | string | - | Filter by specific provider (returns single result) |
| `page` | integer | 1 | Page number |
| `per_page` | integer | 50 | Results per page (max 100) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "data": [
    {
      "provider_id": "ip_openai-001",
      "provider_name": "openai",
      "spending": 789.45,
      "request_count": 12456,
      "avg_cost_per_request": 0.0634,
      "agent_count": 8
    },
    {
      "provider_id": "ip_anthropic-001",
      "provider_name": "anthropic",
      "spending": 456.22,
      "request_count": 9123,
      "avg_cost_per_request": 0.0500,
      "agent_count": 5
    }
  ],
  "summary": {
    "total_spend": 1245.67,
    "total_requests": 21579,
    "average_cost_per_request": 0.0577
  },
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 2,
    "total_pages": 1
  },
  "period": "last-30-days",
  "calculated_at": "2025-12-10T15:30:00Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `data[]` | array | Provider spending records (sorted by spending desc) |
| `data[].provider_id` | string | Provider identifier |
| `data[].provider_name` | string | Provider name |
| `data[].spending` | number | Amount spent (USD) |
| `data[].request_count` | integer | Number of requests |
| `data[].avg_cost_per_request` | number | Average cost per request (USD, 4 decimal places) |
| `data[].agent_count` | integer | Number of agents using provider |
| `summary` | object | Aggregate statistics |

**Authorization:**
- **User:** See own usage only
- **Admin:** See all usage


#### 5. Request Usage

**Endpoint:** `GET /api/v1/analytics/usage/requests`

**Description:** Returns request count statistics by time period.

**Use Case:** "How many requests have been made today?"

**Request:**

```
GET /api/v1/analytics/usage/requests?period=today&agent_id=agent-abc123
Authorization: Bearer <user-token or api-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `period` | string | `today` | Time range |
| `agent_id` | integer | - | Filter by specific agent |
| `provider_id` | string | - | Filter by specific provider |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "total_requests": 892,
  "successful_requests": 870,
  "failed_requests": 22,
  "success_rate": 97.53,
  "period": "today",
  "filters": {
    "agent_id": "agent_abc123",
    "provider_id": null
  },
  "calculated_at": "2025-12-10T15:30:00Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `total_requests` | integer | Total number of requests |
| `successful_requests` | integer | Requests with 2xx status |
| `failed_requests` | integer | Requests with 4xx/5xx status |
| `success_rate` | number | Success percentage (0-100, 2 decimal places) |
| `period` | string | Time range |
| `filters` | object | Applied filters |

**Authorization:**
- **User:** See own data only
- **Admin:** See all data


#### 6. Token Usage by Agent

**Endpoint:** `GET /api/v1/analytics/usage/tokens/by-agent`

**Description:** Returns token usage breakdown by agent.

**Use Case:** "What's the token usage by agent?"

**Request:**

```
GET /api/v1/analytics/usage/tokens/by-agent?period=last-7-days&page=1&per_page=50
Authorization: Bearer <user-token or api-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `period` | string | `all-time` | Time range |
| `agent_id` | integer | - | Filter by specific agent |
| `provider_id` | string | - | Filter by specific provider |
| `page` | integer | 1 | Page number |
| `per_page` | integer | 50 | Results per page (max 100) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "data": [
    {
      "agent_id": "agent_abc123",
      "agent_name": "Production Agent 1",
      "input_tokens": 1234567,
      "output_tokens": 567890,
      "total_tokens": 1802457,
      "request_count": 2341,
      "avg_tokens_per_request": 770
    },
    {
      "agent_id": "agent_def456",
      "agent_name": "Test Agent",
      "input_tokens": 789012,
      "output_tokens": 345678,
      "total_tokens": 1134690,
      "request_count": 1205,
      "avg_tokens_per_request": 941
    }
  ],
  "summary": {
    "total_input_tokens": 2023579,
    "total_output_tokens": 913568,
    "total_tokens": 2937147,
    "total_requests": 3546,
    "average_tokens_per_request": 828
  },
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 2,
    "total_pages": 1
  },
  "period": "last-7-days",
  "calculated_at": "2025-12-10T15:30:00Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `data[].input_tokens` | integer | Total input tokens |
| `data[].output_tokens` | integer | Total output tokens |
| `data[].total_tokens` | integer | Sum of input + output tokens |
| `data[].request_count` | integer | Number of requests |
| `data[].avg_tokens_per_request` | integer | Average total tokens per request |
| `summary` | object | Aggregate token statistics |

**Authorization:**
- **User:** See own agents only
- **Admin:** See all agents


#### 7. Model Usage

**Endpoint:** `GET /api/v1/analytics/usage/models`

**Description:** Returns usage statistics by model, showing which models are used most.

**Use Case:** "Which models are being used most?"

**Request:**

```
GET /api/v1/analytics/usage/models?period=last-30-days&page=1&per_page=50
Authorization: Bearer <user-token or api-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `period` | string | `all-time` | Time range |
| `agent_id` | integer | - | Filter by specific agent |
| `provider_id` | string | - | Filter by specific provider |
| `page` | integer | 1 | Page number |
| `per_page` | integer | 50 | Results per page (max 100) |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "data": [
    {
      "model": "gpt-4",
      "provider_id": "ip_openai-001",
      "provider_name": "openai",
      "request_count": 8945,
      "spending": 567.89,
      "input_tokens": 1456789,
      "output_tokens": 678901,
      "total_tokens": 2135690,
      "avg_cost_per_request": 0.0635
    },
    {
      "model": "claude-3-opus",
      "provider_id": "ip_anthropic-001",
      "provider_name": "anthropic",
      "request_count": 5234,
      "spending": 345.67,
      "input_tokens": 923456,
      "output_tokens": 456789,
      "total_tokens": 1380245,
      "avg_cost_per_request": 0.0660
    }
  ],
  "summary": {
    "total_requests": 14179,
    "total_spend": 913.56,
    "total_tokens": 3515935,
    "unique_models": 2
  },
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 2,
    "total_pages": 1
  },
  "period": "last-30-days",
  "calculated_at": "2025-12-10T15:30:00Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `data[]` | array | Model usage records (sorted by request_count desc) |
| `data[].model` | string | Model identifier (e.g., "gpt-4", "claude-3-opus") |
| `data[].provider_id` | string | Provider identifier |
| `data[].provider_name` | string | Provider name |
| `data[].request_count` | integer | Number of requests to this model |
| `data[].spending` | number | Total spending on this model (USD) |
| `data[].input_tokens` | integer | Total input tokens |
| `data[].output_tokens` | integer | Total output tokens |
| `data[].total_tokens` | integer | Total tokens (input + output) |
| `data[].avg_cost_per_request` | number | Average cost per request (USD, 4 decimal places) |
| `summary.unique_models` | integer | Number of distinct models used |

**Authorization:**
- **User:** See own usage only
- **Admin:** See all usage


#### 8. Average Cost Per Request

**Endpoint:** `GET /api/v1/analytics/spending/avg-per-request`

**Description:** Returns average cost per request statistics, useful for capacity planning.

**Use Case:** "What's the average cost per request?"

**Request:**

```
GET /api/v1/analytics/spending/avg-per-request?period=last-30-days&agent_id=agent-abc123
Authorization: Bearer <user-token or api-token>
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `period` | string | `all-time` | Time range |
| `agent_id` | integer | - | Filter by specific agent |
| `provider_id` | string | - | Filter by specific provider |

**Success Response:**

```json
HTTP 200 OK
Content-Type: application/json

{
  "average_cost_per_request": 0.0577,
  "total_requests": 21579,
  "total_spend": 1245.67,
  "median_cost_per_request": 0.0534,
  "min_cost_per_request": 0.0012,
  "max_cost_per_request": 0.2345,
  "period": "last-30-days",
  "filters": {
    "agent_id": "agent_abc123",
    "provider_id": null
  },
  "calculated_at": "2025-12-10T15:30:00Z"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `average_cost_per_request` | number | Mean cost per request (USD, 4 decimal places) |
| `total_requests` | integer | Number of requests in calculation |
| `total_spend` | number | Total spending (USD) |
| `median_cost_per_request` | number | Median cost per request (USD, 4 decimal places) |
| `min_cost_per_request` | number | Minimum cost per request (USD, 4 decimal places) |
| `max_cost_per_request` | number | Maximum cost per request (USD, 4 decimal places) |
| `period` | string | Time range |
| `filters` | object | Applied filters |

**Authorization:**
- **User:** See own data only
- **Admin:** See all data


### Common Parameters

#### Period Values

| Value | Description | Time Range |
|-------|-------------|------------|
| `today` | Today's data | 00:00:00 UTC today to now |
| `yesterday` | Yesterday's data | 00:00:00 to 23:59:59 UTC yesterday |
| `last-7-days` | Last 7 days | Last 7 complete days + today |
| `last-30-days` | Last 30 days | Last 30 complete days + today |
| `all-time` | Since agent creation | From first request to now |

#### Pagination

All list endpoints support pagination:

| Parameter | Type | Default | Max | Description |
|-----------|------|---------|-----|-------------|
| `page` | integer | 1 | - | Page number (1-indexed) |
| `per_page` | integer | 50 | 100 | Results per page |

**Response structure:**

```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 125,
    "total_pages": 3
  }
}
```

#### Filtering

| Parameter | Type | Description |
|-----------|------|-------------|
| `agent_id` | integer | Filter by specific agent |
| `provider_id` | string | Filter by specific provider |

**Note:** Filters are optional. Omitting filters returns data for all accessible agents/providers (based on user authorization).


### Error Handling

#### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Invalid query parameters |
| `INVALID_PERIOD` | 400 | Invalid period value |
| `UNAUTHORIZED` | 401 | Missing/invalid authentication |
| `TOKEN_EXPIRED` | 401 | Authentication token expired |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `AGENT_NOT_FOUND` | 404 | Agent does not exist (when filtering by agent_id) |
| `PROVIDER_NOT_FOUND` | 404 | Provider does not exist (when filtering by provider_id) |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `QUERY_TIMEOUT` | 504 | Query took too long (>30 seconds) |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

#### Empty Results

Analytics endpoints return 200 with empty data (not 404):

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


### Rate Limiting

#### Limits (per user)

| Endpoint | Limit | Window |
|----------|-------|--------|
| All analytics endpoints | 20 | 1 minute |

**Reasoning:** Analytics queries can be expensive. 20 req/min = polling every 3 seconds, sufficient for dashboards.


### Performance

#### Query Optimization

- **Real-time data:** Calculated from live database (no caching)
- **Daily aggregations:** Pre-calculated nightly, fast queries
- **Indexes:** Optimized for agent_id, provider_id, timestamp filters
- **Query timeout:** 30 seconds (returns 504 if exceeded)

#### Response Times

| Endpoint | Target | Max |
|----------|--------|-----|
| Total spending | <100ms | 1s |
| Spending by agent | <200ms | 2s |
| Budget status | <200ms | 2s |
| Spending by provider | <200ms | 2s |
| Request usage | <100ms | 1s |
| Token usage | <300ms | 3s |
| Model usage | <300ms | 3s |
| Average cost | <150ms | 1.5s |

**Note:** Times for user queries (10-50 agents). Admin queries (1000+ agents) may be slower.


### CLI Integration

#### iron analytics spending

```bash
# Total spend
iron analytics spending total
iron analytics spending total --period today
iron analytics spending total --agent agent-abc123

# By agent
iron analytics spending by-agent
iron analytics spending by-agent --period last-7-days
iron analytics spending by-agent --provider ip_openai_001

# By provider
iron analytics spending by-provider
iron analytics spending by-provider --period last-30-days
iron analytics spending by-provider --agent agent-abc123

# Average cost
iron analytics spending avg-per-request
iron analytics spending avg-per-request --period today
```

#### iron analytics usage

```bash
# Requests
iron analytics usage requests
iron analytics usage requests --period today
iron analytics usage requests --agent agent-abc123

# Tokens
iron analytics usage tokens by-agent
iron analytics usage tokens by-agent --period last-7-days

# Models
iron analytics usage models
iron analytics usage models --period last-30-days
iron analytics usage models --provider ip_openai_001
```

#### iron analytics budget

```bash
# Budget status
iron analytics budget status
iron analytics budget status --threshold 80
iron analytics budget status --status active

# Output:
# AGENT                 BUDGET    SPENT     REMAINING  USED    RISK
# agent-abc123 (Prod 1) $1000.00  $956.78   $43.22     95.68%  CRITICAL
# agent-def456 (Test)   $500.00   $434.56   $65.44     86.91%  HIGH
# agent-ghi789 (Dev)    $100.00   $100.00   $0.00      100.00% EXHAUSTED
#
# Summary: 3 agents (2 active, 1 exhausted, 1 critical, 1 high)
```


### Usage vs Analytics Endpoints

The Iron Cage API provides two distinct endpoint families for tracking usage and analytics. They serve different purposes and use different data sources.

#### `/api/usage/*` - Project-Level Usage Tracking - Project-Level Usage Tracking

**Purpose:** Simple token usage tracking at project/provider level

**Data Source:** `UsageTracker` from `iron_token_manager` crate

**Endpoints:**
- `GET /api/usage/aggregate` - Aggregate usage across all projects
- `GET /api/usage/by-project/:project_id` - Usage for specific project
- `GET /api/usage/by-provider/:provider` - Usage for specific provider

**Returns:**
- `total_tokens` - Token count
- `total_requests` - Request count
- `total_cost_cents` - Cost in cents
- `providers[]` - Provider breakdown

**Authentication:** None visible (basic usage tracking)

**Use Case:** Basic project-level usage reporting and cost tracking for API token usage

#### `/api/v1/analytics/*` - Comprehensive Analytics Platform - Comprehensive Analytics Platform

**Purpose:** Full-featured analytics with spending, budgets, agent breakdowns, and event ingestion

**Data Source:** `analytics_events` table (event-based analytics)

**Endpoints:**
- `POST /api/v1/analytics/events` - Event ingestion (IC token auth)
- `GET /api/v1/analytics/spending/*` - Spending analytics (5 endpoints)
- `GET /api/v1/analytics/budget/status` - Budget monitoring
- `GET /api/v1/analytics/usage/requests` - Request statistics
- `GET /api/v1/analytics/usage/tokens/by-agent` - Token usage by agent
- `GET /api/v1/analytics/usage/models` - Model usage

**Returns:**
- Cost in USD (converted from microdollars)
- Period filters (`today`, `yesterday`, `last-7-days`, `last-30-days`, `all-time`)
- Pagination support
- Budget status and risk levels
- Agent-level breakdowns

**Authentication:** JWT authentication (user tokens) for queries, IC tokens for event ingestion

**Use Case:** Dashboard analytics, budget monitoring, spending analysis, comprehensive reporting

#### Key Differences

| Aspect | `/api/usage/*` | `/api/v1/analytics/*` |
|--------|----------------|----------------------|
| **Granularity** | Project-level | Agent-level |
| **Data Source** | UsageTracker | analytics_events table |
| **Authentication** | None | JWT / IC tokens |
| **Features** | Basic stats | Period filters, pagination, budgets |
| **Cost Units** | Cents | USD (from microdollars) |
| **Purpose** | Simple reporting | Comprehensive analytics |

**When to Use:**
- Use `/api/usage/*` for simple project-level usage tracking
- Use `/api/v1/analytics/*` for dashboard analytics, budget monitoring, and detailed reporting


### Cross-References

#### Related Principles Documents

None.

#### Related Architecture Documents

None.

#### Used By

None currently. Analytics data consumed by dashboard UI (not yet documented).

#### Dependencies

- Protocol 002: REST API Protocol - General REST API standards and conventions
- Protocol 005: Budget Control Protocol - Budget limits and enforcement referenced in budget status queries
- Protocol 010: Agents API - Agent entity for analytics event association and filtering
- Protocol 011: Providers API - Provider entity for analytics event attribution and cost breakdown

#### Implementation

- `/home/user1/pro/lib/wip_iron/iron_runtime/dev/module/iron_control_api/src/routes/analytics/` - Analytics API endpoint handlers
- `/home/user1/pro/lib/wip_iron/iron_runtime/dev/module/iron_runtime_analytics/` - Analytics event processing and storage
