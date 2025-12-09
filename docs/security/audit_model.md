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

*Related: [credential_flow.md](credential_flow.md) | [threat_model.md](threat_model.md)*
