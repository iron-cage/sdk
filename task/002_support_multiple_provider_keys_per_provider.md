# Task 002: Support multiple keys for the same inference provider

## Goal

Enable users to register and manage multiple API keys for the same inference provider without overwriting existing keys. The result is observable through the API returning distinct key IDs for multiple keys created under the same provider, and through cross-tenant isolation preventing key leakage. Scoped to provider key creation, lookup, assignment, and handshake selection logic. Testable by creating two keys for the same provider and verifying both exist with unique IDs, and by confirming cross-user isolation.

## Dependencies
- Task 001

## In Scope

- Changing provider key creation to always insert a new record instead of overwriting
- Owner-scoped storage queries for provider key lookup and selection
- Owner/admin access checks for update, delete, and assignment operations
- Removal of global "first key" fallback in handshake
- Explicit error responses for unauthorized key access and missing agent-assigned key
- Tests for same-provider multi-key and cross-tenant isolation

## Out of Scope

- Per-key spending limits or budget enforcement (covered by Task 003)
- Provider key encryption at rest changes
- Migration of existing single-key data to multi-key format

## Description

The database schema already supports multiple provider keys, but the API layer effectively behaves as single-key - creating a new key for a provider that already has one overwrites the existing record. This prevents teams from managing multiple API keys for the same provider, which is necessary for separating billing, rate limits, or access tiers.

This task changes provider key creation semantics so that each creation always produces a new record with a unique ID. Storage queries are updated to be owner-scoped, ensuring users can only see and use their own keys. The handshake flow is updated to remove the global "first key" fallback - instead requiring either an explicit `provider_key_id` or an agent-assigned key. Access checks enforce that users cannot reference or use keys belonging to other users.

## Context
Database schema allows multiple provider keys, but API behavior is still effectively single-key in key creation and fallback selection logic.

Critical areas:
- `module/iron_control_api/src/routes/providers.rs`
- `module/iron_control_api/src/routes/budget/handshake.rs`
- `module/iron_token_manager/src/provider_key_storage.rs`
- `module/iron_token_manager/migrations/019_add_agent_provider_key_id.sql`

## Work Procedure

1. Audit current provider key creation logic in `providers.rs` to understand overwrite behavior
2. Modify key creation to always insert a new record, returning a unique `provider_key_id`
3. Update storage queries in `provider_key_storage.rs` to filter by owner context
4. Add owner/admin authorization checks on update, delete, and assignment operations
5. Remove the global "first key by provider" fallback from handshake logic
6. Implement `provider_key_id` validation in handshake - verify ownership and provider match
7. Implement agent-assigned key resolution as the fallback when `provider_key_id` is absent
8. Define explicit error types for unauthorized key access and missing agent-assigned key
9. Write tests for multi-key creation under one user, cross-user isolation, and unauthorized access
10. Run full test suite to verify no regressions in existing single-key workflows

## Implementation plan
1. Change provider key creation semantics to always create a new record instead of overwriting the first key by provider.
2. Add owner-scoped storage queries for provider key lookup and selection.
3. Enforce owner/admin access checks for update, delete, and assignment operations.
4. Remove global "first key" fallback in handshake.
   - If `provider_key_id` is present, validate ownership and provider match.
   - If missing, use agent-assigned key only.
5. Add explicit errors for unauthorized key access and missing agent-assigned key.
6. Add tests for same-provider multi-key and cross-tenant isolation.

## Test Matrix

| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Create two keys for `openai` under same user | Both keys created with unique IDs | API returns two distinct `provider_key_id` values |
| Create key for `openai` under user B | User A keys unchanged | User A key list unmodified |
| List keys as user A | Only user A keys returned | No keys from other users appear |
| Handshake with explicit valid `provider_key_id` | Handshake succeeds | Budget reservation granted |
| Handshake with `provider_key_id` owned by another user | Handshake rejected | Access error returned |
| Handshake without `provider_key_id`, agent has assigned key | Handshake uses agent-assigned key | Budget reservation granted |
| Handshake without `provider_key_id`, no agent-assigned key | Handshake rejected | Missing key error returned |
| Delete key owned by another user | Operation rejected | Authorization error returned |
| Update key owned by another user | Operation rejected | Authorization error returned |

## Validation Checklist

- [ ] Provider key creation always inserts a new record
- [ ] Multiple keys for the same provider can coexist under one user
- [ ] Storage queries are owner-scoped
- [ ] Owner/admin access checks enforced on update, delete, and assignment
- [ ] Global "first key" fallback removed from handshake
- [ ] Cross-tenant key isolation verified
- [ ] Explicit error messages for unauthorized access and missing keys
- [ ] Tests cover multi-key, cross-tenant, and unauthorized scenarios

## Validation Procedure

1. Run existing test suite to establish baseline
2. Create two provider keys for `openai` under the same user via API - verify both have unique IDs
3. Create a provider key under a second user - verify first user's keys are unchanged
4. Attempt handshake with explicit `provider_key_id` owned by the requesting user - verify success
5. Attempt handshake with `provider_key_id` owned by a different user - verify access error
6. Attempt handshake without `provider_key_id` using an agent with an assigned key - verify success
7. Attempt handshake without `provider_key_id` using an agent without an assigned key - verify error
8. Attempt to delete or update a key owned by another user - verify authorization error
9. Run full test suite and confirm zero regressions

## Acceptance Criteria
- A single user can create at least two keys for `openai`, both with unique IDs.
- Creating keys in user B context does not mutate user A keys.
- `POST /api/v1/providers` no longer updates existing provider keys implicitly.
- Handshake does not select keys from another user or from global provider list fallback.
- Unauthorized key usage returns deterministic access error.
- Tests cover multi-key happy path, owner scoping, and unauthorized negatives.
