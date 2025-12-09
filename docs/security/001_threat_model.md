# Threat Model

**Purpose:** What attacks Iron Cage defends against.

---

## User Need

Understand the attack surface and adversary capabilities we consider.

## Core Idea

**AI-specific threats + traditional infrastructure threats**

## AI-Specific Threats

| Threat | Description | Mitigation |
|--------|-------------|------------|
| **Prompt Injection** | Malicious input hijacks agent | Input firewall, ML classifier |
| **Data Exfiltration** | Agent leaks sensitive data | Output firewall, PII detection |
| **Credential Theft** | Agent accesses unauthorized secrets | Scoped credentials, audit |
| **Runaway Costs** | Agent burns budget | Hard limits, circuit breaker |
| **Tool Abuse** | Agent misuses authorized tools | Parameter validation, rate limits |

## Traditional Threats

| Threat | Description | Mitigation |
|--------|-------------|------------|
| **Unauthorized Access** | Attacker accesses API | Authentication, rate limiting |
| **Data Breach** | Database compromise | Encryption at rest/transit |
| **Denial of Service** | Overwhelm services | Rate limiting, auto-scaling |

## Adversary Model

**We assume:**
- Malicious prompts from external users
- Compromised agent code (supply chain)
- Network eavesdropping (mitigated by TLS)

**We do NOT assume:**
- Compromised infrastructure (out of scope)
- Insider threat with admin access (different problem)

## Defense Priority

1. **Prompt injection** - Most common, highest impact
2. **Data exfiltration** - Compliance risk
3. **Cost overrun** - Financial risk

---

*Related: [002_isolation_layers.md](002_isolation_layers.md) | [004_audit_model.md](004_audit_model.md)*
