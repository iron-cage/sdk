BEGIN;

-- SQLite cannot add CHECK constraints to existing columns via ALTER TABLE.
-- Use a trigger to enforce spending_used_microdollars >= 0 at the storage layer.
CREATE TRIGGER IF NOT EXISTS trg_spending_non_negative
  BEFORE UPDATE OF spending_used_microdollars ON ai_provider_keys
  WHEN NEW.spending_used_microdollars < 0
BEGIN
  SELECT RAISE(ABORT, 'spending_used_microdollars cannot be negative');
END;

CREATE TABLE IF NOT EXISTS _migration_026_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_026_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);

COMMIT;
