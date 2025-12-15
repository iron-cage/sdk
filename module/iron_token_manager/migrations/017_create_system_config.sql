-- Migration 017: Create system_config table and seed development data
--
-- This migration:
-- 1. Creates system_config table for persistent configuration
-- 2. Stores the IC token secret and pre-generated IC token
-- 3. Seeds a default agent and budget for development/testing
--
-- The IC token is generated with secret: dev-ic-token-secret-change-in-production
-- and will work on any server using that same secret.

-- Create system_config table for persistent settings
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    description TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now') * 1000),
    updated_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
);

-- Insert IC token secret (used for signing/verifying IC tokens)
INSERT OR IGNORE INTO system_config (key, value, description)
VALUES (
    'ic_token_secret',
    'dev-ic-token-secret-change-in-production',
    'Secret key for signing IC tokens (Protocol 005). Change in production!'
);

-- Insert pre-generated IC token for agent_1 with budget_test
-- This token works with secret: dev-ic-token-secret-change-in-production
-- Claims: agent_id=agent_1, budget_id=budget_test, permissions=[llm:call, analytics:write]
INSERT OR IGNORE INTO system_config (key, value, description)
VALUES (
    'dev_ic_token',
    'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJhZ2VudF9pZCI6ImFnZW50XzEiLCJidWRnZXRfaWQiOiJidWRnZXRfdGVzdCIsImlhdCI6MTc2NTU0OTExNSwiaXNzIjoiaXJvbi1jb250cm9sLXBhbmVsIiwicGVybWlzc2lvbnMiOlsibGxtOmNhbGwiLCJhbmFseXRpY3M6d3JpdGUiXX0.9cJZnr4OP7pIp30ntTR9NRgfNhDykEPFW_Ew1PYn-zU',
    'Pre-generated IC token for development. agent_id=agent_1, budget_id=budget_test'
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
CREATE TABLE IF NOT EXISTS _migration_017_completed (id INTEGER PRIMARY KEY);