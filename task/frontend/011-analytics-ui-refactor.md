# Task: Analytics UI Refactor

## Goal
Refactor the analytics UI to better utilize existing backend endpoints, surface unused data, and add new sections for spending by agent and cost efficiency.

## Relevant Files
- `module/iron_dashboard/src/views/UsageView.vue` — main analytics page
- `module/iron_dashboard/src/views/DashboardView.vue` — summary dashboard
- `module/iron_dashboard/src/views/LimitsView.vue` — agent budget table
- `module/iron_dashboard/src/views/BudgetRequestsView.vue` — budget request workflow
- `module/iron_dashboard/src/composables/useApi.ts` — all API methods and types
- `module/iron_control_api/src/routes/analytics/` — backend handlers
- `docs/protocol/012_analytics_api.md` — backend API spec

## Backend Endpoints (Protocol 012)

### Currently used by frontend
| Endpoint | Used in |
|---|---|
| `GET /api/v1/analytics/spending/total` | UsageView, DashboardView |
| `GET /api/v1/analytics/spending/by-provider` | UsageView |
| `GET /api/v1/analytics/usage/requests` | UsageView, DashboardView |
| `GET /api/v1/analytics/usage/models` | UsageView |
| `GET /api/v1/analytics/events/list` | UsageView |
| `GET /api/v1/analytics/budget/status` | LimitsView |

### NOT used by frontend (need client methods)
| Endpoint | Returns |
|---|---|
| `GET /api/v1/analytics/spending/by-agent` | Per-agent: spending, budget, % used, request_count + summary totals |
| `GET /api/v1/analytics/spending/avg-per-request` | avg/median/min/max cost per request |
| `GET /api/v1/analytics/usage/tokens/by-agent` | input/output/total tokens per agent |

## Gaps Found

### 1. Missing API methods in useApi.ts
- No `getAnalyticsSpendingByAgent()` method
- No `getAnalyticsSpendingAvgPerRequest()` method
- No response types: `SpendingByAgentResponse`, `AgentSpending`, `AvgCostResponse`
- Bug: `getAnalyticsSpendingByProvider()` accepts `AnalyticsFilters` but silently drops `provider_id` (never appended to params, line 514-516)

### 2. Parameters supported by backend but never sent
| Call | Missing param |
|---|---|
| `getAnalyticsSpendingByProvider` | `provider_id` (also missing from method impl) |
| `getAnalyticsUsageModels` | `provider_id`, pagination not used in UsageView |
| `getAnalyticsEventsList` | `provider_id` |
| `getBudgetStatus` | `status`, `threshold`, pagination not used |
| `listBudgetRequests` | `status` filtered client-side only, not sent to backend |

### 3. Response fields returned but ignored
| Endpoint | Ignored fields |
|---|---|
| `spending/by-provider` | `avg_cost_per_request`, `agent_count`, `summary` |
| `usage/models` | `provider` column, `summary`, `pagination` |
| `events/list` | `provider`, `error_code`, `error_message`, `agent_id` |
| `budget/status` | `status`, `risk_level`, full `summary` breakdown (critical/high/medium/low counts) |
| `usage/requests` | `failed_requests` (only success rate shown) |

## What needs to be done

- Wire up the missing API methods for spending by agent and avg cost per request — the backend already has these endpoints, frontend just never calls them
- Add a provider filter dropdown and a model filter to the analytics page so users can slice the data further
- Add a "Spending by Agent" table so you can see how much each agent is spending, their budget usage, and risk level at a glance
- Add an avg cost per request card — something like "$0.006 avg, median $0.005" — so users understand efficiency not just totals
- Show failed request count on the success rate card instead of hiding it
- Show error details (error code, message) in the event log for failed requests
- Add a provider column to the model breakdown table
- Add cost per 1k tokens to the model breakdown table — this can be calculated from existing data (spending / total tokens × 1000), no backend changes needed
- Add trend indicators to the summary cards showing % change vs the previous period (e.g. "$12.40 +34% vs last period") — this requires backend support: each analytics endpoint needs to return both current and previous period values, or a separate comparison endpoint

## Backend work needed

- Trend comparison — tracked in task/010-analytics-trend-comparison.md

## Notes
- All amounts from backend are USD floats (already converted from microdollars server-side), except `cost_micros` in events which uses `formatMicrodollars()`
- DashboardView hardcodes `period: 'all-time'` — intentional, leave as-is
- Backend auth: admins see all agents, regular users see only their own — agent dropdown already handles this via `getAgents()`
- Model filter has no backend param — must be client-side filtering of already-fetched data
- `spending/by-agent` params: `period`, `agent_id`, `provider_id`, `page`, `per_page`
- `spending/avg-per-request` params: `period`, `agent_id`, `provider_id`