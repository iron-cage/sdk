# iron_safety - Specification

**Module:** iron_safety
**Layer:** 3 (Feature)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

PII detection and output redaction for compliance. Prevents GDPR violations by detecting and removing personally identifiable information (email, phone, SSN) from agent outputs before logging.

---

## Scope

**In Scope:**
- Email detection (regex pattern matching)
- Phone number detection (US format)
- SSN detection (US format)
- Automatic redaction with placeholders ([EMAIL_REDACTED])
- SHA256 hash of redacted values for audit trail

**Out of Scope:**
- Credit card detection (full platform feature)
- International formats (full platform feature)
- ML-based PII detection (full platform feature)
- Prompt injection detection (separate security domain)
- Content filtering (separate safety domain)

---

## Dependencies

**Required Modules:**
- iron_types - Foundation types
- iron_state - Audit logging
- iron_telemetry - Logging

**Required External:**
- regex - Pattern matching

**Optional:**
- None

---

## Core Concepts

**Key Components:**
- **PII Detector:** Regex-based scanner for email, phone, SSN
- **Redactor:** Replaces PII with placeholder text
- **Hash Generator:** SHA256 for audit trail
- **Audit Logger:** Records PII detections to iron_state

---

## Integration Points

**Used by:**
- iron_runtime - Validates LLM outputs before logging

**Uses:**
- iron_state - Persists PII detection audit logs

---

*For detailed regex patterns, see spec/-archived_detailed_spec.md*
*For security model, see docs/security/001_threat_model.md*
