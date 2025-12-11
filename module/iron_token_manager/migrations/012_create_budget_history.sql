-- Migration 012: Budget Modification History Table (Protocol 017)
CREATE TABLE IF NOT EXISTS budget_modification_history
(
  id TEXT PRIMARY KEY,
  agent_id INTEGER NOT NULL,
  modification_type TEXT NOT NULL CHECK( modification_type IN ( 'increase', 'decrease', 'reset' ) ),
  old_budget_micros INTEGER NOT NULL,
  new_budget_micros INTEGER NOT NULL,
  change_amount_micros INTEGER NOT NULL,
  modifier_id TEXT NOT NULL,
  reason TEXT NOT NULL CHECK( LENGTH( reason ) >= 10 AND LENGTH( reason ) <= 500 ),
  related_request_id TEXT,
  created_at INTEGER NOT NULL,
  FOREIGN KEY ( agent_id ) REFERENCES agents( id ) ON DELETE CASCADE,
  FOREIGN KEY ( related_request_id ) REFERENCES budget_change_requests( id ) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_budget_history_agent ON budget_modification_history( agent_id );
CREATE TABLE _migration_012_completed ( id INTEGER PRIMARY KEY );
INSERT INTO _migration_012_completed ( id ) VALUES ( 1 );
