-- Migration 002: Add length constraints for defense-in-depth
--
-- Fix(issue-001): Runtime enforcement of string length limits
-- Root cause: No database-level constraints on input size
-- Pitfall: Always add database constraints as defense-in-depth
--
-- Fix(issue-003): Migration idempotency to prevent data loss
-- Root cause: Migration dropped api_tokens table on every run, cascading to token_usage deletion
-- Pitfall: Migrations must be idempotent - use guard table to track application state
--
-- NOTE: This migration is conditionally executed by Rust code in storage.rs
-- The Rust code checks if _migration_002_completed table exists before running this.
-- This SQL file should ONLY be executed if the guard table doesn't exist.
--
-- SQLite doesn't support ALTER TABLE ADD CONSTRAINT CHECK, so we must:
-- 1. Create new table with CHECK constraints
-- 2. Copy data from old table
-- 3. Drop old table
-- 4. Rename new table

-- Clean up any partial migration attempts
DROP TABLE IF EXISTS api_tokens_new;

-- Step 1: Create new table with CHECK constraints
CREATE TABLE api_tokens_new
(
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Token hash (never store plaintext)
  token_hash TEXT NOT NULL UNIQUE,

  -- User/Project association with length constraints
  user_id TEXT NOT NULL CHECK (LENGTH(user_id) > 0 AND LENGTH(user_id) <= 500),
  project_id TEXT CHECK (project_id IS NULL OR (LENGTH(project_id) > 0 AND LENGTH(project_id) <= 500)),

  -- Token metadata
  name TEXT,
  scopes TEXT,
  is_active BOOLEAN NOT NULL DEFAULT 1,

  -- Timestamps
  created_at INTEGER NOT NULL,
  last_used_at INTEGER,
  expires_at INTEGER,

  -- Unique constraint
  CONSTRAINT token_hash_unique UNIQUE( token_hash )
);

-- Step 2: Copy data from old table (if it exists and has data)
INSERT INTO api_tokens_new (id, token_hash, user_id, project_id, name, scopes, is_active, created_at, last_used_at, expires_at)
SELECT id, token_hash, user_id, project_id, name, scopes, is_active, created_at, last_used_at, expires_at
FROM api_tokens
WHERE EXISTS (SELECT 1 FROM api_tokens LIMIT 1);

-- Step 3: Drop old table
DROP TABLE IF EXISTS api_tokens;

-- Step 4: Rename new table to original name
ALTER TABLE api_tokens_new RENAME TO api_tokens;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_api_tokens_user_id ON api_tokens( user_id );
CREATE INDEX IF NOT EXISTS idx_api_tokens_project_id ON api_tokens( project_id );
CREATE INDEX IF NOT EXISTS idx_api_tokens_is_active ON api_tokens( is_active );

-- Create guard table to mark migration as completed
CREATE TABLE _migration_002_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_002_completed (applied_at) VALUES (1733600000000); -- 2025-12-07
