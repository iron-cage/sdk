-- Migration 018: Convert budget columns from REAL (USD) to INTEGER (microdollars)
--
-- ## Rationale
--
-- The Rust codebase was migrated from f64 (USD) to i64 (microdollars) for precision
-- and to avoid floating-point arithmetic issues. This migration updates the database
-- schema to match.
--
-- ## Conversion
--
-- 1 USD = 1,000,000 microdollars
-- All REAL values are multiplied by 1,000,000 and stored as INTEGER
--
-- ## Tables Affected
--
-- - budget_leases: budget_granted, budget_spent, returned_amount
-- - agent_budgets: total_allocated, total_spent, budget_remaining

-- Check if migration already applied
CREATE TABLE IF NOT EXISTS _migration_018_completed (applied_at INTEGER NOT NULL);

-- Only proceed if not already applied
INSERT INTO _migration_018_completed (applied_at)
SELECT strftime('%s', 'now') * 1000
WHERE NOT EXISTS (SELECT 1 FROM _migration_018_completed);

-- ============================================================================
-- PART 1: Convert budget_leases table (must be first due to FK dependency)
-- ============================================================================

-- Create temporary table with new schema (includes columns from migration 016)
CREATE TABLE IF NOT EXISTS budget_leases_new
(
  id TEXT PRIMARY KEY,  -- Keep TEXT as defined in migration 009
  agent_id INTEGER NOT NULL,
  budget_id INTEGER NOT NULL,
  budget_granted INTEGER NOT NULL,  -- Changed from REAL to INTEGER (microdollars)
  budget_spent INTEGER NOT NULL DEFAULT 0,  -- Changed from REAL to INTEGER (microdollars)
  lease_status TEXT NOT NULL DEFAULT 'active',
  created_at INTEGER NOT NULL,
  expires_at INTEGER,
  returned_amount INTEGER DEFAULT 0,  -- From migration 016, converted to microdollars
  closed_at INTEGER,  -- From migration 016
  updated_at INTEGER,  -- From migration 016
  FOREIGN KEY ( agent_id ) REFERENCES agents( id ) ON DELETE CASCADE,
  FOREIGN KEY ( budget_id ) REFERENCES agent_budgets( agent_id ) ON DELETE CASCADE
);

-- Copy data from old table to new table (convert REAL to INTEGER)
INSERT INTO budget_leases_new
(
  id, agent_id, budget_id, budget_granted, budget_spent,
  lease_status, created_at, expires_at, returned_amount, closed_at, updated_at
)
SELECT
  id, agent_id, budget_id,
  CAST( budget_granted * 1000000 AS INTEGER ),
  CAST( budget_spent * 1000000 AS INTEGER ),
  lease_status, created_at, expires_at,
  CAST( COALESCE(returned_amount, 0.0) * 1000000 AS INTEGER ),
  closed_at, updated_at
FROM budget_leases;

-- Drop old table and rename new table
DROP TABLE budget_leases;
ALTER TABLE budget_leases_new RENAME TO budget_leases;

-- Recreate indexes (from migration 009 and 016)
CREATE INDEX IF NOT EXISTS idx_budget_leases_agent ON budget_leases( agent_id );
CREATE INDEX IF NOT EXISTS idx_budget_leases_status ON budget_leases( lease_status );
CREATE INDEX IF NOT EXISTS idx_budget_leases_created ON budget_leases( created_at );
CREATE INDEX IF NOT EXISTS idx_budget_leases_updated ON budget_leases( updated_at );

-- ============================================================================
-- PART 2: Convert agent_budgets table
-- ============================================================================

-- Create temporary table with new schema
CREATE TABLE IF NOT EXISTS agent_budgets_new
(
  agent_id INTEGER PRIMARY KEY,
  total_allocated INTEGER NOT NULL,  -- Changed from REAL to INTEGER (microdollars)
  total_spent INTEGER NOT NULL DEFAULT 0,  -- Changed from REAL to INTEGER (microdollars)
  budget_remaining INTEGER NOT NULL,  -- Changed from REAL to INTEGER (microdollars)
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY ( agent_id ) REFERENCES agents( id ) ON DELETE CASCADE
);

-- Copy data from old table to new table (convert REAL to INTEGER)
INSERT INTO agent_budgets_new
(
  agent_id, total_allocated, total_spent, budget_remaining,
  created_at, updated_at
)
SELECT
  agent_id,
  CAST( total_allocated * 1000000 AS INTEGER ),
  CAST( total_spent * 1000000 AS INTEGER ),
  CAST( budget_remaining * 1000000 AS INTEGER ),
  created_at, updated_at
FROM agent_budgets;

-- Drop old table and rename new table
DROP TABLE agent_budgets;
ALTER TABLE agent_budgets_new RENAME TO agent_budgets;
