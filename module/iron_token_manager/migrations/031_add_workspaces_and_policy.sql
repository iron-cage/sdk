BEGIN;

CREATE TABLE IF NOT EXISTS workspaces (
  id    INTEGER PRIMARY KEY AUTOINCREMENT,
  name  TEXT NOT NULL,
  domain TEXT NOT NULL,
  account_type TEXT NOT NULL CHECK (account_type IN ('client', 'internal')),
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000)
);

-- One policy row per workspace; workspace_id doubles as primary key.
CREATE TABLE IF NOT EXISTS workspace_policy (
  workspace_id      INTEGER PRIMARY KEY REFERENCES workspaces(id),
  budget_amount_cents INTEGER NOT NULL CHECK (budget_amount_cents >= 0),
  budget_period     TEXT NOT NULL CHECK (budget_period IN ('day', 'week', 'month')),
  default_model     TEXT,
  updated_at        INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000)
);

CREATE TABLE IF NOT EXISTS _migration_031_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_031_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);

COMMIT;
