# Protocol 012: Analytics API

**Status:** Specification
**Version:** 1.5.0
**Last Updated:** 2025-12-11

---

## Overview

The Analytics API provides insights into agent spending, provider usage, budget status, and request patterns. Supports both event ingestion from LlmRouter and query endpoints for dashboard display.

**Key characteristics:**
- **Event ingestion:** LlmRouter reports events via POST (async, non-blocking)
- **8 critical use cases:** Answers essential questions about spending, usage, and budget status
- **Real-time + daily aggregations:** Supports today, yesterday, last-7-days, last-30-days, all-time
- **Pagination:** Consistent offset pagination across all endpoints
- **Filtering:** By agent_id and provider_id
- **Authorization:** Users see own data, admins see all data
- **Multi-agent:** Single database stores events from all agents, filtered by `agent_id`

---

## Standards Compliance

This protocol adheres to the following Iron Cage standards:

**ID Format Standards** ([id_format_standards.md](../standards/id_format_standards.md))
- All entity IDs use `prefix_uuid` format with underscore separator
- `agent_id`: `agent_<uuid>`
- `provider_id`: `provider_<uuid>`

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

---

## Use Cases

The Analytics API answers 8 critical questions:

1. **Total Spend:** "What's the total spend across all agents?"
2. **Spend by Agent:** "How much has each agent spent?"
3. **Budget Status:** "Which agents are near their budget limits?"
4. **Spend by Provider:** "What's the cost breakdown by provider?"
5. **Request Count:** "How many requests have been made today?"
6. **Token Usage:** "What's the token usage by agent?"
7. **Model Usage:** "Which models are being used most?"
8. **Average Cost:** "What's the average cost per request?"

---

## Event Ingestion

LlmRouter reports analytics events after each LLM request. Events are sent asynchronously (non-blocking) to avoid impacting request latency.

### POST /api/v1/analytics/events

**Description:** Report LLM request events from LlmRouter to server.

**Use Case:** Record successful/failed LLM requests for analytics and dashboard.

**Request:**

```http
POST /api/v1/analytics/events
Content-Type: application/json
Authorization: Bearer <ic_token>
```

**Request Body:**

```json
{
  "event_id": "evt_7c9e6679-7425-40de-944b",
  "timestamp_ms": 1733830245123,
  "event_type": "llm_request_completed",
  "model": "gpt-4o-mini",
  "provider": "openai",
  "input_tokens": 150,
  "output_tokens": 50,
  "cost_micros": 1250,
  "agent_id": "agent_abc123",
  "provider_id": "ip_openai-001"
}
```

**Request Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `event_id` | string | YES | UUID for deduplication, unique per agent (`evt_<uuid>`) |
| `timestamp_ms` | integer | YES | Unix timestamp in milliseconds |
| `event_type` | string | YES | `llm_request_completed` or `llm_request_failed` |
| `model` | string | YES | Model name (e.g., `gpt-4o-mini`, `claude-3-opus`) |
| `provider` | string | YES | Provider: `openai`, `anthropic`, `unknown` |
| `input_tokens` | integer | YES* | Input token count (*required for completed) |
| `output_tokens` | integer | YES* | Output token count (*required for completed) |
| `cost_micros` | integer | YES* | Cost in microdollars (1 USD = 1,000,000) |
| `agent_id` | string | NO | Agent identifier (optional) |
| `provider_id` | string | NO | Provider key identifier (optional) |
| `error_code` | string | NO | Error code (for failed events) |
| `error_message` | string | NO | Error message (for failed events) |

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
- **Idempotent:** Duplicate `event_id` returns 200 (not error)
- **Non-blocking:** LlmRouter sends fire-and-forget, doesn't wait for response
- **Retry:** On network failure, LlmRouter retries with exponential backoff

**Cost Units:**

Cost is reported in **microdollars** (1 USD = 1,000,000 microdollars) for precision:

```
$0.001250 USD = 1250 microdollars
$1.00 USD = 1,000,000 microdollars
```

**Example: Failed Request**

```json
{
  "event_id": "evt_8d0f7780-8536-51ef-955c",
  "timestamp_ms": 1733830246456,
  "event_type": "llm_request_failed",
  "model": "gpt-4o-mini",
  "provider": "openai",
  "agent_id": "agent_abc123",
  "provider_id": "ip_openai-001",
  "error_code": "rate_limit_exceeded",
  "error_message": "Rate limit exceeded. Please retry after 60 seconds."
}
```

**Authorization:**

- Requires valid IC Token
- Events are associated with token's user

**Rate Limiting:**

| Limit | Value | Window |
|-------|-------|--------|
| Events per token | 1000 | 1 minute |

---

## Query Endpoints

### 1. Total Spending

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
| `agent_id` | string | - | Filter by specific agent (optional) |
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

---

### 2. Spending by Agent

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
| `agent_id` | string | - | Filter by specific agent (returns single result) |
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
| `data[].agent_id` | string | Agent identifier |
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

---

### 3. Budget Status

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
| `agent_id` | string | - | Filter by specific agent |
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

---

### 4. Spending by Provider

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
| `agent_id` | string | - | Filter by specific agent |
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

---

### 5. Request Usage

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
| `agent_id` | string | - | Filter by specific agent |
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

---

### 6. Token Usage by Agent

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
| `agent_id` | string | - | Filter by specific agent |
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

---

### 7. Model Usage

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
| `agent_id` | string | - | Filter by specific agent |
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

---

### 8. Average Cost Per Request

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
| `agent_id` | string | - | Filter by specific agent |
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

---

## Common Parameters

### Period Values

| Value | Description | Time Range |
|-------|-------------|------------|
| `today` | Today's data | 00:00:00 UTC today to now |
| `yesterday` | Yesterday's data | 00:00:00 to 23:59:59 UTC yesterday |
| `last-7-days` | Last 7 days | Last 7 complete days + today |
| `last-30-days` | Last 30 days | Last 30 complete days + today |
| `all-time` | Since agent creation | From first request to now |

### Pagination

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

### Filtering

| Parameter | Type | Description |
|-----------|------|-------------|
| `agent_id` | string | Filter by specific agent |
| `provider_id` | string | Filter by specific provider |

**Note:** Filters are optional. Omitting filters returns data for all accessible agents/providers (based on user authorization).

---

## Error Handling

### Error Codes

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

### Empty Results

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

---

## Rate Limiting

### Limits (per user)

| Endpoint | Limit | Window |
|----------|-------|--------|
| All analytics endpoints | 20 | 1 minute |

**Reasoning:** Analytics queries can be expensive. 20 req/min = polling every 3 seconds, sufficient for dashboards.

---

## Performance

### Query Optimization

- **Real-time data:** Calculated from live database (no caching)
- **Daily aggregations:** Pre-calculated nightly, fast queries
- **Indexes:** Optimized for agent_id, provider_id, timestamp filters
- **Query timeout:** 30 seconds (returns 504 if exceeded)

### Response Times

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

---

## CLI Integration

### iron analytics spending

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

### iron analytics usage

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

### iron analytics budget

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

---

## References

**Related Protocols:**
- [010: Agents API](010_agents_api.md) - Agent management
- [011: Providers API](011_providers_api.md) - Provider management
- [005: Budget Control Protocol](005_budget_control_protocol.md) - Budget enforcement
- [002: REST API Protocol](002_rest_api_protocol.md) - General standards

---

**Protocol 012 Version:** 1.5.0
**Status:** Specification
**Last Updated:** 2025-12-12
