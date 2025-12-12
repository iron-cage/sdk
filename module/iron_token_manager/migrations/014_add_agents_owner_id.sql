-- Migration 014: Add owner_id column to agents table
--
-- This migration adds user ownership to agents table for multi-tenant isolation.
-- All agents must belong to a user who created them.
--
-- Security: Protocol 005 requires authorization checks to prevent cross-user
-- access to agent budgets and leases. This migration implements database-level
-- user isolation.

-- Add owner_id column to agents table
-- Note: SQLite requires a multi-step process for adding NOT NULL columns with foreign keys
-- Step 1: Add column as nullable
ALTER TABLE agents ADD COLUMN owner_id TEXT REFERENCES users(id) ON DELETE CASCADE;

-- Step 2: Update existing agents to have an owner (use first user if exists, or fail if no users)
-- In production, this should be handled by application logic or data migration
-- For now, set to NULL and application will validate on first use
UPDATE agents SET owner_id = (SELECT id FROM users LIMIT 1) WHERE owner_id IS NULL;

-- Step 3: Create index on owner_id for fast lookups
CREATE INDEX IF NOT EXISTS idx_agents_owner_id ON agents(owner_id);

-- Create guard table to prevent re-running this migration
CREATE TABLE IF NOT EXISTS _migration_014_completed (id INTEGER PRIMARY KEY);
