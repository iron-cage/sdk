# Seed Data Reference

This document provides a complete reference of all test data created by the `seed_all()` function in `src/seed.rs`.

**Purpose:** Eliminate confusion about what data exists during manual testing. Developers can reference this document to know exact IDs, usernames, balances, and relationships.

**Location:** This seed data is created by:
- `iron_token_manager::seed::seed_all( &pool )`
- Individual seed functions: `seed_users()`, `seed_provider_keys()`, `seed_api_tokens()`, `seed_usage_limits()`, `seed_project_assignments()`

**Usage:**
```rust
use iron_token_manager::seed::{ seed_all, wipe_database };

// In tests or manual setup
wipe_database( &pool ).await?;
seed_all( &pool ).await?;
```

---

## Users (5 total)

| Username    | Role  | Active | Password        | Created At      | Notes                          |
|-------------|-------|--------|-----------------|-----------------|--------------------------------|
| `admin`     | admin | ✓ Yes  | `IronDemo2025!` | now             | Admin user with unlimited access |
| `demo`      | user  | ✓ Yes  | `IronDemo2025!` | now             | Standard demo account (tiered limits) |
| `viewer`    | user  | ✗ No   | `IronDemo2025!` | now             | Inactive user (for revocation testing) |
| `tester`    | user  | ✓ Yes  | `IronDemo2025!` | 7 days ago      | Older account (backdated created_at) |
| `guest`     | user  | ✓ Yes  | `IronDemo2025!` | now             | No tokens (edge case) |

**Password Hash:** All users use bcrypt hash of `IronDemo2025!` with cost=12
- Hash: `$2b$12$AJbkR5cbO1NDN8vXQ2FSr.02E7lvpf6X7fp7yfBkppqHWtHF8vh86`

**User IDs:**
- Primary key: `users.id` (string)
- Values: `user_admin`, `user_demo`, `user_viewer`, `user_tester`, `user_guest`
- Referenced by foreign key columns as `user_*`

---

## AI Provider Keys (2 total)

| ID  | Provider    | Description                 | Enabled | Balance  | User ID      | Created At | Notes                           |
|-----|-------------|-----------------------------|---------|---------|--------------|-----------|---------------------------------|
| 1   | `openai`    | Development OpenAI key      | ✓ Yes   | $50.00  | `user_admin` | now       | Fake encrypted key for testing  |
| 2   | `anthropic` | Development Anthropic key   | ✓ Yes   | $100.00 | `user_admin` | now       | Fake encrypted key for testing  |

**Encryption Details:**
- **OpenAI encrypted key:** `ZmFrZV9lbmNyeXB0ZWRfa2V5X29wZW5haQ==` (base64 of "fake_encrypted_key_openai")
- **OpenAI nonce:** `YWFhYWFhYWFhYWFh` (base64 of "aaaaaaaaaaaa")
- **Anthropic encrypted key:** `ZmFrZV9lbmNyeXB0ZWRfa2V5X2FudGhyb3BpYw==` (base64 of "fake_encrypted_key_anthropic")
- **Anthropic nonce:** `YmJiYmJiYmJiYmJi` (base64 of "bbbbbbbbbbbb")

⚠️ **NOT SECURE:** These are placeholder values for development only!

**Balance Format:** Stored as cents (integer)
- $50.00 = 5000 cents
- $100.00 = 10000 cents

---

## API Tokens (8 total)

| Token Hash                            | Name                  | User ID        | Project ID      | Active | Expires At       | Notes                              |
|---------------------------------------|-----------------------|----------------|-----------------|--------|------------------|------------------------------------|
| `admin_token_hash_placeholder_aaa111` | Admin Master Token    | `user_admin`   | NULL            | ✓ Yes  | Never            | Permanent admin token              |
| `dev_token_hash_placeholder_bbb222`   | Developer Token       | `user_demo`    | NULL            | ✓ Yes  | now + 30 days    | Expires in 30 days                 |
| `project_token_hash_placeholder_ccc333` | Project Alpha Token | `user_demo`    | `project_alpha` | ✓ Yes  | Never            | Project-scoped token               |
| `inactive_token_hash_placeholder_ddd444` | Inactive Token     | `user_viewer`  | NULL            | ✗ No   | Never            | Inactive (for revocation testing)  |
| `expired_token_hash_placeholder_eee555` | Expired Token       | `user_demo`    | NULL            | ✓ Yes  | now - 30 days    | Already expired (for expiry tests) |
| `expiring_soon_token_hash_placeholder_fff666` | Expiring Soon Token | `user_demo` | `project_beta` | ✓ Yes  | now + 7 days     | Expires soon (rotation testing)    |
| `tester_token_hash_placeholder_ggg777` | Tester Token        | `user_tester`  | NULL            | ✓ Yes  | now + 14 days    | Short expiry (created 7 days ago)  |
| `tester_token_2_hash_placeholder_hhh888` | Tester Token 2     | `user_tester`  | `project_alpha` | ✓ Yes  | Never            | Second tester token (rotation)     |

**Token Hash Format:** Placeholder strings (not real hashes, for testing only)

**Expiration Calculation:**
- `day_ms = 24 * 60 * 60 * 1000` (milliseconds per day)
- "now + 30 days" = `now_ms + (30 * day_ms)`
- "now - 30 days" = `now_ms - (30 * day_ms)`

**Project Scoping:**
- Tokens with `project_id = NULL` are user-level tokens (access all projects)
- Tokens with `project_id = "project_alpha"` are project-scoped (limited to that project)

---

## Usage Limits (3 total)

| User ID        | Project ID | Max Tokens/Day | Max Requests/Min | Max Cost/Month (microdollars) | Current Tokens Today | Current Requests/Min | Current Cost This Month (microdollars) | Notes                        |
|----------------|------------|----------------|------------------|----------------|----------------------|----------------------|-------------------------|------------------------------|
| `user_admin`   | NULL       | Unlimited      | Unlimited        | Unlimited      | 0                    | 0                    | $0.00                   | Admin has no limits          |
| `user_demo`    | NULL       | 1,000,000      | 60               | $50.00 (50,000,000) | 250,000              | 15                   | $12.50 (12,500,000)        | Standard demo tier           |
| `user_viewer`  | NULL       | 100,000        | 10               | $0.00 (0)      | 95,000 ⚠️             | 2                    | $0.00 (0)               | Free tier, near daily limit! |

**Limit Representation:**
- `NULL` = Unlimited (no limit enforced)
- Integer values = Hard limit

**Cost Format:** Stored as microdollars (`$1.00 = 1_000_000` microdollars)
- $50.00 = 50_000_000 microdollars
- $12.50 = 12_500_000 microdollars
- $0.00 = 0 microdollars

**Current Usage:**
- These are *current* counters that would be reset periodically
- `user_viewer` is intentionally near limit (95k/100k tokens) for testing limit enforcement

---

## Project Provider Key Assignments (2 total)

| Project ID      | Provider Key ID | Provider    | Assigned At | Notes                               |
|-----------------|-----------------|-------------|-------------|-------------------------------------|
| `project_alpha` | 1               | `openai`    | now         | project_alpha can use OpenAI key    |
| `project_alpha` | 2               | `anthropic` | now         | project_alpha can use Anthropic key |

**Lookup Logic:**
- Provider key IDs are looked up dynamically (not hardcoded) via:
  ```sql
  SELECT id FROM ai_provider_keys WHERE provider = 'openai' LIMIT 1
  ```
- This prevents issues with AUTOINCREMENT differences across test runs

**Multi-Provider Support:**
- `project_alpha` has access to **both** OpenAI and Anthropic keys
- Applications can choose which provider to use per API call

---

## Foreign Key Relationships

```
users
  └─> ai_provider_keys (user_id references users.id)
  └─> api_tokens (user_id references users.id)
  └─> usage_limits (user_id references users.id)

ai_provider_keys
  └─> project_provider_key_assignments (provider_key_id references ai_provider_keys.id)

api_tokens
  └─> token_usage (token_hash references api_tokens.token_hash)
  └─> api_call_traces (token_hash references api_tokens.token_hash)
  └─> audit_log (token_hash references api_tokens.token_hash)
  └─> token_blacklist (token_hash references api_tokens.token_hash)
```

**Deletion Order (for wiping):**
1. Child tables first: `token_usage`, `api_call_traces`, `audit_log`, `project_provider_key_assignments`, `token_blacklist`, `user_audit_log`
2. Parent tables: `api_tokens`, `ai_provider_keys`, `usage_limits`, `users`, `agents`

See `wipe_database()` in `src/seed.rs` for the exact deletion order.

---

## Common Testing Scenarios

### Scenario 1: Admin Testing
- **User:** `admin` / `IronDemo2025!`
- **Token:** `admin_token_hash_placeholder_aaa111`
- **Limits:** Unlimited (no restrictions)
- **Access:** All projects, all providers

### Scenario 2: Standard Demo Testing
- **User:** `demo` / `IronDemo2025!`
- **Token:** `dev_token_hash_placeholder_bbb222` (user-level) or `project_token_hash_placeholder_ccc333` (project-scoped)
- **Limits:** 1M tokens/day (250k used), 60 req/min (15 used), $50/month ($12.50 used)
- **Projects:** `project_alpha` (via project token)
- **Providers:** OpenAI + Anthropic (via project assignments)

### Scenario 3: Near-Limit Testing
- **User:** `viewer` / `IronDemo2025!`
- **Limits:** 100k tokens/day (**95k used** ⚠️ near limit!)
- **Purpose:** Test rate limiting and quota enforcement

### Scenario 4: Revocation Testing
- **Token:** `inactive_token_hash_placeholder_ddd444`
- **Status:** Inactive (is_active = 0)
- **Purpose:** Test token revocation flows

### Scenario 5: Expiration Testing
- **Token:** `expired_token_hash_placeholder_eee555`
- **Status:** Active but expired 30 days ago
- **Purpose:** Test expiration enforcement

---

## Time Constants

All timestamps are stored as **milliseconds since Unix epoch** (`i64`).

```rust
let now_ms = std::time::SystemTime::now()
  .duration_since( std::time::UNIX_EPOCH )
  .unwrap()
  .as_millis() as i64;

let day_ms = 24 * 60 * 60 * 1000;  // Milliseconds per day
```

**Time Calculations:**
- "now" = Current system time in milliseconds
- "now + 30 days" = `now_ms + (30 * day_ms)`
- "now - 30 days" = `now_ms - (30 * day_ms)`
- "now - 60 days" = `now_ms - (60 * day_ms)`

---

## Safety Warnings

⚠️ **NEVER USE IN PRODUCTION:**
- Passwords are predictable (`IronDemo2025!`)
- Encryption keys are fake placeholders
- Token hashes are simple strings (not cryptographic hashes)
- Predictable seed data is for test/demo environments only

✅ **Safe for:**
- Development environments
- Automated tests
- Manual testing
- CI/CD pipelines

🔒 **Before Production:**
- Generate real cryptographic keys
- Use strong password hashing (bcrypt cost ≥ 12)
- Implement proper key management (e.g., HashiCorp Vault)
- Remove all seed functions from production builds

---

## Maintenance Notes

**When Adding New Seed Data:**
1. Update the corresponding `seed_*()` function in `src/seed.rs`
2. Update this documentation file with new entities
3. Update `wipe_database()` deletion order if new tables added
4. Add test validation in `tests` module at bottom of `src/seed.rs`

**When Modifying Schema:**
1. Update migration files (NOT seed data!)
2. Update seed functions to match new schema
3. Update this documentation to reflect schema changes
4. Re-run tests to validate seed data still works

**Foreign Key Ordering:**
- Always insert parent tables before child tables (users before api_tokens)
- Always delete child tables before parent tables (api_tokens before users)
- Use dynamic lookups for AUTOINCREMENT IDs (see `seed_project_assignments()`)
