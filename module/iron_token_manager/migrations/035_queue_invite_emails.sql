-- Task 029 Slide 11: queue invite emails ("queued, not yet invited") parsed from
-- the FreeForm providers paste. A queued seat has a NULL invite_link_id and a
-- non-NULL email; an accepted seat has a non-NULL invite_link_id (and user_id).
--
-- SQLite cannot drop a NOT NULL constraint via ALTER COLUMN, so invite_link_id is
-- made nullable through the official table-recreate procedure. foreign_keys is
-- toggled OFF outside the transaction (PRAGMA is a no-op inside one) per the
-- SQLite "Making Other Kinds Of Table Schema Changes" guidance.
PRAGMA foreign_keys=OFF;

BEGIN;

CREATE TABLE invite_seats_new (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  invite_link_id TEXT REFERENCES invite_links(id),
  user_id        TEXT REFERENCES users(id),
  email          TEXT,
  accepted_at    INTEGER,
  approved_at    INTEGER,
  approved_by    TEXT REFERENCES users(id),
  -- A seat is either claimed against a link or queued by email; never empty.
  CHECK (invite_link_id IS NOT NULL OR email IS NOT NULL)
);

INSERT INTO invite_seats_new (id, invite_link_id, user_id, accepted_at, approved_at, approved_by)
  SELECT id, invite_link_id, user_id, accepted_at, approved_at, approved_by FROM invite_seats;

DROP TABLE invite_seats;
ALTER TABLE invite_seats_new RENAME TO invite_seats;

CREATE INDEX IF NOT EXISTS idx_invite_seats_link ON invite_seats (invite_link_id);
CREATE INDEX IF NOT EXISTS idx_invite_seats_user ON invite_seats (user_id);
CREATE INDEX IF NOT EXISTS idx_invite_seats_email ON invite_seats (email);

CREATE TABLE IF NOT EXISTS _migration_035_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_035_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);

COMMIT;

PRAGMA foreign_keys=ON;
