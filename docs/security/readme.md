# Security

**Purpose:** Conceptual overview of Iron Cage security model.

---

## Directory Responsibilities

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| **threat_model.md** | Document security threats and mitigations | Attack scenario → Threat analysis | AI-specific threats (prompt injection, data exfiltration, credential theft, runaway costs), traditional threats (unauthorized access, data breach), mitigation strategies | NOT isolation design (→ isolation_layers.md), NOT credential handling (→ credential_flow.md), NOT audit logging (→ audit_model.md) |
| **isolation_layers.md** | Explain defense-in-depth isolation architecture | Isolation question → Layer model | Four isolation layers (network, filesystem, syscall, process), layer independence, container/seccomp technology | NOT threat enumeration (→ threat_model.md), NOT credential handling (→ credential_flow.md), NOT implementation (→ module/iron_sandbox/) |
| **credential_flow.md** | Describe just-in-time credential injection pattern | Credential access question → Flow diagram | Scoped credential access, vault integration, rate limiting, agent authorization, JIT injection | NOT threat model (→ threat_model.md), NOT audit details (→ audit_model.md), NOT implementation (→ module/iron_secrets/spec.md) |
| **audit_model.md** | Define audit logging model for compliance | Audit question → Logging specification | Event types (LLM calls, tool invocations, credential access, safety violations), data captured, retention policies, log structure | NOT threat analysis (→ threat_model.md), NOT credential handling (→ credential_flow.md), NOT implementation (→ module/iron_telemetry/spec.md) |

---

## The Four Security Concepts

| # | Concept | Core Idea |
|---|---------|-----------|
| 1 | [Threat Model](threat_model.md) | What we defend against |
| 2 | [Isolation Layers](isolation_layers.md) | Defense in depth |
| 3 | [Credential Flow](credential_flow.md) | How secrets are protected |
| 4 | [Audit Model](audit_model.md) | Compliance and logging |

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

*For capability details, see [capabilities/ai_safety_guardrails.md](../capabilities/ai_safety_guardrails.md)*
