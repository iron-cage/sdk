CREATE INDEX IF NOT EXISTS idx_ai_provider_keys_user_provider
ON ai_provider_keys(user_id, provider);

CREATE TABLE IF NOT EXISTS _migration_026_completed (completed_at TEXT DEFAULT CURRENT_TIMESTAMP);
