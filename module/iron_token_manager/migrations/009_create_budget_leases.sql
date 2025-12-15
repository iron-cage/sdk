-- Migration 009: Create budget_leases table
--
-- This migration creates the budget_leases table for Protocol 005 (Budget Control Protocol).
-- Leases track individual budget allocations per agent session, enabling:
-- - Budget handshake (IC Token → IP Token exchange)
-- - Per-session budget tracking
-- - Lease expiration and revocation
-- - Usage reporting

-- Create budget_leases table
CREATE TABLE IF NOT EXISTS budget_leases
(
  id TEXT PRIMARY KEY,  -- Format: lease_<uuid> (follows ID standards)
  agent_id INTEGER NOT NULL,  -- References agents(id)
  budget_id INTEGER NOT NULL,  -- References agent_budgets(agent_id)
  budget_granted REAL NOT NULL,  -- USD allocated for this lease
  budget_spent REAL NOT NULL DEFAULT 0.0,  -- USD spent in this lease
  lease_status TEXT NOT NULL DEFAULT 'active',  -- Status: active, expired, revoked
  created_at INTEGER NOT NULL,  -- Unix timestamp (milliseconds)
  expires_at INTEGER,  -- Unix timestamp (milliseconds), NULL for no expiration
  FOREIGN KEY ( agent_id ) REFERENCES agents( id ) ON DELETE CASCADE,
  FOREIGN KEY ( budget_id ) REFERENCES agent_budgets( agent_id ) ON DELETE CASCADE
);

-- Create indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_budget_leases_agent ON budget_leases( agent_id );
CREATE INDEX IF NOT EXISTS idx_budget_leases_status ON budget_leases( lease_status );
CREATE INDEX IF NOT EXISTS idx_budget_leases_created ON budget_leases( created_at );

-- Create guard table to prevent re-running this migration
CREATE TABLE IF NOT EXISTS _migration_009_completed ( id INTEGER PRIMARY KEY );
