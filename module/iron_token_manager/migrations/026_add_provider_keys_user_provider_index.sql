CREATE INDEX IF NOT EXISTS idx_ai_provider_keys_user_provider
ON ai_provider_keys(user_id, provider);
