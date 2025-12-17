# Budget Concurrent Server Test

## Overview

Test file: `test_budget_concurrent_server.py`

Runs 10 concurrent threads making LLM requests through the server until budget is depleted.

## Current Status: WORKING

**Test Successfully Completed** (2025-12-14)

Results from latest run:
- 33 successful LLM requests across 10 concurrent threads
- All 10 threads blocked when $0.01 budget exhausted
- Budget enforcement working correctly
- 0 errors

**Key Issues Resolved:**

1. **IC Token header order**: Python-generated JWT tokens fail because PyJWT produces different header field order than Rust `jsonwebtoken`. Solution: Generate IC tokens using Rust.

2. **Missing provider_key_id column**: The `agents` table was missing the `provider_key_id` column needed for Feature 014 (agent provider key assignment).

## Prerequisites

### 1. Server Requirements

The server (`iron_control_api_server`) must be running on `localhost:3001` with these environment variables:

```bash
# Required in secret/-iron.sh
DATABASE_URL="sqlite://./iron.db?mode=rwc"
JWT_SECRET="<your-jwt-secret>"
IRON_SECRETS_MASTER_KEY="cgNibSOEmcQlx2Ax64o8s7kUjr2p/NE4S7eMgrhT7ZY="
IC_TOKEN_SECRET="dev-ic-token-secret-change-in-production"
```

### 2. Database Setup

Agent must exist in the database with `provider_key_id` pointing to a valid provider key.

```bash
# Ensure provider_key_id column exists (may need to add if missing)
sqlite3 iron.db "ALTER TABLE agents ADD COLUMN provider_key_id INTEGER REFERENCES ai_provider_keys(id);" 2>/dev/null || true

# Create agent 9999 with provider key assignment
sqlite3 iron.db "
INSERT OR REPLACE INTO agents (id, name, providers, created_at, owner_id, provider_key_id)
VALUES (9999, 'agent_9999', '[\"openai\"]', strftime('%s','now') * 1000, 'user_admin', 1);

INSERT OR REPLACE INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
VALUES (9999, 10000, 0, 10000, strftime('%s','now') * 1000, strftime('%s','now') * 1000);

-- Ensure usage_limits exists for user_admin
INSERT OR IGNORE INTO usage_limits (user_id, max_cost_microdollars_per_month, current_cost_microdollars_this_month, created_at, updated_at)
VALUES ('user_admin', 100000000, 0, strftime('%s','now') * 1000, strftime('%s','now') * 1000);
"
```

**Note**:
- Budget is in microdollars (10000 = $0.01)
- `provider_key_id = 1` points to the first provider key (OpenAI)

### 3. Provider API Key

OpenAI API key must be encrypted and stored in `ai_provider_keys` table:

```python
import base64, os, sqlite3
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

master_key_b64 = "cgNibSOEmcQlx2Ax64o8s7kUjr2p/NE4S7eMgrhT7ZY="  # From -iron.sh
master_key = base64.b64decode(master_key_b64)
nonce = os.urandom(12)
api_key = "sk-proj-YOUR-OPENAI-KEY"  # Replace with real key

aesgcm = AESGCM(master_key)
encrypted = aesgcm.encrypt(nonce, api_key.encode(), None)

conn = sqlite3.connect("iron.db")
conn.execute("""
UPDATE ai_provider_keys
SET encrypted_api_key = ?, encryption_nonce = ?
WHERE id = 1 AND provider = 'openai'
""", (base64.b64encode(encrypted).decode(), base64.b64encode(nonce).decode()))
conn.commit()
print("API key encrypted and stored")
```

## Generating IC Tokens

**Important**: Python-generated tokens don't work due to JWT header field order differences. Use Rust to generate tokens.

### Option 1: Use the Rust Example

```bash
# Edit the example to set your agent_id
cat > module/iron_control_api/examples/gen_ic_token.rs << 'EOF'
use iron_control_api::ic_token::{IcTokenClaims, IcTokenManager};

fn main() {
    let secret = "dev-ic-token-secret-change-in-production";
    let manager = IcTokenManager::new(secret.to_string());

    let claims = IcTokenClaims::new(
        "agent_9999".to_string(),       // Must match database agent ID
        "budget_9999".to_string(),
        vec!["llm:call".to_string(), "analytics:write".to_string()],
        None, // No expiration
    );

    let token = manager.generate_token(&claims).expect("Failed to generate token");
    println!("{}", token);
}
EOF

# Generate token
cargo run --package iron_control_api --example gen_ic_token
```

### Option 2: Pre-generated Token for agent_9999

```
eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJhZ2VudF9pZCI6ImFnZW50Xzk5OTkiLCJidWRnZXRfaWQiOiJidWRnZXRfOTk5OSIsImlhdCI6MTc2NTcyODQyMywiaXNzIjoiaXJvbi1jb250cm9sLXBhbmVsIiwicGVybWlzc2lvbnMiOlsibGxtOmNhbGwiLCJhbmFseXRpY3M6d3JpdGUiXX0.j6F-glMhP5DPEZ9A1crRfSbk7RLUyJoZyiBloiJ4GpY
```

**Note**: This token uses the default `IC_TOKEN_SECRET`. Regenerate if using a different secret.

## Running the Test

```bash
# From iron_runtime root directory
source .venv/bin/activate

# Reset budget before test
sqlite3 iron.db "UPDATE agent_budgets SET total_spent = 0, budget_remaining = 10000 WHERE agent_id = 9999;"

# Run test
RUST_LOG=info python module/iron_runtime/python/tests/test_budget_concurrent_server.py
```

## Expected Output

### First Run (Fresh Budget)

```
Budget from server handshake: $0.010000 (10000μ$), lease_id: lease_xxx
LlmRouter proxy listening on http://127.0.0.1:xxxxx

CONCURRENT BUDGET TEST (through server)
  Server: http://localhost:3001
  Threads: 10
  Server budget limit: $100.00

[T05] #1 | $0.0003 | Artificial Intelligence (AI) refers...
[T00] #1 | $0.0006 | Artificial Intelligence (AI) refers...
...
[T04] #4 | $0.0099 | Artificial Intelligence (AI) refers...
[T04] BLOCKED! Spent: $0.0099

RESULTS (elapsed: 40.1s)
  Successful requests: 33
  Blocked by budget:   10
  Errors:              0

  LOCAL spent:  $0.0099
  LOCAL budget: 0.01
  Budget returned to server: $0.000067 (spent: $0.009933)
```

### Second Run (Budget Exhausted)

When running again without resetting budget, all threads are immediately blocked:

```
Budget from server handshake: $0.000067 (67μ$), lease_id: lease_xxx

RESULTS (elapsed: 0.3s)
  Successful requests: 0
  Blocked by budget:   10
  Errors:              0

  LOCAL budget: 6.7e-05  ← Only 67 microdollars remaining (not enough for 1 request)
```

This confirms budget persistence works correctly - the server remembers spent budget across sessions.

## Known Issues

### 1. "Invalid IC Token" Error

**Possible Causes**:

1. **Python-generated token**: JWT header order differs from Rust
   - **Fix**: Generate token using Rust (see "Generating IC Tokens" section)

2. **Agent doesn't exist**: Server returns "Invalid IC Token" when agent not found (security measure to prevent enumeration)
   - **Fix**: Create agent in database (see "Database Setup" section)
   - **Debug**: `sqlite3 iron.db "SELECT id, name FROM agents;"`

3. **Secret mismatch**: `IC_TOKEN_SECRET` differs between token generation and server
   - **Fix**: Ensure both use `dev-ic-token-secret-change-in-production`

### 2. "Failed to decrypt provider key"

**Cause**: API key not encrypted with correct `IRON_SECRETS_MASTER_KEY`.

**Fix**: Re-encrypt API key using the master key from `secret/-iron.sh` (see "Provider API Key" section).

### 3. "Budget limit exceeded" with null values

**Cause**: Agent exists but has no budget record.

**Fix**: Create agent_budgets record:
```bash
sqlite3 iron.db "INSERT INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at) VALUES (9999, 10000, 0, 10000, strftime('%s','now') * 1000, strftime('%s','now') * 1000);"
```

### 4. Budget not enforced (requests keep going)

**Cause**: Bug in budget microdollar conversion (was multiplying by 1,000,000 twice).

**Fixed in**: `router.rs` - handshake returns microdollars, no conversion needed.

## Architecture

```
Python Test
    │
    ▼
LlmRouter (Rust)  ──handshake──►  iron_control_api (Server)
    │                                    │
    │ (local proxy)                      │ (budget lease)
    │                                    │
    ▼                                    ▼
OpenAI API                          SQLite DB
                                   (agent_budgets)
```

## IC Token Validation Flow

```
1. POST /api/v1/budget/handshake
   └─► Validate request fields
       └─► Verify IC Token signature (HMAC-SHA256)
           └─► Parse agent_id from claims (format: agent_<numeric_id>)
               └─► Lookup agent in database
                   ├─► If not found: Return "Invalid IC Token" (prevents enumeration)
                   └─► If found: Check budget, decrypt provider key, return IP token
```

## Budget Flow

1. **Handshake**: Client sends IC token → Server validates → Returns budget lease (microdollars)
2. **Requests**: Client tracks spending locally via CostController
3. **Enforcement**: CostController blocks when `spent + reserved >= limit`
4. **Return**: On shutdown, unused budget returned to server via `/api/v1/budget/return`

## Code Changes Made

### 1. Budget Return Fix (`iron_control_api/src/routes/budget/usage.rs`)

Added `restore_reserved_budget()` call in `return_budget` endpoint to properly restore unused budget to agent's balance.

### 2. Budget Microdollar Fix (`iron_runtime/src/llm_router/router.rs`)

Fixed double conversion bug - handshake returns microdollars, no longer multiplied by 1,000,000 again:

```rust
let budget_micros = if budget.is_some() {
  // Explicit budget parameter is in USD, convert to microdollars
  (effective_budget * 1_000_000.0) as i64
} else {
  // Handshake returns microdollars directly - no conversion needed
  effective_budget as i64
};
```

### 3. Log Display Fix (`iron_runtime/src/llm_router/router.rs`)

Fixed log messages to display dollars instead of raw microdollars:
- `Budget from server handshake: $0.010000` (was showing $10000)
- `Budget returned to server: $0.010000` (was showing microdollars)

### 4. Secrets Configuration

Added `IC_TOKEN_SECRET` to:
- `secret/-iron.sh` (with dev value)
- `secret/iron.example.sh` (with empty placeholder)

### 5. ConnectInfo Fix (`iron_control_api/src/bin/iron_control_api_server.rs`)

Fixed "Missing ConnectInfo" error on login by using:
```rust
axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
```

## Debug Commands

```bash
# Check server environment
cat /proc/$(pgrep -f iron_control_api_server)/environ | tr '\0' '\n' | grep -E "IC_TOKEN|JWT|IRON"

# Generate IC token
cargo run --package iron_control_api --example gen_ic_token

# Test handshake directly
TOKEN="<your-rust-generated-token>"
curl -s -X POST http://localhost:3001/api/v1/budget/handshake \
  -H "Content-Type: application/json" \
  -d "{\"ic_token\": \"$TOKEN\", \"provider\": \"openai\"}"

# Check agents in database
sqlite3 iron.db "SELECT id, name, owner_id FROM agents;"

# Check agent budget
sqlite3 iron.db "SELECT * FROM agent_budgets WHERE agent_id = 9999;"

# Reset budget
sqlite3 iron.db "UPDATE agent_budgets SET total_spent = 0, budget_remaining = 10000 WHERE agent_id = 9999;"

# Check provider keys
sqlite3 iron.db "SELECT id, provider, is_enabled FROM ai_provider_keys;"
```

## Technical Details: JWT Header Order Issue

The Rust `jsonwebtoken` crate and Python `PyJWT` produce different JWT header field orders:

| Library | Header JSON | Base64 |
|---------|-------------|--------|
| Rust | `{"typ":"JWT","alg":"HS256"}` | `eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9` |
| Python | `{"alg":"HS256","typ":"JWT"}` | `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9` |

Since the header is part of the signed data (`header.payload`), different header encoding produces different signatures, causing cross-library validation to fail.

**Workaround**: Generate IC tokens using Rust, not Python.

**Potential Fix**: Modify server to accept both header orders (would require custom JWT validation).

---

## Complete Debugging Session Summary (2025-12-14)

This section documents all issues encountered and fixes applied during the debugging session.

### Issue 1: IC Token Validation Failure

**Symptom**: `"Invalid IC Token"` error from handshake endpoint.

**Root Cause**: JWT header field order differs between Python and Rust:
- Rust `jsonwebtoken` crate: `{"typ":"JWT","alg":"HS256"}`
- Python `PyJWT` library: `{"alg":"HS256","typ":"JWT"}`

Since the header is part of the signed data (`base64(header).base64(payload)`), different header encoding produces different signatures.

**Solution**: Generate IC tokens using Rust instead of Python:
```bash
cargo run --package iron_control_api --example gen_ic_token
```

**Files Modified**:
- `module/iron_runtime/python/tests/test_budget_concurrent_server.py` - Changed `create_ic_token()` to use subprocess calling Rust

---

### Issue 2: Agent Not Found Returns "Invalid IC Token"

**Symptom**: `"Invalid IC Token"` error even with valid Rust-generated token.

**Root Cause**: The server intentionally returns "Invalid IC Token" when the agent doesn't exist (security measure to prevent agent enumeration attacks). See `handshake.rs:235-246`.

**Solution**: Ensure agent exists in database with matching ID:
```sql
INSERT INTO agents (id, name, providers, created_at, owner_id)
VALUES (9999, 'agent_9999', '["openai"]', strftime('%s','now') * 1000, 'user_admin');
```

---

### Issue 3: Missing `provider_key_id` Column

**Symptom**: `"Key fetch failed: 500 Internal Server Error: Database error (INTERNAL_ERROR)"`

**Root Cause**: The `agents` table was missing the `provider_key_id` column required by Feature 014 (agent provider key assignment). The endpoint `POST /api/v1/agents/provider-key` queries:
```sql
SELECT provider_key_id FROM agents WHERE id = ?
```

**Solution**: Add the column and assign provider key:
```sql
ALTER TABLE agents ADD COLUMN provider_key_id INTEGER REFERENCES ai_provider_keys(id);
UPDATE agents SET provider_key_id = 1 WHERE id = 9999;
```

**Files Affected**:
- `module/iron_control_api/src/routes/agent_provider_key.rs:151-180` - Endpoint expecting `provider_key_id`

---

### Issue 4: Missing `usage_limits` Row

**Symptom**: Database errors during handshake when updating usage limits.

**Root Cause**: The `usage_limits` table had no row for `user_admin` (the agent's owner).

**Solution**: Insert usage limits for the user:
```sql
INSERT INTO usage_limits (user_id, max_cost_microdollars_per_month, current_cost_microdollars_this_month, created_at, updated_at)
VALUES ('user_admin', 100000000, 0, strftime('%s','now') * 1000, strftime('%s','now') * 1000);
```

---

### Issue 5: Provider Key Decryption Failure

**Symptom**: `"Failed to decrypt provider key"`

**Root Cause**: The OpenAI API key in database was either:
1. A placeholder value (`fake_encrypted_key_openai`)
2. Encrypted with wrong `IRON_SECRETS_MASTER_KEY`

**Solution**: Re-encrypt with correct master key:
```python
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
import base64, os

master_key = base64.b64decode("cgNibSOEmcQlx2Ax64o8s7kUjr2p/NE4S7eMgrhT7ZY=")
aesgcm = AESGCM(master_key)
nonce = os.urandom(12)
encrypted = aesgcm.encrypt(nonce, b"sk-proj-YOUR-KEY", None)
# Store base64(encrypted) and base64(nonce) in ai_provider_keys
```

---

### Issue 6: Budget Double Conversion (Historical)

**Symptom**: Budget showing $10000 instead of $0.01.

**Root Cause**: Handshake returns microdollars but code was multiplying by 1,000,000 again.

**Solution**: Check if budget comes from explicit parameter vs handshake:
```rust
let budget_micros = if budget.is_some() {
  (effective_budget * 1_000_000.0) as i64  // Convert dollars to microdollars
} else {
  effective_budget as i64  // Already microdollars from handshake
};
```

**Files Modified**:
- `module/iron_runtime/src/llm_router/router.rs`

---

### Issue 7: ConnectInfo Missing (Historical)

**Symptom**: `"Missing ConnectInfo"` error on login endpoint.

**Root Cause**: Server not configured with `ConnectInfo` for rate limiting (client IP extraction).

**Solution**: Use `into_make_service_with_connect_info`:
```rust
axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
```

**Files Modified**:
- `module/iron_control_api/src/bin/iron_control_api_server.rs`

---

### Issue 8: IC_TOKEN_SECRET Not Configured (Historical)

**Symptom**: IC tokens rejected despite correct format.

**Root Cause**: `IC_TOKEN_SECRET` environment variable not set in server.

**Solution**: Add to `secret/-iron.sh`:
```bash
IC_TOKEN_SECRET="dev-ic-token-secret-change-in-production"
```

**Files Modified**:
- `secret/-iron.sh` - Added IC_TOKEN_SECRET
- `secret/iron.example.sh` - Added placeholder

---

### Database Schema Requirements

The test requires these tables/columns:

```sql
-- agents table with provider_key_id
CREATE TABLE agents (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  providers TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  owner_id TEXT,
  provider_key_id INTEGER REFERENCES ai_provider_keys(id)  -- REQUIRED
);

-- agent_budgets table
CREATE TABLE agent_budgets (
  agent_id INTEGER PRIMARY KEY,
  total_allocated INTEGER,
  total_spent INTEGER,
  budget_remaining INTEGER,
  created_at INTEGER,
  updated_at INTEGER
);

-- usage_limits table
CREATE TABLE usage_limits (
  id INTEGER PRIMARY KEY,
  user_id TEXT NOT NULL,
  max_cost_microdollars_per_month INTEGER,
  current_cost_microdollars_this_month INTEGER DEFAULT 0,
  created_at INTEGER,
  updated_at INTEGER
);

-- ai_provider_keys table
CREATE TABLE ai_provider_keys (
  id INTEGER PRIMARY KEY,
  provider TEXT NOT NULL,
  encrypted_api_key TEXT NOT NULL,
  encryption_nonce TEXT NOT NULL,
  is_enabled INTEGER DEFAULT 1,
  created_at INTEGER,
  user_id TEXT NOT NULL
);
```

---

### Test File Changes

**`test_budget_concurrent_server.py`**:

1. Removed `import jwt` (no longer using PyJWT)
2. Added `import subprocess`
3. Changed `create_ic_token()` to:
   - Write Rust source to `gen_ic_token.rs`
   - Run `cargo run --package iron_control_api --example gen_ic_token`
   - Return stdout (the generated token)
4. Added flush to print statements for correct log ordering

---

### Issue 9: Duplicate Migration Number 019 (2025-12-14)

**Symptom**: After deleting database and restarting server:
- `"no such column: provider_key_id"` errors
- Seed fails with `"LOUD FAILURE: Failed to seed database: Generic"`

**Root Cause**: Two separate migrations were created with the same number (019):
- `019_add_agent_provider_key_id.sql` - Added Dec 13 (commit f93cff7) - Feature 014
- `019_add_account_lockout_fields.sql` - Added Dec 14 (commit 26878f4) - Protocol 007

Both used `_migration_019_completed` as their guard table, causing conflicts:
1. On fresh DB, one migration would run and create `_migration_019_completed`
2. The other migration would see guard table exists and skip
3. Result: Missing columns, seed failures

**Solution**: Renumber account lockout migration to 020:

| File | Change |
|------|--------|
| `migrations/019_add_agent_provider_key_id.sql` | Keep as 019 (original) |
| `migrations/019_add_account_lockout_fields.sql` | Rename to `020_add_account_lockout_fields.sql` |
| `020_add_account_lockout_fields.sql` | Update comment to "Migration 020" |
| `020_add_account_lockout_fields.sql` | Update guard table to `_migration_020_completed` |
| `iron_token_manager/src/migrations.rs` | Add `apply_migration_020()` function |
| `iron_control_api/src/routes/auth/shared.rs` | Update to reference migration 020 |

**Files Modified**:
- `module/iron_token_manager/migrations/020_add_account_lockout_fields.sql` (renamed from 019)
- `module/iron_token_manager/src/migrations.rs` - Added migration 020 call and function
- `module/iron_control_api/src/routes/auth/shared.rs` - Updated include_str! path

**Verification**:
```bash
# Check migration files
ls module/iron_token_manager/migrations/0{19,20}*.sql
# Expected:
# 019_add_agent_provider_key_id.sql
# 020_add_account_lockout_fields.sql

# Run tests
cargo test --package iron_token_manager --test seed_test
# Expected: 3 passed

# Build full project
cargo build --package iron_control_api
# Expected: Success
```

---

### Verification Commands

```bash
# Verify all prerequisites
sqlite3 iron.db "
  SELECT 'agents' as tbl, id, name, provider_key_id FROM agents WHERE id = 9999;
  SELECT 'budgets' as tbl, agent_id, budget_remaining FROM agent_budgets WHERE agent_id = 9999;
  SELECT 'limits' as tbl, user_id, max_cost_microdollars_per_month FROM usage_limits WHERE user_id = 'user_admin';
  SELECT 'keys' as tbl, id, provider, is_enabled FROM ai_provider_keys WHERE id = 1;
"

# Test handshake
TOKEN=$(cargo run --package iron_control_api --example gen_ic_token 2>/dev/null)
curl -s -X POST http://localhost:3001/api/v1/budget/handshake \
  -H "Content-Type: application/json" \
  -d "{\"ic_token\": \"$TOKEN\", \"provider\": \"openai\"}" | jq .

# Expected: {"ip_token":"...", "lease_id":"...", "budget_granted":10000, ...}
```
