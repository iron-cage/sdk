-- Migration 029: Add alias column to provider keys
ALTER TABLE ai_provider_keys ADD COLUMN alias TEXT;

CREATE TABLE IF NOT EXISTS _migration_029_completed (id INTEGER PRIMARY KEY);
