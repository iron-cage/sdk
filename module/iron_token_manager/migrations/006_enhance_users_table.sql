-- Migration 006: Enhance users table with additional fields
--
-- Adds email, last_login, suspended_at, suspended_by, deleted_at, deleted_by, force_password_change

ALTER TABLE users ADD COLUMN email TEXT;
ALTER TABLE users ADD COLUMN last_login INTEGER;
ALTER TABLE users ADD COLUMN suspended_at INTEGER;
ALTER TABLE users ADD COLUMN suspended_by INTEGER REFERENCES users(id);
ALTER TABLE users ADD COLUMN deleted_at INTEGER;
ALTER TABLE users ADD COLUMN deleted_by INTEGER REFERENCES users(id);
ALTER TABLE users ADD COLUMN force_password_change INTEGER NOT NULL DEFAULT 0;

-- Create guard table
CREATE TABLE IF NOT EXISTS _migration_006_completed (id INTEGER PRIMARY KEY);
