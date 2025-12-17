-- Migration 020: Add account lockout fields
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

-- Add failed login tracking
ALTER TABLE users ADD COLUMN failed_login_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN last_failed_login INTEGER;
ALTER TABLE users ADD COLUMN locked_until INTEGER;

-- Create guard table to mark migration as completed
CREATE TABLE IF NOT EXISTS _migration_020_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_020_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);
