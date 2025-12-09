# Iron Control Store - Specification

**Module:** iron_control_store
**Type:** Spec-only (no implementation)
**Purpose:** PostgreSQL schema for production Control Panel
**Status:** Planning

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. This is a spec-only module - implementation planned for production phase. Schema definitions document intent, not enforce exact structure.

---

## Overview

PostgreSQL schema for Iron Cage Control Panel in production deployment mode. This is separate from iron_state (SQLite, local agent execution) and handles centralized cloud data.

**Key Difference from iron_state:**
- iron_state: SQLite, local per-machine, agent execution state
- iron_control_store: PostgreSQL, centralized cloud, user/token/secret data

---

## Schema Tables

### 1. users

Stores user accounts for Control Panel access.

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| id | UUID | PRIMARY KEY | Unique user ID |
| email | TEXT | UNIQUE, NOT NULL | User email |
| created_at | TIMESTAMP | NOT NULL | Account creation |
| last_login | TIMESTAMP | NULL | Last login time |

### 2. tokens

Stores JWT tokens for API authentication.

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| id | UUID | PRIMARY KEY | Token ID |
| user_id | UUID | FOREIGN KEY (users.id) | Owner |
| name | TEXT | NOT NULL | Human-readable name |
| hash | TEXT | NOT NULL | SHA256 hash of token |
| created_at | TIMESTAMP | NOT NULL | Creation time |
| expires_at | TIMESTAMP | NULL | Expiry (NULL = no expiry) |
| last_used | TIMESTAMP | NULL | Last usage time |

### 3. secrets

Stores encrypted credentials (LLM API keys, etc.).

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| id | UUID | PRIMARY KEY | Secret ID |
| user_id | UUID | FOREIGN KEY (users.id) | Owner |
| name | TEXT | NOT NULL | Secret identifier |
| encrypted_value | BYTEA | NOT NULL | AES-GCM encrypted |
| created_at | TIMESTAMP | NOT NULL | Creation time |
| rotated_at | TIMESTAMP | NULL | Last rotation |

### 4. telemetry

Aggregated metrics from distributed agents.

| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| id | UUID | PRIMARY KEY | Event ID |
| user_id | UUID | FOREIGN KEY (users.id) | Agent owner |
| agent_id | TEXT | NOT NULL | Agent identifier |
| event_type | TEXT | NOT NULL | llm_call, tool_invoke, etc. |
| tokens | INTEGER | NULL | Token count |
| cost_usd | NUMERIC(10,6) | NULL | Cost in USD |
| timestamp | TIMESTAMP | NOT NULL | Event time |
| data | JSONB | NULL | Additional event data |

---

## Indexes

```sql
CREATE INDEX idx_tokens_user_id ON tokens(user_id);
CREATE INDEX idx_tokens_hash ON tokens(hash);
CREATE INDEX idx_secrets_user_id ON secrets(user_id);
CREATE INDEX idx_telemetry_user_id ON telemetry(user_id);
CREATE INDEX idx_telemetry_timestamp ON telemetry(timestamp DESC);
```

---

## Implementation Plan

**Phase 1 (Q1 2026):**
- Create sqlx migrations in `module/iron_control_store/migrations/`
- Implement schema types as Rust structs
- Add query builders for common operations

**Phase 2 (Q2 2026):**
- Connection pooling
- Transaction helpers
- Performance optimization

---

*This is a spec-only module. Implementation will begin in production deployment phase.*
