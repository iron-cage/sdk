BEGIN;

-- One row per generated invite link.
-- token_hash stores SHA-256 of the raw token; raw value is returned once at creation.
CREATE TABLE IF NOT EXISTS invite_links (
  id          TEXT PRIMARY KEY,
  workspace_id INTEGER NOT NULL REFERENCES workspaces(id),
  token_hash  TEXT NOT NULL UNIQUE,
  seats_total INTEGER NOT NULL DEFAULT 1 CHECK (seats_total >= 1),
  seats_used  INTEGER NOT NULL DEFAULT 0 CHECK (seats_used >= 0),
  expires_at  INTEGER,
  created_by  TEXT NOT NULL REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_invite_links_token_hash ON invite_links (token_hash);
CREATE INDEX IF NOT EXISTS idx_invite_links_workspace ON invite_links (workspace_id);

CREATE TABLE IF NOT EXISTS _migration_032_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_032_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);

COMMIT;
