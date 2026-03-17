-- Migration 027: Add Gemini and xAI provider support
--
-- SQLite does not support ALTER TABLE to modify CHECK constraints.
-- This migration recreates ai_provider_keys and analytics_events
-- with updated CHECK constraints to include 'gemini' and 'xai'.
--
-- Full column sets include all additions from prior migrations:
--   ai_provider_keys: spending cap columns added by migration 024
--   analytics_events: provider_key_id column added by migration 026

PRAGMA foreign_keys = OFF;

-- Recreate ai_provider_keys with extended provider CHECK
-- (includes spending_cap_microdollars, spending_used_microdollars from migration 024)
CREATE TABLE ai_provider_keys_new
(
  id                          INTEGER PRIMARY KEY AUTOINCREMENT,
  provider                    TEXT    NOT NULL CHECK ( provider IN ( 'openai', 'anthropic', 'gemini', 'xai' ) ),
  encrypted_api_key           TEXT    NOT NULL,
  encryption_nonce            TEXT    NOT NULL,
  base_url                    TEXT    CHECK ( base_url IS NULL OR LENGTH( base_url ) <= 2000 ),
  description                 TEXT    CHECK ( description IS NULL OR LENGTH( description ) <= 500 ),
  is_enabled                  INTEGER NOT NULL DEFAULT 1,
  created_at                  INTEGER NOT NULL,
  last_used_at                INTEGER,
  balance_cents               INTEGER,
  balance_updated_at          INTEGER,
  user_id                     TEXT    NOT NULL CHECK ( LENGTH( user_id ) > 0 AND LENGTH( user_id ) <= 500 ),
  spending_cap_microdollars   INTEGER,
  spending_used_microdollars  INTEGER NOT NULL DEFAULT 0
);

INSERT INTO ai_provider_keys_new
SELECT id, provider, encrypted_api_key, encryption_nonce, base_url, description,
       is_enabled, created_at, last_used_at, balance_cents, balance_updated_at, user_id,
       spending_cap_microdollars, spending_used_microdollars
FROM ai_provider_keys;

DROP TABLE ai_provider_keys;
ALTER TABLE ai_provider_keys_new RENAME TO ai_provider_keys;

-- Restore indexes
CREATE INDEX IF NOT EXISTS idx_ai_provider_keys_user_id    ON ai_provider_keys( user_id );
CREATE INDEX IF NOT EXISTS idx_ai_provider_keys_provider   ON ai_provider_keys( provider );
CREATE INDEX IF NOT EXISTS idx_ai_provider_keys_is_enabled ON ai_provider_keys( is_enabled );

-- Recreate analytics_events with extended provider CHECK
-- (includes provider_key_id column from migration 026)
CREATE TABLE analytics_events_new
(
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  event_id        TEXT    NOT NULL,
  timestamp_ms    INTEGER NOT NULL,
  event_type      TEXT    NOT NULL CHECK (
    event_type IN ('llm_request_completed', 'llm_request_failed')
  ),
  model           TEXT    NOT NULL,
  provider        TEXT    NOT NULL CHECK (
    provider IN ('openai', 'anthropic', 'gemini', 'xai', 'unknown')
  ),
  input_tokens    INTEGER NOT NULL DEFAULT 0,
  output_tokens   INTEGER NOT NULL DEFAULT 0,
  cost_micros     INTEGER NOT NULL DEFAULT 0,
  agent_id        INTEGER,
  provider_id     TEXT,
  error_code      TEXT,
  error_message   TEXT,
  received_at     INTEGER NOT NULL,
  provider_key_id INTEGER,
  UNIQUE(agent_id, event_id)
);

INSERT INTO analytics_events_new
SELECT id, event_id, timestamp_ms, event_type, model, provider,
       input_tokens, output_tokens, cost_micros, agent_id, provider_id,
       error_code, error_message, received_at, provider_key_id
FROM analytics_events;

DROP TABLE analytics_events;
ALTER TABLE analytics_events_new RENAME TO analytics_events;

-- Restore indexes
CREATE INDEX IF NOT EXISTS idx_analytics_events_timestamp
  ON analytics_events(timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_analytics_events_agent_id
  ON analytics_events(agent_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_provider
  ON analytics_events(provider);
CREATE INDEX IF NOT EXISTS idx_analytics_events_provider_id
  ON analytics_events(provider_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_event_type
  ON analytics_events(event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_events_model
  ON analytics_events(model);
CREATE INDEX IF NOT EXISTS idx_analytics_events_agent_timestamp
  ON analytics_events(agent_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_analytics_events_provider_timestamp
  ON analytics_events(provider, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_analytics_events_provider_key
  ON analytics_events(provider_key_id);

PRAGMA foreign_keys = ON;

-- Guard table
CREATE TABLE IF NOT EXISTS _migration_027_completed ( id INTEGER PRIMARY KEY );
INSERT INTO _migration_027_completed (id) VALUES (1);
