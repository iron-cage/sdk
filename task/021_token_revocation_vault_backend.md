# Task 021: Token revocation and vault backend

## Goal
Replace the current token deletion mechanism with an explicit revocation workflow that produces audit events and supports a configurable grace period, and introduce a pluggable vault backend for IP Token storage so that secrets are never stored as plain environment variables in production. This directly addresses promises #7 and #8 by providing observable revocation state and secure secret storage.

## Dependencies
- Task 001

## In Scope
- Explicit revocation action distinct from deletion for IC Tokens
- Configurable grace period before revoked tokens are fully rejected
- Audit event emission on revocation
- Pluggable `VaultBackend` trait for IP Token storage
- Environment variable vault implementation (default) and sealed secrets implementation
- Key rotation workflow without downtime

## Out of Scope
- Cloud KMS vault implementations (AWS KMS, GCP KMS) - deferred to future task
- UI for managing revocation in the dashboard
- Automatic token expiry policies (separate from revocation)

## Description
Currently, revoking an IC Token is implemented as a simple database delete, which means there is no audit trail and no way to provide a grace period for dependent services to migrate. This task introduces a proper revocation lifecycle: tokens transition to a "revoked" state that is recorded as an audit event, remain queryable during a configurable grace period, and are then rejected on use after the grace window expires.

For IP Tokens, the existing AES-GCM encryption primitives provide the cryptographic foundation, but secrets are still stored directly in environment variables. This task introduces a `VaultBackend` trait that abstracts secret storage, with an environment variable implementation as the default and a sealed secrets implementation for production use. A key rotation workflow ensures that rotating encryption keys does not cause downtime or require re-provisioning all tokens simultaneously.

## Context
IC Token "revocation" is currently a database delete. No audit event, no grace period. IP Token storage has AES-GCM encryption primitives but no vault backend. Contradicts promises #7 and #8.

Critical areas:
- `module/iron_control_api/src/ic_token.rs`
- `module/iron_token_manager/src/`
- `module/iron_secrets/src/crypto.rs`

## Work Procedure
1. Add a `TokenStatus` enum with variants `Active`, `Revoked(revoked_at)`, and `Deleted` to `ic_token.rs`.
2. Implement the revoke endpoint that transitions a token to `Revoked` state and emits an audit event.
3. Add grace period configuration (default 24 hours) and middleware that checks token status on each request.
4. Define the `VaultBackend` trait with methods `store_secret`, `retrieve_secret`, `delete_secret`, and `rotate_key`.
5. Implement `EnvVaultBackend` as the default vault using environment variables.
6. Implement `SealedSecretsVaultBackend` using the existing AES-GCM primitives in `iron_secrets`.
7. Wire the vault backend into the IP Token manager so that new tokens are stored via the configured backend.
8. Implement key rotation: encrypt all existing secrets with the new key, verify, then remove old key material.
9. Write integration tests covering revocation lifecycle, grace period enforcement, and vault backend switching.

## Implementation plan
1. Add explicit revocation action (distinct from delete) with audit event.
2. Add revocation grace period (configurable).
3. Implement pluggable vault backend for IP Token storage.
4. Vault implementations: environment variables (default), sealed secrets, cloud KMS (future).
5. Add key rotation workflow.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Revoke an active IC Token | Token transitions to Revoked state | Status is Revoked, audit event emitted |
| Use revoked token within grace period | Request succeeds with deprecation warning | Response includes warning header |
| Use revoked token after grace period | Request rejected with 401 | Error message indicates revoked token |
| Store IP Token via EnvVaultBackend | Secret stored in environment variable | Retrieve returns same secret |
| Store IP Token via SealedSecretsVaultBackend | Secret stored encrypted | Retrieve returns decrypted secret |
| Rotate encryption key | All secrets re-encrypted with new key | All secrets still retrievable after rotation |
| Rotate key under concurrent reads | No failed reads during rotation | Zero errors during rotation window |

## Validation List
- [ ] Revoke endpoint returns success and emits audit event
- [ ] Token status transitions follow the correct lifecycle (Active -> Revoked -> rejected)
- [ ] Grace period is configurable and defaults to 24 hours
- [ ] `VaultBackend` trait is implemented by both EnvVaultBackend and SealedSecretsVaultBackend
- [ ] IP Tokens are stored and retrieved correctly through each vault implementation
- [ ] Key rotation completes without downtime
- [ ] Existing tests for IC Token CRUD still pass
- [ ] Audit log contains revocation events with timestamp and actor

## Validation Procedure
1. Run `cargo test -p iron_control_api` and `cargo test -p iron_token_manager` to verify all tests pass.
2. Revoke a token via the API, then query the audit log to confirm the revocation event exists.
3. Attempt to use a revoked token within the grace period and verify the request succeeds with a warning.
4. Wait for the grace period to expire (or use a test-configured short period) and verify the token is rejected.
5. Configure each vault backend and store/retrieve an IP Token to confirm round-trip correctness.
6. Perform a key rotation and verify all existing tokens remain accessible.
7. Run concurrent read requests during a key rotation and verify zero errors.

## Acceptance criteria
- Revoke action produces audit event.
- Revoked tokens are rejected after grace period.
- IP Tokens stored via vault backend, not plain environment variables.
- Key rotation does not cause downtime.
