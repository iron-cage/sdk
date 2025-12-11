# iron_token_manager

User management, API token management, authentication, and rate limiting.

### Scope

**Responsibilities:**
Manages user accounts with RBAC (admin/user/viewer roles) and comprehensive audit logging. Handles API token lifecycle with secure generation, SHA-256 hashing, and SQLite storage. Provides JWT authentication, usage tracking, quota enforcement, and token bucket rate limiting for API access control.

**In Scope:**
- **User Management:** Account lifecycle (create, suspend, activate, delete with soft delete)
- **RBAC:** Three roles (admin, user, viewer) with permission-based access control
- **Audit Logging:** Append-only user_audit_log with immutability guarantees
- **Password Security:** BCrypt hashing (cost 12), secure password reset
- Cryptographic token generation (Base64, high-entropy)
- SHA-256 token hashing (never store plaintext)
- Token CRUD operations (create, verify, revoke, list)
- Token expiration and deactivation
- Usage tracking per token (requests, tokens, cost)
- Quota enforcement (daily limits, cost caps)
- Token bucket rate limiting (requests per second)
- JWT authentication and validation
- SQLite persistence with proper constraints

**Out of Scope:**
- OAuth2/OIDC integration (future)
- API key rotation automation (future)
- Multi-tenant token isolation (future)
- Token analytics and reporting (future)
- REST API endpoints (see iron_control_api)
- Dashboard UI (see iron_dashboard)
- Cost calculation (see iron_cost)
- Budget tracking (see iron_cost)

## Token Types

This module manages API tokens for Control Panel authentication.

**IC Token (Internal Control Token):**
- Purpose: Link agent to budget allocation (Model C architecture)
- Visibility: Developer-visible (JWT format)
- **See:** [docs/protocol/005_budget_control_protocol.md](../../docs/protocol/005_budget_control_protocol.md) § IC Token Format

**API Token:**
- Purpose: Authenticate Control Panel REST API requests
- Visibility: Developer-visible (for API access)
- Format: Opaque Base64 string (SHA-256 hashed in storage)
- Lifetime: Long-lived (manually revoked)

**Distinction:**
- IC Token: For Runtime (agent execution, budget-linked)
- API Token: For Control Panel API (CRUD operations)

---

## Installation

```toml
[dependencies]
iron_token_manager = { path = "../iron_token_manager" }
```

## Example

### User Management

```rust
use iron_token_manager::UserService;

// Create user service with SQLite storage
let service = UserService::new("./users.db")?;

// Create a new user account
let user = service.create_user(
  "john_doe",
  "SecurePass123!",
  "john@example.com",
  "user",
  1 // admin_id performing the action
)?;

// Suspend user account
service.suspend_user(
  user.id,
  1, // admin_id
  Some("Violates acceptable use policy")
)?;

// Change user role
service.change_user_role(user.id, "admin", 1)?;

// Get audit log for user
let audit_entries = service.get_user_audit_log(user.id)?;
```

### Token Management

```rust
use iron_token_manager::{TokenManager, RateLimiter};

// Create token manager with SQLite storage
let manager = TokenManager::new("./tokens.db")?;

// Generate new API token
let token = manager.create_token("user-001", "my-api-key")?;

// Verify and track usage
if manager.verify(&token)? {
  manager.record_usage(&token, 1000, 0.05)?;
}

// Rate limiting
let limiter = RateLimiter::new(100); // 100 req/sec
if limiter.check("user-001")? {
  // Process request
}
```

## Development

### Quick Start: Fresh Environment

Get a clean, validated development environment in one command:

```bash
# Full workflow (reset + seed + validate)
make dev-fresh

# With full test suite
make dev-fresh-test
```

### Database Path Standards

**Canonical Development Path: `./iron.db`**

All development uses this single path:
- Scripts default to `./iron.db`
- Config: `sqlite:///./iron.db?mode=rwc`
- Tests use in-memory databases (`sqlite::memory:`)

**Validation System:**

```bash
# Run all validators
make validate

# Individual validators
make validate-paths    # Check for forbidden paths
make validate-schema   # Verify schema correctness
make validate-seed     # Validate seed data
```

**Enforcement:**

1. **Pre-commit hook** - Blocks commits with path violations
   ```bash
   make install-hooks
   ```

2. **CI/CD validation** - GitHub Actions on every PR

3. **Makefile integration** - Commands include validation

**Quick Reference:**

```bash
make reset-seed     # Reset and seed database
make validate       # Run all validators
make test          # Full test suite
make dev-fresh     # Complete fresh start
```

**Test Tokens for Manual Testing:**

```
Admin:      iron_dev_admin_token_001
Developer:  iron_dev_pm_token_002
Viewer:     iron_dev_viewer_token_003
```

**Detailed Documentation:**
- [Database Path Standards](./docs/database_path_standards.md) - Complete guide
- [Database Initialization](./docs/database_initialization.md) - Schema and migrations
- [Configuration](./docs/configuration.md) - Config file reference

## Testing

### Test Database Infrastructure

This crate uses `iron_test_db` for standardized test database management with automatic cleanup and seed data support.

**Key Features:**
- RAII cleanup (no manual TempDir management)
- Shared pool across components (no `/tmp` workarounds)
- Automatic migrations
- Seed data population for realistic testing
- In-memory storage for speed

### Using v2 Test Helpers

The `tests/common/mod.rs` module provides v2 helpers using `iron_test_db`:

```rust
use crate::common::{ create_test_db_v2, create_test_db_with_seed, create_test_storage_v2 };

#[ tokio::test ]
async fn test_basic_database()
{
  // Create test database with migrations applied
  let db = create_test_db_v2().await;

  // Use pool directly for SQL queries
  let result: i64 = sqlx::query_scalar( "SELECT 1" )
    .fetch_one( db.pool() )
    .await?;

  // No cleanup needed - TestDatabase handles it via Drop
}

#[ tokio::test ]
async fn test_with_seed_data()
{
  // Create database with realistic seed data
  let db = create_test_db_with_seed().await;

  // Database now contains:
  // - 3 users (admin, developer, viewer)
  // - 2 AI provider keys (OpenAI, Anthropic)
  // - 5 API tokens (various states)
  // - 3 usage limits (different tiers)

  let user_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM users" )
    .fetch_one( db.pool() )
    .await?;

  assert_eq!( user_count, 3 );
}

#[ tokio::test ]
async fn test_with_components()
{
  // Create components sharing the same database
  let ( storage, db ) = create_test_storage_v2().await;

  // Storage and db share the same pool
  let token_id = storage.create_token( "hash", "user", None, None, None, None ).await?;

  // Query directly via pool if needed
  let exists: bool = sqlx::query_scalar(
    "SELECT EXISTS(SELECT 1 FROM api_tokens WHERE id = ?)"
  )
  .bind( token_id )
  .fetch_one( db.pool() )
  .await?;

  assert!( exists );
}
```

### Seed Data Reference

See `tests/fixtures/seed_data_reference.md` for complete documentation of seeded entities and their properties.

The seed data validation tests in `tests/seed_data_validation.rs` ensure the documentation stays in sync with actual seed data.

### Migration from v1 to v2 Helpers

**Old v1 approach:**
```rust
let ( pool, _temp ) = create_test_db().await;
// Manual pool and TempDir management
```

**New v2 approach:**
```rust
let db = create_test_db_v2().await;
// Automatic cleanup, cleaner API
```

**Benefits:**
- No manual TempDir tracking
- Shared pool across multiple components
- Consistent RAII cleanup pattern
- Better ergonomics

Both approaches are currently supported for backward compatibility.

## License

Apache-2.0
