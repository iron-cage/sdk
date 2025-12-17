-- Seed dev IC token hash for agent_1 (demo/dev only)
-- Hash corresponds to dev_ic_token inserted in migration 017
-- SHA-256: 897b52e23fde48c0c98b1f5aa80b80292cf1d8301adc51e7475d36068d53733a

UPDATE agents
SET ic_token_hash = '897b52e23fde48c0c98b1f5aa80b80292cf1d8301adc51e7475d36068d53733a',
    ic_token_created_at = strftime('%s','now')
WHERE id = 1 AND ic_token_hash IS NULL;

CREATE TABLE IF NOT EXISTS _migration_022_completed (id INTEGER PRIMARY KEY);
