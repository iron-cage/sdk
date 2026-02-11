# Task 002: Support multiple keys for the same inference provider

## Dependencies
- Task 001

## Context
Database schema allows multiple provider keys, but API behavior is still effectively single-key in key creation and fallback selection logic.

Critical areas:
- `module/iron_control_api/src/routes/providers.rs`
- `module/iron_control_api/src/routes/budget/handshake.rs`
- `module/iron_token_manager/src/provider_key_storage.rs`
- `module/iron_token_manager/migrations/019_add_agent_provider_key_id.sql`

## Implementation plan
1. Change provider key creation semantics to always create a new record instead of overwriting the first key by provider.
2. Add owner-scoped storage queries for provider key lookup and selection.
3. Enforce owner/admin access checks for update, delete, and assignment operations.
4. Remove global "first key" fallback in handshake.
   - If `provider_key_id` is present, validate ownership and provider match.
   - If missing, use agent-assigned key only.
5. Add explicit errors for unauthorized key access and missing agent-assigned key.
6. Add tests for same-provider multi-key and cross-tenant isolation.

## Acceptance criteria
- A single user can create at least two keys for `openai`, both with unique IDs.
- Creating keys in user B context does not mutate user A keys.
- `POST /api/v1/providers` no longer updates existing provider keys implicitly.
- Handshake does not select keys from another user or from global provider list fallback.
- Unauthorized key usage returns deterministic access error.
- Tests cover multi-key happy path, owner scoping, and unauthorized negatives.
