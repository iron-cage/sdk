-- Migration 005: Enhance users table for user management
--
-- This migration adds fields required for admin user management operations:
-- - email for user identification and communication
-- - last_login for tracking user activity
-- - suspension tracking (suspended_at, suspended_by)
-- - deletion tracking (deleted_at, deleted_by)
-- - force_password_change for admin password resets
--
-- Adds indexes for performance on email, role, is_active searches

-- Add last_login timestamp (Unix epoch milliseconds)
ALTER TABLE users ADD COLUMN last_login INTEGER;

-- Add account suspension tracking
ALTER TABLE users ADD COLUMN suspended_at INTEGER;
ALTER TABLE users ADD COLUMN suspended_by TEXT REFERENCES users(id);

-- Add account deletion tracking (soft delete)
ALTER TABLE users ADD COLUMN deleted_at INTEGER;
ALTER TABLE users ADD COLUMN deleted_by TEXT REFERENCES users(id);

-- Add password reset fields
ALTER TABLE users ADD COLUMN force_password_change INTEGER NOT NULL DEFAULT 0;

-- Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active);
CREATE INDEX IF NOT EXISTS idx_users_username_search ON users(username);

-- Create guard table to mark migration as completed
CREATE TABLE IF NOT EXISTS _migration_005_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_005_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);
-- INSERT OR IGNORE INTO users (id, email, username, password_hash, role, is_active, created_at) VALUES ('admin', 'admin@admin.com', 'admin', '$2b$12$zZOfQakwkynHa0mBVlSvQ.rmzFZxkkN6OelZE/bLDCY1whIW.IWf2', 'admin', 1, strftime('%s', 'now'));"

