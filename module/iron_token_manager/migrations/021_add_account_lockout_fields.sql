-- Migration 021: Add account lockout fields
--
-- **Authority:** Protocol 007 § Security Considerations (line 158)
--
-- This migration adds fields required for account lockout after failed login attempts:
-- - failed_login_count: Counter for consecutive failed login attempts
-- - last_failed_login: Timestamp of most recent failed login (Unix epoch milliseconds)
-- - locked_until: Timestamp when account lockout expires (Unix epoch milliseconds)
--
-- Security Requirement:
-- "Account lockout after 10 failed attempts (manual unlock by admin)"
--
-- Lockout Policy:
-- - 10 consecutive failed login attempts triggers lockout
-- - Lockout duration: 15-30 minutes (configurable)
-- - Counter resets on successful login
-- - Admin can manually unlock by setting locked_until = NULL

-- Idempotent migration: Only add columns if table doesn't already have them
-- We check by attempting to query the column; if it exists, skip the ALTER TABLE

-- Try to add columns with error handling via guard table
-- First, check if we've already run this migration
CREATE TABLE IF NOT EXISTS _migration_021_completed (applied_at INTEGER NOT NULL);

-- The Rust code checks this guard table before running the SQL below
-- If the guard table has rows, the migration is skipped entirely
-- This SQL only runs if the guard table is empty

-- Add failed login tracking
ALTER TABLE users ADD COLUMN failed_login_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN last_failed_login INTEGER;
ALTER TABLE users ADD COLUMN locked_until INTEGER;

-- Mark migration as completed
INSERT INTO _migration_021_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);
