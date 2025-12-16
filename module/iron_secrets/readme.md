# iron_secrets

Encrypted secrets storage and access control for AI agents.

[![Documentation](https://img.shields.io/badge/docs-ironcage.ai-E5E7EB.svg)](https://ironcage.ai/docs)

## Installation

```toml
[dependencies]
iron_secrets = { path = "../iron_secrets" }
```


## Quick Start

```rust
use iron_secrets::SecretsManager;

// Initialize with master key from environment
let manager = SecretsManager::new("./secrets.db")?;

// Store encrypted secret
manager.create("openai-api-key", "sk-proj-abc123...")?;

// Retrieve decrypted secret for agent use
let api_key = manager.get("openai-api-key")?;

// Audit trail is automatically maintained
```


<details>
<summary>Scope & Boundaries</summary>

**Responsibilities:**
Provides secure secrets management with AES-256-GCM encryption at rest, Argon2id key derivation, role-based access control, and comprehensive audit logging. Enables safe storage and runtime injection of sensitive credentials (API keys, database passwords, tokens).

**In Scope:**
- AES-256-GCM encryption for secrets at rest
- Argon2id key derivation from master key
- SQLite storage for encrypted blobs and metadata
- CRUD operations (create, read, update, delete, list)
- Role-based access control (Admin, Viewer, Agent)
- Audit logging for all secret operations
- Environment isolation (Development, Staging, Production)
- Secret masking for display (`sk-proj-abc...xyz`)
- Master key from environment variable

**Out of Scope:**
- AWS KMS integration (future)
- HashiCorp Vault integration (future)
- Secret versioning and history (future)
- Secret expiration and auto-rotation (future)
- Multi-tenancy isolation (future)
- External secret providers (GitHub Secrets, Azure Key Vault)
- REST API endpoints (see iron_control_api)
- Dashboard UI (see iron_dashboard)

</details>


<details>
<summary>Directory Structure</summary>

### Source Files

| File | Responsibility |
|------|----------------|
| lib.rs | Secure secrets management for AI agents |
| access_control.rs | Access control for secrets |
| audit.rs | Audit logging for secrets access |
| crypto.rs | Cryptographic operations for secret encryption/decryption |
| error.rs | Error types |
| secrets_manager.rs | Secrets manager service |
| storage.rs | Encrypted storage backend |

**Notes:**
- Entries marked 'TBD' require manual documentation
- Entries marked '⚠️ ANTI-PATTERN' should be renamed to specific responsibilities

</details>


## License

Apache-2.0
