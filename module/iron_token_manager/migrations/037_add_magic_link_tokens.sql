-- Task 029 Slide 1: magic-link sign-in is the only authentication surface. The
-- admin enters an email and receives a one-time login link; clicking it returns
-- them authenticated. token_hash stores SHA-256 of the raw token (the raw value
-- is surfaced once at creation, as with invite_links); used_at is stamped on
-- redemption so a link cannot be replayed. Email delivery is out of scope, so the
-- send endpoint returns the link for copy rather than mailing it.
BEGIN;

CREATE TABLE IF NOT EXISTS magic_link_tokens (
  token_hash TEXT PRIMARY KEY,
  email      TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  used_at    INTEGER
);

CREATE INDEX IF NOT EXISTS idx_magic_link_tokens_email ON magic_link_tokens (email);

CREATE TABLE IF NOT EXISTS _migration_037_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_037_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);

COMMIT;
