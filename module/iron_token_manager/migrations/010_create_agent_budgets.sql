-- Migration 010: Create agent_budgets table
--
-- This migration creates the agent_budgets table for Protocol 005 (Budget Control Protocol).
-- Agent budgets track the overall budget allocation per agent, enabling:
-- - Per-agent budget limits
-- - Total spending tracking across all leases
-- - Budget remaining calculations
-- - Budget allocation and adjustment

-- Create agent_budgets table
CREATE TABLE IF NOT EXISTS agent_budgets
(
  agent_id TEXT PRIMARY KEY,  -- 1:1 relationship with agents(id)
  total_allocated REAL NOT NULL,  -- USD total budget allocated
  total_spent REAL NOT NULL DEFAULT 0.0,  -- USD total spent across all leases
  budget_remaining REAL NOT NULL,  -- USD remaining (total_allocated - total_spent)
  created_at INTEGER NOT NULL,  -- Unix timestamp (milliseconds)
  updated_at INTEGER NOT NULL,  -- Unix timestamp (milliseconds)
  FOREIGN KEY ( agent_id ) REFERENCES agents( id ) ON DELETE CASCADE
);

-- Create index for fast lookups by update time
CREATE INDEX IF NOT EXISTS idx_agent_budgets_updated ON agent_budgets( updated_at );

-- Create guard table to prevent re-running this migration
CREATE TABLE IF NOT EXISTS _migration_010_completed ( id INTEGER PRIMARY KEY );
