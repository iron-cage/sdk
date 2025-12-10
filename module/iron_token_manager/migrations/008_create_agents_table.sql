-- Migration 008: Create agents table and update api_tokens
--
-- This migration:
-- 1. Creates the agents table to store agent configurations
-- 2. Adds agent_id and provider columns to api_tokens
-- 3. Removes project_id from api_tokens (tokens are now user-centric)

-- Create agents table
CREATE TABLE IF NOT EXISTS agents (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  providers TEXT NOT NULL,  -- JSON array of supported providers
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_agents_created_at ON agents(created_at);

-- Add new columns to api_tokens
ALTER TABLE api_tokens ADD COLUMN agent_id INTEGER REFERENCES agents(id) ON DELETE CASCADE;
ALTER TABLE api_tokens ADD COLUMN provider TEXT;  -- Current provider for this token

-- Create index on agent_id for fast lookups
CREATE INDEX IF NOT EXISTS idx_api_tokens_agent_id ON api_tokens(agent_id);

-- Note: SQLite doesn't support DROP COLUMN, so project_id will remain but should not be used
-- The application layer will ignore project_id and use agent_id + provider instead

-- Create guard table to prevent re-running this migration
CREATE TABLE IF NOT EXISTS _migration_008_completed (id INTEGER PRIMARY KEY);
