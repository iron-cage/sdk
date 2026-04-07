-- Seed IC token hash for agent_1 (fresh deployment).
--
-- Uses randomblob(32) so every new deployment gets an unpredictable token hash.
-- The corresponding plaintext token is never inserted, making this slot
-- effectively locked until an admin assigns a real IC token via the control API.
-- This prevents any attacker who has read the migration source from
-- authenticating as agent_1 on a fresh install.

BEGIN;

UPDATE agents
SET ic_token_hash = lower(hex(randomblob(32))),
    ic_token_created_at = strftime('%s','now') * 1000
WHERE id = 1 AND ic_token_hash IS NULL;

CREATE TABLE IF NOT EXISTS _migration_023_completed (id INTEGER PRIMARY KEY);
INSERT INTO _migration_023_completed (id) VALUES (1);

COMMIT;
