# Seed Data Reference

## Overview

This document provides a comprehensive reference for the test seed data used in the iron_token_manager module. The seed data is designed to provide realistic test scenarios for both automated testing and manual API validation.

## Two Seed Implementations

### Bash Seed (`scripts/seed_dev_data.sh`)

**Purpose:** Quick, simple seed data for basic manual testing

**Contents:**
- 3 users (admin, developer, viewer)
- 3 tokens (one per user)
- 7 usage records
- 3 usage limits

**Use when:** You need a quick development database for basic API testing

```bash
./scripts/reset_and_seed.sh
```

### Rust Seed (`src/seed.rs`)

**Purpose:** Comprehensive seed data with edge cases for thorough testing

**Contents:**
- 5 users (admin, developer, viewer, tester, guest)
- 8 tokens (various expiration states and assignments)
- 10+ usage records (diverse patterns)
- 3 usage limits (same as bash)

**Use when:** Running automated tests or testing edge cases

```rust
// Called automatically in tests
seed::seed_all(&pool).await?;
```

## User Profiles

### Core Users (Both Implementations)

#### 1. Admin User
- **Username:** `admin`
- **Role:** `admin`
- **Status:** Active (`is_active=1`)
- **Purpose:** Test administrative privileges and unlimited access
- **Usage Limit:** Unlimited (500,000 tokens/day)
- **Tokens:** 1 active token
- **Test Scenarios:**
  - Administrative API operations
  - High-volume token usage
  - Never-expiring tokens

#### 2. Developer User
- **Username:** `developer`
- **Role:** `user`
- **Status:** Active (`is_active=1`)
- **Purpose:** Test standard user with typical limits
- **Usage Limit:** Standard tier (10,000 tokens/day)
- **Tokens:** 1 active token (bash), 3 tokens (Rust - includes project and expiring tokens)
- **Test Scenarios:**
  - Normal API usage
  - Quota enforcement
  - Multiple token management
  - Project-scoped tokens

#### 3. Viewer User
- **Username:** `viewer`
- **Role:** `user`
- **Status:** **Inactive** (`is_active=0`)
- **Purpose:** Test inactive user handling
- **Usage Limit:** Restricted (1,000 tokens/day, near limit)
- **Tokens:** 1 active token (bash), 2 tokens (Rust - includes expired token)
- **Test Scenarios:**
  - Inactive user behavior
  - Near-limit quota scenarios
  - Expired token handling

### Extended Users (Rust Implementation Only)

#### 4. Tester User
- **Username:** `tester`
- **Role:** `user`
- **Status:** Active (`is_active=1`)
- **Purpose:** Test unlimited usage scenarios
- **Usage Limit:** **None** (unlimited testing)
- **Tokens:** 2 active tokens
- **Test Scenarios:**
  - Unlimited quota behavior
  - Token rotation
  - Mixed provider usage (OpenAI + Anthropic)

#### 5. Guest User
- **Username:** `guest`
- **Role:** `user`
- **Status:** Active (`is_active=1`)
- **Purpose:** Test newly registered user with no tokens
- **Usage Limit:** **None**
- **Tokens:** **None** (edge case)
- **Test Scenarios:**
  - User without tokens
  - Initial registration state
  - Token creation workflow

## Token Catalog

### Core Tokens (Both Implementations)

#### Token 1: Admin Token
- **Value:** `iron_dev_admin_token_001`
- **User:** admin
- **Project:** `project_alpha`
- **Status:** Active
- **Expiration:** Never expires
- **Purpose:** Administrative operations testing

#### Token 2: Developer Token
- **Value:** `iron_dev_pm_token_002`
- **User:** developer
- **Project:** None (personal token)
- **Status:** Active
- **Expiration:** Never expires
- **Purpose:** Standard user operations

#### Token 3: Viewer Token
- **Value:** `iron_dev_viewer_token_003`
- **User:** viewer
- **Project:** `project_beta`
- **Status:** Active
- **Expiration:** Never expires
- **Purpose:** Inactive user with active token testing

### Extended Tokens (Rust Implementation Only)

#### Token 4: Expired Token
- **Value:** `iron_dev_expired_token_004`
- **User:** viewer
- **Project:** None
- **Status:** Inactive (`is_active=0`)
- **Expiration:** 30 days ago
- **Purpose:** Test expired token handling and cleanup

#### Token 5: Project Token
- **Value:** `iron_dev_project_token_005`
- **User:** developer
- **Project:** `project_alpha`
- **Status:** Active
- **Expiration:** Never expires
- **Purpose:** Test project-scoped token operations

#### Token 6: Expiring Soon Token
- **Value:** `iron_dev_expiring_token_006`
- **User:** developer
- **Project:** `project_beta`
- **Status:** Active
- **Expiration:** 7 days from creation
- **Purpose:** Test token expiration warnings and renewal

#### Token 7: Tester Token
- **Value:** `iron_dev_tester_token_007`
- **User:** tester
- **Project:** None
- **Status:** Active
- **Expiration:** 14 days from creation
- **Purpose:** Test unlimited user scenarios

#### Token 8: Tester Token 2
- **Value:** `iron_dev_tester_token_008`
- **User:** tester
- **Project:** `project_alpha`
- **Status:** Active
- **Expiration:** Never expires
- **Purpose:** Test token rotation and multiple tokens per user

## Usage Patterns

### Bash Implementation (7 Records)

**Pattern:** Basic usage across all core users

- Admin token: Moderate usage over multiple days
- Developer token: Standard daily usage
- Viewer token: Low usage (near limit scenarios)

### Rust Implementation (10+ Records)

**Pattern 1: Admin Token - Moderate Usage**
- 7 records over 7 days
- Gradually increasing usage (500-850 tokens/day)
- Tests incremental growth patterns

**Pattern 2: Developer Token - High Usage**
- 5 records over 5 days
- High usage approaching limits
- Tests quota enforcement

**Pattern 3: Project Token - Sporadic Usage**
- 2 records (day 0 and day 3)
- Irregular access patterns
- Tests intermittent usage tracking

**Pattern 4: Tester Token - Mixed Providers**
- 4 records with both OpenAI and Anthropic
- Tests multi-provider usage
- Validates provider-specific metrics

**Edge Cases Tested:**
- Zero usage tokens (newly created)
- High-frequency usage
- Cross-provider usage
- Time-based usage patterns

## Usage Limits

All implementations have 3 usage limits:

### Admin Limit
- **User:** admin
- **Daily Tokens:** 500,000 (effectively unlimited)
- **Daily Requests:** 10,000
- **Purpose:** Administrative tier testing

### Developer Limit
- **User:** developer
- **Daily Tokens:** 10,000
- **Daily Requests:** 1,000
- **Purpose:** Standard tier testing, quota enforcement

### Viewer Limit
- **User:** viewer
- **Daily Tokens:** 1,000
- **Daily Requests:** 100
- **Current Usage:** 950 tokens (95% of limit)
- **Purpose:** Near-limit scenario testing

### Special Cases

**Tester User:** No limits (tests unlimited access)
**Guest User:** No limits (tests users without limits)

## Manual Testing Guide

### Using Bash Seed Data

```bash
# 1. Reset and seed database
make reset-seed

# 2. Test with admin token
curl -H "Authorization: Bearer iron_dev_admin_token_001" \
  http://localhost:8080/api/usage

# 3. Test with developer token
curl -H "Authorization: Bearer iron_dev_pm_token_002" \
  http://localhost:8080/api/usage

# 4. Test with inactive user (viewer)
curl -H "Authorization: Bearer iron_dev_viewer_token_003" \
  http://localhost:8080/api/usage
```

### Testing Edge Cases (Rust Seed)

```bash
# 1. Test expired token (should fail)
curl -H "Authorization: Bearer iron_dev_expired_token_004" \
  http://localhost:8080/api/usage

# 2. Test expiring soon token (should warn)
curl -H "Authorization: Bearer iron_dev_expiring_token_006" \
  http://localhost:8080/api/usage

# 3. Test unlimited user (tester)
curl -H "Authorization: Bearer iron_dev_tester_token_007" \
  http://localhost:8080/api/usage

# 4. Test user without tokens (guest)
# Should fail with "no valid token" error
```

### Testing Quota Limits

```bash
# 1. Check viewer usage (near limit - 95%)
curl -H "Authorization: Bearer iron_dev_viewer_token_003" \
  http://localhost:8080/api/usage/current

# 2. Try to exceed limit
curl -X POST \
  -H "Authorization: Bearer iron_dev_viewer_token_003" \
  -H "Content-Type: application/json" \
  -d '{"tokens": 100}' \
  http://localhost:8080/api/usage/record
# Should fail with quota exceeded

# 3. Check unlimited user (tester)
curl -H "Authorization: Bearer iron_dev_tester_token_007" \
  http://localhost:8080/api/usage/current
# Should show no limits
```

## Validation

### Bash Validator

The seed data validator accepts both implementations:

```bash
./scripts/validate_seed_data.sh ./iron.db
```

**Validation Rules:**
- Users: 3-5 (bash: 3, Rust: 5)
- Tokens: 3-8 (bash: 3, Rust: 8)
- Usage records: ≥7
- Usage limits: 3
- Core users present: admin, developer, viewer
- Optional users: tester, guest

## Test Scenarios Covered

### Authentication & Authorization
- ✅ Valid token authentication
- ✅ Expired token rejection
- ✅ Inactive user handling
- ✅ User without tokens
- ✅ Multiple tokens per user

### Quota Management
- ✅ Standard usage limits
- ✅ Unlimited access (admin, tester)
- ✅ Near-limit scenarios (viewer)
- ✅ Quota enforcement
- ✅ Users without limits

### Token Lifecycle
- ✅ Never-expiring tokens
- ✅ Time-limited tokens
- ✅ Expiring soon warnings
- ✅ Expired token cleanup
- ✅ Token rotation

### Usage Tracking
- ✅ Daily usage patterns
- ✅ Multi-day usage history
- ✅ Provider-specific usage
- ✅ Zero usage tokens
- ✅ Sporadic access patterns

### Edge Cases
- ✅ Inactive users with active tokens
- ✅ Active users with expired tokens
- ✅ Users with no tokens (guest)
- ✅ Users with no limits (tester)
- ✅ Multiple tokens with different expiration states

## Maintenance

### Keeping Implementations in Sync

When adding new test scenarios:

1. **Decide scope:**
   - Core scenarios → Update both bash and Rust
   - Edge cases → Rust only

2. **Update bash seed (`scripts/seed_dev_data.sh`):**
   - Add SQL INSERT statements
   - Update counts in summary

3. **Update Rust seed (`src/seed.rs`):**
   - Modify `seed_users()`, `seed_api_tokens()`, or `seed_token_usage()`
   - Update function documentation
   - Update test assertions

4. **Update validator (`scripts/validate_seed_data.sh`):**
   - Adjust count ranges if needed
   - Add optional checks for Rust-only features
   - Update documentation

5. **Update this documentation:**
   - Add new user/token descriptions
   - Update test scenarios
   - Add manual testing examples

## Related Documentation

- [Database Initialization](./database_initialization.md) - Schema and migrations
- [Database Path Standards](./database_path_standards.md) - Path conventions and validation
- [Quick Reference](./quick_reference_database.md) - Command cheat sheet
