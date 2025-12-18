# analytics/ - Analytics REST API Endpoints (Protocol 012)

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `budget.rs` | Budget status monitoring endpoint |
| `ingestion.rs` | Event ingestion and listing from LlmRouter |
| `mod.rs` | Export analytics module public API |
| `shared.rs` | Common types and response structures |
| `spending.rs` | Spending analytics endpoints (4 handlers) |
| `usage.rs` | Usage statistics endpoints (3 handlers) |

## Directory Purpose

Analytics REST API implementation (Protocol 012). Provides event ingestion, spending analytics, budget monitoring, and usage statistics. All costs use microdollars (1 USD = 1,000,000 microdollars) for precision.

## Endpoints

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| POST | `/api/v1/analytics/events` | `post_event` | Ingest analytics event from LlmRouter |
| GET | `/api/v1/analytics/events/list` | `list_events` | List analytics events with pagination |
| GET | `/api/v1/analytics/spending/total` | `get_spending_total` | Total spending (filterable by agent/period) |
| GET | `/api/v1/analytics/spending/by-agent` | `get_spending_by_agent` | Spending breakdown by agent |
| GET | `/api/v1/analytics/spending/by-provider` | `get_spending_by_provider` | Spending breakdown by provider |
| GET | `/api/v1/analytics/spending/avg-per-request` | `get_spending_avg` | Average cost per request |
| GET | `/api/v1/analytics/budget/status` | `get_budget_status` | Budget status for all agents |
| GET | `/api/v1/analytics/usage/requests` | `get_usage_requests` | Request success/failure stats |
| GET | `/api/v1/analytics/usage/tokens/by-agent` | `get_usage_tokens` | Token usage by agent |
| GET | `/api/v1/analytics/usage/models` | `get_usage_models` | Usage breakdown by model |
