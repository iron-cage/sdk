-- Migration 006: Create user audit log table
--
-- This migration creates the user_audit_log table for tracking all user management
-- operations performed by admins. Every create, suspend, activate, delete, role change,
-- and password reset operation is logged here for security auditing.
--
-- Audit logs are append-only and preserved even when users are deleted (RESTRICT constraint).

CREATE TABLE IF NOT EXISTS user_audit_log
(
  -- Primary key
  id TEXT PRIMARY KEY CHECK (LENGTH(id) > 0 AND LENGTH(id) <= 255),

  -- Operation type
  operation TEXT NOT NULL CHECK (
    operation IN (
      'create',
      'suspend',
      'activate',
      'delete',
      'role_change',
      'password_reset'
    )
  ),

  -- Target user (who was affected)
  target_user_id TEXT NOT NULL CHECK (LENGTH(target_user_id) > 0 AND LENGTH(target_user_id) <= 255),

  -- Admin who performed the operation
  performed_by TEXT NOT NULL CHECK (LENGTH(performed_by) > 0 AND LENGTH(performed_by) <= 255),

  -- When the operation occurred (Unix epoch milliseconds)
  timestamp INTEGER NOT NULL,

  -- Previous state (JSON) - for updates only
  previous_state TEXT,

  -- New state (JSON) - for updates only
  new_state TEXT,

  -- Optional reason for action
  reason TEXT,

  -- Foreign key constraints (RESTRICT to preserve audit trail)
  FOREIGN KEY (target_user_id) REFERENCES users(id) ON DELETE RESTRICT,
  FOREIGN KEY (performed_by) REFERENCES users(id) ON DELETE RESTRICT
);

-- Indexes for audit log queries
CREATE INDEX IF NOT EXISTS idx_user_audit_target ON user_audit_log(target_user_id);
CREATE INDEX IF NOT EXISTS idx_user_audit_performer ON user_audit_log(performed_by);
CREATE INDEX IF NOT EXISTS idx_user_audit_timestamp ON user_audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_user_audit_operation ON user_audit_log(operation);

-- Create guard table to mark migration as completed
CREATE TABLE IF NOT EXISTS _migration_006_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_006_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);
