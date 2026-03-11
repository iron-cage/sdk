-- Migration 028: Enforce non-negative invariant on spending_used_microdollars
--
-- SQLite does not support adding CHECK constraints to existing columns via
-- ALTER TABLE.  A BEFORE UPDATE trigger provides equivalent protection:
-- any UPDATE that would set spending_used_microdollars < 0 is aborted.

CREATE TRIGGER IF NOT EXISTS trg_spending_used_non_negative
BEFORE UPDATE OF spending_used_microdollars ON ai_provider_keys
FOR EACH ROW
WHEN NEW.spending_used_microdollars < 0
BEGIN
  SELECT RAISE(ABORT, 'spending_used_microdollars cannot be negative');
END;

CREATE TABLE IF NOT EXISTS _migration_028_completed ( applied_at INTEGER NOT NULL );
INSERT INTO _migration_028_completed ( applied_at ) VALUES ( strftime( '%s', 'now' ) * 1000 );
