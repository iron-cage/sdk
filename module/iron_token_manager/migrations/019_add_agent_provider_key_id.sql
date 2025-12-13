-- Migration 019: Add provider_key_id to agents table
--
-- Feature 014: Agent-Provider Key Assignment
-- Adds a foreign key from agents to ai_provider_keys, enabling each agent
-- to have one assigned provider key.

-- Add provider_key_id column (nullable FK to ai_provider_keys)
ALTER TABLE agents ADD COLUMN provider_key_id INTEGER REFERENCES ai_provider_keys(id) ON DELETE SET NULL;

-- Create index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_agents_provider_key_id ON agents(provider_key_id);
