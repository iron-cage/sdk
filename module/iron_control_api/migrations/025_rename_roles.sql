-- Migration 025: Rename RBAC roles for Task 008
--
-- Renames role values in the users table:
--   "user"   -> "manager"     (key/token management, no role assignment)
--   "viewer" -> "developer"   (read-only + own-token inspection)
--
-- "admin" remains unchanged.
--
-- NOTE: This migration is conditionally executed by Rust code.
-- The Rust code checks if _migration_025_completed table exists before running this.

UPDATE users SET role = 'manager'   WHERE role = 'user';
UPDATE users SET role = 'developer' WHERE role = 'viewer';

-- Create guard table to mark migration as completed
CREATE TABLE IF NOT EXISTS _migration_025_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_025_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);
