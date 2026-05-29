-- Task 029 Slides 2-4, 15-17: the Complete Your Registration form captures First
-- Name, Last Name, and Birthday for both admin and member onboarding. The users
-- table previously held only a single optional `name`; these three nullable
-- columns back the structured registration fields. birthday is stored as TEXT in
-- ISO-8601 date-only form (YYYY-MM-DD): no time component, no timezone. All three
-- are nullable so existing rows and seed users remain valid (reversible ADD
-- COLUMN, no table recreate; mirrors migration 036).
BEGIN;

ALTER TABLE users ADD COLUMN first_name TEXT;
ALTER TABLE users ADD COLUMN last_name TEXT;
ALTER TABLE users ADD COLUMN birthday TEXT;

CREATE TABLE IF NOT EXISTS _migration_038_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_038_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);

COMMIT;
