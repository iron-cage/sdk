# Task 001: Fix IC token invalidation after regeneration

## Dependencies
- None

## Context
The platform stores `ic_token_hash` in `agents`, but runtime endpoints currently trust only JWT validation. This allows previously issued IC tokens to remain usable after regenerate or revoke.

Critical areas:
- `module/iron_control_api/src/routes/budget/handshake.rs`
- `module/iron_control_api/src/routes/budget/refresh.rs`
- `module/iron_control_api/src/routes/agent_provider_key.rs`
- `module/iron_control_api/src/routes/analytics/ingestion.rs`
- `module/iron_control_api/src/ic_token.rs`

## Implementation plan
1. Add one shared runtime IC-token validator used by all IC-auth runtime routes.
   - Verify JWT signature and claims.
   - Parse `agent_id` from claims.
   - Hash raw token and compare against `agents.ic_token_hash`.
   - Reject when agent does not exist, hash is null, or hash mismatch.
2. Replace direct IC-token verification in runtime routes with the shared validator.
3. Keep `generate`, `regenerate`, and `revoke` lifecycle endpoints aligned with validator expectations.
4. Standardize authentication error status and payload for invalid, rotated, and revoked states.
5. Add integration tests for full token lifecycle and endpoint coverage.

## Acceptance criteria
- After `POST /api/v1/agents/:id/ic-token/regenerate`, the old token returns `401` on:
  - `/api/v1/budget/handshake`
  - `/api/v1/budget/refresh`
  - `/api/v1/agents/provider-key`
  - `/api/v1/analytics/events`
- After `DELETE /api/v1/agents/:id/ic-token`, the previous token returns `401` on the same endpoints.
- The newly issued token works on all IC-auth runtime endpoints.
- A token with valid signature but stale hash is rejected with `401`.
- Integration tests cover positive and negative lifecycle cases.
