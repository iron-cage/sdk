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
- Budget lease management (Protocol 005)
- Agent budget tracking (1:1 with agents)
- IC Token generation and validation (JWT with agent_id, budget_id)
- IP Token encryption/decryption (AES-256-GCM)
- Budget request workflow (Protocol 012)
- Budget change request CRUD (create, get, list, approve, reject)
- Budget modification history tracking
- Atomic budget application with transaction guarantees

**Out of Scope:**
- OAuth2/OIDC integration (future enhancement)
- API key rotation automation (future enhancement)
- Multi-tenant token isolation (future enhancement)
- Token analytics and reporting UI (see iron_dashboard)
- REST API endpoints (see iron_control_api)
- Cost calculation logic (see iron_cost)

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

### Budget Lease Management (Protocol 005)

```rust
use iron_token_manager::{ LeaseManager, AgentBudgetManager };
use sqlx::SqlitePool;

// Create lease manager
let pool = SqlitePool::connect( "./tokens.db" ).await?;
let lease_mgr = LeaseManager::from_pool( pool.clone() );

// Create budget lease
lease_mgr.create_lease(
  "lease_abc123",     // lease_id (format: lease_<uuid>)
  42,                 // agent_id
  42,                 // budget_id (1:1 with agent)
  10.0,               // budget_granted (USD)
  None                // expires_at (optional)
).await?;

// Record usage
lease_mgr.record_usage( "lease_abc123", 2.5 ).await?;

// Get lease status
let lease = lease_mgr.get_lease( "lease_abc123" ).await?;
println!( "Spent: ${}, Remaining: ${}", lease.budget_spent, lease.budget_granted - lease.budget_spent );

// Create agent budget
let budget_mgr = AgentBudgetManager::from_pool( pool );
budget_mgr.create_budget( 42, 100.0 ).await?;

// Check budget availability
let has_budget = budget_mgr.has_sufficient_budget( 42, 10.0 ).await?;

// Record spending (updates budget_remaining)
budget_mgr.record_spending( 42, 2.5 ).await?;

// Get budget status
let budget = budget_mgr.get_budget( 42 ).await?;
println!( "Total: ${}, Spent: ${}, Remaining: ${}",
  budget.total_allocated, budget.total_spent, budget.budget_remaining );
```

### Budget Request Workflow (Protocol 012)

```rust
use iron_token_manager::budget_request;
use sqlx::SqlitePool;

// Create pool
let pool = SqlitePool::connect( "./tokens.db" ).await?;

// Create budget request
let request_id = "breq_550e8400-e29b-41d4-a716-446655440000";
let now_ms = chrono::Utc::now().timestamp_millis();

budget_request::create_budget_request(
  &pool,
  request_id,
  1,                      // agent_id
  "user-123",             // requester_id
  100_000_000,            // current_budget_micros ($100)
  250_000_000,            // requested_budget_micros ($250)
  "Need increased budget for expanded testing",
  now_ms
).await?;

// Get request by ID
let request = budget_request::get_budget_request( &pool, request_id ).await?;
println!( "Status: {}, Current: ${}, Requested: ${}",
  request.status, request.current_budget_usd, request.requested_budget_usd );

// List all pending requests
let pending = budget_request::list_budget_requests( &pool, None, Some( "pending" ) ).await?;
println!( "Pending requests: {}", pending.len() );

// Approve request (atomically updates budget and records history)
budget_request::approve_budget_request(
  &pool,
  request_id,
  "admin-approver",       // approver_id
  chrono::Utc::now().timestamp_millis()
).await?;

// Reject request
budget_request::reject_budget_request(
  &pool,
  request_id,
  chrono::Utc::now().timestamp_millis()
).await?;
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
│   ├── lease_manager.rs        # Budget lease management (Protocol 005)
│   ├── agent_budget.rs         # Agent budget tracking (Protocol 005)
│   ├── budget_request.rs       # Budget request workflow (Protocol 012)
│   └── error.rs                # Error types
├── migrations/                 # SQLite schema migrations
│   ├── 005_enhance_users_table.sql        # User management fields
│   ├── 006_create_user_audit_log.sql      # Audit logging
│   ├── 009_create_budget_leases.sql       # Budget leases table (Protocol 005)
│   ├── 010_create_agent_budgets.sql       # Agent budgets table (Protocol 005)
│   ├── 011_create_budget_requests.sql     # Budget change requests (Protocol 012)
│   └── 012_create_budget_history.sql      # Budget modification history (Protocol 012)
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

**LeaseManager (Protocol 005):**
- Budget lease CRUD operations
- Temporary budget allocations per agent session
- Usage tracking per lease (budget_spent updates)
- Lease status management (active, expired, revoked)
- Foreign key enforcement (agent_id, budget_id)

**AgentBudgetManager (Protocol 005):**
- Per-agent total budget tracking (1:1 with agents)
- Budget invariant maintenance (allocated = spent + remaining)
- Spending records with automatic budget_remaining updates
- Sufficient budget checks before lease creation
- Integrates with LeaseManager for session-level tracking

**budget_request Module (Protocol 012):**
- Budget change request workflow (create → approve/reject)
- Request CRUD operations (create, get by ID, list with filters)
- Atomic approval with budget application and history recording
- Optimistic locking for concurrent modification prevention
- Request status management (pending → approved/rejected)
- Budget modification history tracking with full audit trail
- Database transaction guarantees for consistency

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

### budget_leases Table (Protocol 005)

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| id | TEXT | PRIMARY KEY | Lease ID (format: lease_<uuid>) |
| agent_id | INTEGER | NOT NULL, FOREIGN KEY (agents.id) | Agent database ID |
| budget_id | INTEGER | NOT NULL, FOREIGN KEY (agent_budgets.agent_id) | Budget database ID (1:1 with agent) |
| budget_granted | REAL | NOT NULL | USD allocated for this lease |
| budget_spent | REAL | NOT NULL, DEFAULT 0.0 | USD consumed in this lease |
| lease_status | TEXT | NOT NULL | Lease status (active, expired, revoked) |
| created_at | INTEGER | NOT NULL | Creation timestamp (milliseconds since epoch) |
| expires_at | INTEGER | NULL | Expiration timestamp (milliseconds, NULL = no expiration) |

**Foreign Keys:**
```sql
FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
FOREIGN KEY (budget_id) REFERENCES agent_budgets(agent_id) ON DELETE CASCADE
```

### agent_budgets Table (Protocol 005)

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| agent_id | INTEGER | PRIMARY KEY, FOREIGN KEY (agents.id) | Agent database ID (1:1 relationship) |
| total_allocated | REAL | NOT NULL | Total USD budget allocated to agent |
| total_spent | REAL | NOT NULL, DEFAULT 0.0 | Total USD spent by agent across all leases |
| budget_remaining | REAL | NOT NULL | Remaining budget (invariant: allocated = spent + remaining) |
| created_at | INTEGER | NOT NULL | Creation timestamp (milliseconds since epoch) |
| updated_at | INTEGER | NOT NULL | Last update timestamp (milliseconds since epoch) |

**Foreign Keys:**
```sql
FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
```

**Invariant:** The budget_remaining column maintains the invariant: `total_allocated = total_spent + budget_remaining`. This is enforced by application logic in `AgentBudgetManager::record_spending()`.

### budget_change_requests Table (Protocol 012)

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| id | TEXT | PRIMARY KEY | Request ID (format: breq_<uuid>) |
| agent_id | INTEGER | NOT NULL, FOREIGN KEY (agents.id) | Agent database ID |
| requester_id | TEXT | NOT NULL | User who created the request |
| current_budget_micros | INTEGER | NOT NULL | Budget at time of request creation (microdollars) |
| requested_budget_micros | INTEGER | NOT NULL | Requested budget amount (microdollars) |
| justification | TEXT | NOT NULL, CHECK (LENGTH >= 20 AND <= 500) | Request justification |
| status | TEXT | NOT NULL, CHECK IN ('pending', 'approved', 'rejected', 'cancelled') | Current status |
| created_at | INTEGER | NOT NULL | Creation timestamp (milliseconds since epoch) |
| updated_at | INTEGER | NOT NULL | Last update timestamp (milliseconds since epoch) |

**Foreign Keys:**
```sql
FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
```

**Optimistic Locking:** Status transitions use `WHERE status='pending'` clause to prevent concurrent modifications. Only pending requests can be approved or rejected.

### budget_modification_history Table (Protocol 012)

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| id | TEXT | PRIMARY KEY | History record ID (format: bhist_<uuid>) |
| agent_id | INTEGER | NOT NULL, FOREIGN KEY (agents.id) | Agent database ID |
| modification_type | TEXT | NOT NULL, CHECK IN ('increase', 'decrease', 'reset') | Type of budget change |
| old_budget_micros | INTEGER | NOT NULL | Budget before change (microdollars) |
| new_budget_micros | INTEGER | NOT NULL | Budget after change (microdollars) |
| change_amount_micros | INTEGER | NOT NULL | Delta amount (new - old, microdollars) |
| modifier_id | TEXT | NOT NULL | User/system who made the change |
| reason | TEXT | NOT NULL, CHECK (LENGTH >= 10 AND <= 500) | Reason for change |
| related_request_id | TEXT | FOREIGN KEY (budget_change_requests.id) | Associated request (nullable) |
| created_at | INTEGER | NOT NULL | Change timestamp (milliseconds since epoch) |

**Foreign Keys:**
```sql
FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
FOREIGN KEY (related_request_id) REFERENCES budget_change_requests(id) ON DELETE SET NULL
```

**Audit Trail:** All budget modifications are recorded in this table, including manual changes and request-driven changes. The `related_request_id` links history records to their originating requests.

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
CREATE INDEX idx_budget_requests_status ON budget_change_requests(status);
CREATE INDEX idx_budget_requests_agent ON budget_change_requests(agent_id);
CREATE INDEX idx_budget_history_agent ON budget_modification_history(agent_id);
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
- ✅ Protocol 005: Budget Control Protocol
  - ✅ Budget lease management (LeaseManager)
  - ✅ Agent budget tracking (AgentBudgetManager)
  - ✅ IC Token generation (JWT with agent_id, budget_id)
  - ✅ IP Token encryption (AES-256-GCM)
  - ✅ Multi-layer enforcement (database + schema + API)
  - ✅ 26 Protocol 005 tests (all passing)
- ✅ Protocol 012: Budget Request Workflow
  - ✅ Budget change request CRUD operations (create, get, list)
  - ✅ Approval/rejection workflow with optimistic locking
  - ✅ Atomic budget application with transaction guarantees
  - ✅ Budget modification history tracking
  - ✅ Database schema (budget_change_requests, budget_modification_history)
  - ✅ 19 Protocol 012 API tests (all passing)
  - ✅ 15 Protocol 012 storage tests (all passing)

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

- **2025-12-11 (v0.1.3):** Protocol 012 (Budget Request Workflow)
  - Budget change request CRUD operations (create, get by ID, list with filters)
  - Approval/rejection workflow with optimistic locking
  - Atomic budget application with transaction guarantees (approve updates budget + status + history)
  - Budget modification history tracking with full audit trail
  - Database schema: budget_change_requests and budget_modification_history tables
  - Microdollar precision for budget calculations
  - 19 Protocol 012 API tests + 15 storage tests (all passing)

- **2025-12-11 (v0.1.2):** Protocol 005 (Budget Control Protocol)
  - Budget lease management (LeaseManager API)
  - Agent budget tracking (AgentBudgetManager API, 1:1 with agents)
  - IC Token generation and validation (JWT with agent_id, budget_id)
  - IP Token encryption/decryption (AES-256-GCM)
  - Database schema: budget_leases and agent_budgets tables
  - Multi-layer enforcement (database + schema + API)
  - 26 Protocol 005 tests (all passing)

- **2025-12-11 (v0.1.1):** Database testing infrastructure
  - Database path standardization (test, dev, CI/CD paths)
  - Comprehensive seed data (two-tier approach: bash 3/3/7, Rust 5/8/10+)
  - Three-layer validation enforcement (pre-commit, CI/CD, manual)
  - Testing standards documentation (testing_standards.md, seed_data_reference.md)
  - v2 test helpers for ergonomic database lifecycle management
  - 120 tests (all passing)

- **2025-12-09 (v0.1.0):** Initial specification - comprehensive token management with 288 tests

**Next Milestone:** PostgreSQL migration for production deployment
