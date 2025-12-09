# Integration

**Purpose:** Conceptual overview of external system integration patterns.

---

## Directory Responsibilities

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| **llm_providers.md** | Document LLM provider integration patterns | Provider question → Integration approach | OpenAI, Anthropic, Azure, Google, local models, unified API, fallback chains | NOT implementation (→ module/iron_lang/spec.md), NOT capability overview (→ docs/capabilities/llm_access_control.md), NOT other integrations (→ secret_backends.md, identity_providers.md, observability_backends.md) |
| **secret_backends.md** | Explain secret storage backend options | Secret storage question → Backend comparison | HashiCorp Vault, AWS Secrets Manager, GCP Secret Manager, Azure Key Vault, local file, adapter pattern | NOT implementation (→ module/iron_secrets/spec.md), NOT credential flow (→ docs/security/credential_flow.md), NOT other integrations (→ llm_providers.md, identity_providers.md, observability_backends.md) |
| **identity_providers.md** | Document authentication provider integrations | Identity question → Provider options | SSO via Okta, Auth0, Azure AD, Google Workspace, OIDC/SAML protocols, enterprise delegation | NOT implementation (→ module specifications), NOT secret storage (→ secret_backends.md), NOT other integrations (→ llm_providers.md, observability_backends.md) |
| **observability_backends.md** | Define observability platform integrations | Observability backend question → Platform options | OpenTelemetry export, Datadog, New Relic, Grafana, Prometheus, CloudWatch, custom OTLP | NOT implementation (→ module/iron_telemetry/spec.md), NOT capability overview (→ docs/capabilities/observability.md), NOT other integrations (→ llm_providers.md, secret_backends.md, identity_providers.md) |

---

## The Four Integration Concepts

| # | Concept | Core Idea |
|---|---------|-----------|
| 1 | [LLM Providers](llm_providers.md) | Multi-provider access |
| 2 | [Secret Backends](secret_backends.md) | Enterprise secrets |
| 3 | [Identity Providers](identity_providers.md) | SSO and authentication |
| 4 | [Observability Backends](observability_backends.md) | Metrics and tracing export |

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
