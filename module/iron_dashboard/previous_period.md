# `previous_period` — Endpoints & Logic

## Endpoints that support `previous_period`

Both are `GET` requests that accept the shared `AnalyticsQuery` parameters.

| Endpoint | Response type | Comparison struct |
|---|---|---|
| `GET /api/v1/analytics/spending/total` | `SpendingTotalResponse` | `SpendingTotalComparison` |
| `GET /api/v1/analytics/usage/requests` | `RequestUsageResponse` | `RequestUsageComparison` |

---

## How to request it

Add `compare=true` to the query string:

```
GET /api/v1/analytics/spending/total?period=last7-days&compare=true
GET /api/v1/analytics/usage/requests?period=last7-days&compare=true
```

`compare` defaults to `false`. When `false` (or absent), the `previous_period` key is **omitted entirely** from the response (`skip_serializing_if = "Option::is_none"`).

---

## Response shape

### `SpendingTotalResponse.previous_period` (`SpendingTotalComparison`)

```json
{
  "previous_period": {
    "total_spend": 30.0,
    "change_percent": 41.67
  }
}
```

| Field | Type | Description |
|---|---|---|
| `total_spend` | `f64` | Total spending in the previous period (USD) |
| `change_percent` | `f64 \| null` | `((current - prev) / prev) * 100`. `null` when previous spend is zero |

---

### `RequestUsageResponse.previous_period` (`RequestUsageComparison`)

```json
{
  "previous_period": {
    "total_requests": 80,
    "successful_requests": 75,
    "failed_requests": 5,
    "success_rate": 0.9375,
    "change_percent": 25.0
  }
}
```

| Field | Type | Description |
|---|---|---|
| `total_requests` | `i64` | Total requests in the previous period |
| `successful_requests` | `i64` | Successful requests in the previous period |
| `failed_requests` | `i64` | Failed requests in the previous period |
| `success_rate` | `f64` | Success rate in the previous period (0.0–1.0) |
| `change_percent` | `f64 \| null` | `((current - prev) / prev) * 100`. `null` when previous total is zero |

---

## How the previous period range is calculated

Implemented in `Period::previous_period_range()` (`shared.rs`).
Returns a `(start_ms, end_ms)` Unix-millisecond pair, or `None` for `all-time`.

| `period` param | Previous period window |
|---|---|
| `today` | Yesterday — midnight to midnight (= same range as `yesterday`) |
| `yesterday` | Day before yesterday — midnight to midnight |
| `last7-days` | `[now − 14d, now − 7d)` — the 7-day block immediately before the current one |
| `last30-days` | `[now − 60d, now − 30d)` — the 30-day block immediately before the current one |
| `this-month` | The full previous calendar month (1st 00:00 → 1st 00:00 of current month − 1 ms) |
| `last-month` | The calendar month before last month |
| `all-time` | **Not supported** — `previous_period` is always `null` for this period |

> All ranges are end-exclusive in spirit: `end_ms = boundary_ms - 1` so there is no overlap with the current period.
