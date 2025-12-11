-- Migration 003: Create users table for authentication
--
-- This migration adds the users table required by iron_api's auth module.
-- The table was previously only defined in test code but is needed for
-- production authentication endpoints.
--
-- NOTE: This migration is conditionally executed by Rust code.
-- The Rust code checks if _migration_003_completed table exists before running this.

-- Users table for user authentication
CREATE TABLE IF NOT EXISTS users
(
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Authentication fields
  username TEXT NOT NULL UNIQUE CHECK (LENGTH(username) > 0 AND LENGTH(username) <= 255),
  password_hash TEXT NOT NULL CHECK (LENGTH(password_hash) > 0 AND LENGTH(password_hash) <= 255),
  email TEXT NOT NULL UNIQUE CHECK (LENGTH(email) > 0 AND LENGTH(email) <= 255),

  -- Authorization
  role TEXT NOT NULL DEFAULT 'user' CHECK (LENGTH(role) > 0 AND LENGTH(role) <= 50),

  -- Status
  is_active INTEGER NOT NULL DEFAULT 1,

  -- Timestamps (Unix epoch seconds)
  created_at INTEGER NOT NULL
);

-- Index for username lookups during authentication
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

-- Token blacklist for logout functionality (JWT revocation)
CREATE TABLE IF NOT EXISTS blacklist
(
  -- JWT ID (jti claim)
  jti TEXT PRIMARY KEY CHECK (LENGTH(jti) > 0 AND LENGTH(jti) <= 255),

  -- User who owned the token
  user_id INTEGER NOT NULL,

  -- When the token was blacklisted (Unix epoch seconds)
  blacklisted_at INTEGER NOT NULL,

  -- When the token expires (Unix epoch seconds)
  expires_at INTEGER NOT NULL
);

-- Index for looking up blacklisted tokens by user
CREATE INDEX IF NOT EXISTS idx_blacklist_user_id ON blacklist(user_id);

-- Create guard table to mark migration as completed
CREATE TABLE IF NOT EXISTS _migration_003_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_003_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);