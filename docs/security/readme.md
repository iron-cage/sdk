# Security

**Purpose:** Conceptual overview of Iron Cage security model.

---

## Directory Responsibilities

| ID | Entity | Responsibility |
|----|--------|----------------|
| 001 | **001_threat_model.md** | Document security threats and mitigations (AI-specific threats, traditional threats, mitigation strategies) |
| 002 | **002_isolation_layers.md** | Explain defense-in-depth isolation architecture (four layers: network, filesystem, syscall, process) |
| 003 | **003_credential_flow.md** | Describe credential security pattern (two-token system, scoped access, vault integration, just-in-time injection) |
| 004 | **004_audit_model.md** | Define audit logging model for compliance (event types, data captured, retention policies, immutable logs) |

---

## The Four Security Concepts

| # | Concept | Core Idea |
|---|---------|-----------|
| 1 | [Threat Model](001_threat_model.md) | What we defend against |
| 2 | [Isolation Layers](002_isolation_layers.md) | Defense in depth |
| 3 | [Credential Flow](003_credential_flow.md) | How secrets are protected |
| 4 | [Audit Model](004_audit_model.md) | Compliance and logging |

## Security Principles

1. **Fail-safe defaults:** Block on uncertainty
2. **Defense in depth:** Multiple layers, no single point
3. **Least privilege:** Minimal access by default
4. **Audit everything:** Immutable logs for compliance

## Trust Boundaries

```
+---------------------------------------------+
| UNTRUSTED: User prompts, agent outputs      |
+---------------------------------------------+
| VALIDATED: After input firewall             |
+---------------------------------------------+
| TRUSTED: Internal services, infrastructure  |
+---------------------------------------------+
```

*For capability details, see [Capability 004: AI Safety Guardrails](../capabilities/004_ai_safety_guardrails.md)*
