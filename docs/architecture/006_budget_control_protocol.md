# Budget Control Protocol

**Purpose:** How runtime and Control Panel communicate to enforce budget limits without exposing provider tokens.

---

## User Need

Developers need budget-controlled LLM access without handling provider API keys directly. Admins need centralized budget control and real-time monitoring.

## Core Idea

**Two-token system with budget borrowing:**

```
Admin (Control Panel)          Developer (Runtime)
+------------------+           +------------------+
| Allocates $100   |           | Gets IC Token    |
| Stores IP Token  |           | (visible)        |
+--------+---------+           +--------+---------+
         |                              |
         | 1. Token Handshake           |
         |<-------- IC Token -----------+
         |                              |
         +-------- IP Token + $10 ----->|
         |        (encrypted)           |
         |                              |
         | 2. LLM Requests              |
         |<--- Usage: 500 tok, $0.01 ---+
         |                              |
         | 3. Budget Refresh (at $9)    |
         |<-------- Need more ----------+
         |                              |
         +--------- + $10 more --------->|
```

## The Two Tokens

| Token | Visible To | Stored | Purpose |
|-------|-----------|--------|---------|
| **IC Token** | Developer | Plaintext on disk | Budget ID, authentication with Control Panel |
| **IP Token** | Runtime only | Encrypted in memory | Actual LLM provider API key |

**Key insight:** Developer NEVER sees provider credentials. Runtime acts as secure proxy.

### IC Token Format (JWT Structure)

**Type:** JSON Web Token (JWT) per RFC 7519

**Claims Schema:**
```json
{
  "agent_id": "agent-abc123",
  "budget_id": "budget-xyz789",
  "issued_at": 1702123456,
  "expires_at": 1702209856,
  "issuer": "iron-control-panel",
  "permissions": ["llm:call", "data:read"]
}
```

**Claim Specifications:**

| Claim | Type | Format | Example | Purpose |
|-------|------|--------|---------|---------|
| `agent_id` | string | `^agent-[a-z0-9]{6,32}$` | "agent-abc123" | Unique agent identifier |
| `budget_id` | string | `^budget-[a-z0-9]{6,32}$` | "budget-xyz789" | Links to budget allocation |
| `issued_at` | number | Unix timestamp (seconds) | 1702123456 | Token creation time |
| `expires_at` | number | Unix timestamp (seconds) | 1702209856 | Expiration (24h default) |
| `issuer` | string | Literal "iron-control-panel" | "iron-control-panel" | Token source validation |
| `permissions` | array | Strings from vocabulary | ["llm:call"] | Allowed operations |

**Validation Rules:**
- Signature: HMAC-SHA256 with Control Panel secret key
- Expiration: `expires_at > current_time`
- Issuer: Must be "iron-control-panel"
- Format: IDs match regex patterns

**Lifetime:** 24 hours (renewable by re-login)

## Budget Borrowing Protocol

### Step 1: Initialization (Token Handshake)

**Message 1: INIT_BUDGET_REQUEST**

**Direction:** Runtime → Control Panel

**HTTP Request:**
```http
POST /api/v1/auth/handshake
Content-Type: application/json
```

**Request Schema:**
```json
{
  "ic_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "requested_budget": 10.00,
  "runtime_version": "0.1.0",
  "runtime_id": "runtime-dev-machine-abc"
}
```

**Field Specifications:**

| Field | Type | Required | Constraints | Purpose |
|-------|------|----------|-------------|---------|
| `ic_token` | string | YES | Valid JWT, 200-400 bytes | IC Token from developer |
| `requested_budget` | number | YES | 0 < x <= 1000 USD | Initial budget portion |
| `runtime_version` | string | YES | Semantic version | Runtime version for compatibility |
| `runtime_id` | string | NO | Unique identifier | Identify Runtime instance |

**Message 2: INIT_BUDGET_RESPONSE**

**Direction:** Control Panel → Runtime

**HTTP Response:**
```http
HTTP/1.1 200 OK
Content-Type: application/json
```

**Response Schema (Success):**
```json
{
  "ip_token": "AES256:YWJjZGVm:c2stcHJval9hYmMxMjM=:MTIzNDU2",
  "budget_granted": 10.00,
  "budget_remaining": 90.00,
  "lease_id": "lease-001",
  "provider": "openai",
  "provider_model": "gpt-4"
}
```

**Field Specifications:**

| Field | Type | Required | Description | Security |
|-------|------|----------|-------------|----------|
| `ip_token` | string | YES | Base64(IV:Ciphertext:Tag) | IP Token encrypted with AES-256-GCM |
| `budget_granted` | number | YES | USD amount allocated | Portion approved for this session |
| `budget_remaining` | number | YES | USD total left | Total budget minus granted |
| `lease_id` | string | YES | Unique lease ID | Track this budget allocation |
| `provider` | string | YES | Provider ID ("openai", "anthropic") | Which LLM provider |
| `provider_model` | string | NO | Model name ("gpt-4") | Default model |

**IP Token Encryption Format:**

```
AES256:{IV_base64}:{ciphertext_base64}:{auth_tag_base64}

Example:
AES256:YWJjZGVmZ2hpams=:c2stcHJval9hYmMxMjNkZWY=:MTIzNDU2Nzg5MA==
│      │               │                      │
Algorithm  IV (12 bytes)  Ciphertext           Auth Tag (16 bytes)
```

**Decryption (Runtime):**
```rust
let parts: Vec<&str> = ip_token.split(':').collect();
let iv = base64::decode(parts[1])?;
let ciphertext = base64::decode(parts[2])?;
let tag = base64::decode(parts[3])?;

let cipher = Aes256Gcm::new(&runtime_session_key);
let plaintext = cipher.decrypt(&iv, &ciphertext, &tag)?;
// plaintext = "sk-proj-abc123def456..." (provider API key)

// Store encrypted in memory
encrypted_memory.store(lease_id, plaintext);

// Zero plaintext
plaintext.zeroize();
```

**Runtime State After Init:**
- IP Token: Encrypted in memory (AES-256)
- Budget: $10.00 allocated, $10.00 remaining
- Lease ID: "lease-001"
- Ready to process LLM calls

### Step 2: Request Execution

**Token Translation (< 1ms):**
```
Agent → Runtime: LLM request (with IC Token)
Runtime: Validate IC Token, decrypt IP Token, replace IC → IP
Runtime → Provider: Request with IP Token
Provider → Runtime: Response + usage metadata
```

**Message 3: BUDGET_USAGE_REPORT**

**Direction:** Runtime → Control Panel (async, non-blocking)

**HTTP Request:**
```http
POST /api/v1/budget/report
Content-Type: application/json
```

**Request Schema:**
```json
{
  "lease_id": "lease-001",
  "request_id": "req-uuid-123",
  "tokens": 1523,
  "cost_usd": 0.0457,
  "model": "gpt-4",
  "provider": "openai",
  "timestamp": 1702123456
}
```

**Field Specifications:**

| Field | Type | Required | Description | Purpose |
|-------|------|----------|-------------|---------|
| `lease_id` | string | YES | Current lease ID | Link to budget allocation |
| `request_id` | string | YES | Unique per LLM call | Idempotency, deduplication |
| `tokens` | number | YES | Total tokens used | From provider response |
| `cost_usd` | number | YES | Calculated cost | tokens × provider pricing |
| `model` | string | YES | Model used | For pricing calculation |
| `provider` | string | YES | Provider used | For audit trail |
| `timestamp` | number | YES | Unix timestamp | When request completed |

**Response:** 200 OK (acknowledgment, no body)

**Performance:**
- Async send: 0ms perceived latency (doesn't block agent)
- Actual network: ~5-20ms (happens in background)

**Runtime Updates Local Budget:**
- Spent: $0.0457
- Remaining: $10.00 - $0.0457 = $9.9543

### Step 3: Budget Refresh

**Trigger:** `remaining_budget < $1.00` threshold

**Message 4: BUDGET_REFRESH_REQUEST**

**Direction:** Runtime → Control Panel

**HTTP Request:**
```http
POST /api/v1/budget/refresh
Content-Type: application/json
```

**Request Schema:**
```json
{
  "lease_id": "lease-001",
  "budget_id": "budget-xyz789",
  "requested_budget": 10.00,
  "current_remaining": 0.85,
  "total_spent": 9.15
}
```

**Field Specifications:**

| Field | Type | Required | Description | Purpose |
|-------|------|----------|-------------|---------|
| `lease_id` | string | YES | Current lease ID | Identify current allocation |
| `budget_id` | string | YES | Budget allocation ID | Link to total budget |
| `requested_budget` | number | YES | Amount requested (USD) | How much more needed |
| `current_remaining` | number | YES | Current lease remaining | For Control Panel validation |
| `total_spent` | number | YES | Total spent so far | For reconciliation |

**Message 5: BUDGET_REFRESH_RESPONSE**

**Direction:** Control Panel → Runtime

**Response Schema (Approved):**
```json
{
  "status": "approved",
  "budget_granted": 10.00,
  "budget_remaining": 80.00,
  "lease_id": "lease-002",
  "total_allocated": 100.00,
  "total_spent": 9.15
}
```

**Field Specifications:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `status` | string | YES | "approved" or "denied" |
| `budget_granted` | number | YES (if approved) | Amount allocated |
| `budget_remaining` | number | YES | Total budget left |
| `lease_id` | string | YES (if approved) | New lease ID |
| `total_allocated` | number | YES | Original total budget |
| `total_spent` | number | YES | Cumulative spending |

**Response Schema (Denied):**
```json
{
  "status": "denied",
  "reason": "total_budget_exhausted",
  "budget_remaining": 0.00,
  "total_allocated": 100.00,
  "total_spent": 100.00
}
```

**Runtime Behavior:**
- If approved: Add $10 to local budget, continue processing
- If denied: Stop accepting new LLM calls, return Error::BudgetExhausted to agent

## Budget Overshoot Prevention

**Local check (fast):**
```
Before LLM call:
if local_budget_remaining < estimated_cost:
  return BudgetExceededError  # <1ms, no network call
```

**Centralized enforcement:**
- Control Panel tracks total spent: $20 of $100
- Refresh denied if allocation exceeded
- Admin can increase allocation in real-time

## Security Model

**IP Token protection:**
- Encrypted with AES-256 in runtime memory
- Encryption key derived from IC Token + salt
- NEVER written to disk (memory-only)
- Cleared on runtime shutdown
- Developer cannot extract (memory protection)

**Threat mitigation:**
- Developer compromise: IC Token leaked → Admin revokes → All leases invalidated
- Memory dump attack: IP Token encrypted, key unavailable outside process
- Disk forensics: No IP Token on disk

## Performance

| Operation | Latency | Frequency |
|-----------|---------|-----------|
| Token handshake | ~50ms | Once per runtime startup |
| Usage reporting | ~5ms | Per LLM request (async) |
| Budget refresh | ~50ms | Every ~100 requests (when $10 depleted) |
| Local budget check | <0.1ms | Per LLM request |

**Overhead:** <1ms per request (local check + async reporting)

## Failure Handling

| Scenario | Behavior |
|----------|----------|
| Control Panel unreachable | Use cached budget, queue reports |
| Budget refresh fails | Block new requests, return error |
| IP Token decrypt fails | Fatal error, runtime shutdown |
| Usage report fails | Retry 3x, then cache locally |

---

*Related: [003_service_boundaries.md](003_service_boundaries.md) | [002_layer_model.md](002_layer_model.md)*
