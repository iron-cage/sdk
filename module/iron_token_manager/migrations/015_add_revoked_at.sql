-- Migration 015: Add revoked_at timestamp to distinguish revoked vs rotated tokens
--
-- WHY: Token can be inactive for two reasons:
--   1. Explicitly revoked (should return 409 on second revoke)
--   2. Rotated/deactivated (should return 404 on revoke attempt)
--
-- SOLUTION: Add revoked_at timestamp
--   - Set when token is revoked
--   - NULL when token is rotated or otherwise deactivated
--   - Enables distinguishing revoked (409) from rotated (404) states

-- ALTER TABLE api_tokens ADD COLUMN revoked_at INTEGER;

-- Note: Existing inactive tokens will have revoked_at = NULL
-- This is correct as we cannot retroactively determine the reason
-- New revocations will set this field

-- Create guard table to prevent re-running this migration
CREATE TABLE IF NOT EXISTS _migration_015_completed (id INTEGER PRIMARY KEY)
