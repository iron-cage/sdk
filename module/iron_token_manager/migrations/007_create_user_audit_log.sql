-- Migration 007: Create user audit log
--
-- Tracks all user management operations

CREATE TABLE IF NOT EXISTS user_audit_log
(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  operation TEXT NOT NULL CHECK (operation IN ('create', 'suspend', 'activate', 'delete', 'role_change', 'password_reset')),
  target_user_id INTEGER NOT NULL,
  performed_by INTEGER NOT NULL,
  timestamp INTEGER NOT NULL,
  previous_state TEXT,
  new_state TEXT,
  reason TEXT,
  FOREIGN KEY (target_user_id) REFERENCES users(id) ON DELETE RESTRICT,
  FOREIGN KEY (performed_by) REFERENCES users(id) ON DELETE RESTRICT
);

-- Create guard table
CREATE TABLE IF NOT EXISTS _migration_007_completed (id INTEGER PRIMARY KEY);
