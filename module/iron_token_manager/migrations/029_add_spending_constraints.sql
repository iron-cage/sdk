BEGIN;

-- Note: the non-negative trigger on spending_used_microdollars is already
-- created in migration 028 (trg_spending_used_non_negative). The duplicate
-- trigger (trg_spending_non_negative) that was here has been removed to
-- avoid confusion.

CREATE TABLE IF NOT EXISTS _migration_029_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_029_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);

COMMIT;
