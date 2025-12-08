# Capability 5: Credential Management - Product Specification

**Version:** 1.0.0
**Last Updated:** 2025-01-20
**Status:** Draft - Thin Component Specification
**Build Priority:** Thin Component (42/100 standalone viability - minimal wrapper around Vault)

---

### Scope

**Responsibility:** Product specification for Credential Management capability (Capability 5 of 8 - thin component, 42/100, Vault wrapper)

**In Scope:**
- Market context (secrets management $4.22B→$8.05B 14% CAGR, dominated by HashiCorp Vault/AWS/Azure)
- Strategic approach (thin wrapper around Vault, do NOT rebuild secrets storage, 1-2 months, 1 engineer)
- Problem statement (scattered secrets across 5+ locations, no rotation, no audit, compliance violations)
- Solution architecture (centralized credential service with unified LLM dashboard, 90-day rotation reminders, audit trails, dynamic secrets)
- Build recommendation (LEVERAGE VAULT, add LLM-specific features, integrate with Caps 2/8)
- Platform integration (included in $100K-300K/year, not sold separately)
- Standalone viability score (42/100 - build as thin component only)

**Out of Scope:**
- Other capabilities, strategic analysis, pilot specs, implementation details

---

## Executive Summary

This specification defines the requirements for Iron Cage's Credential Management capability - centralized secrets management for AI agents with rotation reminders, unified dashboard, and audit trails.

**Market Opportunity:** Secrets management $4.22B → $8.05B (14% CAGR), dominated by HashiCorp Vault, AWS Secrets Manager, Azure Key Vault
**Strategic Approach:** Thin wrapper around HashiCorp Vault (market leader, proven technology)
**Build Timeline:** 1-2 months, 1 engineer
**Platform Pricing:** Included in $100K-300K/year Iron Cage platform (not sold separately)

**Core Value Proposition:** Replace scattered secrets (LLM API keys in env vars, database passwords in config files, no rotation) with centralized credential service providing unified LLM credential dashboard, 90-day rotation reminders, audit trails, and dynamic secrets.

**Strategic Recommendation:** LEVERAGE VAULT (don't rebuild secrets storage). Build thin Iron Cage wrapper adding LLM-specific features (unified dashboard, rotation reminders, integration with Caps 2/8).

---

## Pilot vs Full Platform Approach

**Strategic Decision:** Pilot uses custom implementation, full platform uses Vault wrapper.

### Pilot Implementation (iron_secrets)

**Why NOT Vault for pilot:**
- **Timeline:** Vault integration adds 2-3 weeks (HSM setup, lease management, dynamic secrets)
- **Complexity:** Pilot doesnt need enterprise features (HSM, automatic rotation, audit trail exports)
- **Dependencies:** Vault requires dedicated infrastructure (server, unsealing, backups)
- **Overkill:** 28-day pilot with 10-50 secrets doesnt justify Vault overhead

**Pilot Architecture:**
- **Encryption:** AES-256-GCM (AEAD, hardware-accelerated)
- **Key Derivation:** Argon2id from environment variable (`IRON_SECRETS_MASTER_KEY`)
- **Storage:** SQLite (encrypted blobs with unique nonces/salts)
- **Access Control:** Simple RBAC (Admin, Viewer, Agent)
- **Audit Trail:** Append-only SQLite table
- **Scope:** 100-500 secrets, single-instance deployment

**Pilot Limitations (vs Vault):**
- No HSM support (master key in environment variable)
- No dynamic secrets (static secrets only)
- No lease management (secrets dont expire)
- No automatic rotation policies
- No distributed secret coordination (single-instance only)
- Manual key rotation (no `vault operator rotate`)

### Full Platform Migration (HashiCorp Vault)

**When to migrate:** After pilot success (3-6 months), when scaling to 1000+ secrets or multi-tenant.

**Migration Path:**
1. **Export from pilot:** `iron_secrets export --output secrets.json` (encrypted)
2. **Import to Vault:** `vault kv put secret/iron_cage @secrets.json`
3. **Update iron_cage code:** Replace `iron_secrets` calls with `vault` SDK calls
4. **Test migration:** Run pilot agents against Vault (verify no regressions)
5. **Deploy:** Cutover to Vault in production (zero downtime with canary deployment)

**Full Platform Architecture:**
- **Encryption:** Vault's AES-256-GCM (auto-unsealing with AWS KMS)
- **Key Management:** AWS KMS or Cloud HSM (no environment variable keys)
- **Storage:** Vault's integrated storage (Raft) or Consul backend
- **Access Control:** Vault's policy engine (path-based, fine-grained)
- **Audit Trail:** Vault's audit device (JSON logs to S3, CloudWatch, Splunk)
- **Scope:** 10K+ secrets, multi-region, multi-tenant

**Full Platform Benefits (vs Pilot):**
- HSM-backed keys (FIPS 140-2 compliance)
- Dynamic secrets (generate DB passwords on-demand)
- Lease management (secrets auto-expire)
- Automatic rotation policies (rotate every 90 days)
- Distributed secret coordination (multi-instance, multi-region)
- Zero-touch operations (`vault operator rotate` for key rotation)

### Decision Matrix

| Requirement | Pilot (iron_secrets) | Full Platform (Vault) |
|-------------|----------------------|----------------------|
| **Timeline** | 3-4 days | 2-3 weeks |
| **Secrets Count** | 100-500 | 10K+ |
| **Multi-Tenancy** | Single instance | Multi-tenant |
| **Compliance** | Basic audit trail | SOC2, HIPAA, FIPS 140-2 |
| **Key Storage** | Environment variable | AWS KMS / Cloud HSM |
| **Rotation** | Manual | Automatic policies |
| **Dynamic Secrets** | No | Yes |
| **Cost** | $0 (included) | ~$500-2000/month (Vault cluster) |

**Recommendation:** Start with iron_secrets for pilot (prove business value), migrate to Vault for production (prove technical maturity).

**Pilot Specification:** See `/home/user1/pro/lib/willbe/module/iron_secrets/spec.md` for complete pilot implementation details.

---

## 1. Product Overview

### 1.1 Problem Statement

Scattered secrets across multiple systems:

```
CURRENT STATE: Scattered Secrets
┌─────────────────────────────────────────────────────┐
│  LLM API Keys                                        │
│  - OpenAI key in .env file                          │
│  - Anthropic key in config file                     │
│  - Cohere key hardcoded in code (!!)                │
│  - No rotation (keys 2+ years old)                  │
│  - No audit trail (who accessed what)               │
│                                                      │
│  Data Source Credentials                             │
│  - Salesforce OAuth in database                     │
│  - Jira API key in environment variable             │
│  - PostgreSQL password in config file               │
│  - No expiration tracking                           │
│                                                      │
│  RISKS:                                             │
│  ❌ Secrets scattered across 5+ locations           │
│  ❌ No rotation reminders (stale credentials)       │
│  ❌ No audit trail (compliance violations)          │
│  ❌ No unified view (hard to manage)                │
└─────────────────────────────────────────────────────┘
```

### 1.2 Solution: Iron Cage Credential Management

```
IRON CAGE SOLUTION: Centralized Credentials
┌─────────────────────────────────────────────────────┐
│  CREDENTIAL SERVICE (Iron Cage)                     │
│  ┌─────────────────────────────────────────────┐   │
│  │   UNIFIED DASHBOARD                          │   │
│  │   ┌─────────────┐  ┌─────────────┐         │   │
│  │   │ LLM Keys    │  │ Data Sources│         │   │
│  │   │ - OpenAI    │  │ - Salesforce│         │   │
│  │   │ - Anthropic │  │ - Jira      │         │   │
│  │   │ - Cohere    │  │ - PostgreSQL│         │   │
│  │   │             │  │             │         │   │
│  │   │ 🔔 Rotate in│  │ 🔔 Expires  │         │   │
│  │   │    14 days  │  │    tomorrow │         │   │
│  │   └─────────────┘  └─────────────┘         │   │
│  └─────────────────────────────────────────────┘   │
│                    │                                 │
│  ┌─────────────────▼───────────────────────────┐   │
│  │        HASHICORP VAULT                       │   │
│  │  - Encrypted storage (AES-256)               │   │
│  │  - Dynamic secrets (generate on-demand)      │   │
│  │  - Lease management (TTL, renewal)           │   │
│  │  - Audit logs (who accessed what)            │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 2. Functional Requirements

### 2.1 Secret Storage & Retrieval

**Requirement:** CRUD operations for secrets via simple API.

**API:**
```rust
// src/credentials/service.rs

pub struct CredentialService
{
  vault_client: Arc< VaultClient >,
}

impl CredentialService
{
  /// Store secret
  pub async fn store_secret
  (
    &self,
    path: &str, // e.g., "llm/openai/api_key"
    value: &str,
    metadata: SecretMetadata,
  ) -> Result< () >
  {
    self.vault_client
      .write_secret( path, value, metadata )
      .await
  }

  /// Retrieve secret
  pub async fn get_secret
  (
    &self,
    path: &str,
  ) -> Result< SecretValue >
  {
    self.vault_client
      .read_secret( path )
      .await
  }

  /// Delete secret
  pub async fn delete_secret
  (
    &self,
    path: &str,
  ) -> Result< () >
  {
    self.vault_client
      .delete_secret( path )
      .await
  }

  /// List secrets (paths only, not values)
  pub async fn list_secrets
  (
    &self,
    prefix: &str, // e.g., "llm/"
  ) -> Result< Vec< String > >
  {
    self.vault_client
      .list_secrets( prefix )
      .await
  }
}

pub struct SecretMetadata
{
  pub name: String, // Human-readable name
  pub type_: SecretType,
  pub created_at: DateTime< Utc >,
  pub rotation_policy: RotationPolicy,
}

pub enum SecretType
{
  LlmApiKey { provider: String }, // OpenAI, Anthropic, etc.
  DatabasePassword { database: String },
  OAuth2Token { service: String },
  ApiKey { service: String },
}

pub struct RotationPolicy
{
  pub rotation_period: Duration, // e.g., 90 days
  pub reminder_days_before: usize, // e.g., 14 days before expiration
}
```

### 2.2 Rotation Reminders

**Requirement:** Automated reminders for credential rotation (90-day default, configurable).

**Implementation:**
```rust
// src/credentials/rotation_monitor.rs

pub struct RotationMonitor
{
  credential_store: Arc< CredentialService >,
  notifier: Arc< Notifier >, // Slack, email, webhook
}

impl RotationMonitor
{
  /// Check all secrets, send reminders for approaching expiration
  pub async fn check_rotation_reminders( &self ) -> Result< () >
  {
    // 1. Get all secrets with rotation policies
    let secrets = self.credential_store
      .list_secrets( "/" ) // Root path
      .await?;

    for secret_path in secrets
    {
      let secret = self.credential_store
        .get_secret( &secret_path )
        .await?;

      // 2. Check if rotation reminder needed
      let days_until_rotation = self.calculate_days_until_rotation( &secret )?;

      if days_until_rotation <= secret.metadata.rotation_policy.reminder_days_before
      {
        // 3. Send reminder
        self.notifier
          .send_rotation_reminder
          (
            &secret_path,
            &secret.metadata.name,
            days_until_rotation,
          )
          .await?;
      }
    }

    Ok( () )
  }

  fn calculate_days_until_rotation
  (
    &self,
    secret: &SecretValue,
  ) -> Result< usize >
  {
    let rotation_due = secret.metadata.created_at
      + secret.metadata.rotation_policy.rotation_period;

    let days_remaining = ( rotation_due - Utc::now() ).num_days();

    Ok( days_remaining.max( 0 ) as usize )
  }
}
```

**Reminder Channels:**
- Slack (recommended): "@channel LLM API key expires in 14 days"
- Email: Send to admin email list
- Webhook: POST to custom endpoint (for integration with ticketing systems)

**Cron Schedule:** Daily at 9am UTC (via Kubernetes CronJob)

### 2.3 Unified LLM Credential Dashboard

**Requirement:** Web UI showing all LLM API keys with rotation status.

**Dashboard Features:**
- List all LLM providers (OpenAI, Anthropic, Cohere, etc.)
- Show rotation status (✅ Healthy, ⚠️ Expiring soon, ❌ Expired)
- Show last rotated date, next rotation date
- Quick actions: "Rotate now", "Test connection"

**Example Dashboard:**
```
┌─────────────────────────────────────────────────────┐
│  LLM API Keys                                        │
├─────────────────────────────────────────────────────┤
│  Provider      │ Status  │ Last Rotated │ Next      │
│  ────────────  │ ──────  │ ────────────  │ ──────── │
│  OpenAI        │ ✅      │ 2025-01-01   │ 2025-04-01│
│  Anthropic     │ ⚠️ (14d)│ 2024-11-01   │ 2025-01-30│
│  Cohere        │ ❌ (5d) │ 2024-10-01   │ 2024-12-30│
│  xAI           │ ✅      │ 2025-01-15   │ 2025-04-15│
└─────────────────────────────────────────────────────┘
```

### 2.4 Audit Logging

**Requirement:** Complete audit trail for all secret access (SOC2, HIPAA compliance).

**Audit Log Fields:**
- Timestamp
- User/Service ID (who accessed)
- Secret path (what was accessed)
- Operation (read, write, delete)
- Result (success, denied)
- IP address (where from)

**Retention:**
- PostgreSQL: 90 days (hot storage)
- S3: 7 years (cold storage, compliance)

---

## 3. Non-Functional Requirements

### 3.1 Performance

**Latency:**
- Get secret: p50 < 10ms, p99 < 50ms
- Store secret: p50 < 20ms, p99 < 100ms

**Throughput:**
- 10K requests/second (Vault benchmark)

### 3.2 Reliability

**Availability:**
- 99.9% uptime SLA (inherits Vault's 99.9% SLA)
- Multi-AZ deployment (Vault HA cluster)

**Data Durability:**
- Vault storage backend: PostgreSQL (replicated 3 AZs)
- Backup: Daily snapshots to S3

### 3.3 Security

**Encryption:**
- At rest: AES-256 (Vault encryption)
- In transit: TLS 1.3 (all API calls)

**Access Control:**
- RBAC (role-based access control via Vault policies)
- mTLS (mutual TLS for inter-service communication)

---

## 4. Technical Architecture

### 4.1 Technology Stack

**Base Platform:** HashiCorp Vault (proven secrets management)

**Iron Cage Wrapper:**
- Rust (credential service API)
- React (admin dashboard)
- PostgreSQL (audit logs, metadata)
- Kubernetes CronJob (rotation monitor, daily)

### 4.2 Deployment Architecture

```
┌─────────────────────────────────────────────────────┐
│       CREDENTIAL MANAGEMENT (Iron Cage)             │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │   CREDENTIAL SERVICE (Rust)                  │   │
│  │   - CRUD API for secrets                     │   │
│  │   - Rotation reminders                       │   │
│  │   - Audit logging                            │   │
│  └─────────────────┬───────────────────────────┘   │
│                    │                                 │
│                    │ (Vault API)                     │
│                    ▼                                 │
│  ┌─────────────────────────────────────────────┐   │
│  │        HASHICORP VAULT (HA Cluster)         │   │
│  │  - 3 replicas (Multi-AZ)                     │   │
│  │  - PostgreSQL backend (encrypted storage)    │   │
│  │  - Dynamic secrets (generate on-demand)      │   │
│  │  - Lease management (TTL, renewal)           │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │   ROTATION MONITOR (Kubernetes CronJob)     │   │
│  │   - Runs daily at 9am UTC                    │   │
│  │   - Checks all secrets                       │   │
│  │   - Sends reminders (Slack, email)           │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 5. Integration with Other Capabilities

### 5.1 Capability 2 (LLM Gateway)

**Integration:** Gateway fetches LLM API keys from credential service.

**Flow:**
1. Gateway needs OpenAI API key
2. Calls credential service: `GET /credentials/llm/openai`
3. Credential service queries Vault, returns key
4. Gateway uses key for LLM API call
5. Gateway discards key (short-lived lease, 1 hour TTL)

### 5.2 Capability 8 (Enterprise Data Access)

**Integration:** Data connectors fetch credentials (Salesforce OAuth, database passwords).

**Flow:**
1. Connector needs Salesforce OAuth token
2. Calls credential service: `GET /credentials/data_sources/salesforce`
3. Credential service queries Vault, returns OAuth token
4. Connector uses token for Salesforce API call
5. Token expires after 24 hours (Vault dynamic secret)

---

## 6. Build Roadmap

### Phase 1: Core Features (Months 12-13)

- ✅ Vault deployment (HA cluster, 3 replicas)
- ✅ Credential service API (CRUD operations)
- ✅ Audit logging (PostgreSQL + S3)

### Phase 2: LLM-Specific Features (Month 14)

- ✅ Rotation monitor (CronJob, daily reminders)
- ✅ LLM credential dashboard (React UI)
- ✅ Integration with Cap 2 (LLM Gateway)
- ✅ Integration with Cap 8 (Data Access)

---

## 7. Success Metrics

### Product Metrics (Month 14)

**Adoption:**
- 100% of LLM API keys stored in credential service (mandatory)
- 100% of data source credentials stored in credential service

**Performance:**
- p99 get secret latency < 50ms
- 99.9% uptime

**Compliance:**
- Zero credentials stored outside Vault
- 100% audit log coverage (all secret access logged)

---

## 8. Open Questions

1. **Vault Deployment:** Managed Vault Cloud (easier, $X/mo) vs self-hosted (more control, lower cost)?

2. **Secret Versioning:** Keep last N versions of secrets (for rollback) or single version only?

3. **Dynamic Secrets:** Use Vault dynamic secrets (generate on-demand, short TTL) vs static secrets (manually rotated)?

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-01-20 | Platform Engineering | Initial product specification for Capability 5 (Credential Management). Defines functional requirements (secret storage/retrieval, rotation reminders, unified LLM dashboard, audit logging), non-functional requirements (performance <50ms p99, 99.9% uptime, AES-256 encryption), technical architecture (Vault HA cluster, Rust wrapper), integration with Cap 2 (LLM Gateway) and Cap 8 (Data Access), build roadmap (1-2 months, 1 engineer), success metrics. Strategic recommendation: THIN WRAPPER around HashiCorp Vault (don't rebuild secrets storage). Ready for engineering review. |
