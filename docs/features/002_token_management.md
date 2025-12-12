# LLM Token Management - Feature Architecture

**Version:** 1.0.0
**Date:** 2025-12-02
**Status:** Design document

---

### Scope

**Responsibility:** Complete architecture design for LLM Token Management Dashboard and Backend (cross-module feature).

**In Scope:**
- System architecture for token management backend
- Component breakdown (iron_token_manager, iron_control_api, iron_runtime_state extensions)
- Data flow diagrams (token generation, usage tracking, limit enforcement)
- Security architecture (JWT + RBAC, token encryption, API authentication)
- Performance architecture (async-first, database indexing, caching)
- Testing strategy (unit, integration, E2E tests)
- Implementation phases (10-week plan)

**Out of Scope:**
- System-wide Iron Cage architecture (see `architecture.md`)
- Platform capability integration (see `technical_architecture.md`)
- Module implementation details (see individual crate `spec.md` files)
- Deployment procedures (see `deployment_guide.md`)

---

## 1. Overview

This document defines the architecture for implementing an LLM token management system with backend API and dashboard UI. The system allows users to:
- Generate API tokens for LLM inference provider access
- Track usage per user/project/provider
- Enforce hard limits on token usage
- Rate limit API calls
- View analytics dashboard

**Critical Timeline Note:** This task requires approximately 70 days (10 weeks) of implementation effort. With the pilot deadline of December 17, 2025 (23 days away), this task is **recommended for deferral to post-pilot** (Q1 2026).

---

## 2. System Architecture

### 2.1 High-Level Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        EXTERNAL CLIENTS                          │
│  (Dashboard UI, CLI tools, External APIs)                        │
└────────────────┬─────────────────────────────────────────────────┘
                 │
                 │ HTTPS (JWT tokens)
                 │
┌────────────────▼─────────────────────────────────────────────────┐
│                       API GATEWAY LAYER                          │
│  • Authentication (JWT verification)                             │
│  • Authorization (RBAC: Admin/User/Agent roles)                  │
│  • Rate limiting (per-token)                                     │
│  • Request validation                                            │
└────────────────┬─────────────────────────────────────────────────┘
                 │
                 │
┌────────────────▼─────────────────────────────────────────────────┐
│                      BUSINESS LOGIC LAYER                        │
│                                                                  │
│  ┌────────────────────┐  ┌────────────────────┐                │
│  │ Token Management   │  │  Usage Tracking    │                │
│  │ • Generate tokens  │  │  • Record calls    │                │
│  │ • Rotate tokens    │  │  • Calculate costs │                │
│  │ • Revoke tokens    │  │  • Aggregate stats │                │
│  └────────────────────┘  └────────────────────┘                │
│                                                                  │
│  ┌────────────────────┐  ┌────────────────────┐                │
│  │ Limit Enforcement  │  │  Rate Limiting     │                │
│  │ • Check limits     │  │  • Token bucket    │                │
│  │ • Grace periods    │  │  • Per-minute      │                │
│  │ • Notifications    │  │  • Per-hour/day    │                │
│  └────────────────────┘  └────────────────────┘                │
│                                                                  │
└────────────────┬─────────────────────────────────────────────────┘
                 │
                 │
┌────────────────▼─────────────────────────────────────────────────┐
│                    INFRASTRUCTURE LAYER                          │
│                                                                  │
│  ┌────────────────────┐  ┌────────────────────┐                │
│  │   State Storage    │  │   Observability    │                │
│  │ • PostgreSQL/SQLite│  │  • Metrics export  │                │
│  │ • Connection pool  │  │  • Structured logs │                │
│  │ • Migrations       │  │  • Health checks   │                │
│  └────────────────────┘  └────────────────────┘                │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                       PERSISTENCE LAYER                          │
│  • api_tokens (id, hash, user_id, project_id, created_at, ...)  │
│  • token_usage (id, token_id, provider, tokens, cost, ...)      │
│  • usage_limits (id, user_id, project_id, limit_tokens, ...)    │
│  • api_call_traces (id, token_id, timestamp, request, ...)      │
│  • audit_log (id, user_id, action, resource_id, timestamp, ...) │
└──────────────────────────────────────────────────────────────────┘
```

### 2.2 Layered Architecture

**Follows existing iron_cage patterns:**

1. **Presentation Layer** (Dashboard)
   - Vue 3 + TypeScript SPA
   - Separate deployment from backend
   - Uses REST API for all operations

2. **API Layer** (`iron_control_api` enhanced)
   - Axum HTTP server
   - JWT authentication middleware
   - RBAC authorization
   - OpenAPI documentation

3. **Business Logic Layer** (`iron_token_manager` new crate)
   - Token generation/rotation/revocation
   - Usage tracking and aggregation
   - Limit enforcement
   - Rate limiting

4. **Infrastructure Layer** (`iron_runtime_state` enhanced)
   - Database connection management
   - Migration runner
   - Health checks

5. **Persistence Layer**
   - PostgreSQL (production)
   - SQLite (development/testing)

---

## 3. Component Breakdown

### 3.1 Backend Components

#### 3.1.1 iron_token_manager (New Crate)

**Responsibility:** Core token management business logic.

**Modules:**
- `token_generator` - Generate cryptographically secure tokens
- `usage_tracker` - Record and aggregate usage data
- `limit_enforcer` - Validate against usage limits
- `rate_limiter` - Enforce rate limits per token
- `provider_adapter` - Integrate with LLM provider SDKs
- `storage` - Database access layer
- `error` - Error types

**Key Structures:**

```rust
// Token generation
pub struct TokenGenerator
{
  hasher: Blake3Hasher,
  random: SecureRandom,
}

impl TokenGenerator
{
  pub fn generate_token(
    &self,
    user_id: i64,
    project_id: i64,
    provider: Provider,
  ) -> Result< ApiToken >
  {
    // Generate cryptographically secure random token
    let token_bytes: [u8; 32] = self.random.generate();
    let token_string = base64::encode_url_safe(&token_bytes);

    // Hash for storage (SHA-256, never store plaintext)
    let token_hash = sha2::Sha256::digest(&token_bytes);

    Ok(ApiToken
    {
      id: 0, // Set by database
      token_hash: token_hash.to_vec(),
      token_string, // Return once, never stored
      user_id,
      project_id,
      provider,
      created_at: Utc::now(),
      expires_at: None,
      revoked_at: None,
    })
  }
}

// Usage tracking
pub struct UsageTracker
{
  storage: Arc< dyn TokenStorage >,
  cost_calculator: CostCalculator,
}

impl UsageTracker
{
  pub async fn record_usage(
    &self,
    token_id: i64,
    provider: Provider,
    input_tokens: u32,
    output_tokens: u32,
  ) -> Result< UsageRecord >
  {
    // Calculate cost based on provider pricing
    let cost = self.cost_calculator.calculate(
      provider,
      input_tokens,
      output_tokens,
    );

    // Record usage (validate-then-mutate pattern)
    let usage = UsageRecord
    {
      token_id,
      provider,
      input_tokens,
      output_tokens,
      cost_usd: cost,
      timestamp: Utc::now(),
    };

    self.storage.insert_usage(&usage).await?;
    Ok(usage)
  }

  pub async fn aggregate_usage(
    &self,
    token_id: i64,
    window: TimeWindow,
  ) -> Result< AggregateUsage >
  {
    // Aggregate usage data for analytics
    self.storage.query_usage_aggregates(token_id, window).await
  }
}

// Limit enforcement
pub struct LimitEnforcer
{
  storage: Arc< dyn TokenStorage >,
}

impl LimitEnforcer
{
  pub async fn check_limit(
    &self,
    token_id: i64,
    requested_tokens: u32,
  ) -> Result< LimitCheckResult >
  {
    // Fetch current usage and limits
    let limit = self.storage.get_limit_for_token(token_id).await?;
    let usage = self.storage.get_current_usage(token_id).await?;

    // Validate BEFORE allowing operation (validate-then-mutate)
    if usage.total_tokens + requested_tokens > limit.max_tokens
    {
      // Check grace period
      if limit.grace_period_tokens > 0
      {
        let total_with_grace = limit.max_tokens + limit.grace_period_tokens;
        if usage.total_tokens + requested_tokens <= total_with_grace
        {
          return Ok(LimitCheckResult::AllowedWithWarning);
        }
      }

      return Ok(LimitCheckResult::Denied);
    }

    Ok(LimitCheckResult::Allowed)
  }
}

// Rate limiting
pub struct RateLimiter
{
  limiters: DashMap< i64, Governor >, // token_id -> rate limiter
}

impl RateLimiter
{
  pub async fn check_rate_limit(
    &self,
    token_id: i64,
    rate_config: RateConfig,
  ) -> Result< RateLimitResult >
  {
    // Get or create rate limiter for token
    let limiter = self.limiters.entry(token_id).or_insert_with(||
    {
      // Token bucket algorithm
      Governor::new(
        rate_config.requests_per_minute,
        Duration::from_secs(60),
      )
    });

    // Check rate limit (non-blocking)
    match limiter.check()
    {
      Ok(_) => Ok(RateLimitResult::Allowed),
      Err(not_until) =>
      {
        let retry_after = not_until.wait_time_from(Instant::now());
        Ok(RateLimitResult::Denied { retry_after })
      }
    }
  }
}
```

**Dependencies:**
- `iron_types` - Shared type definitions
- `iron_runtime_state` - Database access
- `iron_cost` - Cost calculation (reuse existing)
- `error_tools` - Error handling (per rulebook)
- `rand` - Cryptographic randomness
- `sha2` - SHA-256 hashing
- `blake3` - BLAKE3 hashing (fast)
- `sqlx` - Async database driver
- `tokio` - Async runtime
- `governor` - Rate limiting
- `base64` - Token encoding

#### 3.1.2 iron_control_api (Enhanced)

**Responsibility:** HTTP API server with authentication.

**Enhancements:**
- Add JWT authentication middleware
- Add RBAC authorization
- Add token management endpoints
- Add analytics endpoints

**New Endpoints:**

```rust
// Authentication
POST   /api/v1/auth/login          // JWT login
POST   /api/v1/auth/refresh        // Refresh token

// Token management
POST   /api/v1/tokens               // Generate new token
GET    /api/v1/tokens               // List tokens
GET    /api/v1/tokens/:id           // Get token details
PUT    /api/v1/tokens/:id/rotate   // Rotate token
DELETE /api/v1/tokens/:id           // Revoke token

// Usage analytics
GET    /api/v1/usage                // Aggregate usage
GET    /api/v1/usage/:token_id      // Token-specific usage
GET    /api/v1/usage/by-project     // Project-level usage
GET    /api/v1/usage/by-provider    // Provider breakdown

// Limits management
GET    /api/v1/limits                // List limits
PUT    /api/v1/limits/:id            // Update limit
POST   /api/v1/limits                // Create limit

// Call tracing
GET    /api/v1/traces                // Query call traces
GET    /api/v1/traces/:id            // Get trace details

// Health
GET    /api/v1/health               // Health check
```

**Authentication Middleware:**

```rust
pub struct JwtAuth
{
  secret: Vec< u8 >,
  validator: jsonwebtoken::Validation,
}

impl JwtAuth
{
  pub async fn authenticate(
    &self,
    req: Request< Body >,
  ) -> Result< (Request< Body >, Claims) >
  {
    // Extract JWT from Authorization header
    let auth_header = req.headers()
      .get("Authorization")
      .ok_or(ApiError::Unauthorized)?;

    let token = auth_header
      .to_str()?
      .strip_prefix("Bearer ")
      .ok_or(ApiError::InvalidToken)?;

    // Verify JWT signature
    let token_data = jsonwebtoken::decode::<Claims>(
      token,
      &self.secret,
      &self.validator,
    )?;

    // Return request with claims attached
    Ok((req, token_data.claims))
  }
}

pub struct RbacAuth;

impl RbacAuth
{
  pub fn authorize(
    claims: &Claims,
    required_role: Role,
  ) -> Result< () >
  {
    // Check role hierarchy: Admin > User > Agent
    if claims.role < required_role
    {
      return Err(ApiError::Forbidden);
    }
    Ok(())
  }
}
```

#### 3.1.3 iron_runtime_state (Enhanced)

**Responsibility:** Database schema and migrations.

**New Tables:**

```sql
-- API tokens (stores hashed tokens only)
CREATE TABLE api_tokens
(
  id BIGSERIAL PRIMARY KEY,
  token_hash BYTEA NOT NULL UNIQUE,  -- SHA-256 hash (never plaintext)
  user_id BIGINT NOT NULL,
  project_id BIGINT,
  provider VARCHAR(50) NOT NULL,     -- openai, anthropic, google, etc.
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  last_used_at TIMESTAMPTZ,
  metadata JSONB,                    -- Additional token metadata
  INDEX idx_user_id (user_id),
  INDEX idx_project_id (project_id),
  INDEX idx_token_hash (token_hash)
);

-- Token usage tracking
CREATE TABLE token_usage
(
  id BIGSERIAL PRIMARY KEY,
  token_id BIGINT NOT NULL REFERENCES api_tokens(id),
  provider VARCHAR(50) NOT NULL,
  input_tokens INTEGER NOT NULL,
  output_tokens INTEGER NOT NULL,
  cost_usd DECIMAL(10, 6) NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  INDEX idx_token_id (token_id),
  INDEX idx_timestamp (timestamp),
  INDEX idx_provider (provider)
);

-- Usage limits
CREATE TABLE usage_limits
(
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL,
  project_id BIGINT,
  provider VARCHAR(50),              -- NULL = applies to all providers
  limit_tokens BIGINT NOT NULL,      -- Hard limit
  grace_tokens BIGINT DEFAULT 0,     -- Grace period allowance
  period VARCHAR(20) NOT NULL,       -- hourly, daily, monthly
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  INDEX idx_user_id (user_id),
  INDEX idx_project_id (project_id)
);

-- API call traces (for debugging)
CREATE TABLE api_call_traces
(
  id BIGSERIAL PRIMARY KEY,
  token_id BIGINT NOT NULL REFERENCES api_tokens(id),
  timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  request_id UUID NOT NULL,
  provider VARCHAR(50) NOT NULL,
  model VARCHAR(100) NOT NULL,
  input_tokens INTEGER NOT NULL,
  output_tokens INTEGER NOT NULL,
  latency_ms INTEGER NOT NULL,
  status VARCHAR(20) NOT NULL,       -- success, rate_limited, error
  error_message TEXT,
  INDEX idx_token_id (token_id),
  INDEX idx_timestamp (timestamp),
  INDEX idx_request_id (request_id)
);

-- Audit log
CREATE TABLE audit_log
(
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL,
  action VARCHAR(50) NOT NULL,       -- token_created, token_revoked, limit_updated
  resource_type VARCHAR(50) NOT NULL,
  resource_id BIGINT,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  ip_address INET,
  metadata JSONB,
  INDEX idx_user_id (user_id),
  INDEX idx_timestamp (timestamp),
  INDEX idx_action (action)
);
```

**Indexing Strategy:**
- Primary keys: B-tree indexes (default)
- Foreign keys: B-tree indexes for joins
- Timestamp columns: B-tree indexes for time-range queries
- token_hash: UNIQUE index for fast lookups
- Composite indexes for common query patterns

### 3.2 Frontend Components

#### 3.2.1 Dashboard (Vue 3 + TypeScript)

**Responsibility:** Web-based dashboard UI.

**Technology Stack:**
- Vue 3 (Composition API, script setup)
- TypeScript 5.x (type safety)
- Vite (build tool)
- Vue Router (routing)
- Pinia (state management)
- TanStack Query Vue / VueQuery (data fetching)
- Chart.js with vue-chartjs (visualization)
- Tailwind CSS (styling)
- shadcn-vue (component library based on Radix Vue)

**Views:**

1. **Token Management View**
   - List all tokens (table)
   - Generate new token (modal)
   - Rotate token (action button)
   - Revoke token (action button)
   - Copy token to clipboard
   - Token metadata display

2. **Usage Analytics View**
   - Total usage summary (cards)
   - Usage over time (line chart)
   - Usage by provider (pie chart)
   - Usage by project (bar chart)
   - Cost breakdown (table)
   - Date range selector

3. **Limits Management View**
   - Current limits (table)
   - Create new limit (form)
   - Edit limit (modal)
   - Delete limit (action button)
   - Grace period configuration
   - Period selection (hourly/daily/monthly)

4. **Call Tracing View**
   - Recent API calls (table)
   - Call details (drawer)
   - Filtering (provider, status, date range)
   - Pagination
   - Export to CSV

**Component Structure:**

```typescript
// src/App.vue
<script setup lang="ts">
import { RouterView } from 'vue-router'
import { VueQueryPlugin } from '@tanstack/vue-query'
</script>

<template>
  <AuthProvider>
    <Layout>
      <RouterView />
    </Layout>
  </AuthProvider>
</template>

// src/router/index.ts
import { createRouter, createWebHistory } from 'vue-router'
import TokensView from '@/views/TokensView.vue'
import UsageView from '@/views/UsageView.vue'
import LimitsView from '@/views/LimitsView.vue'
import TracesView from '@/views/TracesView.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/tokens', component: TokensView },
    { path: '/usage', component: UsageView },
    { path: '/limits', component: LimitsView },
    { path: '/traces', component: TracesView },
  ],
})

// src/views/TokensView.vue
<script setup lang="ts">
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { fetchTokens, generateToken } from '@/api/tokens'
import TokensTable from '@/components/TokensTable.vue'
import GenerateTokenButton from '@/components/GenerateTokenButton.vue'

const queryClient = useQueryClient()

const { data: tokens, isLoading } = useQuery({
  queryKey: ['tokens'],
  queryFn: fetchTokens,
})

const generateMutation = useMutation({
  mutationFn: generateToken,
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['tokens'] })
  },
})
</script>

<template>
  <div>
    <TokensTable :tokens="tokens" :is-loading="isLoading" />
    <GenerateTokenButton @click="generateMutation.mutate" />
  </div>
</template>

// src/stores/auth.ts (Pinia store)
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(null)
  const user = ref<User | null>(null)

  const isAuthenticated = computed(() => token.value !== null)

  function setAuth(newToken: string, newUser: User) {
    token.value = newToken
    user.value = newUser
  }

  function clearAuth() {
    token.value = null
    user.value = null
  }

  return { token, user, isAuthenticated, setAuth, clearAuth }
})
```

**API Client:**

```typescript
// src/api/client.ts
import { useAuthStore } from '@/stores/auth'

export class ApiClient {
  private baseUrl: string

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl
  }

  private getToken(): string | null {
    const auth = useAuthStore()
    return auth.token
  }

  async generateToken(req: GenerateTokenRequest): Promise<ApiToken> {
    const response = await fetch(`${this.baseUrl}/api/v1/tokens`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.getToken()}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(req),
    })

    if (!response.ok) {
      throw new ApiError(response.status, await response.text())
    }

    return response.json()
  }

  async fetchUsage(params: UsageQueryParams): Promise<UsageData> {
    const query = new URLSearchParams(params as any)
    const response = await fetch(
      `${this.baseUrl}/api/v1/usage?${query}`,
      {
        headers: { 'Authorization': `Bearer ${this.getToken()}` },
      }
    )

    if (!response.ok) {
      throw new ApiError(response.status, await response.text())
    }

    return response.json()
  }
}

// src/api/tokens.ts
import { apiClient } from './instance'

export const fetchTokens = async (): Promise<ApiToken[]> => {
  return apiClient.fetchTokens()
}

export const generateToken = async (req: GenerateTokenRequest): Promise<ApiToken> => {
  return apiClient.generateToken(req)
}
```

---

## 4. Data Flow Diagrams

### 4.1 Token Generation Flow

```
User (Dashboard)
  │
  │ 1. POST /api/v1/tokens
  │    { user_id, project_id, provider }
  │
  ▼
API Gateway (iron_control_api)
  │
  │ 2. Authenticate JWT
  │ 3. Authorize (RBAC: User role)
  │
  ▼
TokenGenerator (iron_token_manager)
  │
  │ 4. Generate random 256-bit token
  │ 5. Hash token (SHA-256)
  │ 6. Create ApiToken struct
  │
  ▼
TokenStorage (iron_runtime_state)
  │
  │ 7. INSERT INTO api_tokens
  │    (token_hash, user_id, project_id, provider, ...)
  │ 8. Return token_id
  │
  ▼
API Response
  │
  │ 9. Return { id, token_string, created_at, ... }
  │    (token_string returned ONCE, never stored)
  │
  ▼
Dashboard
  │
  │ 10. Display token
  │ 11. Copy to clipboard
  │ 12. Show "Save this token - you won't see it again"
```

### 4.2 Usage Tracking Flow

```
Client (Agent/CLI)
  │
  │ 1. LLM API call with token
  │    Authorization: Bearer <token>
  │
  ▼
Provider Adapter (iron_token_manager)
  │
  │ 2. Hash token, lookup in database
  │ 3. Verify token is valid (not revoked/expired)
  │ 4. Check rate limit (requests per minute)
  │
  ▼
LLM Provider (OpenAI/Anthropic/Google)
  │
  │ 5. Forward request to provider
  │ 6. Receive response with usage data
  │    { input_tokens: 100, output_tokens: 50 }
  │
  ▼
UsageTracker (iron_token_manager)
  │
  │ 7. Calculate cost (provider pricing)
  │ 8. Record usage in database
  │    INSERT INTO token_usage (...)
  │ 9. Record call trace
  │    INSERT INTO api_call_traces (...)
  │
  ▼
Telemetry (iron_observability)
  │
  │ 10. Export metrics
  │     - total_tokens_used
  │     - cost_usd
  │     - api_call_latency_ms
```

### 4.3 Limit Enforcement Flow

```
Client Request
  │
  │ 1. POST /api/v1/inference
  │    Authorization: Bearer <token>
  │    { model, messages, max_tokens: 500 }
  │
  ▼
API Gateway (iron_control_api)
  │
  │ 2. Authenticate token
  │ 3. Extract requested_tokens (500)
  │
  ▼
LimitEnforcer (iron_token_manager)
  │
  │ 4. Fetch limit for token
  │    SELECT * FROM usage_limits WHERE user_id = ?
  │ 5. Fetch current usage
  │    SELECT SUM(input_tokens + output_tokens) FROM token_usage
  │    WHERE token_id = ? AND timestamp >= period_start
  │ 6. Validate: current + requested <= limit
  │
  ▼
Decision Point
  │
  ├─ If within limit
  │   │
  │   │ 7. Allow request
  │   ▼
  │   LLM Provider
  │
  ├─ If within grace period
  │   │
  │   │ 8. Allow with warning
  │   │ 9. Send notification
  │   ▼
  │   LLM Provider
  │
  └─ If exceeds limit + grace
      │
      │ 10. Reject request
      │ 11. Return 429 Too Many Requests
      │     { error: "Usage limit exceeded", retry_after: ... }
      ▼
      Client (receives error)
```

---

## 5. Security Architecture

### 5.1 Authentication

**JWT-Based Authentication:**
- Access tokens (1 hour expiry)
- Refresh tokens (7 days expiry)
- HMAC-SHA256 signature
- Claims: `user_id`, `role`, `exp`, `iat`

**Token Storage:**
- API tokens: SHA-256 hash only (never plaintext)
- Plaintext token returned once during generation
- Client responsibility to store securely

### 5.2 Authorization

**Role-Based Access Control (RBAC):**

| Role  | Permissions |
|-------|-------------|
| Admin | Full access: manage all tokens, view all usage, configure limits |
| User  | Manage own tokens, view own usage, configure own limits |
| Agent | API access only: use tokens for LLM calls, no dashboard access |

**Endpoint Authorization Matrix:**

| Endpoint | Admin | User | Agent |
|----------|-------|------|-------|
| POST /api/v1/tokens | ✅ All users | ✅ Own | ❌ |
| GET /api/v1/tokens | ✅ All tokens | ✅ Own | ❌ |
| DELETE /api/v1/tokens/:id | ✅ All | ✅ Own | ❌ |
| GET /api/v1/usage | ✅ All | ✅ Own | ❌ |
| PUT /api/v1/limits/:id | ✅ All | ✅ Own | ❌ |
| POST /api/v1/inference | ✅ | ✅ | ✅ |

### 5.3 Data Protection

**Encryption:**
- In transit: TLS 1.3
- At rest: Database-level encryption (PostgreSQL + pgcrypto)
- Secrets: Never log tokens or credentials

**Input Validation:**
- Validate all API inputs (JSON schema)
- Sanitize user-provided strings
- Rate limit authentication attempts (10 per minute)
- CSRF protection for dashboard

### 5.4 Audit Logging

**Audit Events:**
- Token created/rotated/revoked
- Limit created/updated/deleted
- Authentication success/failure
- Authorization denied (403)
- Unusual usage patterns (spike detection)

**Log Fields:**
- `user_id`, `action`, `resource_type`, `resource_id`, `timestamp`, `ip_address`, `metadata`

---

## 6. Performance Architecture

### 6.1 Async-First Design

**Tokio Runtime:**
- All I/O operations are async (database, HTTP)
- Non-blocking request handling
- Efficient resource utilization

**Concurrency:**
- Connection pooling (SQLx pool size: 20)
- DashMap for concurrent in-memory caching
- Rate limiter per-token state (lock-free)

### 6.2 Database Optimization

**Indexing Strategy:**
- B-tree indexes on all foreign keys
- Composite indexes for common queries
- Partial indexes for active tokens (`WHERE revoked_at IS NULL`)

**Query Optimization:**
- Use prepared statements (SQLx)
- Batch inserts for usage records
- Pagination for large result sets
- Aggregate queries with materialized views (optional)

**Connection Pooling:**
```rust
let pool = PgPoolOptions::new()
  .max_connections(20)
  .acquire_timeout(Duration::from_secs(5))
  .connect(&database_url).await?;
```

### 6.3 Caching

**In-Memory Cache:**
- DashMap for token metadata (read-heavy)
- TTL: 5 minutes
- Invalidate on token revocation

**Cache Patterns:**
```rust
pub struct TokenCache
{
  cache: DashMap< i64, Arc< ApiToken > >,
}

impl TokenCache
{
  pub async fn get_or_fetch(
    &self,
    token_id: i64,
    storage: &dyn TokenStorage,
  ) -> Result< Arc< ApiToken > >
  {
    // Try cache first
    if let Some(token) = self.cache.get(&token_id)
    {
      return Ok(Arc::clone(token.value()));
    }

    // Fetch from database
    let token = storage.get_token(token_id).await?;
    let token_arc = Arc::new(token);

    // Store in cache
    self.cache.insert(token_id, Arc::clone(&token_arc));
    Ok(token_arc)
  }
}
```

### 6.4 Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| API latency (p50) | < 50ms | Histogram |
| API latency (p95) | < 100ms | Histogram |
| API latency (p99) | < 200ms | Histogram |
| Dashboard load time | < 2s | Browser DevTools |
| Database query time | < 10ms | SQLx logging |
| Concurrent requests | 10,000+ calls/min | Load testing |
| Token validation | < 5ms | Benchmark |

---

## 7. Testing Strategy

### 7.1 Test Pyramid

```
           E2E Tests (10%)
        ┌──────────────┐
        │ Dashboard UI │
        │ API endpoints│
        └──────────────┘
              ▲
              │
     Integration Tests (30%)
   ┌────────────────────────┐
   │ API → Business Logic   │
   │ Business Logic → DB    │
   │ Auth middleware        │
   └────────────────────────┘
              ▲
              │
        Unit Tests (60%)
   ┌────────────────────────┐
   │ Token generation       │
   │ Usage calculation      │
   │ Limit validation       │
   │ Rate limiting          │
   └────────────────────────┘
```

### 7.2 Unit Tests

**Location:** `module/iron_token_manager/tests/`

**Test Cases:**

```rust
// tests/token_generator_tests.rs
#[test]
fn test_generate_token_uniqueness()
{
  let generator = TokenGenerator::new();

  let token1 = generator.generate_token(1, 1, Provider::OpenAI).unwrap();
  let token2 = generator.generate_token(1, 1, Provider::OpenAI).unwrap();

  assert_ne!(token1.token_string, token2.token_string);
  assert_ne!(token1.token_hash, token2.token_hash);
}

#[test]
fn test_token_hash_irreversible()
{
  let generator = TokenGenerator::new();
  let token = generator.generate_token(1, 1, Provider::OpenAI).unwrap();

  // Hash should be 32 bytes (SHA-256)
  assert_eq!(token.token_hash.len(), 32);

  // Should not contain original token
  assert!(!token.token_hash.contains(&token.token_string.as_bytes()[0]));
}

// tests/usage_tracker_tests.rs
#[tokio::test]
async fn test_record_usage_calculates_cost()
{
  let storage = MockStorage::new();
  let tracker = UsageTracker::new(Arc::new(storage));

  let usage = tracker.record_usage(
    1,
    Provider::OpenAI,
    100,  // input tokens
    50,   // output tokens
  ).await.unwrap();

  // OpenAI GPT-4: $30/1M input, $60/1M output
  let expected_cost = (100.0 * 30.0 / 1_000_000.0) + (50.0 * 60.0 / 1_000_000.0);
  assert_eq!(usage.cost_usd, expected_cost);
}

// tests/limit_enforcer_tests.rs
#[tokio::test]
async fn test_limit_enforcement_rejects_exceeded()
{
  let storage = MockStorage::new();
  storage.set_limit(1, 1000); // 1000 token limit
  storage.set_usage(1, 900);  // 900 tokens used

  let enforcer = LimitEnforcer::new(Arc::new(storage));

  // Request 200 tokens (would exceed 1000)
  let result = enforcer.check_limit(1, 200).await.unwrap();

  assert_eq!(result, LimitCheckResult::Denied);
}

#[tokio::test]
async fn test_limit_enforcement_allows_grace_period()
{
  let storage = MockStorage::new();
  storage.set_limit_with_grace(1, 1000, 100); // 1000 + 100 grace
  storage.set_usage(1, 950);  // 950 tokens used

  let enforcer = LimitEnforcer::new(Arc::new(storage));

  // Request 75 tokens (exceeds 1000 but within grace)
  let result = enforcer.check_limit(1, 75).await.unwrap();

  assert_eq!(result, LimitCheckResult::AllowedWithWarning);
}
```

### 7.3 Integration Tests

**Location:** `module/iron_control_api/tests/`

**Test Cases:**

```rust
// tests/api_integration_tests.rs
#[tokio::test]
async fn test_generate_token_endpoint()
{
  let app = setup_test_app().await;

  let response = app.post("/api/v1/tokens")
    .json(&json!({
      "user_id": 1,
      "project_id": 1,
      "provider": "openai"
    }))
    .bearer_auth("test_jwt_token")
    .send()
    .await
    .unwrap();

  assert_eq!(response.status(), 201);

  let token: ApiToken = response.json().await.unwrap();
  assert!(!token.token_string.is_empty());
  assert_eq!(token.user_id, 1);
  assert_eq!(token.provider, "openai");
}

#[tokio::test]
async fn test_usage_endpoint_requires_auth()
{
  let app = setup_test_app().await;

  let response = app.get("/api/v1/usage")
    .send()  // No auth header
    .await
    .unwrap();

  assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_revoke_token_prevents_usage()
{
  let app = setup_test_app().await;

  // Create token
  let token = create_test_token(&app).await;

  // Revoke token
  let response = app.delete(format!("/api/v1/tokens/{}", token.id))
    .bearer_auth("admin_jwt")
    .send()
    .await
    .unwrap();

  assert_eq!(response.status(), 204);

  // Try to use revoked token
  let response = app.post("/api/v1/inference")
    .bearer_auth(&token.token_string)
    .json(&json!({ "model": "gpt-4", "messages": [] }))
    .send()
    .await
    .unwrap();

  assert_eq!(response.status(), 401);
}
```

### 7.4 End-to-End Tests

**Location:** `tests/e2e/`

**Test Cases:**

```rust
// tests/e2e/dashboard_tests.rs
#[tokio::test]
async fn test_complete_token_lifecycle()
{
  let app = setup_test_app().await;
  let browser = setup_test_browser().await;

  // 1. Login to dashboard
  browser.goto("http://localhost:3000/login").await.unwrap();
  browser.fill("#username", "testuser").await.unwrap();
  browser.fill("#password", "testpass").await.unwrap();
  browser.click("button[type=submit]").await.unwrap();

  // 2. Navigate to tokens page
  browser.wait_for_selector(".tokens-table").await.unwrap();

  // 3. Generate new token
  browser.click("button.generate-token").await.unwrap();
  browser.fill("#provider", "openai").await.unwrap();
  browser.click("button.confirm").await.unwrap();

  // 4. Verify token appears in table
  let token_row = browser.wait_for_selector(".token-row").await.unwrap();
  let token_id = token_row.get_attribute("data-token-id").await.unwrap();

  // 5. Use token for API call
  let client = reqwest::Client::new();
  let response = client.post("http://localhost:8080/api/v1/inference")
    .bearer_token(&token_id)
    .json(&json!({ "model": "gpt-4", "messages": [] }))
    .send()
    .await
    .unwrap();

  assert_eq!(response.status(), 200);

  // 6. Verify usage appears in dashboard
  browser.click("a.usage-link").await.unwrap();
  browser.wait_for_selector(".usage-chart").await.unwrap();

  let usage_text = browser.text(".total-usage").await.unwrap();
  assert!(usage_text.contains("1 call"));
}
```

### 7.5 Load Testing

**Tool:** Apache Bench (ab) or k6

**Test Scenarios:**

```bash
# Scenario 1: Token validation throughput
ab -n 10000 -c 100 -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/v1/health

# Scenario 2: Usage recording throughput
k6 run - <<EOF
import http from 'k6/http';
export let options = {
  vus: 100,
  duration: '60s',
};
export default function() {
  http.post('http://localhost:8080/api/v1/inference', JSON.stringify({
    model: 'gpt-4',
    messages: [{ role: 'user', content: 'test' }],
  }), {
    headers: { 'Authorization': 'Bearer <token>' },
  });
}
EOF

# Scenario 3: Dashboard API load
ab -n 5000 -c 50 -H "Authorization: Bearer <jwt>" \
  http://localhost:8080/api/v1/usage
```

**Acceptance Criteria:**
- 10,000+ requests/minute without errors
- p95 latency < 100ms
- Zero failed requests under normal load
- Graceful degradation under 2x expected load

---

## 8. Implementation Phases

### Phase 1: Database Schema + Token Generation (Weeks 1-2)

**Goals:**
- Extend `iron_runtime_state` with token management schema
- Implement `iron_token_manager::token_generator`
- Create database migrations
- Write unit tests

**Deliverables:**
- `api_tokens`, `token_usage`, `usage_limits`, `api_call_traces`, `audit_log` tables
- `TokenGenerator::generate_token()` with SHA-256 hashing
- `TokenStorage` trait + PostgreSQL implementation
- 20+ unit tests

**Verification:**
- `w3 .test l::3` passes
- Manual testing: generate 1000 tokens, verify uniqueness
- Database inspection: verify hashes stored, not plaintext

### Phase 2: Usage Tracking (Weeks 3-4)

**Goals:**
- Implement `iron_token_manager::usage_tracker`
- Implement `iron_token_manager::provider_adapter`
- Integrate with `iron_cost` for cost calculation
- Write unit + integration tests

**Deliverables:**
- `UsageTracker::record_usage()` with cost calculation
- `ProviderAdapter` for OpenAI, Anthropic, Google
- Cost calculator with provider-specific pricing
- 30+ unit tests, 10+ integration tests

**Verification:**
- `w3 .test l::3` passes
- Manual testing: record 100 API calls, verify usage aggregation
- Cost validation: compare calculated costs with provider pricing

### Phase 3: Limits & Rate Limiting (Week 5)

**Goals:**
- Implement `iron_token_manager::limit_enforcer`
- Implement `iron_token_manager::rate_limiter`
- Add grace period support
- Write unit + integration tests

**Deliverables:**
- `LimitEnforcer::check_limit()` with grace period
- `RateLimiter::check_rate_limit()` with token bucket algorithm
- 20+ unit tests, 10+ integration tests

**Verification:**
- `w3 .test l::3` passes
- Manual testing: exceed limit, verify rejection
- Rate limit testing: send 100 requests/sec, verify throttling

### Phase 4: API Endpoints + Authentication (Week 6)

**Goals:**
- Enhance `iron_control_api` with token management endpoints
- Implement JWT authentication middleware
- Implement RBAC authorization
- Write integration tests

**Deliverables:**
- 10+ REST API endpoints (tokens, usage, limits, traces)
- `JwtAuth` middleware with signature verification
- `RbacAuth` with role-based access control
- 30+ integration tests

**Verification:**
- `w3 .test l::3` passes
- API testing: call all endpoints, verify auth/authz
- Postman collection: test happy paths + error cases

### Phase 5: Dashboard UI (Weeks 7-9)

**Goals:**
- Build Vue 3 + TypeScript dashboard
- Implement 4 views (tokens, usage, limits, traces)
- Integrate with backend API
- Write E2E tests

**Deliverables:**
- Vue 3 SPA with routing, authentication (Vue Router, Pinia)
- Token management view with CRUD operations
- Usage analytics view with charts (Chart.js + vue-chartjs)
- Limits management view with forms
- Call tracing view with filtering
- shadcn-vue components for UI consistency
- 20+ E2E tests

**Verification:**
- Dashboard loads in < 2s
- All views functional
- Manual testing: complete token lifecycle
- Browser DevTools: verify no console errors

### Phase 6: Security Hardening + Documentation (Week 10)

**Goals:**
- Conduct security audit
- Add input validation, rate limiting, CSRF protection
- Write comprehensive documentation
- Perform load testing

**Deliverables:**
- Security audit report
- Input validation on all endpoints
- Rate limiting on authentication (10/min)
- CSRF protection for dashboard
- API documentation (OpenAPI spec)
- Deployment guide
- Load testing results

**Verification:**
- Security scan: no high/critical vulnerabilities
- Load testing: 10K+ requests/min, p95 < 100ms
- Documentation review: complete, accurate

---

## 9. Critical Recommendation

**DEFER Task 001 to Post-Pilot (Q1 2026)**

**Rationale:**
- **Implementation effort:** 70 days (10 weeks)
- **Pilot deadline:** December 17, 2025 (23 days away)
- **Timeline conflict:** Mathematically impossible to complete before conference
- **Alternative strategy:** Focus on pilot features (slides-only for Dec 16-17)

**Post-Pilot Timeline:**
- **Start:** January 6, 2026 (after pilot concludes)
- **End:** March 17, 2026 (10 weeks)
- **Target:** Q1 2026 completion with production-ready system

**Pilot Focus:**
- Build demonstration slides showcasing token management concept
- Prepare mockups of dashboard UI for presentation
- Focus on core pilot features (sandbox, safety, cost tracking)

---

## 10. Summary

This architecture design provides a complete, production-ready blueprint for implementing the LLM Token Management Dashboard and Backend. The design:

✅ **Follows existing patterns** - Layered monolith, `error_tools`, async-first
✅ **Comprehensive** - Backend + frontend + security + testing + deployment
✅ **Secure** - JWT + RBAC, token hashing, audit logging, encryption
✅ **Performant** - Async, connection pooling, caching, indexing
✅ **Testable** - 60% unit, 30% integration, 10% E2E tests
✅ **Maintainable** - Clean separation of concerns, documented
✅ **Modern Frontend** - Vue 3 Composition API, TypeScript, shadcn-vue

**Next Steps (if approved to proceed):**
1. Extend `iron_runtime_state` with database schema
2. Implement `iron_token_manager::token_generator`
3. Write unit tests for token generation
4. Proceed through 10-week implementation plan

**Critical:** Confirm timeline alignment before starting implementation (70-day effort vs 23-day deadline).
