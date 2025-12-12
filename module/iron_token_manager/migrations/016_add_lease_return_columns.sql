-- Migration 016: Add return columns to budget_leases
--
-- This migration adds columns to support Protocol 005 lease return flow:
-- - returned_amount: USD returned to agent budget on lease close
-- - closed_at: Timestamp when lease was closed/returned
-- - updated_at: Timestamp of last update (for stale lease detection)

-- Add returned_amount column (USD returned when lease closed)
ALTER TABLE budget_leases ADD COLUMN returned_amount REAL DEFAULT 0.0;

-- Add closed_at column (timestamp when lease was closed)
ALTER TABLE budget_leases ADD COLUMN closed_at INTEGER;

-- Add updated_at column (for tracking last activity, stale lease detection)
ALTER TABLE budget_leases ADD COLUMN updated_at INTEGER;

-- Create index on updated_at for stale lease queries
CREATE INDEX IF NOT EXISTS idx_budget_leases_updated ON budget_leases( updated_at );

-- Create guard table to prevent re-running this migration
CREATE TABLE IF NOT EXISTS _migration_016_completed ( id INTEGER PRIMARY KEY );
