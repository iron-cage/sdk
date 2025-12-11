# Iron Token Manager - Specification

**Module:** iron_token_manager
**Version:** 0.1.0
**Status:** Implemented
**Layer:** 3 (Feature)
**Date:** 2025-12-09

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility.

---

## Scope

**Responsibility:**
Manages API token lifecycle and user account management for Iron Cage with secure generation, SHA-256 hashing, SQLite storage, JWT authentication, usage tracking, quota enforcement, and token bucket rate limiting. Provides complete authentication, authorization, and user management infrastructure for API access control.

**In Scope:**
- Cryptographic token generation (Base64, high-entropy)
- SHA-256 token hashing (never store plaintext)
- Token CRUD operations (create, verify, revoke, list, update)
- Token expiration and deactivation
- Usage tracking per token (requests, tokens consumed, cost in USD)
- Quota enforcement (daily limits, cost caps, hard cutoffs)
- Token bucket rate limiting (requests per second)
- JWT authentication and validation
- User management (create, suspend, activate, delete, role change, password reset)
- User audit logging (comprehensive operation tracking)
- BCrypt password hashing (cost factor 12)
- SQLite persistence with migrations
- Token metadata (name, created_at, last_used, owner)

**Out of Scope:**
- OAuth2/OIDC integration (future enhancement)
- API key rotation automation (future enhancement)
- Multi-tenant token isolation (future enhancement)
- Token analytics and reporting UI (see iron_dashboard)
- REST API endpoints (see iron_control_api)
- Cost calculation logic (see iron_cost)
- Budget tracking implementation (see iron_cost)

## Deployment Context

Iron Cage supports two deployment modes. This module operates in both modes with different storage backends.

**See:** [docs/module_package_matrix.md](../../docs/module_package_matrix.md) for package distribution.

**This Module (iron_token_manager):**

**Pilot Mode:**
- Part of Control Panel (single process)
- Uses SQLite (./tokens.db)
- Tokens shared across API and dashboard in same process

**Production Mode:**
- Part of Control Panel (cloud deployment)
- Uses PostgreSQL (iron_control_schema schema)
- Tokens centralized for multi-user access
- Replicated across Control Panel instances

---

## Dependencies

**Required:**
- iron_types (foundation types, errors)
- iron_runtime_state (state management)
- iron_telemetry (logging)
- iron_cost (cost types and calculations)
- sqlx (database operations)
- tokio (async runtime)
- governor (rate limiting)

**Cryptography:**
- rand (secure random generation)
- sha2 (SHA-256 hashing)
- blake3 (fast hashing)
- base64 (token encoding)

**Optional:**
- None (all features enabled by default)

---

## API Contract

### Token Management

```rust
use iron_token_manager::{TokenManager, Token, TokenConfig};

// Create token manager
let manager = TokenManager::new("./tokens.db").await?;

// Generate new token
let config = TokenConfig {
  name: "Production API".to_string(),
  owner: "user-001".to_string(),
  expires_at: None, // Never expires
  daily_limit_usd: Some(100.0),
  rate_limit_rps: Some(10),
};
let token = manager.create_token(config).await?;

// Verify token
let is_valid = manager.verify(&token.value).await?;

// Record usage
manager.record_usage(&token.id, 1000, 0.05).await?;

// Check quota
let can_proceed = manager.check_quota(&token.id).await?;

// Revoke token
manager.revoke(&token.id).await?;
```

### Rate Limiting

```rust
use iron_token_manager::RateLimiter;

// Create rate limiter (100 requests/second)
let limiter = RateLimiter::new(100);

// Check if request allowed
if limiter.check("token-id-123").await? {
  // Process request
} else {
  // Rate limited - reject request
}
```

### Usage Tracking

```rust
use iron_token_manager::UsageTracker;

// Track token usage
tracker.record(
  token_id: "token-123",
  tokens: 1500,
  cost_usd: 0.045,
  model: "gpt-4"
).await?;

// Get usage stats
let stats = tracker.get_daily_usage("token-123").await?;
println!("Tokens: {}, Cost: ${}", stats.tokens, stats.cost_usd);
```

### User Management

```rust
use iron_token_manager::user_service::{UserService, CreateUserParams};

// Create user service
let pool = SqlitePool::connect("./users.db").await?;
let user_service = UserService::new(pool);

// Create new user
let params = CreateUserParams {
  username: "john_doe".to_string(),
  password: "SecurePass123!".to_string(),
  email: "john.doe@example.com".to_string(),
  role: "user".to_string(),
};
let user = user_service.create_user(params, admin_id).await?;

// Suspend user
let reason = Some("Violation of terms".to_string());
let suspended = user_service.suspend_user(user.id, admin_id, reason).await?;

// Change role
let updated = user_service.change_user_role(user.id, admin_id, "admin".to_string()).await?;

// Reset password
let reset = user_service.reset_password(user.id, admin_id, "NewPass456!".to_string(), true).await?;
```

---

## Architecture

### Module Structure

```
iron_token_manager/
├── src/
│   ├── lib.rs                  # Public API, TokenManager
│   ├── token_generator.rs      # Cryptographic token generation
│   ├── storage.rs              # SQLite persistence layer
│   ├── user_service.rs         # User management service (CRUD, audit logging)
│   ├── usage_tracker.rs        # Usage tracking and quota enforcement
│   ├── rate_limiter.rs         # Token bucket rate limiting
│   ├── limit_enforcer.rs       # Quota enforcement logic
│   ├── cost_calculator.rs      # Cost calculation (uses iron_cost)
│   ├── trace_storage.rs        # Trace storage for debugging
│   ├── provider_adapter.rs     # Adapter for external providers
│   └── error.rs                # Error types
├── migrations/                 # SQLite schema migrations
│   ├── 005_enhance_users_table.sql        # User management fields
│   └── 006_create_user_audit_log.sql      # Audit logging
├── tests/                      # Integration tests
├── Cargo.toml
└── readme.md
```

### Core Components

**TokenManager:**
- Main API entry point
- Coordinates token lifecycle
- Delegates to specialized components

**TokenGenerator:**
- Generates cryptographically secure tokens
- Uses rand for entropy
- Base64 encoding for URL-safe tokens

**Storage:**
- SQLite backend for token persistence
- Schema: tokens table (id, name, hash, owner, created_at, expires_at, last_used)
- Uses sqlx for compile-time SQL verification

**UsageTracker:**
- Tracks requests, tokens, and cost per token
- Daily aggregation for quota enforcement
- Integrates with iron_cost for cost calculation

**RateLimiter:**
- Token bucket algorithm
- Per-token rate limiting
- Configurable requests per second

**LimitEnforcer:**
- Checks quota before allowing requests
- Hard cutoffs (blocks when limit exceeded)
- Daily reset at midnight UTC

**UserService:**
- User account management (create, suspend, activate, delete)
- Role management (change user roles)
- Password management (BCrypt hashing, reset, force change)
- Audit logging (all operations tracked)
- Self-modification prevention (can't delete/change own role)

---

## Database Schema

### tokens Table

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| id | TEXT | PRIMARY KEY | Unique token ID (UUID) |
| name | TEXT | NOT NULL | Human-readable name |
| hash | TEXT | NOT NULL, UNIQUE | SHA256 hash of token value |
| owner | TEXT | NOT NULL | User/service owning token |
| created_at | TEXT | NOT NULL | ISO8601 timestamp |
| expires_at | TEXT | NULL | ISO8601 expiry (NULL = never) |
| last_used | TEXT | NULL | ISO8601 last usage |
| active | INTEGER | NOT NULL, DEFAULT 1 | 0=revoked, 1=active |

### usage Table

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| id | TEXT | PRIMARY KEY | Usage record ID |
| token_id | TEXT | FOREIGN KEY (tokens.id) | Associated token |
| timestamp | TEXT | NOT NULL | ISO8601 timestamp |
| requests | INTEGER | NOT NULL | Request count |
| tokens | INTEGER | NOT NULL | Token count |
| cost_usd | REAL | NOT NULL | Cost in USD |
| model | TEXT | NULL | LLM model used |

### users Table

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | Unique user ID |
| username | TEXT | NOT NULL, UNIQUE | Username (login identifier) |
| password_hash | TEXT | NOT NULL | BCrypt hashed password |
| email | TEXT | NULL | User email address |
| role | TEXT | NOT NULL, DEFAULT 'user' | User role (viewer, user, admin) |
| is_active | INTEGER | NOT NULL, DEFAULT 1 | 0=inactive, 1=active |
| created_at | INTEGER | NOT NULL | Unix epoch milliseconds |
| last_login | INTEGER | NULL | Last login timestamp |
| suspended_at | INTEGER | NULL | Suspension timestamp |
| suspended_by | INTEGER | FOREIGN KEY (users.id) | Admin who suspended |
| deleted_at | INTEGER | NULL | Soft deletion timestamp |
| deleted_by | INTEGER | FOREIGN KEY (users.id) | Admin who deleted |
| force_password_change | INTEGER | NOT NULL, DEFAULT 0 | Force password change flag |

### user_audit_log Table

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | Audit log entry ID |
| operation | TEXT | NOT NULL, CHECK | Operation type (create, suspend, activate, delete, role_change, password_reset) |
| target_user_id | INTEGER | NOT NULL, FOREIGN KEY (users.id) | User affected by operation |
| performed_by | INTEGER | NOT NULL, FOREIGN KEY (users.id) | Admin who performed operation |
| timestamp | INTEGER | NOT NULL | Unix epoch milliseconds |
| previous_state | TEXT | NULL | Previous state (JSON) |
| new_state | TEXT | NULL | New state (JSON) |
| reason | TEXT | NULL | Optional reason |

### Indexes

```sql
CREATE INDEX idx_usage_token_id ON usage(token_id);
CREATE INDEX idx_usage_timestamp ON usage(timestamp DESC);
CREATE INDEX idx_tokens_hash ON tokens(hash);
CREATE INDEX idx_tokens_owner ON tokens(owner);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_is_active ON users(is_active);
CREATE INDEX idx_users_username_search ON users(username);
CREATE INDEX idx_user_audit_target ON user_audit_log(target_user_id);
CREATE INDEX idx_user_audit_performer ON user_audit_log(performed_by);
CREATE INDEX idx_user_audit_timestamp ON user_audit_log(timestamp);
CREATE INDEX idx_user_audit_operation ON user_audit_log(operation);
```

---

## Testing Standards

### Database Path Standards

All database paths follow strict conventions to ensure portability and proper cleanup:

**Test Databases:**
- In-memory: `:memory:` (preferred for unit/integration tests)
- Temporary files: `/tmp/iron_token_manager_test_*.db`
- CI/CD: `./target/test_db_{unique}.db`

**Development Database:**
- Standard path: `./iron.db` (relative to crate root)
- Git-ignored by default

**Validation:**
- Pre-commit hook validates all database paths
- CI/CD validates paths before test execution
- Manual validation via `scripts/validate_db_paths.sh`

### Test Database Lifecycle

**v2 Helpers (Recommended):**

The module provides ergonomic test helpers via `iron_test_db` crate:

```rust
use common::{create_test_db_v2, create_test_db_with_seed};

#[tokio::test]
async fn my_test() {
  // Basic test database with migrations applied
  let db = create_test_db_v2().await;
  // Automatic cleanup via RAII
}

#[tokio::test]
async fn my_test_with_data() {
  // Database with comprehensive seed data
  let db = create_test_db_with_seed().await;
  // 5 users, 8 tokens, usage records, limits
}
```

**Benefits:**
- No manual TempDir management
- Shared pool across components
- Automatic cleanup via Drop trait
- Consistent initialization
- Complete test isolation

### Seed Data Management

**Two-Tier Approach:**

**Tier 1: Bash Seed (Simple)**
- Script: `scripts/seed_dev_data.sh`
- Contents: 3 users, 3 tokens, 7 usage records, 3 limits
- Use case: Manual testing, quick development

**Tier 2: Rust Seed (Comprehensive)**
- Implementation: `src/seed.rs`
- Contents: 5 users, 8 tokens, 10+ usage records, 3 limits
- Edge cases: Expired tokens, users without tokens, unlimited users
- Use case: Automated tests, edge case validation

**Validation:**
- Script: `scripts/validate_seed_data.sh`
- Accepts both implementations (bash: 3/3/7, Rust: 5/8/10+)
- Validates core users, optional users, foreign keys

**Documentation:**
- Complete reference: [docs/seed_data_reference.md](./docs/seed_data_reference.md)
- User profiles, token catalog, usage patterns
- Manual testing guide with examples

### Validation and Enforcement

**Three-Layer Enforcement:**

1. **Pre-commit Hooks** - Catch violations before commit
   - Database path compliance
   - Seed data completeness
   - Schema validation

2. **CI/CD Pipeline** - Enforce in automation
   - All pre-commit checks
   - Full test suite (120 tests)
   - Clippy warnings as errors

3. **Manual Validation** - On-demand verification
   - `scripts/validate_db_paths.sh`
   - `scripts/validate_db_schema.sh`
   - `scripts/validate_seed_data.sh`

### Test Organization

**Standards:**
- All tests in `tests/` directory (no `#[cfg(test)]` modules in `src/`)
- Shared helpers in `tests/common/mod.rs`
- One test file per major component
- Loud failures with descriptive messages
- Foreign key integrity validation
- Test isolation (each test gets independent database)

**Documentation:**
- Comprehensive standards: [docs/testing_standards.md](./docs/testing_standards.md)
- Best practices and anti-patterns
- Troubleshooting guide
- Performance considerations

---

## Development Status

**Current Phase:** Implemented (v0.1.0)

**Completed:**
- ✅ Token generation with crypto-secure random
- ✅ SHA-256 hashing for storage
- ✅ SQLite persistence with migrations
- ✅ CRUD operations (create, verify, revoke, list)
- ✅ Usage tracking (requests, tokens, cost)
- ✅ Quota enforcement (daily limits)
- ✅ Rate limiting (token bucket algorithm)
- ✅ Integration with iron_runtime_state for audit
- ✅ User management (create, suspend, activate, delete, role change, password reset)
- ✅ User audit logging (comprehensive operation tracking)
- ✅ BCrypt password hashing (cost factor 12)
- ✅ Integration tests (120 tests - token + user + database)
- ✅ Database path standardization (validation + enforcement)
- ✅ Comprehensive seed data (two-tier approach with edge cases)
- ✅ Testing infrastructure (v2 helpers, validation scripts, pre-commit hooks)
- ✅ Testing standards documentation

**Pending:**
- 📋 PostgreSQL migration for production mode
- 📋 OAuth2/OIDC integration
- 📋 Automatic token rotation
- 📋 Multi-tenant isolation

---

## Non-Functional Requirements

### NFR1: Security
- **NFR1.1:** Never store plaintext tokens (SHA-256 hash only)
- **NFR1.2:** Cryptographically secure token generation (rand crate)
- **NFR1.3:** Constant-time token comparison (prevent timing attacks)
- **NFR1.4:** Token entropy ≥128 bits

### NFR2: Performance
- **NFR2.1:** Token verification <1ms (database lookup + hash comparison)
- **NFR2.2:** Rate limit check <0.5ms (in-memory token bucket)
- **NFR2.3:** Usage tracking <2ms (database write)

### NFR3: Reliability
- **NFR3.1:** Token database corruption recovery (WAL mode, checksums)
- **NFR3.2:** Rate limiter survives restarts (persist buckets to Redis in production)
- **NFR3.3:** Graceful degradation (if rate limiter down, allow requests with warning)

### NFR4: Scalability
- **NFR4.1:** Support 10,000+ tokens per instance
- **NFR4.2:** Handle 1,000 requests/second rate limiting
- **NFR4.3:** Usage tracking scales to millions of records

---

## Integration Points

### With iron_control_api
- iron_control_api calls `token_manager.verify()` on every API request
- Middleware extracts Bearer token from Authorization header
- Returns 401 if token invalid, 429 if rate limited

### With iron_cost
- Uses iron_cost types for cost tracking
- Delegates cost calculation to iron_cost
- Stores calculated costs in usage table

### With iron_runtime_state
- Emits audit events for token operations (create, revoke, quota exceeded)
- Usage data available for dashboard queries
- PII detections logged (if token used by iron_safety)

### With iron_dashboard
- Dashboard reads token list for display
- Dashboard triggers token creation/revocation
- Real-time usage updates via WebSocket

---

## Migration Path

### Pilot → Production

**Current (Pilot):**
- SQLite database (./tokens.db)
- Single-instance storage

**Target (Production):**
- PostgreSQL database (iron_control_schema schema)
- Multi-instance with replication
- Redis for rate limiter state sharing

**Migration:**
1. Export tokens from SQLite
2. Import to PostgreSQL (iron_control_schema)
3. Update storage backend configuration
4. Deploy Redis for rate limiter
5. Verify quota enforcement across replicas

---

## Revision History

- **2025-12-11 (v0.1.1):** Database testing infrastructure
  - Database path standardization (test, dev, CI/CD paths)
  - Comprehensive seed data (two-tier approach: bash 3/3/7, Rust 5/8/10+)
  - Three-layer validation enforcement (pre-commit, CI/CD, manual)
  - Testing standards documentation (testing_standards.md, seed_data_reference.md)
  - v2 test helpers for ergonomic database lifecycle management
  - 120 tests (all passing)

- **2025-12-09 (v0.1.0):** Initial specification - comprehensive token management with 288 tests

**Next Milestone:** PostgreSQL migration for production deployment
