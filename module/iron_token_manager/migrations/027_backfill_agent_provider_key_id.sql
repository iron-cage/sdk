-- Backfill provider_key_id for agents that have no key assigned.
-- For each agent with owner_id set, link to any one active provider key
-- owned by the same user (prefer most recently created).
-- Agents whose owner has no key at all remain NULL (handled gracefully
-- by the handshake as NO_PROVIDER_ASSIGNED).
UPDATE agents
SET provider_key_id = (
    SELECT id
    FROM ai_provider_keys
    WHERE user_id = agents.owner_id
      AND is_enabled = 1
    ORDER BY created_at DESC
    LIMIT 1
)
WHERE provider_key_id IS NULL
  AND owner_id IS NOT NULL
  AND EXISTS (
    SELECT 1 FROM ai_provider_keys
    WHERE user_id = agents.owner_id
      AND is_enabled = 1
  );

CREATE TABLE IF NOT EXISTS _migration_027_completed (done INTEGER);
