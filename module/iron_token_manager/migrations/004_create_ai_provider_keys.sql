-- Migration 004: Create AI provider keys table
--
-- This migration adds tables for managing AI provider API keys (OpenAI, Anthropic)
-- with encrypted storage and per-project assignment.
--
-- NOTE: This migration is conditionally executed by Rust code.
-- The Rust code checks if _migration_004_completed table exists before running this.

-- AI provider keys table for storing encrypted API keys
CREATE TABLE IF NOT EXISTS ai_provider_keys
(
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Provider type (openai, anthropic)
  provider TEXT NOT NULL CHECK ( provider IN ( 'openai', 'anthropic' ) ),

  -- Encrypted API key (AES-256-GCM encrypted, base64 encoded)
  encrypted_api_key TEXT NOT NULL,

  -- Encryption nonce (12 bytes, base64 encoded)
  encryption_nonce TEXT NOT NULL,

  -- Optional custom base URL for proxy/self-hosted endpoints
  base_url TEXT CHECK ( base_url IS NULL OR LENGTH( base_url ) <= 2000 ),

  models TEXT CHECK ( models IS NULL OR LENGTH( models ) <= 2000 ),

  -- Human-friendly description
  description TEXT CHECK ( description IS NULL OR LENGTH( description ) <= 500 ),

  -- Enabled/disabled status
  is_enabled INTEGER NOT NULL DEFAULT 1,

  -- Timestamps (milliseconds since epoch)
  created_at INTEGER NOT NULL,
  last_used_at INTEGER,

  -- Balance tracking (cents)
  balance_cents INTEGER,
  balance_updated_at INTEGER,

  preview TEXT CHECK ( preview IS NULL OR LENGTH( preview ) <= 10 ),

  -- Owner
  user_id TEXT NOT NULL CHECK ( LENGTH( user_id ) > 0 AND LENGTH( user_id ) <= 500 )
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_ai_provider_keys_user_id ON ai_provider_keys( user_id );
CREATE INDEX IF NOT EXISTS idx_ai_provider_keys_provider ON ai_provider_keys( provider );
CREATE INDEX IF NOT EXISTS idx_ai_provider_keys_is_enabled ON ai_provider_keys( is_enabled );

-- Project to provider key assignments (many-to-many)
CREATE TABLE IF NOT EXISTS project_provider_key_assignments
(
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Project identifier
  project_id TEXT NOT NULL CHECK ( LENGTH( project_id ) > 0 AND LENGTH( project_id ) <= 500 ),

  -- Foreign key to ai_provider_keys
  provider_key_id INTEGER NOT NULL REFERENCES ai_provider_keys( id ) ON DELETE CASCADE,

  -- When the assignment was made (milliseconds since epoch)
  assigned_at INTEGER NOT NULL,

  -- Unique constraint: one key per project per provider
  UNIQUE ( project_id, provider_key_id )
);

-- Indexes for project lookups
CREATE INDEX IF NOT EXISTS idx_project_key_assignments_project_id ON project_provider_key_assignments( project_id );
CREATE INDEX IF NOT EXISTS idx_project_key_assignments_key_id ON project_provider_key_assignments( provider_key_id );

-- Create guard table to mark migration as completed
CREATE TABLE IF NOT EXISTS _migration_004_completed ( applied_at INTEGER NOT NULL );
INSERT INTO _migration_004_completed ( applied_at ) VALUES ( strftime( '%s', 'now' ) * 1000 );
