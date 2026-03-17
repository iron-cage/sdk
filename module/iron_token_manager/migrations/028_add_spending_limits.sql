-- Migration 028: Per-IC-key and Per-IP-key spending limits
--
-- Adds spending cap/used columns to agent_budgets (IC-key limits)
-- and provider_key_id attribution to leases and analytics.

-- Per-IC-key (agent) spending limit
ALTER TABLE agent_budgets ADD COLUMN spending_cap_microdollars INTEGER;
ALTER TABLE agent_budgets ADD COLUMN spending_used_microdollars INTEGER NOT NULL DEFAULT 0;

-- Per-IP-key attribution in leases
ALTER TABLE budget_leases ADD COLUMN provider_key_id INTEGER REFERENCES ai_provider_keys(id) ON DELETE SET NULL;
CREATE INDEX idx_budget_leases_provider_key ON budget_leases(provider_key_id);

-- Per-IP-key attribution in analytics
ALTER TABLE analytics_events ADD COLUMN provider_key_id INTEGER;
CREATE INDEX idx_analytics_events_provider_key ON analytics_events(provider_key_id);

-- Guard table
CREATE TABLE IF NOT EXISTS _migration_028_completed (applied_at INTEGER NOT NULL);
INSERT OR IGNORE INTO _migration_028_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);
