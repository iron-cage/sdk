# LLM Inference Providers Landscape

**Created:** 2025-12-07
**Scope:** Project-wide reference for LLM provider capabilities, pricing, and limitations
**Purpose:** Comprehensive analysis of major LLM inference providers to inform Iron Cage integration, demo scenarios, and architectural decisions

---

## Document Responsibility

### ✅ IN SCOPE

**What this document contains:**
- Complete catalog of major LLM inference providers (8+ tier-1 providers)
- Detailed capabilities analysis: models, pricing, rate limits, budget controls
- Provider-specific constraints and limitations (especially budget enforcement)
- Comparison matrix for integration decisions
- Provider reliability patterns and failure modes
- Circuit breaker and fallback strategy recommendations
- Pricing tiers and cost optimization insights

**Usage contexts:**
- Integration planning: Which providers to support first
- Demo scenarios: Which provider to use for demos
- Circuit breaker design: Primary/fallback provider selection
- Cost optimization: Provider selection based on use case
- Customer guidance: "Which provider should I use?"
- Architecture decisions: Multi-provider support strategy

### ❌ OUT OF SCOPE

**What this document does NOT contain:**
- Implementation details (belongs in technical specs)
- API integration code (belongs in codebase)
- Provider API credentials or secrets (belongs in secure config)
- Temporary research notes (use hyphenated files for that)
- Conference-specific Q&A (belongs in conference-specific docs)
- Historical provider comparisons (keep only current state)

### 🔄 UPDATE TRIGGERS

**This document should be updated when:**
- New major provider launches or gains market share
- Existing provider changes capabilities (especially budget controls)
- Pricing changes significantly (>20% shift)
- New rate limit tiers or features announced
- Provider reliability issues discovered
- Integration priorities change

**Update frequency:** Quarterly review recommended, ad-hoc for major changes

---

## Provider Classification

### Tier 1: Proprietary Model Providers
High-quality models, highest cost, extensive API features

### Tier 2: Cloud Platform Services
Multi-model offerings, enterprise integration, SLA guarantees

### Tier 3: Open Source Inference Platforms
Cost leaders, open models, developer-friendly

### Tier 4: Specialized Infrastructure
Unique value propositions (speed, hardware, specific use cases)

---

## Provider Catalog (8 Major Providers)

### Tier 1: Proprietary Model Providers

---

#### 1. OpenAI

**Market Position:** Industry leader, most widely deployed

**Models:**
- GPT-4 Turbo (128k context)
- GPT-4 (8k/32k context)
- GPT-3.5 Turbo (16k context)
- o1 (reasoning model)
- Embeddings (text-embedding-3-small/large)

**Pricing (Dec 2024):**
- GPT-4 Turbo: $0.01/1K input tokens, $0.03/1K output
- GPT-4: $0.03/1K input, $0.06/1K output
- GPT-3.5 Turbo: $0.0005/1K input, $0.0015/1K output
- o1: $15/1M input, $60/1M output tokens

**Rate Limits:**
- Tier-based (usage-dependent)
- Tier 1: 10K TPM (tokens per minute)
- Tier 5: 10M TPM
- Request limits: 500-10,000 RPM depending on tier

**Budget Controls:**
- ✅ **Monthly hard limits available** (dashboard setting)
- ✅ Email notifications at thresholds (50%, 75%, 90%)
- ✅ Automatic blocking when monthly limit reached
- ⚠️ **Monthly granularity only** (not hourly/daily)
- ⚠️ Race conditions possible with high concurrency
- ⚠️ In-flight requests complete before blocking

**Reliability Patterns:**
- Most frequent outages due to high demand
- HTTP 429 on rate limit exceeded
- Retry-After header provided
- Status page: status.openai.com

**Iron Cage Integration Notes:**
- Primary provider for demos (most recognizable)
- Need hourly budget enforcement (OpenAI only has monthly)
- Circuit breaker critical (frequent 429s during peak)
- Cost tracking: Track per-agent spend within monthly cap

**Key Limitation for Iron Cage Value Prop:**
Monthly hard limits don't prevent hourly bursts. Example: $10K/month limit, early month ($600 spent), agent bug burns $8,200 in 6 hours → still under monthly cap, not blocked.

---

#### 2. Anthropic (Claude)

**Market Position:** Strong enterprise adoption, constitutional AI focus, safety-first

**Models:**
- Claude 3.5 Sonnet (200K context, newest)
- Claude 3 Opus (200K context, most capable)
- Claude 3 Sonnet (200K context, balanced)
- Claude 3 Haiku (200K context, fastest)

**Pricing (Dec 2024):**
- Claude 3.5 Sonnet: $3/MTok input, $15/MTok output
- Claude 3 Opus: $15/MTok input, $75/MTok output
- Claude 3 Sonnet: $3/MTok input, $15/MTok output
- Claude 3 Haiku: $0.25/MTok input, $1.25/MTok output

**Rate Limits:**
- Model-dependent
- Claude 3.5 Sonnet: 4K RPM, 400K TPM
- Claude 3 Opus: 4K RPM, 400K TPM
- Burst capacity available

**Budget Controls:**
- ❌ No hard budget limits
- ✅ Workspace spending limits (soft - alerts only)
- ✅ Usage notifications via email
- ✅ Dashboard monitoring
- Reality: Informational alerts, no automatic blocking

**Reliability Patterns:**
- Fewer outages than OpenAI (lower traffic)
- HTTP 429 on rate limit exceeded
- Retry-After header provided
- Generally stable

**Iron Cage Integration Notes:**
- Excellent fallback provider (high reliability)
- Higher cost than OpenAI for similar tasks
- Better safety features (constitutional AI)
- Good for compliance-focused customers

---

#### 3. Google AI (Gemini / Vertex AI)

**Market Position:** Google ecosystem integration, multimodal capabilities

**Models:**
- Gemini 1.5 Pro (2M context, multimodal)
- Gemini 1.5 Flash (1M context, fast)
- PaLM 2 (legacy, being phased out)

**Pricing (Dec 2024):**
- Gemini 1.5 Pro: $1.25/MTok input (<128K), $2.50/MTok input (>128K), $5/MTok output
- Gemini 1.5 Flash: $0.075/MTok input (<128K), $0.15/MTok input (>128K), $0.30/MTok output
- Volume discounts available

**Rate Limits:**
- Vertex AI: 360 RPM (free tier), 1500 RPM (paid)
- TPM varies by model
- Quota increases available

**Budget Controls:**
- ❌ No hard budget limits
- ✅ Google Cloud Budgets (alerts only)
- ✅ Budget notifications
- **Quote from docs:** "Budget alerts notify you when costs approach or exceed a threshold. They do not prevent spending."

**Reliability Patterns:**
- Stable (Google infrastructure)
- Regional availability varies
- HTTP 429 on quota exceeded
- Retry-After header

**Iron Cage Integration Notes:**
- Good for multimodal use cases
- Google Cloud integration (IAM, logging)
- Lower adoption than OpenAI/Anthropic
- Complex pricing structure

---

### Tier 2: Cloud Platform Services

---

#### 4. AWS Bedrock

**Market Position:** Multi-model offering, enterprise AWS integration

**Models Available:**
- Anthropic Claude (all versions)
- Meta Llama 3 (8B, 70B, 405B)
- Mistral AI models
- Cohere Command models
- Amazon Titan models

**Pricing (Dec 2024):**
- Claude 3.5 Sonnet: $3/MTok input, $15/MTok output (same as Anthropic)
- Llama 3 70B: $0.99/MTok input, $0.99/MTok output
- Mistral Large: $4/MTok input, $12/MTok output
- Pricing varies by model

**Rate Limits:**
- Service Quotas (configurable per region)
- Default: 200 requests/minute per model
- Can request quota increases
- Throttling via AWS throttling mechanisms

**Budget Controls:**
- ❌ No hard budget limits
- ✅ AWS Budgets (alerts only)
- ✅ Cost Management dashboards
- **Quote from docs:** "AWS Budgets gives you the ability to set custom budgets that **alert you** when your costs exceed your budgeted amount."

**Reliability Patterns:**
- SLA guarantees (AWS infrastructure)
- Regional redundancy available
- Better uptime than individual providers
- HTTP 429 (ThrottlingException) on quota exceeded

**Iron Cage Integration Notes:**
- Best for enterprise AWS customers
- Model choice flexibility
- Higher reliability (AWS SLA)
- Good fallback: Claude → Llama 3 within Bedrock

---

#### 5. Azure OpenAI Service

**Market Position:** Enterprise Microsoft integration, compliance focus

**Models:**
- GPT-4 Turbo (all versions)
- GPT-4 (8k/32k context)
- GPT-3.5 Turbo
- Embeddings (Ada v2, text-embedding-3)

**Pricing (Dec 2024):**
- Same as OpenAI base pricing + Azure overhead (~10%)
- GPT-4 Turbo: $0.01/1K input, $0.03/1K output
- Enterprise agreements available (volume discounts)

**Rate Limits:**
- Tokens Per Minute (TPM) quotas
- Configurable per deployment
- Default: 60K TPM for GPT-4
- Request limits: 360 RPM default

**Budget Controls:**
- ❌ No hard cost-based limits
- ✅ TPM quotas (rate-based, not cost-based)
- ✅ Azure Cost Management (alerts only)
- ✅ Budget alerts via Action Groups
- Reality: Alerts trigger workflows, don't block API calls

**Reliability Patterns:**
- More stable than OpenAI public API
- Regional redundancy
- Private endpoints available
- SLA guarantees for enterprise

**Iron Cage Integration Notes:**
- Best for Microsoft enterprise customers
- Better reliability than OpenAI public
- Complex quota management
- Good for compliance (SOC 2, HIPAA, etc.)

---

### Tier 3: Open Source Inference Platforms

---

#### 6. Together AI

**Market Position:** Cost leader, open source focus, fast inference

**Models:**
- Llama 3.1 (8B, 70B, 405B)
- Mixtral 8x7B, 8x22B
- Qwen 2.5 (various sizes)
- 50+ open source models

**Pricing (Dec 2024):**
- Llama 3.1 70B: $0.88/MTok input, $0.88/MTok output
- Llama 3.1 405B: $3.50/MTok input, $3.50/MTok output
- Mixtral 8x22B: $1.20/MTok input, $1.20/MTok output
- **Cost: 5-10x cheaper than proprietary models**

**Rate Limits:**
- High throughput (optimized infrastructure)
- Default: 600 RPM
- Custom limits negotiable
- Burst capacity available

**Budget Controls:**
- ❌ No hard budget limits
- ✅ Usage monitoring dashboard
- ✅ Email notifications
- Reality: Pay-as-you-go, no automatic enforcement

**Reliability Patterns:**
- Generally stable
- Smaller scale than OpenAI/Anthropic
- HTTP 429 on rate limits
- Uptime: ~99.5% (community reported)

**Iron Cage Integration Notes:**
- Excellent fallback for cost optimization
- Good quality for many use cases
- OpenAI-compatible API (easy migration)
- Primary for cost-sensitive demos

---

#### 7. Fireworks AI

**Market Position:** Speed-focused, function calling optimized

**Models:**
- Llama 3.1 (8B, 70B, 405B)
- Mixtral 8x7B, 8x22B
- Firefunction v2 (function calling optimized)
- 40+ open source models

**Pricing (Dec 2024):**
- Llama 3.1 70B: $0.90/MTok input, $0.90/MTok output
- Llama 3.1 405B: $3.00/MTok input, $3.00/MTok output
- Mixtral 8x22B: $1.20/MTok input, $1.20/MTok output
- Similar to Together AI

**Rate Limits:**
- Optimized for low latency
- High throughput
- Default: 600 RPM
- Custom limits available

**Budget Controls:**
- ❌ No hard budget limits
- ✅ Usage monitoring
- Reality: Pay-as-you-go

**Reliability Patterns:**
- Speed focus (2-3x faster than competitors)
- Generally stable
- Good uptime

**Iron Cage Integration Notes:**
- Best for latency-sensitive use cases
- Function calling optimization
- OpenAI-compatible API
- Good alternative to Together AI

---

### Tier 4: Specialized Infrastructure

---

#### 8. Groq

**Market Position:** Fastest inference, specialized LPU (Language Processing Unit) hardware

**Models:**
- Llama 3.1 (8B, 70B)
- Mixtral 8x7B
- Gemma 7B
- Limited model selection (hardware-optimized only)

**Pricing (Dec 2024):**
- Free tier: 30 requests/min, 6K tokens/min
- Paid tier: Competitive (exact pricing varies)
- Llama 3.1 70B: ~$0.80/MTok (estimated)

**Rate Limits:**
- Free tier: 30 RPM, 6K TPM
- Paid tier: Higher limits (negotiable)
- Burst capacity limited (hardware constraints)

**Budget Controls:**
- ❌ No hard budget limits
- ✅ Free tier automatic limiting
- Reality: Rate limits enforced, not cost limits

**Reliability Patterns:**
- Speed: 800+ tokens/sec (fastest in industry)
- Specialized hardware (limited scaling)
- Occasional capacity issues
- Good uptime overall

**Iron Cage Integration Notes:**
- Use case: Ultra-low latency demos
- Unique value prop (speed)
- Limited model selection
- Good for showcasing Iron Cage speed optimization

---

## Capability Comparison Matrix

| Provider | Monthly Hard Limits | Hourly/Daily Limits | Per-Agent Budgets | Real-time Cost Blocking | Multi-Provider Support |
|----------|-------------------|-------------------|------------------|----------------------|---------------------|
| OpenAI | ✅ Yes | ❌ No | ❌ No | ⚠️ Monthly only | N/A |
| Anthropic | ❌ No | ❌ No | ❌ No | ❌ No | N/A |
| Google Gemini | ❌ No | ❌ No | ❌ No | ❌ No | N/A |
| AWS Bedrock | ❌ No | ❌ No | ❌ No | ❌ No | ✅ Multi-model |
| Azure OpenAI | ❌ No | ❌ No | ❌ No | ❌ No | N/A |
| Together AI | ❌ No | ❌ No | ❌ No | ❌ No | ✅ Multi-model |
| Fireworks AI | ❌ No | ❌ No | ❌ No | ❌ No | ✅ Multi-model |
| Groq | ❌ No | ❌ No | ❌ No | ❌ No | N/A |
| **Iron Cage** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | ✅ All providers |

**Key finding:** Only OpenAI offers any form of hard budget limit (monthly). No provider offers hourly/daily/per-agent budget enforcement.

---

## Rate Limit Comparison

| Provider | Rate Limit Type | Default Limits | Enforcement Mechanism |
|----------|----------------|---------------|---------------------|
| OpenAI | TPM + RPM | 10K-10M TPM, 500-10K RPM | HTTP 429, Retry-After |
| Anthropic | TPM + RPM | 400K TPM, 4K RPM | HTTP 429, Retry-After |
| Google Gemini | RPM + Quota | 360-1500 RPM | HTTP 429, Quota exceeded |
| AWS Bedrock | Model-specific | 200 RPM per model | ThrottlingException |
| Azure OpenAI | TPM quotas | 60K TPM | HTTP 429, Quota exceeded |
| Together AI | RPM | 600 RPM | HTTP 429 |
| Fireworks AI | RPM | 600 RPM | HTTP 429 |
| Groq | RPM + TPM | 30 RPM (free), 6K TPM | HTTP 429 |

**Note:** All providers enforce rate limits (count-based). None enforce cost limits (dollar-based) at sub-monthly granularity.

---

## Cost Optimization Strategies

### Primary → Fallback Chains

**Strategy 1: Quality-first with cost fallback**
1. Primary: OpenAI GPT-4 Turbo (highest quality)
2. Fallback 1: Anthropic Claude 3.5 Sonnet (comparable quality)
3. Fallback 2: Together AI Llama 3.1 70B (10x cheaper)

**Strategy 2: Cost-first with quality escalation**
1. Primary: Together AI Llama 3.1 70B (cheapest)
2. Fallback 1: OpenAI GPT-3.5 Turbo (good quality, medium cost)
3. Fallback 2: OpenAI GPT-4 Turbo (best quality, highest cost)

**Strategy 3: Enterprise compliance**
1. Primary: Azure OpenAI GPT-4 (compliance guarantees)
2. Fallback: AWS Bedrock Claude (multi-region redundancy)

---

## Iron Cage Circuit Breaker Design

### Provider Selection Logic

**For demos:**
- Primary: OpenAI (most recognizable)
- Show circuit breaker: OpenAI → Together AI (cost optimization)
- Show speed: Groq (fastest)

**For production:**
- Primary: Customer choice (OpenAI/Anthropic/Azure)
- Fallback: Together AI (cost optimization)
- Emergency: Fireworks AI (speed + cost)

### Rate Limit Handling

**All providers return HTTP 429 on rate limit:**
```
Iron Cage behavior:
1. Detect 429 response
2. Check Retry-After header (if present)
3. Open circuit breaker immediately
4. Attempt fallback provider
5. If all providers exhausted → return BudgetExceeded error
```

---

## Provider Reliability Patterns

### Observed Failure Modes

**OpenAI:**
- Most frequent outages (high demand)
- 429 errors during peak hours
- Occasional 5xx errors (infrastructure)
- Status: status.openai.com

**Anthropic:**
- Fewer outages (lower traffic)
- Stable during OpenAI outages (good fallback)
- Occasional rate limit issues

**Google Gemini:**
- Very stable (Google infrastructure)
- Regional availability issues
- Complex quota system (can be confusing)

**AWS Bedrock:**
- Most reliable (AWS SLA)
- Multi-region redundancy
- Throttling during cold starts

**Together AI / Fireworks AI:**
- Generally stable
- Smaller scale (capacity issues possible)
- Good uptime overall

**Groq:**
- Speed leader
- Occasional capacity issues (hardware limited)
- Limited model selection

---

## Integration Priority Recommendations

### Phase 1 (MVP)
1. OpenAI (most demand)
2. Anthropic (quality fallback)
3. Together AI (cost fallback)

### Phase 2 (Enterprise)
4. AWS Bedrock (enterprise customers)
5. Azure OpenAI (Microsoft customers)

### Phase 3 (Specialized)
6. Fireworks AI (speed optimization)
7. Groq (ultra-low latency)
8. Google Gemini (multimodal)

---

## Document Maintenance

**Last updated:** 2025-12-07

**Next review:** 2026-03-07 (quarterly)

**Major changes since last update:**
- 2025-12-07: Initial document creation
- 2025-12-07: Corrected OpenAI hard limits (monthly granularity)

**Update triggers:**
- Provider pricing changes >20%
- New provider capabilities (especially budget controls)
- New major provider enters market
- Integration priority changes

---

**Status:** Current as of December 2025
