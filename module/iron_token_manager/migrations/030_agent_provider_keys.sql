-- Migration 030: Create agent_provider_keys join table for multi-key support
CREATE TABLE IF NOT EXISTS agent_provider_keys (
    agent_id        INTEGER NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    provider_key_id INTEGER NOT NULL REFERENCES ai_provider_keys(id) ON DELETE CASCADE,
    PRIMARY KEY (agent_id, provider_key_id)
);

-- Migrate existing single-key assignments
INSERT OR IGNORE INTO agent_provider_keys (agent_id, provider_key_id)
SELECT id, provider_key_id FROM agents WHERE provider_key_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS _migration_030_completed (id INTEGER PRIMARY KEY);
