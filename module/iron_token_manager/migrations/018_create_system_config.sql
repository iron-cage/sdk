-- Migration 018: Create system_config table and seed initial configuration
--
-- This migration:
-- 1. Creates system_config table for persistent configuration
-- 2. Seeds the IC token secret with a randomly generated value
-- 3. Seeds a default agent and budget for development/testing
--
-- NOTE: The ic_token_secret is seeded with hex(randomblob(64)) to prevent
-- predictable secrets on fresh deployments. Use init_admin to view or rotate it.

BEGIN;

-- Create system_config table for persistent settings
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    description TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now') * 1000),
    updated_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
);

-- Insert IC token secret with a randomly generated value.
-- Never predictable from git history. Use init_admin to obtain or rotate.
INSERT OR IGNORE INTO system_config (key, value, description)
VALUES (
    'ic_token_secret',
    hex(randomblob(64)),
    'Secret key for signing IC tokens (Protocol 005). View or rotate via init_admin.'
);

-- Seed default agent for development (agent_1)
INSERT OR IGNORE INTO agents (id, name, providers, created_at)
VALUES (
    1,
    'agent_1',
    '["openai", "anthropic"]',
    strftime('%s', 'now') * 1000
);

-- Seed default budget for agent_1 ($100 USD)
INSERT OR IGNORE INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
VALUES (
    1,
    100.0,
    0.0,
    100.0,
    strftime('%s', 'now') * 1000,
    strftime('%s', 'now') * 1000
);

-- Create guard table to prevent re-running this migration
CREATE TABLE IF NOT EXISTS _migration_018_completed (id INTEGER PRIMARY KEY);
INSERT INTO _migration_018_completed (id) VALUES (1);

COMMIT;
