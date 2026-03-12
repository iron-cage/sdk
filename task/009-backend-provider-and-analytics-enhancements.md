# Task 009: Backend Provider & Analytics Enhancements

## Dependencies
- None blocking

## Context

Bundle of backend changes needed to support the updated frontend (provider key management, agent multi-key assignment, analytics improvements).

Critical files:
- `module/iron_control_api/src/routes/providers.rs`
- `module/iron_control_api/src/routes/agents.rs`
- `module/iron_control_api/src/routes/analytics/spending.rs`
- `module/iron_control_api/src/routes/analytics/usage.rs`
- `module/iron_control_api/src/routes/analytics/ingestion.rs`
- `module/iron_control_api/src/routes/analytics/shared.rs`
- `module/iron_token_manager/src/provider_key_storage.rs`
- `module/iron_token_manager/migrations/`

---

## Task 1: Provider Key Alias

**What:** Add optional `alias` (friendly name) field to provider keys.

**Why:** Users want to name keys (e.g. "Production", "Team A key") for display instead of just provider type.

### Changes

**Migration** (`029_add_provider_key_alias.sql`):
```sql
ALTER TABLE ai_provider_keys ADD COLUMN alias TEXT;
```

**Storage** (`provider_key_storage.rs`):
- Add `alias: Option<String>` to ProviderKey struct
- Include `alias` in SELECT, INSERT, UPDATE queries

**API** (`providers.rs`):
- `POST /api/v1/provider-keys` — accept `alias` in request body
- `PUT /api/v1/provider-keys/:id` — accept `alias` in request body
- `GET /api/v1/provider-keys` — include `alias` in response

---

## Task 2: Multiple Provider Keys per Agent

**What:** Allow an agent to be associated with multiple provider keys (one per provider type) via a join table.

**Why:** Agents may call OpenAI for some requests and Anthropic for others. Currently only one `provider_key_id` is stored per agent.

### Changes

**Migration** (`030_create_agent_provider_keys.sql`):
```sql
CREATE TABLE agent_provider_keys (
    agent_id        INTEGER NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    provider_key_id INTEGER NOT NULL REFERENCES ai_provider_keys(id) ON DELETE CASCADE,
    PRIMARY KEY (agent_id, provider_key_id)
);

-- Migrate existing data
INSERT INTO agent_provider_keys (agent_id, provider_key_id)
SELECT id, provider_key_id FROM agents WHERE provider_key_id IS NOT NULL;
```

**Agents struct/storage:**
- Replace `provider_key_id: Option<i64>` with `provider_key_ids: Vec<i64>`
- List: query `agent_provider_keys` join table per agent
- Create/Update: insert/replace rows in join table

**API** (`agents.rs`):
- `POST /api/v1/agents` — accept `provider_key_ids: Vec<i64>`
- `PUT /api/v1/agents/:id` — accept `provider_key_ids: Vec<i64>`
- `GET /api/v1/agents` — return `provider_key_ids: number[]`

**Routing** (`agent_provider_key.rs`):
- Resolve which key to use based on requested model's provider type

---

## Task 3: Provider Key Spend in List Response

**What:** `GET /api/v1/provider-keys` includes `total_spend_usd` per key.

**Why:** Frontend "Spend (all-time)" column currently requires separate parallel API calls per key.

### Changes

**Storage** (`provider_key_storage.rs`):
- Add `total_spend_usd: f64` to ProviderKey response struct
- Update list query with LEFT JOIN:
```sql
SELECT pk.*, COALESCE(SUM(ae.cost_micros), 0) / 1000000.0 as total_spend_usd
FROM ai_provider_keys pk
LEFT JOIN analytics_events ae ON ae.provider_key_id = pk.id
  AND ae.event_type = 'llm_request_completed'
WHERE pk.user_id = ?
GROUP BY pk.id
```

---

## Task 4: Analytics `by-provider` — Group by Provider Key ID

**What:** `GET /api/v1/analytics/spending/by-provider` optionally groups by `provider_key_id`.

**Why:** With multiple keys per provider type, aggregating by provider name loses granularity.

### Changes

**Query params** (`shared.rs`):
- Add `group_by: Option<String>` to `AnalyticsQuery` (accepted value: `"key"`)

**Handler** (`spending.rs` — `get_spending_by_provider`):
- When `group_by=key`: GROUP BY `provider_key_id`, join `ai_provider_keys` for alias
- Response includes `provider_key_id: Option<i64>` and `alias: Option<String>`

---

## Task 5: Median Cost in avg-per-request

**What:** `GET /api/v1/analytics/spending/avg-per-request` returns `median_cost_per_request`.

**Why:** Average alone is skewed by outliers; median gives a better sense of typical request cost.

### Changes

**Response struct** (`shared.rs` — `AvgCostResponse`):
- Add `median_cost_per_request: f64`

**Handler** (`spending.rs` — `get_spending_avg`):
- Fetch ordered `cost_micros` values and compute median in Rust (SQLite lacks native median)
- Alternative: use SQLite window function with `PERCENTILE_CONT` if extension available

---

## Task 6: Add Missing `provider_key_id` and `provider_id` Filters to Analytics Endpoints

**What:** Most analytics endpoints parse `provider_key_id` and `provider_id` from query params but silently ignore them in SQL. One endpoint (`events/list`) doesn't even accept the params.

**Why:** Frontend filters by provider key and provider type, but the API returns unfiltered data. E.g. filtering by-agent to "only Anthropic spending" has no effect.

### Current filter status

| Endpoint | `provider_id` in SQL | `provider_key_id` in SQL |
|---|---|---|
| `spending/by-provider` | ❌ ignored | ✅ works |
| `spending/by-agent` | ❌ ignored | ❌ ignored |
| `spending/avg-per-request` | ✅ works | ✅ works |
| `usage/models` | ❌ ignored | ❌ ignored |
| `usage/tokens/by-agent` | ❌ ignored | ❌ ignored |
| `events/list` | ❌ not in struct | ❌ not in struct |

### Changes needed

**Endpoints that already parse params via `AnalyticsQuery` — add SQL filters:**

1. **`spending/by-provider`** (`spending.rs` — `get_spending_by_provider`)
   - Add `AND provider = ?` when `provider_id` is present

2. **`spending/by-agent`** (`spending.rs` — `get_spending_by_agent`)
   - Add `AND provider = ?` when `provider_id` is present
   - Add `AND provider_key_id = ?` when `provider_key_id` is present

3. **`usage/models`** (`usage.rs` — `get_usage_models`)
   - Add `AND provider = ?` when `provider_id` is present
   - Add `AND provider_key_id = ?` when `provider_key_id` is present

4. **`usage/tokens/by-agent`** (`usage.rs` — `get_usage_tokens`)
   - Add `AND provider = ?` when `provider_id` is present
   - Add `AND provider_key_id = ?` when `provider_key_id` is present

**Endpoint needing params added to struct + SQL filters:**

5. **`events/list`** (`ingestion.rs` — `list_events`)
   - Add `provider_id: Option<String>` to `EventsListQuery` struct
   - Add `provider_key_id: Option<i64>` to `EventsListQuery` struct
   - Add both SQL filters in query builder

### Already working (no changes needed)

- `spending/avg-per-request` — ✅ both `provider_id` and `provider_key_id` filters work
- `spending/by-provider` — ✅ `provider_key_id` filter works (only `provider_id` missing)

---

## Acceptance Criteria

- [x] Provider keys support optional `alias` in CRUD and list responses
- [x] Agents support multiple provider key IDs via join table; routing resolves correct key by provider type
- [x] Provider key list includes `total_spend_usd` per key
- [x] `spending/by-provider` supports `group_by=key` returning per-key breakdown with alias
- [x] `avg-per-request` returns `median_cost_per_request`
- [x] All 6 analytics endpoints filter by `provider_key_id` when provided
- [x] All 6 analytics endpoints filter by `provider_id` when provided
- [x] Existing tests pass; new tests cover multi-key agents and analytics filters

---

## E2E Tests

### `analytics_spending_e2e_tests.rs`

Tests for analytics spending endpoint fixes (provider filter, median, group-by-key).

| Test | Validates | Endpoint |
|---|---|---|
| `test_spending_total_filters_by_provider` | `provider_id` query param correctly filters by `provider` column (not `provider_id`) | `GET /api/v1/analytics/spending/total` |
| `test_spending_avg_includes_median` | `median_cost_per_request` field is returned with correct value (odd count) | `GET /api/v1/analytics/spending/avg-per-request` |
| `test_spending_avg_median_with_provider_filter` | Median calculation respects `provider_id` filter using correct column | `GET /api/v1/analytics/spending/avg-per-request` |
| `test_spending_by_provider_group_by_key` | `group_by=key` returns per-key breakdown with `provider_key_id` and `alias` | `GET /api/v1/analytics/spending/by-provider` |
| `test_spending_by_agent_with_provider_key_filter` | `provider_key_id` filter correctly scopes spending by agent | `GET /api/v1/analytics/spending/by-agent` |

### `agents_transaction_e2e_tests.rs`

Tests for agent multi-key support, transaction atomicity, and response correctness.

| Test | Validates | Endpoint |
|---|---|---|
| `test_create_agent_with_multiple_keys` | Creating agent with `provider_key_ids: [1,2,3]` populates join table and budget | `POST /api/agents` |
| `test_create_agent_atomic_rollback` | Invalid key ID (99999) rolls back entire transaction — no orphaned agent | `POST /api/agents` |
| `test_update_agent_replaces_provider_keys` | Updating `provider_key_ids` replaces old join rows atomically | `PUT /api/agents/{id}` |
| `test_update_agent_keys_atomic` | Invalid key ID in update rolls back — original keys preserved | `PUT /api/agents/{id}` |
| `test_list_agents_includes_provider_key_ids` | List response includes `provider_key_ids` array per agent | `GET /api/agents` |
| `test_get_agent_includes_provider_key_ids` | Single agent response includes `provider_key_ids` array | `GET /api/agents/{id}` |

### Review Bug Fixes Covered

| Bug | Fix | Test Coverage |
|---|---|---|
| Wrong SQL column `provider_id` instead of `provider` in 3 spending functions | Changed to `AND provider = ?` | `test_spending_total_filters_by_provider`, `test_spending_avg_median_with_provider_filter` |
| `compute_median` loaded all rows into memory | Two-step COUNT + LIMIT/OFFSET approach | `test_spending_avg_includes_median` |
| Missing transactions in `create_agent` / `update_agent` | Wrapped in `pool.begin()` / `tx.commit()` | `test_create_agent_atomic_rollback`, `test_update_agent_keys_atomic` |
| `LIMIT 1` without `ORDER BY` in agent key lookup | Added `ORDER BY apk.provider_key_id ASC` | Deterministic behavior tested via `test_create_agent_with_multiple_keys` |
| Silent `unwrap_or_default()` on DB errors (12 instances) | Replaced with `unwrap_or_else` + `tracing::warn!`/`error!` | Logging verified at code level |