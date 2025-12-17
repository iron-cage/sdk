-- Migration 007: Create token blacklist table
--
-- This migration creates the blacklist table for tracking invalidated JWT tokens.
-- When a user logs out, their token is added to this blacklist to prevent reuse.
-- Tokens remain blacklisted until their original expiration time.

CREATE TABLE IF NOT EXISTS blacklist
(
  -- Token ID (jti claim from JWT) - Primary key
  jti TEXT PRIMARY KEY NOT NULL,

  -- User ID who owned the token
  user_id TEXT NOT NULL,

  -- When the token was blacklisted (Unix timestamp)
  blacklisted_at INTEGER NOT NULL,

  -- Original token expiration time (Unix timestamp)
  -- Used for cleanup - can remove blacklist entries after expiration
  expires_at INTEGER NOT NULL
);

-- Index for querying tokens by user
CREATE INDEX IF NOT EXISTS idx_blacklist_user_id ON blacklist(user_id);

-- Index for cleanup queries (remove expired blacklist entries)
CREATE INDEX IF NOT EXISTS idx_blacklist_expires_at ON blacklist(expires_at);

-- Create guard table to mark migration as completed
CREATE TABLE IF NOT EXISTS _migration_007_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_007_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);
