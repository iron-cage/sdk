-- Migration 011: Create analytics_events table
--
-- Stores LLM request events for Protocol 012 Analytics API.
-- Supports per-agent event deduplication, period filtering, and aggregation queries.
-- Cost stored in microdollars (1 USD = 1,000,000 microdollars) for precision.

CREATE TABLE IF NOT EXISTS analytics_events
(
  -- Primary key (auto-increment for SQLite)
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Event identification (for per-agent deduplication)
  event_id TEXT NOT NULL,

  -- Timestamp when event occurred (milliseconds since epoch)
  timestamp_ms INTEGER NOT NULL,

  -- Event type: 'llm_request_completed' or 'llm_request_failed'
  event_type TEXT NOT NULL CHECK (
    event_type IN ('llm_request_completed', 'llm_request_failed')
  ),

  -- Model and provider info
  model TEXT NOT NULL,
  provider TEXT NOT NULL CHECK (
    provider IN ('openai', 'anthropic', 'unknown')
  ),

  -- Token counts (0 for failed events)
  input_tokens INTEGER NOT NULL DEFAULT 0,
  output_tokens INTEGER NOT NULL DEFAULT 0,

  -- Cost in microdollars (1 USD = 1,000,000 microdollars)
  cost_micros INTEGER NOT NULL DEFAULT 0,

  -- Optional associations
  agent_id INTEGER,      -- References agents(id)
  provider_id TEXT,      -- Provider key identifier (ip_xxx)

  -- Error info (for failed events)
  error_code TEXT,
  error_message TEXT,

  -- Server metadata
  received_at INTEGER NOT NULL,  -- When server received the event

  -- Per-agent deduplication: same event_id from different agents are distinct
  UNIQUE(agent_id, event_id)
);

-- Indexes for query performance
CREATE INDEX IF NOT EXISTS idx_analytics_events_timestamp ON analytics_events(timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_analytics_events_agent_id ON analytics_events(agent_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_provider ON analytics_events(provider);
CREATE INDEX IF NOT EXISTS idx_analytics_events_provider_id ON analytics_events(provider_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_event_type ON analytics_events(event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_events_model ON analytics_events(model);

-- Composite index for common query patterns
CREATE INDEX IF NOT EXISTS idx_analytics_events_agent_timestamp
  ON analytics_events(agent_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_analytics_events_provider_timestamp
  ON analytics_events(provider, timestamp_ms);

-- Guard table to prevent re-running this migration
CREATE TABLE IF NOT EXISTS _migration_011_completed (id INTEGER PRIMARY KEY);
