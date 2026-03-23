# Task 021: Token revocation and vault backend

## Dependencies
- Task 001

## Context
IC Token "revocation" is currently a database delete. No audit event, no grace period. IP Token storage has AES-GCM encryption primitives but no vault backend. Contradicts promises #7 and #8.

Critical areas:
- `module/iron_control_api/src/ic_token.rs`
- `module/iron_token_manager/src/`
- `module/iron_secrets/src/crypto.rs`

## Implementation plan
1. Add explicit revocation action (distinct from delete) with audit event.
2. Add revocation grace period (configurable).
3. Implement pluggable vault backend for IP Token storage.
4. Vault implementations: environment variables (default), sealed secrets, cloud KMS (future).
5. Add key rotation workflow.

## Acceptance criteria
- Revoke action produces audit event.
- Revoked tokens are rejected after grace period.
- IP Tokens stored via vault backend, not plain environment variables.
- Key rotation does not cause downtime.
