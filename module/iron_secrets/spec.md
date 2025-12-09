# iron_secrets - Specification

**Module:** iron_secrets
**Layer:** 3 (Feature)
**Status:** Active

---

## Responsibility

Encrypted secrets management for LLM provider credentials. Stores API keys with AES-256 encryption, manages key rotation, provides scoped access control for agents.

---

## Scope

**In Scope:**
- Secret encryption (AES-256-GCM)
- Secret storage (SQLite with encryption)
- Key rotation scheduling
- Scoped access (agent can only access its secrets)
- Integration with iron_token_manager for authorization

**Out of Scope:**
- External secret backends (see docs/integration/002_secret_backends.md)
- OAuth2/OIDC integration (future enhancement)
- Multi-tenant secret isolation (future enhancement)

---

## Dependencies

**Required Modules:**
- iron_types - Foundation types
- iron_state - Encrypted storage
- iron_telemetry - Audit logging

**Required External:**
- aes-gcm - AES encryption
- argon2 - Key derivation
- zeroize - Secure memory clearing

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **Secret Vault:** AES-256 encrypted storage
- **Key Deriver:** Argon2 key derivation from passphrase
- **Access Controller:** Scoped permissions per agent
- **Rotation Manager:** Scheduled key rotation

---

## Integration Points

**Used by:**
- iron_api - Secret retrieval for agents
- iron_runtime - Just-in-time secret injection

**Uses:**
- iron_state - Encrypted persistence

---

*For detailed encryption specifications, see spec/-archived_detailed_spec.md*
*For credential flow, see docs/security/003_credential_flow.md*
