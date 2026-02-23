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

## Integration Points

**Used by:** iron_dashboard, iron_sdk/CLI, iron_runtime (provider-key fetch, budget leases)
**Uses:** iron_token_manager, iron_runtime_state, iron_cost, iron_secrets, iron_telemetry
**External Services:** Database (sqlite/sqlx), LLM provider APIs (indirect via provider keys)

---

Cross-references: spec/-archived_detailed_spec.md; docs/protocol/002_rest_api_protocol.md; docs/protocol/005_budget_control_protocol.md; docs/protocol/012_analytics_api.md; docs/architecture/006_budget_control_protocol.md.
