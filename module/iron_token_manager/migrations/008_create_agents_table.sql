-- Migration 008: Create agents table and update api_tokens
--
-- This migration:
-- 1. Creates the agents table to store agent configurations
-- 2. Adds agent_id and provider columns to api_tokens
-- 3. Removes project_id from api_tokens (tokens are now user-centric)

-- Create agents table
CREATE TABLE IF NOT EXISTS agents (
  id TEXT PRIMARY KEY CHECK (LENGTH(id) > 0 AND LENGTH(id) <= 50),
  name TEXT NOT NULL CHECK ( LENGTH( name ) > 0 AND LENGTH( name ) <= 100 ),
  user_id TEXT NOT NULL CHECK ( LENGTH( user_id ) > 0 AND LENGTH( user_id ) <= 500 ),
  budget FLOAT NOT NULL CHECK ( budget >= 0.01 ),
  status TEXT NOT NULL DEFAULT 'active' CHECK ( status IN ( 'active', 'inactive', 'exhausted' ) ),
  tags TEXT CHECK ( tags IS NULL OR LENGTH( tags ) <= 500 ), -- JSON array of tags
  description TEXT CHECK ( description IS NULL OR LENGTH( description ) <= 500 ),
  project_id TEXT CHECK ( LENGTH( project_id ) > 0 AND LENGTH( project_id ) <= 500 ),
  created_at INTEGER NOT NULL
  updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_agents_created_at ON agents(created_at);

-- Add new columns to api_tokens
ALTER TABLE api_tokens ADD COLUMN agent_id INTEGER REFERENCES agents(id) ON DELETE CASCADE;
ALTER TABLE api_tokens ADD COLUMN provider TEXT;  -- Current provider for this token

FOREIGN KEY user_id REFERENCES users(id) ON DELETE CASCADE;
FOREIGN KEY agent_budget_id REFERENCES agent_budgets(id) ON DELETE CASCADE;

-- Create index on agent_id for fast lookups
CREATE INDEX IF NOT EXISTS idx_api_tokens_agent_id ON api_tokens(agent_id);

-- Note: SQLite doesn't support DROP COLUMN, so project_id will remain but should not be used
-- The application layer will ignore project_id and use agent_id + provider instead

-- Create guard table to prevent re-running this migration
CREATE TABLE IF NOT EXISTS _migration_008_completed (id INTEGER PRIMARY KEY);
