BEGIN;

-- One row per invite acceptance; user_id is NULL until the member registers.
CREATE TABLE IF NOT EXISTS invite_seats (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  invite_link_id TEXT NOT NULL REFERENCES invite_links(id),
  user_id        TEXT REFERENCES users(id),
  accepted_at    INTEGER,
  approved_at    INTEGER,
  approved_by    TEXT REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_invite_seats_link ON invite_seats (invite_link_id);
CREATE INDEX IF NOT EXISTS idx_invite_seats_user ON invite_seats (user_id);

CREATE TABLE IF NOT EXISTS _migration_033_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_033_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);

COMMIT;
