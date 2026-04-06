BEGIN;

-- Add unique index on agents.ic_token_hash to:
-- 1. Speed up hash-based lookups in the proxy (currently a full table scan).
-- 2. Enforce that no two agents share the same IC token hash.
-- The WHERE clause excludes agents with no IC token assigned (ic_token_hash IS NULL).
CREATE UNIQUE INDEX IF NOT EXISTS idx_agents_ic_token_hash
  ON agents (ic_token_hash)
  WHERE ic_token_hash IS NOT NULL;

CREATE TABLE IF NOT EXISTS _migration_027_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_027_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);

COMMIT;
