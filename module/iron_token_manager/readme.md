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

## License

Apache-2.0
