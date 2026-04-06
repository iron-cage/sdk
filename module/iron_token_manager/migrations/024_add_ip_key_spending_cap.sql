-- Migration 024: Add per-IP-key spending cap
--
-- Adds spending cap and usage tracking columns to ai_provider_keys table.
-- spending_cap_microdollars = NULL means unlimited (no cap).
-- spending_used_microdollars tracks cumulative spending, incremented atomically
-- after each successful LLM request through the server proxy.
--
-- NOTE: This migration is conditionally executed by Rust code.
-- The Rust code checks if _migration_024_completed table exists before running this.

BEGIN;

-- Add spending cap column (NULL = unlimited)
ALTER TABLE ai_provider_keys ADD COLUMN spending_cap_microdollars INTEGER;

-- Add spending used counter (default 0)
-- //qqq: [Medium] no CHECK (spending_used_microdollars >= 0) constraint — negative values possible via buggy adjust_spending
-- aaa: Addressed by migration 028 (BEFORE UPDATE trigger) and Rust-layer MAX(0, ...) clamp in refund path.
ALTER TABLE ai_provider_keys ADD COLUMN spending_used_microdollars INTEGER NOT NULL DEFAULT 0;

-- Create guard table to mark migration as completed
CREATE TABLE IF NOT EXISTS _migration_024_completed ( applied_at INTEGER NOT NULL );
INSERT INTO _migration_024_completed ( applied_at ) VALUES ( strftime( '%s', 'now' ) * 1000 );

COMMIT;
