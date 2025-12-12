-- Migration 014: Enhance agents table
--
-- This migration adds additional fields to agents table.
ALTER TABLE agents ADD COLUMN tags TEXT; -- JSON array of tags
ALTER TABLE agents ADD COLUMN description TEXT; -- Agent description
ALTER TABLE agents ADD COLUMN status TEXT CHECK(status IN ('active', 'inactive', 'exhausted')); -- Agent status
ALTER TABLE agents ADD COLUMN updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000); -- Unix timestamp (milliseconds)
ALTER TABLE agents ADD COLUMN project_id TEXT; -- Project ID
-- Step 2: Update existing agents to have an owner (use first user if exists, or fail if no users)
-- In production, this should be handled by application logic or data migration
-- For now, set to NULL and application will validate on first use
UPDATE agents SET owner_id = (SELECT id FROM users LIMIT 1) WHERE owner_id IS NULL;

-- Step 3: Create index on owner_id for fast lookups
CREATE INDEX IF NOT EXISTS idx_agents_owner_id ON agents(owner_id);

-- Create guard table to prevent re-running this migration
CREATE TABLE IF NOT EXISTS _migration_014_completed (id INTEGER PRIMARY KEY);
