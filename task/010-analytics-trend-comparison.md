# Task: Analytics Trend Comparison (Backend)

## Goal
Add period-over-period comparison to analytics endpoints so the frontend can show trend indicators like "$12.40 +34% vs last period" on summary cards.

## What needs to be done

- Add an optional `compare=true` query param to spending/total and usage/requests endpoints
- When `compare=true`, the response should include a `previous_period` block with the same fields calculated for the preceding period (e.g. if current period is "last 7 days", previous period is the 7 days before that)
- Return a `change_percent` field so the frontend doesn't have to calculate it

## Example response shape

```json
{
  "total_spend": 12.40,
  "previous_period": {
    "total_spend": 9.20,
    "change_percent": 34.8
  }
}
```

## Notes
- "all-time" period has no meaningful previous period — return null for previous_period in that case
- "yesterday" previous period = the day before yesterday
- "today" previous period = yesterday
- "last 7 days" previous period = 7 days before that window
- "last 30 days" previous period = 30 days before that window
- "this month" previous period = last month

## Related
- Depends on: —
- Required by: task/010-analytics-ui-refactor.md (trend indicators on summary cards)