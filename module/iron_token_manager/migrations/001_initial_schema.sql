-- Migration 001: Initial token management schema
--
-- Five tables per specification:
-- 1. api_tokens - Token metadata and hashed values
-- 2. token_usage - Usage tracking per token/provider
-- 3. usage_limits - Hard limits per user/project
-- 4. api_call_traces - Detailed call traces for debugging
-- 5. audit_log - Audit trail for compliance

-- Table 1: API Tokens
-- Stores generated tokens (hashed), associated user/project, and metadata
CREATE TABLE IF NOT EXISTS api_tokens
(
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Token hash (SHA-256, never store plaintext)
  token_hash TEXT NOT NULL UNIQUE,

  -- User/Project association
  user_id TEXT NOT NULL,
  project_id TEXT,  -- nullable for user-level tokens

  -- Token metadata
  name TEXT,  -- human-friendly name
  scopes TEXT,  -- JSON array of allowed scopes
  is_active BOOLEAN NOT NULL DEFAULT 1,

  -- Timestamps
  created_at INTEGER NOT NULL,  -- milliseconds since epoch
  last_used_at INTEGER,  -- nullable
  expires_at INTEGER,  -- nullable (NULL = never expires)

  -- Indexes for fast lookups
  CONSTRAINT token_hash_unique UNIQUE( token_hash )
);

CREATE INDEX IF NOT EXISTS idx_api_tokens_user_id ON api_tokens( user_id );
CREATE INDEX IF NOT EXISTS idx_api_tokens_project_id ON api_tokens( project_id );
CREATE INDEX IF NOT EXISTS idx_api_tokens_is_active ON api_tokens( is_active );

-- Table 2: Token Usage
-- Tracks usage per token/provider (tokens consumed, requests made, costs)
CREATE TABLE IF NOT EXISTS token_usage
(
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Foreign key to api_tokens
  token_id INTEGER NOT NULL,

  -- Provider and model info
  provider TEXT NOT NULL,  -- "openai", "anthropic", "gemini"
  model TEXT NOT NULL,  -- "gpt-4", "claude-sonnet-4-5-20250929", etc.

  -- Usage metrics
  input_tokens INTEGER NOT NULL DEFAULT 0,
  output_tokens INTEGER NOT NULL DEFAULT 0,
  total_tokens INTEGER NOT NULL DEFAULT 0,
  requests_count INTEGER NOT NULL DEFAULT 1,

  -- Cost tracking (in USD cents to avoid floating point)
  cost_cents INTEGER NOT NULL DEFAULT 0,

  -- Timestamp (when usage occurred)
  recorded_at INTEGER NOT NULL,  -- milliseconds since epoch

  -- Foreign key constraint
  FOREIGN KEY( token_id ) REFERENCES api_tokens( id ) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_token_usage_token_id ON token_usage( token_id );
CREATE INDEX IF NOT EXISTS idx_token_usage_provider ON token_usage( provider );
CREATE INDEX IF NOT EXISTS idx_token_usage_recorded_at ON token_usage( recorded_at );

-- Table 3: Usage Limits
-- Hard limits per user/project (token quotas, request rates, cost caps)
CREATE TABLE IF NOT EXISTS usage_limits
(
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- User/Project association (matches api_tokens)
  user_id TEXT NOT NULL,
  project_id TEXT,  -- nullable for user-level limits

  -- Limit types and values
  max_tokens_per_day INTEGER,  -- nullable = no limit
  max_requests_per_minute INTEGER,  -- nullable = no limit
  max_cost_cents_per_month INTEGER,  -- nullable = no limit

  -- Current usage counters (reset periodically)
  current_tokens_today INTEGER NOT NULL DEFAULT 0,
  current_requests_this_minute INTEGER NOT NULL DEFAULT 0,
  current_cost_cents_this_month INTEGER NOT NULL DEFAULT 0,

  -- Reset timestamps
  tokens_reset_at INTEGER,  -- last daily reset
  requests_reset_at INTEGER,  -- last minute reset
  cost_reset_at INTEGER,  -- last monthly reset

  -- Timestamps
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,

  -- Unique constraint per user/project
  CONSTRAINT user_project_unique UNIQUE( user_id, project_id )
);

CREATE INDEX IF NOT EXISTS idx_usage_limits_user_id ON usage_limits( user_id );
CREATE INDEX IF NOT EXISTS idx_usage_limits_project_id ON usage_limits( project_id );

-- Table 4: API Call Traces
-- Detailed call traces for debugging and analytics
CREATE TABLE IF NOT EXISTS api_call_traces
(
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Foreign key to api_tokens
  token_id INTEGER NOT NULL,

  -- Request details
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  endpoint TEXT NOT NULL,  -- "/v1/chat/completions", etc.

  -- Request/response metadata
  request_payload TEXT,  -- JSON (nullable to save space)
  response_status INTEGER,  -- HTTP status code
  response_payload TEXT,  -- JSON (nullable to save space)

  -- Timing metrics
  duration_ms INTEGER,  -- request duration

  -- Usage metrics (denormalized from response)
  input_tokens INTEGER,
  output_tokens INTEGER,
  total_tokens INTEGER,
  cost_cents INTEGER,

  -- Timestamp
  traced_at INTEGER NOT NULL,  -- milliseconds since epoch

  -- Foreign key constraint
  FOREIGN KEY( token_id ) REFERENCES api_tokens( id ) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_api_call_traces_token_id ON api_call_traces( token_id );
CREATE INDEX IF NOT EXISTS idx_api_call_traces_provider ON api_call_traces( provider );
CREATE INDEX IF NOT EXISTS idx_api_call_traces_traced_at ON api_call_traces( traced_at );

-- Table 5: Audit Log
-- Audit trail for compliance (token creation/deletion/updates, limit changes)
CREATE TABLE IF NOT EXISTS audit_log
(
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Entity type and ID
  entity_type TEXT NOT NULL,  -- "token", "limit", "usage"
  entity_id INTEGER NOT NULL,

  -- Action performed
  action TEXT NOT NULL,  -- "created", "updated", "deleted", "activated", "deactivated"

  -- Actor (who performed the action)
  actor_user_id TEXT NOT NULL,  -- user who performed action

  -- Change details (JSON)
  changes TEXT,  -- JSON object with before/after values

  -- Timestamp
  logged_at INTEGER NOT NULL,  -- milliseconds since epoch

  -- Metadata
  ip_address TEXT,  -- optional
  user_agent TEXT  -- optional
);

CREATE INDEX IF NOT EXISTS idx_audit_log_entity_type ON audit_log( entity_type );
CREATE INDEX IF NOT EXISTS idx_audit_log_entity_id ON audit_log( entity_id );
CREATE INDEX IF NOT EXISTS idx_audit_log_actor_user_id ON audit_log( actor_user_id );
CREATE INDEX IF NOT EXISTS idx_audit_log_logged_at ON audit_log( logged_at );
