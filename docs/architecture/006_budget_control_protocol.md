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

## Budget Borrowing Protocol

### Step 1: Initialization (Token Handshake)

```
Developer → Runtime: IC Token
Runtime → Control Panel: POST /api/v1/auth/handshake
  {
    "ic_token": "ic_abc123...",
    "requested_budget": 10.00
  }

Control Panel → Runtime: 200 OK
  {
    "ip_token": "sk-proj-xyz..." (encrypted),
    "budget_granted": 10.00,
    "budget_remaining": 90.00,
    "lease_id": "lease-001"
  }

Runtime stores in memory:
- IP Token: AES-256 encrypted, never touches disk
- Budget lease: $10 borrowed from $100 total
```

### Step 2: Request Execution

```
Agent → Runtime: LLM request (uses IC Token for auth)
Runtime → Provider: Translates request
  - Replaces IC Token with IP Token
  - Forwards to OpenAI/Anthropic/etc
Provider → Runtime: Response + usage metadata
  {
    "response": "...",
    "usage": {"tokens": 500, "cost": 0.015}
  }

Runtime → Control Panel: POST /api/v1/budget/report
  {
    "lease_id": "lease-001",
    "tokens": 500,
    "cost_usd": 0.015,
    "timestamp": "2025-12-09T09:00:00Z"
  }

Runtime updates local budget:
- Spent: $0.015
- Remaining in lease: $9.985
```

### Step 3: Budget Refresh

```
Runtime checks: if remaining < $1 threshold
Runtime → Control Panel: POST /api/v1/budget/refresh
  {
    "lease_id": "lease-001",
    "requested_budget": 10.00
  }

Control Panel checks: $90 still available
Control Panel → Runtime: 200 OK
  {
    "budget_granted": 10.00,
    "budget_remaining": 80.00,
    "lease_id": "lease-002"
  }

Runtime updates:
- New lease: $10
- Total spent tracking continues
```

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
