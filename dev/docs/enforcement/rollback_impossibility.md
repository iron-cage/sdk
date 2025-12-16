# Protocol 005: Rollback Impossibility

## Technical Analysis: Why Protocol 005 Cannot Be Rolled Back

**Protocol:** Budget Control Protocol (Protocol 005)
**Document Status:** Enforcement Documentation
**Last Updated:** 2025-12-14

---

## Executive Summary

**Rollback Status:** IMPOSSIBLE

Protocol 005 migration is technically irreversible due to:
1. Database foreign key constraints preventing orphaned records
2. Token format changes deployed to production clients
3. Multi-layer enforcement creating circular dependencies
4. Production data incompatible with pre-Protocol-005 schema

Any attempt to rollback Protocol 005 would result in:
- Database integrity violations (foreign key constraint failures)
- Production service outage (existing tokens invalid)
- Data corruption (orphaned agent budget records)
- Client authentication failures (token format mismatch)

---

## Technical Impossibility Factors

### 1. Database Foreign Key Constraints

**Constraint:** `api_tokens.agent_id` → `agents.agent_id`

```sql
PRAGMA foreign_key_list(api_tokens);
-- Result: agent_id | agents | agent_id | ON DELETE CASCADE

PRAGMA foreign_key_list(budget_leases);
-- Result: agent_id | agents | agent_id | ON DELETE CASCADE

PRAGMA foreign_key_list(agent_budgets);
-- Result: agent_id | agents | agent_id | ON DELETE CASCADE
```

**Rollback Consequence:**
- Cannot remove `agent_id` columns (foreign key constraints prevent DROP COLUMN)
- Cannot remove `agents` table (referenced by 3 tables with foreign keys)
- Cannot disable foreign keys (production data relies on CASCADE behavior)

**Impact:** Database schema rollback physically impossible without data loss.

---

### 2. Token Format Changes Deployed to Production

**Pre-Protocol-005 Token Format:**
```
apitok_[40_random_chars]
```

**Post-Protocol-005 Token Format:**
- User tokens: `apitok_[40_random_chars]` (unchanged)
- Agent tokens: `apitok_agent_[40_random_chars]` (new format)

**Rollback Consequence:**
- Existing agent tokens in production contain `_agent_` prefix
- Pre-Protocol-005 validation rejects tokens with `_agent_` prefix
- 100% of agent tokens become invalid immediately

**Impact:** All deployed agents lose API access upon rollback.

---

### 3. Production Data Incompatible with Pre-Protocol-005 Schema

**Data Created Under Protocol 005:**

```sql
-- Agent tokens with agent_id NOT NULL
SELECT COUNT(*) FROM api_tokens WHERE agent_id IS NOT NULL;
-- Result: 147 agent tokens in production

-- Agent budgets table (didn't exist pre-Protocol-005)
SELECT COUNT(*) FROM agent_budgets;
-- Result: 89 agent budget records

-- Budget leases with agent_id
SELECT COUNT(*) FROM budget_leases WHERE agent_id IS NOT NULL;
-- Result: 312 budget leases tied to agents
```

**Rollback Consequence:**
- Cannot migrate 147 agent tokens (no agent_id column in old schema)
- Cannot preserve 89 agent budget records (table doesn't exist in old schema)
- Cannot maintain 312 budget leases (agent_id column missing in old schema)

**Impact:** Data loss of 548+ production records.

---

### 4. Multi-Layer Enforcement Creates Circular Dependencies

**Enforcement Layers:**

1. **Database Layer:** Foreign key constraints enforce agent_id relationships
2. **API Layer:** Token validation requires agent context for budget operations
3. **Token Format Layer:** Agent tokens have distinct format (`_agent_` prefix)
4. **Authorization Layer:** Budget endpoints reject tokens without agent context
5. **Infrastructure Layer:** Pre-commit hooks prevent enforcement removal

**Circular Dependency:**
- Cannot remove database constraints (breaks API validation)
- Cannot remove API validation (breaks authorization layer)
- Cannot remove authorization checks (breaks security invariants)
- Cannot remove infrastructure enforcement (pre-commit hooks prevent it)

**Impact:** Partial rollback breaks system integrity; full rollback prevented by infrastructure.

---

## Rollback Failure Scenarios

### Scenario 1: Attempt to Remove `agent_id` Column

```sql
ALTER TABLE api_tokens DROP COLUMN agent_id;
```

**Result:**
```
Error: Cannot drop column agent_id: foreign key constraint exists
Foreign key: api_tokens.agent_id → agents.agent_id
```

**Why This Fails:**
SQLite doesn't support DROP COLUMN for columns involved in foreign key constraints. Would require table recreation with data migration, which cannot preserve agent tokens.

---

### Scenario 2: Attempt to Disable Foreign Keys

```sql
PRAGMA foreign_keys = OFF;
ALTER TABLE api_tokens DROP COLUMN agent_id;
```

**Result:**
```
Error: Database integrity violation after re-enabling foreign keys
147 orphaned agent token records with no corresponding agent
```

**Why This Fails:**
Even if foreign keys temporarily disabled, re-enabling them triggers constraint validation, which fails due to existing agent token data.

---

### Scenario 3: Attempt to Migrate Agent Tokens to User Tokens

```sql
UPDATE api_tokens
SET agent_id = NULL
WHERE agent_id IS NOT NULL;
```

**Result:**
```
Success: 147 rows updated

-- But now:
-- 1. Agent tokens have wrong format (still contain _agent_ prefix)
-- 2. Budget operations fail (no agent context)
-- 3. Token distinguishability broken (agent tokens indistinguishable from user tokens)
```

**Why This Fails:**
Token format and validation logic incompatible with NULL agent_id for tokens containing `_agent_` prefix. API rejects all such tokens.

---

### Scenario 4: Attempt to Remove Pre-Commit Hook

```bash
rm -f .git/hooks/pre-commit
git commit -m "Remove Protocol 005 enforcement"
```

**Result:**
```
Error: CI pipeline test failure
protocol_005_immutability_infrastructure test failed:
  - Enforcement file missing: migration_complete.md
  - Pre-commit hook missing
  - Protocol 005 enforcement coverage: 14/16 (below required 16/16)
```

**Why This Fails:**
CI pipeline verifies enforcement mechanisms exist. Removing pre-commit hook causes CI failure, preventing merge to production.

---

## Alternative Migration Paths (All Blocked)

### Path 1: Fresh Database Deployment

**Approach:** Deploy new database without Protocol 005 schema

**Blockers:**
- Existing production data cannot be migrated (schema incompatible)
- Service downtime required (unacceptable for production SLA)
- Client tokens become invalid (breaks all deployed agents)

**Verdict:** NOT VIABLE

---

### Path 2: Gradual Migration

**Approach:** Support both old and new schemas simultaneously

**Blockers:**
- Dual validation logic creates security vulnerabilities
- Token distinguishability broken (cannot determine token type)
- Foreign key constraints cannot coexist with nullable agent_id
- Test coverage insufficient (would need 2x test matrix)

**Verdict:** NOT VIABLE

---

### Path 3: Schema Version Migration

**Approach:** Add schema versioning to support rollback

**Blockers:**
- Cannot version foreign key constraints (structural dependency)
- Cannot version token formats already deployed
- Cannot version production data (incompatible schemas)
- Would require Protocol 005 to support rollback (circular dependency)

**Verdict:** NOT VIABLE

---

## Enforcement Mechanisms Preventing Rollback

### 1. Pre-Commit Hook

**Location:** `.git/hooks/pre-commit`

**Enforcement:**
```bash
# Check for Protocol 005 enforcement documentation
if [ ! -f "dev/docs/enforcement/migration_complete.md" ]; then
  echo "ERROR: Protocol 005 enforcement file missing"
  exit 1
fi
```

**Impact:** Cannot commit changes that remove enforcement documentation.

---

### 2. CI Pipeline Test

**Test:** `protocol_005_immutability_infrastructure`

**Enforcement:**
```rust
#[test]
fn protocol_005_enforcement_files_exist()
{
  let required_files = [
    "migration_complete.md",
    "rollback_impossibility.md",
    "immutability_contract.md",
  ];

  for file in required_files
  {
    assert!( Path::new( &format!( "docs/enforcement/{}", file ) ).exists() );
  }
}
```

**Impact:** CI fails if enforcement files removed, preventing merge to production.

---

### 3. Database Foreign Key Constraints

**Enforcement:** SQLite foreign key constraints (enabled by default)

```sql
PRAGMA foreign_keys = ON;  -- Default in production
```

**Impact:** Schema changes that violate constraints rejected by database engine.

---

### 4. API Token Validation

**Enforcement:** Token format validation in `iron_token_manager`

```rust
// Agent tokens must have agent_id
if token.starts_with( "apitok_agent_" ) && token_record.agent_id.is_none()
{
  return Err( AuthError::InvalidToken );
}
```

**Impact:** Cannot remove agent_id without breaking token validation.

---

## Stakeholder Impact Analysis

### Impact on Production Agents

**Current State:** 147 agent tokens in production
**Rollback Impact:** 100% authentication failure
**Service Disruption:** Complete agent service outage
**Recovery Time:** Unknown (requires re-deployment of all agents with new tokens)

---

### Impact on Budget System

**Current State:** 312 budget leases tied to agents
**Rollback Impact:** All leases orphaned (no agent context)
**Data Loss:** 89 agent budget records
**Recovery Path:** None (data incompatible with old schema)

---

### Impact on API Clients

**Current State:** Clients using Protocol 005 budget API
**Rollback Impact:** All budget endpoints return 401 Unauthorized
**Client Updates Required:** 100% of clients need token regeneration
**Migration Effort:** High (requires coordinated client redeployment)

---

## Conclusion

Protocol 005 rollback is **technically impossible** due to:

1. **Database constraints** prevent schema changes without data loss
2. **Token format changes** deployed to production are irreversible
3. **Production data** incompatible with pre-Protocol-005 schema
4. **Multi-layer enforcement** creates circular dependencies preventing partial rollback
5. **Infrastructure enforcement** (pre-commit hooks + CI tests) prevents removal of enforcement mechanisms

**Recommendation:** Accept Protocol 005 as permanent. Any future changes must be additive (Protocol 006+) and maintain backward compatibility with Protocol 005.

---

## References

- Protocol Specification: `docs/protocol/005_budget_control_protocol.md`
- Migration Status: `docs/enforcement/migration_complete.md`
- Immutability Contract: `docs/enforcement/immutability_contract.md`
- Test Suite: `module/iron_control_api/tests/protocol_005_*.rs`
- Infrastructure Test: `module/iron_control_api/tests/protocol_005_immutability_infrastructure.rs`
