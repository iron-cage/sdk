-- Migration 011: Budget Change Requests Table (Protocol 012)
CREATE TABLE IF NOT EXISTS budget_change_requests
(
  id TEXT PRIMARY KEY,
  agent_id INTEGER NOT NULL,
  requester_id TEXT NOT NULL,
  current_budget_micros INTEGER NOT NULL,
  requested_budget_micros INTEGER NOT NULL,
  justification TEXT NOT NULL CHECK( LENGTH( justification ) >= 20 AND LENGTH( justification ) <= 500 ),
  status TEXT NOT NULL CHECK( status IN ( 'pending', 'approved', 'rejected', 'cancelled' ) ),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY ( agent_id ) REFERENCES agents( id ) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_budget_requests_status ON budget_change_requests( status );
CREATE INDEX IF NOT EXISTS idx_budget_requests_agent ON budget_change_requests( agent_id );
CREATE TABLE _migration_011_completed ( id INTEGER PRIMARY KEY );
INSERT INTO _migration_011_completed ( id ) VALUES ( 1 );
