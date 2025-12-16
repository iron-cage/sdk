-- Migration 021: Add IC token columns to agents table
--
-- This migration adds columns for IC (Iron Cage) token management:
-- - ic_token_hash: SHA-256 hash of the IC token (for validation without storing plaintext)
-- - ic_token_created_at: Timestamp when IC token was generated
--
-- IC tokens are JWTs used by agents to authenticate with the budget runtime.
-- The actual token is shown only once on creation (like API tokens).

-- Add IC token hash column (stores SHA-256 hash, not the actual token)
ALTER TABLE agents ADD COLUMN ic_token_hash TEXT;

-- Add IC token creation timestamp
ALTER TABLE agents ADD COLUMN ic_token_created_at INTEGER;

-- Index for potential token hash lookups (e.g., token validation)
CREATE INDEX IF NOT EXISTS idx_agents_ic_token_hash ON agents(ic_token_hash);

-- Create guard table to prevent re-running this migration
CREATE TABLE IF NOT EXISTS _migration_021_completed (id INTEGER PRIMARY KEY);
