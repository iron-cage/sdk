# Task 001: Fix IC token invalidation after regeneration

## Goal

Ensure that regenerating or revoking an IC token immediately invalidates the old token across all runtime endpoints. The result is observable through consistent 401 responses on all IC-auth routes when using a stale or revoked token. Scoped to the shared IC-token validator and the four runtime routes that accept IC-token authentication. Testable by integration tests covering the full token lifecycle - generate, use, regenerate, verify old rejected, verify new accepted.

## Dependencies
- None

## In Scope

- Shared runtime IC-token validator combining JWT verification and hash comparison
- Replacement of direct IC-token verification in handshake, refresh, provider-key, and analytics routes
- Standardized 401 error responses for invalid, rotated, and revoked token states
- Integration tests covering positive and negative token lifecycle scenarios
- Alignment of generate, regenerate, and revoke endpoints with validator expectations

## Out of Scope

- Changes to the IC token generation algorithm or JWT claims structure
- Admin session authentication or non-IC-token auth flows
- Performance optimization of hash comparison lookups

## Description

The platform stores `ic_token_hash` in the agents table, but runtime endpoints currently rely solely on JWT signature validation without verifying the token hash against the stored value. This means that after an IC token is regenerated or revoked, the old token remains usable as long as its JWT signature is still valid. This is a security gap that allows stale tokens to access budget handshake, budget refresh, provider key, and analytics ingestion endpoints.

This task introduces a shared IC-token validator that performs both JWT signature verification and hash comparison against the stored `ic_token_hash`. All four IC-auth runtime routes will be updated to use this shared validator, ensuring consistent rejection of stale tokens. The validator will parse the `agent_id` from JWT claims, hash the raw token, and compare it against the database value - rejecting requests when the agent does not exist, the hash is null, or the hash does not match.

Error responses will be standardized across all routes so that invalid, rotated, and revoked token states all return a clear 401 with a deterministic payload.

## Context
The platform stores `ic_token_hash` in `agents`, but runtime endpoints currently trust only JWT validation. This allows previously issued IC tokens to remain usable after regenerate or revoke.

Critical areas:
- `module/iron_control_api/src/routes/budget/handshake.rs`
- `module/iron_control_api/src/routes/budget/refresh.rs`
- `module/iron_control_api/src/routes/agent_provider_key.rs`
- `module/iron_control_api/src/routes/analytics/ingestion.rs`
- `module/iron_control_api/src/ic_token.rs`

## Work Procedure

1. Audit all four IC-auth runtime routes to catalog current token verification logic
2. Design the shared validator interface - input (raw token), output (validated agent context or error)
3. Implement the shared validator in `ic_token.rs` with JWT verification, hash computation, and DB lookup
4. Define standardized error types and 401 response payloads for each rejection reason
5. Replace direct verification in `handshake.rs` with the shared validator
6. Replace direct verification in `refresh.rs` with the shared validator
7. Replace direct verification in `agent_provider_key.rs` with the shared validator
8. Replace direct verification in `analytics/ingestion.rs` with the shared validator
9. Write integration tests covering: generate-use, regenerate-old-rejected, regenerate-new-accepted, revoke-rejected
10. Run full test suite and verify no regressions in existing endpoint behavior

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

## Test Matrix

| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Valid token on handshake endpoint | Request succeeds with 200 | Response contains valid handshake payload |
| Regenerated (old) token on handshake | Request rejected | 401 with hash mismatch error |
| Regenerated (new) token on handshake | Request succeeds with 200 | Response contains valid handshake payload |
| Revoked token on handshake | Request rejected | 401 with revoked/null hash error |
| Valid token on refresh endpoint | Request succeeds with 200 | Budget refresh completes |
| Regenerated (old) token on refresh | Request rejected | 401 with hash mismatch error |
| Valid token on provider-key endpoint | Request succeeds with 200 | Provider key returned |
| Regenerated (old) token on provider-key | Request rejected | 401 with hash mismatch error |
| Valid token on analytics ingestion | Request succeeds with 200 | Events accepted |
| Regenerated (old) token on analytics | Request rejected | 401 with hash mismatch error |
| Token for non-existent agent | Request rejected | 401 with agent not found error |
| Token with valid JWT but tampered hash | Request rejected | 401 with hash mismatch error |

## Validation Checklist

- [ ] Shared IC-token validator exists in `ic_token.rs`
- [ ] All four runtime routes use the shared validator
- [ ] No direct JWT-only verification remains in runtime route handlers
- [ ] 401 error response payload is consistent across all routes
- [ ] Old token returns 401 after regeneration on all four endpoints
- [ ] Old token returns 401 after revocation on all four endpoints
- [ ] New token works on all four endpoints after regeneration
- [ ] Integration tests cover both positive and negative lifecycle cases
- [ ] No regressions in existing endpoint tests

## Validation Procedure

1. Run existing test suite to establish baseline - all tests must pass
2. Verify the shared validator module exists and contains JWT verification plus hash comparison logic
3. Inspect each of the four runtime route files to confirm they call the shared validator
4. Execute integration test: generate IC token, call all four endpoints, verify 200 responses
5. Execute integration test: regenerate IC token, call all four endpoints with old token, verify 401 responses
6. Execute integration test: call all four endpoints with new token, verify 200 responses
7. Execute integration test: revoke IC token, call all four endpoints, verify 401 responses
8. Verify error response bodies contain deterministic, distinguishable error codes
9. Run full test suite and confirm zero regressions

## Acceptance Criteria
- After `POST /api/v1/agents/:id/ic-token/regenerate`, the old token returns `401` on:
  - `/api/v1/budget/handshake`
  - `/api/v1/budget/refresh`
  - `/api/v1/agents/provider-key`
  - `/api/v1/analytics/events`
- After `DELETE /api/v1/agents/:id/ic-token`, the previous token returns `401` on the same endpoints.
- The newly issued token works on all IC-auth runtime endpoints.
- A token with valid signature but stale hash is rejected with `401`.
- Integration tests cover positive and negative lifecycle cases.
