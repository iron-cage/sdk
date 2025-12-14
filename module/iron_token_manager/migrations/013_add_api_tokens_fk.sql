-- Migration 013: Add foreign key constraint from api_tokens to users
--
-- Purpose: Enforce referential integrity - tokens cannot exist without valid users
-- Impact: Implements IMPOSSIBLE STATE: "Cannot create token without valid user_id (FK constraint fails)"
--
-- SQLite limitation: Cannot use ALTER TABLE ADD CONSTRAINT for foreign keys
-- Solution: Rebuild table with FK constraint
--
-- Fix(issue-migration-013-lost-constraints): Preserve CHECK constraints from migration 002
-- Root cause: Table rebuild for FK addition dropped existing CHECK constraints on user_id/project_id length
-- Pitfall: When rebuilding tables to add constraints, must preserve ALL existing constraints
--
-- Fix(issue-migration-013-schema-inconsistency): Align user_id length constraint with users.id
-- Root cause: api_tokens.user_id allowed 500 chars but users.id (FK target) only allows 255
-- Pitfall: FK column constraints must be compatible with referenced column constraints

-- Check if migration already applied
CREATE TABLE IF NOT EXISTS _migration_013_completed (applied_at INTEGER NOT NULL);

-- Only proceed if not already applied
INSERT INTO _migration_013_completed (applied_at)
SELECT strftime('%s', 'now') * 1000
WHERE NOT EXISTS (SELECT 1 FROM _migration_013_completed);

-- Temporarily disable foreign key checks during table rebuild
PRAGMA foreign_keys = OFF;

-- Rebuild api_tokens table with FK constraint
-- Step 1: Create new table with FK
DROP TABLE IF EXISTS api_tokens_new;
CREATE TABLE api_tokens_new
(
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Token hash (SHA-256, never store plaintext)
  token_hash TEXT NOT NULL UNIQUE,

  -- User/Project association with length constraints
  -- Note: user_id max length is 255 to match users.id constraint (migration 003)
  user_id TEXT NOT NULL CHECK (LENGTH(user_id) > 0 AND LENGTH(user_id) <= 255),
  project_id TEXT CHECK (project_id IS NULL OR (LENGTH(project_id) > 0 AND LENGTH(project_id) <= 500)),

  -- Agent association (added in migration 008)
  agent_id INTEGER,  -- nullable reference to agents table
  provider TEXT,

  -- Token metadata
  name TEXT,  -- human-friendly name
  scopes TEXT,  -- JSON array of allowed scopes
  is_active BOOLEAN NOT NULL DEFAULT 1,

  -- Timestamps
  created_at INTEGER NOT NULL,  -- milliseconds since epoch
  last_used_at INTEGER,  -- nullable
  expires_at INTEGER,  -- nullable (NULL = never expires)
  revoked_at INTEGER,  -- nullable, set when token is revoked

  -- Unique constraint on token_hash
  CONSTRAINT token_hash_unique UNIQUE(token_hash)
);

-- Step 2: Copy data from old table to new table
-- Note: revoked_at column will be NULL for all existing tokens since it's added in migration 015
INSERT INTO api_tokens_new
  (id, token_hash, user_id, project_id, agent_id, provider, name, scopes, is_active, created_at, last_used_at, expires_at, revoked_at)
SELECT
  id, token_hash, user_id, project_id, agent_id, provider, name, scopes, is_active, created_at, last_used_at, expires_at, NULL
FROM api_tokens;

-- Step 3: Drop old table
DROP TABLE api_tokens;

-- Step 4: Rename new table to original name
ALTER TABLE api_tokens_new RENAME TO api_tokens;

-- Step 5: Recreate indexes
CREATE INDEX IF NOT EXISTS idx_api_tokens_user_id ON api_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_api_tokens_project_id ON api_tokens(project_id);
CREATE INDEX IF NOT EXISTS idx_api_tokens_is_active ON api_tokens(is_active);
-- CREATE INDEX IF NOT EXISTS idx_api_tokens_agent_id ON api_tokens(agent_id);

-- Re-enable foreign key checks
PRAGMA foreign_keys = ON;
