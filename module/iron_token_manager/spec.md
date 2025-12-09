# Iron Token Manager - Specification

**Module:** iron_token_manager
**Version:** 0.1.0
**Status:** Implemented
**Layer:** 3 (Feature)
**Date:** 2025-12-09

---

## Scope

**Responsibility:**
Manages API token lifecycle for Iron Cage with secure generation, SHA-256 hashing, SQLite storage, JWT authentication, usage tracking, quota enforcement, and token bucket rate limiting. Provides complete authentication and authorization infrastructure for API access control.

**In Scope:**
- Cryptographic token generation (Base64, high-entropy)
- SHA-256 token hashing (never store plaintext)
- Token CRUD operations (create, verify, revoke, list, update)
- Token expiration and deactivation
- Usage tracking per token (requests, tokens consumed, cost in USD)
- Quota enforcement (daily limits, cost caps, hard cutoffs)
- Token bucket rate limiting (requests per second)
- JWT authentication and validation
- SQLite persistence with migrations
- Token metadata (name, created_at, last_used, owner)

**Out of Scope:**
- OAuth2/OIDC integration (future enhancement)
- API key rotation automation (future enhancement)
- Multi-tenant token isolation (future enhancement)
- Token analytics and reporting UI (see iron_dashboard)
- REST API endpoints (see iron_api)
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
- Uses PostgreSQL (iron_control_store schema)
- Tokens centralized for multi-user access
- Replicated across Control Panel instances

---

## Dependencies

**Required:**
- iron_types (foundation types, errors)
- iron_state (state management)
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

---

## Architecture

### Module Structure

```
iron_token_manager/
├── src/
│   ├── lib.rs                  # Public API, TokenManager
│   ├── token_generator.rs      # Cryptographic token generation
│   ├── storage.rs              # SQLite persistence layer
│   ├── usage_tracker.rs        # Usage tracking and quota enforcement
│   ├── rate_limiter.rs         # Token bucket rate limiting
│   ├── limit_enforcer.rs       # Quota enforcement logic
│   ├── cost_calculator.rs      # Cost calculation (uses iron_cost)
│   ├── trace_storage.rs        # Trace storage for debugging
│   ├── provider_adapter.rs     # Adapter for external providers
│   └── error.rs                # Error types
├── migrations/                 # SQLite schema migrations
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

### Indexes

```sql
CREATE INDEX idx_usage_token_id ON usage(token_id);
CREATE INDEX idx_usage_timestamp ON usage(timestamp DESC);
CREATE INDEX idx_tokens_hash ON tokens(hash);
CREATE INDEX idx_tokens_owner ON tokens(owner);
```

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
- ✅ Integration with iron_state for audit
- ✅ Integration tests (288 tests)

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

### With iron_api
- iron_api calls `token_manager.verify()` on every API request
- Middleware extracts Bearer token from Authorization header
- Returns 401 if token invalid, 429 if rate limited

### With iron_cost
- Uses iron_cost types for cost tracking
- Delegates cost calculation to iron_cost
- Stores calculated costs in usage table

### With iron_state
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
- PostgreSQL database (iron_control_store schema)
- Multi-instance with replication
- Redis for rate limiter state sharing

**Migration:**
1. Export tokens from SQLite
2. Import to PostgreSQL (iron_control_store)
3. Update storage backend configuration
4. Deploy Redis for rate limiter
5. Verify quota enforcement across replicas

---

## Revision History

- **2025-12-09 (v0.1.0):** Initial specification - comprehensive token management with 288 tests

**Next Milestone:** PostgreSQL migration for production deployment
