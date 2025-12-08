# Capability 3: Unified LLM Access Control - Product Specification

**Version:** 1.0.0
**Last Updated:** 2025-01-20
**Status:** Draft - Platform Component Specification
**Build Priority:** Platform Component (58/100 standalone viability - build as component)

---

### Scope

**Responsibility:** Product specification for Unified LLM Access Control capability (Capability 3 of 8 - platform component, 58/100 standalone viability)

**In Scope:**
- Market opportunity ($3.9B AI gateway market 2024, 875% YoY growth, Gartner 70% adoption by 2028)
- Strategic approach (platform component, NOT standalone - fork LiteLLM, add governance layer)
- Problem statement (fragmented multi-LLM access, no cost visibility, no governance, no caching)
- Solution architecture (unified gateway with cost attribution, rate limiting, semantic caching 60% reduction, compliance audit)
- Build recommendation (FORK LiteLLM, do NOT build from scratch, 2-3 months, 1-2 engineers)
- Platform integration (included in $100K-300K/year Iron Cage platform, not sold separately)
- Feature specifications (cost tracking, budget enforcement, semantic caching, failover, compliance trails)
- Competitive positioning (vs LiteLLM, Kong, Portkey, Helicone)
- Standalone viability score (58/100 - build as component, NOT standalone product)

**Out of Scope:**
- System architecture implementation (see `/docs/architecture.md` for HOW to build)
- Warsaw pilot specifications (see `../pilot/spec.md` for 28 pilot features with basic cost tracking)
- Implementation guide (see `/runtime/pilot_guide.md` for pilot build instructions)
- Other 7 capabilities (see `capability_1_enterprise_data_access.md` through `capability_8_agent_runtime.md`)
- Strategic analysis across capabilities (see `/business/strategy/executive_summary.md` for all 8 ranked)
- Competitor detailed research (see `/research/competitors/capability_3_competitors_2025.md`)

---

## Executive Summary

This specification defines the requirements for Iron Cage's Unified LLM Access Control capability - a centralized gateway for all LLM API calls with cost tracking, rate limiting, and semantic caching.

**Market Opportunity:** $3.9B AI gateway market (2024), 875% YoY growth, Gartner predicts 70% adoption by 2028
**Strategic Approach:** Platform component (NOT standalone product)
**Build Timeline:** 2-3 months, fork LiteLLM + add Iron Cage governance layer
**Platform Pricing:** Included in $100K-300K/year Iron Cage platform (not sold separately)

**Core Value Proposition:** Replace fragmented multi-LLM access (direct OpenAI + Anthropic + Cohere API calls, no cost visibility, no governance) with unified gateway providing cost attribution, real-time budget enforcement, semantic caching (60% cost reduction), and compliance audit trails.

**Strategic Recommendation:** FORK LITELLLM (12K stars, proven tech) and add Iron Cage-specific features (cost attribution, compliance, guardrails integration). Do NOT build from scratch. Do NOT compete with LiteLLM or Kong directly.

---

## 1. Product Overview

### 1.1 Problem Statement

Enterprise AI deployments face fragmented LLM access:

```
CURRENT STATE: Direct Provider Access (No Gateway)
┌─────────────────────────────────────────────────────┐
│  AGENTS                                              │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐            │
│  │ Agent 1 │  │ Agent 2 │  │ Agent 3 │            │
│  └────┬────┘  └────┬────┘  └────┬────┘            │
│       │            │            │                   │
│       ├────────────┼────────────┤                   │
│       │            │            │                   │
│  ┌────▼────┐  ┌───▼────┐  ┌───▼────┐              │
│  │ OpenAI  │  │Anthropic│  │ Cohere │              │
│  │   API   │  │  API    │  │  API   │              │
│  └─────────┘  └─────────┘  └─────────┘              │
├─────────────────────────────────────────────────────┤
│ PAIN POINTS:                                        │
│ - No cost visibility (who spent what?)              │
│ - No budget enforcement (runaway costs)             │
│ - No caching (redundant API calls)                  │
│ - No compliance audit trail (SOC2, HIPAA)           │
│ - No failover (if OpenAI down, agents fail)         │
└─────────────────────────────────────────────────────┘
```

### 1.2 Solution: Iron Cage LLM Gateway

```
IRON CAGE SOLUTION: Unified LLM Gateway
┌─────────────────────────────────────────────────────┐
│  AGENTS                                              │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐            │
│  │ Agent 1 │  │ Agent 2 │  │ Agent 3 │            │
│  └────┬────┘  └────┬────┘  └────┬────┘            │
│       │            │            │                   │
│       └────────────┴────────────┘                   │
│                    │                                 │
│         ┌──────────▼──────────┐                     │
│         │   LLM GATEWAY       │                     │
│         │  (Iron Cage Cap 2)  │                     │
│         │                     │                     │
│         │ ✅ Cost tracking    │                     │
│         │ ✅ Budget enforce   │                     │
│         │ ✅ Semantic cache   │                     │
│         │ ✅ Rate limiting    │                     │
│         │ ✅ Failover         │                     │
│         │ ✅ Audit logs       │                     │
│         └──────────┬──────────┘                     │
│                    │                                 │
│       ┌────────────┼────────────┐                   │
│       │            │            │                   │
│  ┌────▼────┐  ┌───▼────┐  ┌───▼────┐              │
│  │ OpenAI  │  │Anthropic│  │ Cohere │              │
│  └─────────┘  └─────────┘  └─────────┘              │
├─────────────────────────────────────────────────────┤
│ ✅ Unified access (single API for all LLMs)         │
│ ✅ Cost attribution (by agent/user/tenant)          │
│ ✅ Real-time cutoffs ($100 budget hit → block)      │
│ ✅ 60% cost reduction (semantic caching)            │
│ ✅ Compliance audit trail (SOC2/HIPAA/GDPR)         │
└─────────────────────────────────────────────────────┘
```

### 1.3 Strategic Positioning

**NOT:** "LLM Gateway" (too crowded - LiteLLM dominates open-source, Kong dominates enterprise)

**YES:** "Enterprise AI Governance Platform Component" (Cost control is ONE PIECE of unified governance: cost + security + compliance + observability)

**Partnership Strategy:**
- Fork LiteLLM (12K stars, proven) as base technology
- Add Iron Cage-specific governance layer on top
- Position as platform component ($100K-300K/year), not point tool ($49-$500/mo)

---

## 2. Functional Requirements

### 2.1 Multi-Provider Routing

**Requirement:** Support 100+ LLM providers with unified OpenAI-compatible API.

**Providers (Priority 1 - Launch):**
- OpenAI (GPT-4, GPT-3.5, o1, o1-mini)
- Anthropic (Claude 3.5 Sonnet, Claude 3 Opus/Haiku)
- Google (Gemini Pro, Gemini Flash)
- AWS Bedrock (all models: Claude, Llama, Mistral)
- Azure OpenAI (all models)
- Cohere (Command, Command-Light, Embed)
- xAI (Grok)
- Open-source (Ollama, HuggingFace, vLLM)

**Providers (Priority 2 - Month 6):**
- Mistral AI, Groq, Together AI, Perplexity, Replicate
- Custom self-hosted models (via OpenAI-compatible endpoints)

**Architecture:**
```rust
// src/gateway/router.rs (simplified - actual implementation in Python/LiteLLM)

pub struct LlmRouter
{
  providers: Arc< ProviderRegistry >,
  fallback_chain: Vec< String >, // e.g., ["openai-gpt4", "anthropic-sonnet", "gemini-pro"]
}

impl LlmRouter
{
  pub async fn route_request
  (
    &self,
    request: LlmRequest,
    routing_config: RoutingConfig,
  ) -> Result< LlmResponse >
  {
    // 1. Determine target provider (explicit or automatic)
    let provider = match routing_config.explicit_provider
    {
      Some( p ) => p,
      None => self.select_best_provider( &request, &routing_config ).await?,
    };

    // 2. Try primary provider
    match self.call_provider( &provider, &request ).await
    {
      Ok( response ) => return Ok( response ),
      Err( err ) if routing_config.enable_fallback =>
      {
        // 3. Fallback to next provider in chain
        for fallback in &self.fallback_chain
        {
          if fallback != &provider
          {
            if let Ok( response ) = self.call_provider( fallback, &request ).await
            {
              return Ok( response );
            }
          }
        }
        Err( err ) // All fallbacks failed
      }
      Err( err ) => Err( err ), // Fallback disabled
    }
  }
}
```

### 2.2 Cost Tracking & Attribution

**Requirement:** Track LLM spend at token-level granularity with multi-dimensional attribution.

**Attribution Dimensions:**
- **Agent:** Which agent made the call
- **User:** Which end-user triggered the agent
- **Tenant:** Which organization/tenant (for multi-tenant deployments)
- **Project:** Which project/application
- **Model:** Which LLM model (GPT-4, Claude, etc.)

**Implementation:**
```rust
// src/gateway/cost_tracker.rs

pub struct CostTracker
{
  db: Arc< PostgresPool >,
  pricing: Arc< PricingTable >,
}

impl CostTracker
{
  pub async fn track_usage
  (
    &self,
    request: &LlmRequest,
    response: &LlmResponse,
    metadata: &RequestMetadata,
  ) -> Result< ()>
  {
    // 1. Count tokens
    let input_tokens = response.usage.prompt_tokens;
    let output_tokens = response.usage.completion_tokens;

    // 2. Calculate cost (per-model pricing)
    let model_pricing = self.pricing.get_pricing( &response.model )?;
    let cost_usd = model_pricing.calculate_cost( input_tokens, output_tokens );

    // 3. Store usage record (multi-dimensional)
    let record = UsageRecord
    {
      timestamp: Utc::now(),
      agent_id: metadata.agent_id.clone(),
      user_id: metadata.user_id.clone(),
      tenant_id: metadata.tenant_id.clone(),
      project_id: metadata.project_id.clone(),
      model: response.model.clone(),
      input_tokens,
      output_tokens,
      cost_usd,
      request_id: request.id.clone(),
    };

    self.db.insert_usage_record( &record ).await?;

    Ok( () )
  }

  pub async fn get_usage_report
  (
    &self,
    filters: UsageFilters,
    group_by: Vec< AttributionDimension >,
  ) -> Result< UsageReport >
  {
    // Query database with filters and aggregation
    let query = self.build_query( filters, group_by )?;
    let results = self.db.execute_query( &query ).await?;

    Ok( UsageReport::from_results( results ) )
  }
}

pub enum AttributionDimension
{
  Agent,
  User,
  Tenant,
  Project,
  Model,
  Day,
  Week,
  Month,
}
```

**Pricing Table (Sample):**
| Model | Input ($/1M tokens) | Output ($/1M tokens) |
|-------|---------------------|----------------------|
| gpt-4-turbo | $10.00 | $30.00 |
| gpt-3.5-turbo | $0.50 | $1.50 |
| claude-3-sonnet | $3.00 | $15.00 |
| gemini-pro | $0.50 | $1.50 |

### 2.3 Budget Enforcement

**Requirement:** Real-time budget enforcement with hard limits and soft alerts.

**Budget Policies:**
```rust
// src/gateway/budget.rs

pub struct BudgetPolicy
{
  pub id: String,
  pub scope: BudgetScope,
  pub limit_usd: f64, // e.g., $100.00
  pub period: BudgetPeriod, // Daily, Weekly, Monthly
  pub enforcement: EnforcementMode,
  pub alerts: Vec< AlertThreshold >, // e.g., [50%, 75%, 90%]
}

pub enum BudgetScope
{
  Agent( String ), // Per-agent budget
  User( String ), // Per-user budget
  Tenant( String ), // Per-tenant budget
  Project( String ), // Per-project budget
  Global, // Global budget (all usage)
}

pub enum BudgetPeriod
{
  Daily,
  Weekly,
  Monthly,
  Yearly,
  Lifetime, // One-time limit (never resets)
}

pub enum EnforcementMode
{
  HardLimit, // Block requests when limit reached
  SoftLimit, // Alert but allow requests
}

pub struct AlertThreshold
{
  pub percentage: f64, // e.g., 0.75 (75%)
  pub channels: Vec< AlertChannel >, // Slack, email, webhook
}
```

**Enforcement Logic:**
```rust
// src/gateway/budget_enforcer.rs

pub struct BudgetEnforcer
{
  cost_tracker: Arc< CostTracker >,
  policy_store: Arc< PolicyStore >,
  notifier: Arc< Notifier >,
}

impl BudgetEnforcer
{
  pub async fn check_budget_before_request
  (
    &self,
    metadata: &RequestMetadata,
    estimated_cost_usd: f64, // Estimated based on prompt length
  ) -> Result< BudgetCheckResult >
  {
    // 1. Find applicable budget policies
    let policies = self.policy_store
      .get_policies_for_request( metadata )
      .await?;

    // 2. Check each policy
    for policy in policies
    {
      let current_usage = self.cost_tracker
        .get_usage_for_period( &policy.scope, &policy.period )
        .await?;

      let projected_usage = current_usage + estimated_cost_usd;

      // 3. Hard limit check
      if policy.enforcement == EnforcementMode::HardLimit
        && projected_usage >= policy.limit_usd
      {
        return Ok( BudgetCheckResult::Denied
        {
          reason: format!(
            "Budget limit reached: ${:.2}/{:.2} for {:?}",
            current_usage, policy.limit_usd, policy.scope
          ),
        });
      }

      // 4. Alert thresholds
      for alert in &policy.alerts
      {
        let threshold_amount = policy.limit_usd * alert.percentage;
        if current_usage < threshold_amount && projected_usage >= threshold_amount
        {
          // Just crossed threshold, send alert
          self.notifier.send_budget_alert( &policy, current_usage, projected_usage ).await?;
        }
      }
    }

    Ok( BudgetCheckResult::Allowed )
  }
}

pub enum BudgetCheckResult
{
  Allowed,
  Denied { reason: String },
}
```

### 2.4 Semantic Caching

**Requirement:** Cache LLM responses with semantic similarity matching (60% cost reduction target).

**Architecture:**
```rust
// src/gateway/semantic_cache.rs

pub struct SemanticCache
{
  embedding_model: Arc< EmbeddingModel >, // MiniLM-L6 (384 dim, fast)
  vector_store: Arc< dyn VectorStore >, // Redis with vector extension or Qdrant
  ttl: Duration, // Default: 24 hours
  similarity_threshold: f32, // Default: 0.95 (95% similarity)
}

impl SemanticCache
{
  pub async fn lookup
  (
    &self,
    prompt: &str,
    model: &str,
  ) -> Result< Option< CachedResponse > >
  {
    // 1. Generate prompt embedding (fast model)
    let embedding = self.embedding_model.encode( prompt ).await?;

    // 2. Vector similarity search
    let matches = self.vector_store
      .search
      (
        &embedding,
        top_k: 1,
        filter: json!({ "model": model }), // Only match same model
        min_score: self.similarity_threshold,
      )
      .await?;

    // 3. Return cached response if found
    if let Some( best_match ) = matches.first()
    {
      Ok( Some( best_match.cached_response.clone() ) )
    }
    else
    {
      Ok( None )
    }
  }

  pub async fn store
  (
    &self,
    prompt: &str,
    model: &str,
    response: LlmResponse,
  ) -> Result< () >
  {
    // 1. Generate embedding
    let embedding = self.embedding_model.encode( prompt ).await?;

    // 2. Store in vector DB with TTL
    let cached = CachedResponse
    {
      prompt: prompt.to_string(),
      model: model.to_string(),
      response,
      embedding,
      created_at: Utc::now(),
      expires_at: Utc::now() + self.ttl,
    };

    self.vector_store.upsert( cached ).await?;

    Ok( () )
  }
}
```

**Cost Savings Example:**
- **Baseline:** 1M LLM requests, $0.01/request = $10,000
- **With 60% cache hit rate:** 400K API calls, $4,000 (60% savings)

### 2.5 Rate Limiting

**Requirement:** Multi-level rate limiting to prevent abuse and runaway costs.

**Rate Limit Policies:**
```rust
// src/gateway/rate_limiter.rs

pub struct RateLimitPolicy
{
  pub scope: RateLimitScope,
  pub limits: Vec< RateLimit >,
}

pub enum RateLimitScope
{
  Agent( String ),
  User( String ),
  Tenant( String ),
  Global,
}

pub struct RateLimit
{
  pub window: Duration, // e.g., 1 hour
  pub max_requests: usize, // e.g., 1000 requests
}
```

**Example Policies:**
- Per-user: 100 requests/hour
- Per-agent: 1000 requests/hour
- Per-tenant: 10,000 requests/hour
- Global: 100,000 requests/hour

### 2.6 Compliance & Audit Logging

**Requirement:** Complete audit trail for SOC2, HIPAA, GDPR compliance.

**Audit Log Fields:**
- Timestamp
- Agent ID
- User ID
- Tenant ID
- Model
- Prompt (redacted PII/secrets)
- Response (redacted PII/secrets)
- Input tokens, output tokens, cost
- Latency
- Status (success, error, budget_denied, rate_limited)

**Retention:**
- PostgreSQL: 90 days (hot storage)
- S3: 7 years (cold storage, compliance requirement)

---

## 3. Non-Functional Requirements

### 3.1 Performance

**Latency Overhead:**
- p50 < 20ms (gateway overhead, excluding LLM API time)
- p99 < 50ms
- Cache hit latency: p50 < 10ms, p99 < 30ms

**Throughput:**
- 10K requests/second (single region)
- 100K requests/second (multi-region)

**Cache Hit Rate:**
- Target: 60%+ (semantic caching)
- Baseline: 40%+ (first 3 months)

### 3.2 Reliability

**Availability:**
- 99.9% uptime SLA
- Multi-AZ deployment

**Failover:**
- Automatic provider failover (<1s detection, <5s switch)
- Fallback chain: Primary → Secondary → Tertiary

**Data Durability:**
- Audit logs: 99.999999999% (S3)
- Cost tracking: Replicated across 3 AZs

### 3.3 Security

**Authentication:**
- API key authentication (HMAC-SHA256)
- API key rotation every 90 days

**PII Protection:**
- Redact PII/secrets before logging (integrate with Cap 4)

**Encryption:**
- At rest: AES-256
- In transit: TLS 1.3

---

## 4. Technical Architecture

### 4.1 Technology Stack

**Base Platform:** Fork LiteLLM (Python, 12K GitHub stars)

**Iron Cage Additions:**
- **Cost Attribution:** PostgreSQL schema for multi-dimensional usage tracking
- **Budget Enforcement:** Real-time budget checks (before request) + policy engine
- **Semantic Caching:** Redis with vector extension OR Qdrant (lightweight)
- **Compliance:** Audit log pipeline (PostgreSQL → S3), PII redaction hooks

**Infrastructure:**
- Kubernetes (3 replicas, t3.xlarge)
- PostgreSQL (db.r5.large, Multi-AZ)
- Redis (cache.r5.large, 2 replicas)
- S3 (audit log archival)

### 4.2 Deployment Architecture

```
┌─────────────────────────────────────────────────────┐
│              LLM GATEWAY (Kubernetes)                │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │         AXUM WEB SERVER (Rust)              │   │
│  │  - Authentication (API keys)                 │   │
│  │  - Rate limiting (Redis)                     │   │
│  │  - Budget enforcement (PostgreSQL)           │   │
│  └─────────────────┬───────────────────────────┘   │
│                    │                                 │
│  ┌─────────────────▼───────────────────────────┐   │
│  │         LITELLLM PROXY (Python)             │   │
│  │  - Multi-provider routing                    │   │
│  │  - Failover logic                            │   │
│  │  - OpenAI-compatible API                     │   │
│  └─────────────────┬───────────────────────────┘   │
│                    │                                 │
│  ┌─────────────────▼───────────────────────────┐   │
│  │         SEMANTIC CACHE (Redis)              │   │
│  │  - Vector search (embeddings)                │   │
│  │  - TTL management (24 hours)                 │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 5. Integration with Other Capabilities

### 5.1 Capability 1 (Agent Runtime)

**Integration:** Agents use LLM gateway for all LLM API calls.

**Implementation:**
- Inject gateway endpoint as environment variable: `LLM_API_BASE_URL=https://gateway.ironcage.ai/v1`
- Agents use OpenAI SDK (compatible with all LLMs via gateway)

### 5.2 Capability 4 (Safety Guardrails)

**Integration:** All prompts/responses flow through input/output firewalls.

**Flow:**
1. Agent sends prompt → Gateway
2. Gateway → Input firewall (prompt injection detection, PII detection)
3. If safe → Forward to LLM provider
4. LLM response → Output firewall (secret scanning, PII redaction)
5. If safe → Return to agent

### 5.3 Capability 5 (Credential Management)

**Integration:** Gateway fetches LLM API keys from credential service.

**Implementation:**
- Gateway queries Vault: `GET /credentials/llm/openai`
- Vault returns API key (encrypted, short-lived lease)
- Gateway uses key for LLM API call, discards after use

### 5.4 Capability 7 (Observability)

**Integration:** Gateway reports metrics to observability service.

**Metrics:**
- Request rate (QPS by model, agent, tenant)
- Latency (p50, p95, p99)
- Cost ($ spent by model, agent, tenant)
- Cache hit rate (% of requests cached)
- Error rate (by provider, error type)

---

## 6. Build Roadmap

### Phase 1: Fork & Core Features (Months 6-7)

- ✅ Fork LiteLLM (base gateway functionality)
- ✅ Add cost tracking schema (PostgreSQL)
- ✅ Add budget enforcement logic (hard limits, soft alerts)
- ✅ Basic semantic caching (Redis)

### Phase 2: Governance Integration (Months 8-9)

- ✅ Integrate with Cap 4 (Safety Guardrails)
- ✅ Integrate with Cap 5 (Credentials)
- ✅ Integrate with Cap 7 (Observability)
- ✅ Multi-tenant support (tenant-level budgets, rate limits)

### Phase 3: Compliance & Polish (Month 9)

- ✅ Audit log pipeline (PostgreSQL → S3)
- ✅ SOC2/HIPAA/GDPR dashboards
- ✅ Cost attribution reports (by agent, user, tenant, project)
- ✅ Admin UI (policy management, usage reports)

---

## 7. Success Metrics

### Product Metrics (Month 9)

**Adoption:**
- 100% of agents use gateway (mandatory for Iron Cage platform)
- 10K+ LLM requests/day per deployment

**Performance:**
- p99 gateway latency < 50ms
- 60%+ semantic cache hit rate
- 99.9% uptime

**Cost Optimization:**
- 60%+ LLM cost reduction (via caching)
- Zero budget overruns (hard limit enforcement)

---

## 8. Open Questions

1. **LiteLLM Fork Strategy:** Fork once and maintain (higher effort) vs stay close to upstream (easier updates)?

2. **Cache Storage:** Redis with vector extension (simpler) vs Qdrant (better performance)?

3. **Budget Period Resets:** UTC midnight (simpler) vs tenant-specific timezone (better UX)?

4. **Failover Latency:** 1s detection + 5s switch (conservative) vs 100ms detection + 1s switch (aggressive, risk false positives)?

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-01-20 | Platform Engineering | Initial product specification for Capability 2 (Unified LLM Access Control). Defines functional requirements (multi-provider routing, cost tracking/attribution, budget enforcement, semantic caching, rate limiting, audit logging), non-functional requirements (performance <20ms overhead, 99.9% uptime, 60%+ cache hit rate), technical architecture (fork LiteLLM, add Iron Cage governance layer), integration with other capabilities (Caps 1, 4, 5, 7), build roadmap (2-3 months), success metrics. Strategic recommendation: BUILD AS COMPONENT (not standalone), fork LiteLLM as base. Ready for engineering review. |
