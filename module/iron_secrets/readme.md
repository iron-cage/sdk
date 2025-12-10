# iron_secrets

Encrypted secrets storage and access control for AI agents.

### Scope

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

## Installation

```toml
[dependencies]
iron_secrets = { path = "../iron_secrets" }
```

## Example

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

## License

Apache-2.0
