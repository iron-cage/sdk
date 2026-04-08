# iron_control_api - Specification

**Module:** iron_control_api  
**Layer:** 5 (Integration)  
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

## Responsibility

REST + WebSocket control surface for Iron Cage. Exposes token/usage/budget/analytics endpoints, validates IC Tokens and JWT/RBAC, enforces agent token rules, and streams real-time events to the dashboard.

## Scope

**In Scope**

- REST API for tokens, usage limits, traces, auth handshake, and user management
- Budget Control (Protocol 005) handshake/report/return/refresh flows
- Analytics ingress and queries (Protocol 012) for spending/usage
- Agent token enforcement to protect credential endpoints
- WebSocket broadcasting of agent/runtime events for dashboards
- Authentication/authorization (IC Token validation, JWT, RBAC)

**Out of Scope**

- UI components (see iron_dashboard)
- Token generation/rotation logic (see iron_token_manager)
- Budget price computation (see iron_cost)
- Runtime execution and LLM routing (see iron_runtime)

## Dependencies

**Required Modules:** iron_token_manager, iron_runtime_state, iron_telemetry, iron_cost, iron_secrets  
**External:** axum/axum-extra, tower/tower-http, tokio, serde/serde_json, jsonwebtoken, sqlx (sqlite), reqwest, aes-gcm, bcrypt, tracing  
**Features:** `enabled` (default core), `full` (alias)

## Core Concepts

- **REST Router:** HTTP surface for tokens, limits, usage, auth, and admin operations.
- **Budget Control Router:** Protocol 005 endpoints for lease handshake, usage reporting, refresh/return of budgets.
- **Analytics Router:** Protocol 012 ingestion and spending/usage queries.
- **Agent Token Enforcement:** Guards provider-key endpoints so agents only access their assigned keys.
- **WebSocket Broadcaster:** Pushes live agent/runtime events to dashboard clients.
- **Auth Middleware:** IC Token + JWT validation with role-based access controls.

## IC Token Runtime Validation

IC Tokens authenticate agents at every runtime endpoint (budget handshake/report/return/refresh, analytics ingestion, provider-key fetch). Validation runs in four ordered layers:

1. **Rate limiting** — per-token-hash, sliding window (20 failures / 60 s). Checked first, before any cryptographic work, to bound brute-force cost. Keyed by the first 128 bits of the token's SHA-256 hash (not per-IP) because runtime endpoints lack socket address context and per-token targeting is more precise against timing attacks.
2. **JWT verification + claim validation** — HMAC-SHA256 signature check, issuer ("iron-control-panel"), optional `exp` field. Custom validation disables the default `exp`-required rule so long-lived tokens (null expiry) are valid by design.
3. **Database hash-check** — the raw token's SHA-256 hash is compared against `agents.ic_token_hash` using constant-time comparison. This layer enforces revocation (NULL hash) and rotation (hash mismatch) that JWT signature alone cannot express.
4. **Timing normalization** — all validation paths (success, auth error, rate-limit) are padded to a 50 ms floor so external observers cannot distinguish error types by response latency.

**Why hash-check in addition to JWT signature?**
JWT signature validity is a static property of the token string. Revocation and rotation are stateful events. Without the hash-check, a leaked or superseded token remains valid until its `exp` — potentially forever for long-lived tokens.

**JTI uniqueness guarantee**
Each `IcTokenClaims::new()` embeds a UUID v4 as `jti` (RFC 7519 §4.1.7). This ensures that two tokens issued in the same second with identical claims produce different SHA-256 hashes, so regeneration always invalidates the old token. Without `jti`, same-second regeneration would produce a hash collision that silently preserved the old token.

**HMAC secret protection**
`IcTokenManager` uses a manual `Debug` implementation that substitutes the `secret` field with `"<redacted>"`. The auto-derived `Debug` would expose the HMAC-SHA256 signing key in tracing output, panic messages, and test logs.

## Provider Key Management

Users with the `ManageProviderKeys` permission can create and manage AI provider keys (OpenAI, Anthropic, Gemini, xAI). Keys are stored AES-GCM encrypted; the API never returns the plaintext key — only a masked representation.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/providers` | Create a new provider key |
| `GET` | `/api/v1/providers` | List the caller's provider keys |
| `GET` | `/api/v1/providers/:id` | Get details for a specific key |
| `PUT` | `/api/v1/providers/:id` | Update key fields |
| `DELETE` | `/api/v1/providers/:id` | Delete a key |
| `POST` | `/api/v1/providers/:id/projects/:project_id` | Assign key to a project |
| `DELETE` | `/api/v1/providers/projects/:project_id` | Unassign key from a project |

All endpoints require a valid JWT with the `ManageProviderKeys` RBAC permission. Ownership is always verified: all responses return `404` for missing or foreign keys to avoid leaking key existence.

### Create Key — `POST /api/v1/providers`

**Request body:**

```json
{
  "provider": "openai",          // required: "openai" | "anthropic" | "gemini" | "xai"
  "api_key": "sk-...",           // required, 1–500 chars, no NULL bytes
  "base_url": "https://...",     // optional, HTTPS only, max 2000 chars
  "description": "...",          // optional, max 500 chars, no NULL bytes
  "spending_cap_usd": 100.0      // optional, 0–1,000,000 USD, finite, non-negative
}
```

**Response:** `201 Created` with a `ProviderKeyResponse` body. The `spending_cap_usd` field is set atomically at creation if provided, eliminating any uncapped window before a subsequent `PUT`.

**Quota:** maximum 20 keys per user per provider; exceeding returns `429`.

**Crypto prerequisite:** `IRON_SECRETS_MASTER_KEY` must be set; if absent, the endpoint returns `503 Service Unavailable` (feature is transient, not permanently unimplemented — an operator can re-enable it by supplying the key).

### List Keys — `GET /api/v1/providers`

Returns all keys owned by the authenticated user. `masked_key` is always `"***"` in list responses. Project assignments are fetched in a single bulk query and joined to each key.

### Get Key — `GET /api/v1/providers/:id`

Returns a single key with its assigned projects. Returns `404` if the key does not exist or belongs to another user.

### Update Key — `PUT /api/v1/providers/:id`

**Request body** (all fields optional):

```json
{
  "description": null | "new text",   // absent = skip; null = clear; string = set
  "base_url": "https://..." | "",     // absent/omit = skip; "" = revert to provider default; string = set (HTTPS only)
  "is_enabled": true | false,         // absent = skip
  "spending_cap_usd": null | 50.0     // absent = skip; null = remove cap; number = set cap
}
```

**Breaking change:** `description` and `spending_cap_usd` are `Option<Option<T>>` with `#[serde(default)]`. This is a 3-way null semantic:

- Field absent from JSON → skip (do not change)
- Field present as `null` → clear the value (set DB column to NULL)
- Field present as a value → update to that value

Note: `base_url` uses a different convention — an empty string `""` clears it (reverts to the provider default endpoint), while `null`/absent means skip. This asymmetry exists because `base_url` is a URL field where NULL has a natural "use default" meaning.

**Returns:** `200 OK` with the updated `ProviderKeyResponse`.

### Delete Key — `DELETE /api/v1/providers/:id`

Verifies ownership then deletes. Returns `204 No Content` on success, `404` if not found or not owned by caller.

### Project Assignment

**Assign:** `POST /api/v1/providers/:id/projects/:project_id` — assigns key `:id` to project `:project_id`. Both key and project ownership are verified. Assignment history is append-only; the most-recently inserted row (highest `assigned_at`) is the active key for the project.

**Unassign:** `DELETE /api/v1/providers/projects/:project_id` — removes the active assignment. The key owner is verified before removal. Returns `404` consistently (never `403`) to avoid leaking key or project existence.

### ProviderKeyResponse shape

```json
{
  "id": 42,
  "provider": "openai",
  "base_url": null,
  "description": "My key",
  "is_enabled": true,
  "created_at": 1700000000000,
  "last_used_at": null,
  "masked_key": "sk-...abc",
  "assigned_projects": ["proj_abc"],
  "spending_cap_usd": 100.0,      // omitted if no cap set
  "spending_used_usd": 3.14
}
```

### Input Validation

| Field | Constraint |
|-------|-----------|
| `provider` | Must be one of `"openai"`, `"anthropic"`, `"gemini"`, `"xai"` |
| `api_key` | Non-empty, max 500 chars, no NULL bytes |
| `base_url` | HTTPS scheme required if non-empty, max 2000 chars, no NULL bytes |
| `description` | Max 500 chars, no NULL bytes |
| `spending_cap_usd` | Finite, non-negative, max $1,000,000 |

## Budget Handshake — Multi-Key Changes

The `POST /api/budget/handshake` endpoint (Protocol 005) has been updated to support agent-assigned provider keys with full ownership validation and spending-cap enforcement.

### Key Resolution

Provider key selection now follows a strict ownership-scoped model:

1. **Explicit key (`provider_key_id` in request body):** The key must exist and belong to the agent's owner. An early ownership pre-check (using `get_key_metadata`) rejects obviously unauthorized keys before any budget work. A TOCTOU-safe re-validation is then performed on the freshly-fetched full key record after all other checks.

2. **Agent-assigned key (no `provider_key_id` in request):** The `agents.provider_key_id` column is read. If `NULL`, the handshake is rejected with `403 NO_PROVIDER_ASSIGNED`. There is no global or "first available" fallback.

3. **Dev-mode auto-creation (`IRON_ALLOW_DEV_KEYS`):** When the environment variable `IRON_ALLOW_DEV_KEYS` is set, agent_1 with no assigned key is permitted to auto-create a placeholder dev key. This is a development-only bypass that panics the server on startup in production environments (see Startup Guards below).

### Validation Order (before budget reservation)

All of the following are checked before any budget is reserved, ensuring no budget is ever depleted for an invalid key:

1. IC Token validation (JWT + hash-check)
2. Ownership pre-check on key metadata
3. Full key record fetch
4. TOCTOU ownership re-check on freshly-fetched record
5. Provider type match (`key_record.metadata.provider == requested_provider`)
6. Key enabled check (`is_enabled == true`)
7. Agent budget `check_and_reserve_budget` (atomic, TOCTOU-safe)
8. Provider key `reserve_spending` (atomic cap enforcement)

### Spending Cap Enforcement

After agent budget is reserved, `reserve_spending` atomically checks the provider key's `spending_cap_microdollars` and increments the used amount. If the cap would be exceeded, it returns `TokenError::SpendingCapExceeded`, the agent budget reservation is immediately refunded, and the handshake returns `403 PROVIDER_KEY_SPENDING_CAP_EXCEEDED`.

### `refund_reservations` Helper

A new `refund_reservations` helper centralizes error-path cleanup for the multi-step reservation sequence. It:

1. Calls `restore_reserved_budget` to reverse the agent budget debit.
2. Calls `adjust_spending(key_id, amount, 0)` to reverse the provider key spending reservation.
3. Optionally reverses the `usage_limits` debit if that step had already run (controlled by `reverse_usage_limits_owner: Option<&str>`).

Failures within `refund_reservations` are logged but do not propagate, since the primary request has already failed and there is no recovery path. Known limitation: if the database is unavailable during refund, the deducted amounts remain permanently inconsistent until manual reconciliation.

### Startup Guards

**`IRON_ALLOW_DEV_KEYS` in production:** The server panics at startup if `IRON_ALLOW_DEV_KEYS` is set and the environment is detected as production. This prevents the dev-mode key bypass from being accidentally enabled in production.

## RBAC Changes

### `parse_role` Helper

A `parse_role(claims)` helper function replaces raw string comparisons (`user.0.role == "admin"`) throughout `agents.rs`. It calls `Role::from_str(&claims.role)` and returns `Err((401, "Unrecognized role: ..."))` for any role string that does not parse to a known `Role` variant. This closes a gap where a malformed or future role string would silently match (or fail to match) string literals.

All existing admin guards in agent endpoints (`list_agents`, `get_agent`, `create_agent`, `update_agent`, `delete_agent`, `update_agent_budget`, `list_agent_tokens`) now use `parse_role` instead of direct string comparison.

### Admin vs Non-Admin Agent Listing

The agent list endpoint applies `parse_role` to determine admin status. If role parsing fails (unexpected role string), the failure is logged and the user is treated as non-admin (sees only their own agents) rather than returning an error, to avoid breaking authenticated non-admin users with unusual roles.

### `ManageProviderKeys` Permission

The `ManageProviderKeys` permission is required for all provider key endpoints (create, list, get, update, delete, assign, unassign). It is checked via `check_manage_provider_keys(role_str)` which uses `PermissionChecker` — separate from the admin-only gates used by agent endpoints.

## New Error Variants

Two new `ApiError` variants have been added:

- `TooManyRequests(String)` → `429 Too Many Requests`: used when a per-user key quota is exceeded.
- `ServiceUnavailable(String)` → `503 Service Unavailable`: used when the provider key feature is disabled because `IRON_SECRETS_MASTER_KEY` is not set.

## Integration Points

**Used by:** iron_dashboard, iron_sdk/CLI, iron_runtime (provider-key fetch, budget leases)
**Uses:** iron_token_manager, iron_runtime_state, iron_cost, iron_secrets, iron_telemetry
**External Services:** Database (sqlite/sqlx), LLM provider APIs (indirect via provider keys)

---

Cross-references: spec/-archived_detailed_spec.md; docs/protocol/002_rest_api_protocol.md; docs/protocol/005_budget_control_protocol.md; docs/protocol/012_analytics_api.md; docs/architecture/006_budget_control_protocol.md.
