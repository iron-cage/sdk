# Audit Model

**Purpose:** What gets logged for compliance and debugging.

---

## User Need

Prove compliance, debug incidents, attribute actions to agents.

## Core Idea

**Immutable, comprehensive audit trail for all AI actions:**

## What Gets Logged

| Event | Data Captured | Retention |
|-------|--------------|-----------|
| LLM Call | Prompt (hashed), response (hashed), tokens, cost | 90 days |
| Tool Invocation | Tool name, parameters, result, duration | 90 days |
| Credential Access | Secret name, agent ID, timestamp | 1 year |
| Safety Violation | Input/output, violation type, action taken | 1 year |
| **User Management** | Action, user_id, admin_id, old_value, new_value, reason, timestamp | **Permanent** |

### User Management Audit Log

All administrative user operations are logged to an **append-only** `user_audit_log` table:

**Logged Actions:**
- User creation (captures initial role, email)
- User suspension/activation (captures reason if provided)
- User deletion (soft delete, captures deleted_by admin)
- Role changes (captures old_role → new_role)
- Password resets (captures force_change flag)

**Log Structure:**
```sql
CREATE TABLE user_audit_log
(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  admin_id INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  action TEXT NOT NULL,
  old_value TEXT,
  new_value TEXT,
  reason TEXT,
  created_at INTEGER NOT NULL
);
```

**Immutability Guarantee:**
- Foreign key constraints with `ON DELETE RESTRICT` prevent audit log deletion
- User accounts cannot be deleted if they appear in audit log (as user or admin)
- Soft delete pattern preserves user records indefinitely
- Append-only design (no UPDATE or DELETE operations on audit log)

**Example Audit Entries:**
```json
{
  "action": "user_created",
  "user_id": 1001,
  "admin_id": 1,
  "new_value": "role=user,email=john@example.com",
  "created_at": 1702345678
}

{
  "action": "user_suspended",
  "user_id": 1001,
  "admin_id": 1,
  "old_value": "is_active=true",
  "new_value": "is_active=false",
  "reason": "Violates acceptable use policy",
  "created_at": 1702456789
}

{
  "action": "role_changed",
  "user_id": 1001,
  "admin_id": 1,
  "old_value": "user",
  "new_value": "admin",
  "created_at": 1702567890
}
```

## Log Structure

```json
{
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "agent_id": "agent-123",
  "event_type": "llm_call",
  "data": { "model": "gpt-4", "tokens": 1240, "cost_usd": 0.015 },
  "safety": { "input_clean": true, "output_redacted": false }
}
```

## Compliance Support

| Standard | Supported Features |
|----------|-------------------|
| SOC 2 | Audit logs, access control, encryption |
| GDPR | PII detection, redaction, data export |
| HIPAA | PHI logging, access audit, encryption |

## Immutability

- Logs written to append-only storage
- Cryptographic checksums for tamper detection
- Archived to S3 with versioning

---

*Related: [003_credential_flow.md](003_credential_flow.md) | [001_threat_model.md](001_threat_model.md)*
