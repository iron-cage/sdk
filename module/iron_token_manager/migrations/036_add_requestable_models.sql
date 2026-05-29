-- Task 029 Slide 12: gated models a member may request access to, set via the
-- `requestable:` usage-policy directive. Stored as a comma-separated list in a
-- single nullable column (model ids never contain commas), mirroring the
-- existing nullable `default_model` column. NULL means "no requestable models".
BEGIN;

ALTER TABLE workspace_policy ADD COLUMN requestable_models TEXT;

CREATE TABLE IF NOT EXISTS _migration_036_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_036_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);

COMMIT;
