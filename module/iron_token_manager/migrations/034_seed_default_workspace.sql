BEGIN;

-- Single-workspace deployments use id=1. Seed a placeholder row so that
-- GET /api/v1/me/workspace returns a valid response on a fresh database,
-- even before an admin has run the FreeForm company-setup flow. The
-- placeholder is overwritten by POST /api/v1/freeform/company on first use.
INSERT OR IGNORE INTO workspaces (id, name, domain, account_type, created_at)
VALUES (1, 'Workspace', 'example.com', 'internal', strftime('%s', 'now') * 1000);

CREATE TABLE IF NOT EXISTS _migration_034_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_034_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);

COMMIT;
