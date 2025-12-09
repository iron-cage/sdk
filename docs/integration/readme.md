# Integration

**Purpose:** Conceptual overview of external system integration patterns.

---

## Directory Responsibilities

| ID | Entity | Responsibility | Input → Output | Scope | Out of Scope |
|----|--------|----------------|----------------|-------|--------------|
| 001 | **001_llm_providers.md** | Document LLM provider integration patterns | Provider question → Integration approach | OpenAI, Anthropic, Azure, Google, local models, unified API, fallback chains | NOT implementation (→ module/iron_lang/spec.md), NOT capability overview (→ docs/capabilities/), NOT other integrations (→ 002, 003, 004) |
| 002 | **002_secret_backends.md** | Explain secret storage backend options | Secret storage question → Backend comparison | HashiCorp Vault, AWS Secrets Manager, GCP Secret Manager, Azure Key Vault, local file, adapter pattern | NOT implementation (→ module/iron_secrets/spec.md), NOT credential flow (→ docs/security/credential_flow.md), NOT other integrations (→ 001, 003, 004) |
| 003 | **003_identity_providers.md** | Document authentication provider integrations | Identity question → Provider options | SSO via Okta, Auth0, Azure AD, Google Workspace, OIDC/SAML protocols, enterprise delegation | NOT implementation (→ module specifications), NOT secret storage (→ 002), NOT other integrations (→ 001, 004) |
| 004 | **004_observability_backends.md** | Define observability platform integrations | Observability backend question → Platform options | OpenTelemetry export, Datadog, New Relic, Grafana, Prometheus, CloudWatch, custom OTLP | NOT implementation (→ module/iron_telemetry/spec.md), NOT capability overview (→ docs/capabilities/), NOT other integrations (→ 001, 002, 003) |

---

## Integration Collection

| ID | Name | Purpose |
|----|------|---------|
| 001 | [LLM Providers](001_llm_providers.md) | Multi-provider LLM access |
| 002 | [Secret Backends](002_secret_backends.md) | Enterprise secrets management |
| 003 | [Identity Providers](003_identity_providers.md) | SSO and authentication |
| 004 | [Observability Backends](004_observability_backends.md) | Metrics and tracing export |

## Integration Principles

1. **Don't rebuild:** Integrate with existing enterprise systems
2. **Abstraction layer:** Unified API over multiple backends
3. **Bring your own:** Support customer's existing tools
4. **Standards-based:** OpenTelemetry, OIDC, etc.

## Provider Landscape

```
+-------------------------------------------------+
|                 Iron Cage                        |
+----------+----------+----------+----------------+
| LLM      | Secrets  | Identity | Observability  |
+----------+----------+----------+----------------+
| OpenAI   | Vault    | Okta     | Datadog        |
| Anthropic| AWS SM   | Auth0    | New Relic      |
| Azure    | GCP SM   | Azure AD | Grafana        |
| Google   |          |          | Custom         |
+----------+----------+----------+----------------+
```

**Note:** This directory follows Design Collections format with NNN_ numbered instances (001-004) per documentation.rulebook.md § integration/ standards.
