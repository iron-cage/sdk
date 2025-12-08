# Iron Cage Runtime: Technical Glossary

**Version:** 1.0.0
**Date:** 2025-11-17
**Purpose:** Comprehensive definitions of technical terms, design patterns, and domain concepts

---

### Scope

**Responsibility:** Comprehensive technical glossary for all Iron Cage terminology (business domain, AI/ML concepts, infrastructure patterns, compliance terms)

**In Scope:**
- Business domain concepts (lead, lead generation, MQL, SQL, firmographic data, lead scoring)
- AI/ML terminology (LLM, agent, prompt, token, embedding, fine-tuning, RAG, chain-of-thought)
- Infrastructure patterns (circuit breaker, fallback chain, health check, state management, PyO3 FFI)
- Safety and compliance terms (PII, GDPR, SOC2, HIPAA, prompt injection, jailbreaking, redaction)
- Cost control terminology (budget, attribution, token counting, rate limiting, caching)
- Rust-specific terms (ownership, borrowing, lifetime, async, Tokio, trait, macro)
- Python agent framework terms (LangChain, CrewAI, AutoGPT, tool, memory, callback)
- Observable and monitoring terms (metric, trace, span, log level, alert, control panel)

**Out of Scope:**
- Implementation details and code examples (see `/runtime/PILOT_GUIDE.md` for HOW to build)
- Architecture diagrams (see `architecture.md` for system design and component interactions)
- Requirements specifications (see `requirements.md` for functional/non-functional requirements)
- Warsaw pilot specifications (see `../pilot/spec.md` for 28 pilot features)
- Business strategy and market analysis (see `/business/strategy/` for GTM and positioning)
- Competitor analysis (see `/research/competitors/` for competitive landscape)
- Capability specifications (see `/spec/capability_*.md` for detailed feature specs)

---

## Business Domain Concepts

### Lead (Lead Generation)

**Definition:** A potential customer who has shown interest in a company's products or services, typically identified through contact information collection or engagement signals.

**In Business Context:**

A "lead" is the fundamental unit of B2B (business-to-business) and B2C (business-to-consumer) sales pipelines. Leads represent potential revenue opportunities that sales teams nurture into paying customers.

**Lead Lifecycle:**
```
Raw Contact → Qualified Lead → Marketing Qualified Lead (MQL)
→ Sales Qualified Lead (SQL) → Opportunity → Customer
```

**Lead Components:**

1. **Contact Information** (Required)
   - Name: "John Smith"
   - Email: john.smith@acme-corp.com
   - Phone: +1-555-0123
   - Company: "Acme Corporation"
   - Title: "VP of Engineering"

2. **Firmographic Data** (B2B specific)
   - Company size: 500-1000 employees
   - Industry: SaaS, Financial Services
   - Revenue: $50M-$100M ARR
   - Location: San Francisco, CA
   - Technology stack: AWS, Kubernetes, Python

3. **Behavioral Data** (Engagement signals)
   - Downloaded whitepaper on "AI Safety"
   - Attended webinar on "Enterprise AI Deployment"
   - Visited pricing page 3 times
   - Opened 5 marketing emails in last 30 days

4. **Lead Score** (Qualification metric)
   - Score: 85/100 (high-quality lead)
   - Factors: Job title (20 pts), Company size (15 pts), Engagement (50 pts)
   - Classification: "Hot lead" (ready for sales outreach)

**Lead Generation Methods:**

1. **Manual Lead Generation** (Traditional, slow)
   - Sales rep searches LinkedIn: 10-20 leads/hour
   - Manually enriches data from company websites
   - Copy-pastes into CRM (Salesforce, HubSpot)
   - **Cost:** ~$5-10 per lead (labor hours)
   - **Quality:** High (human judgment) but inconsistent

2. **Automated Lead Generation** (AI agents)
   - Agent scrapes LinkedIn: 500-1000 leads/hour
   - Automatically enriches from 10+ data sources
   - Direct CRM integration via API
   - **Cost:** ~$0.20-$0.50 per lead (API costs + compute)
   - **Quality:** Consistent, rules-based filtering

**Why Lead Generation for Iron Cage Demo?**

Lead generation is the **perfect demo use case** because:

1. **High Enterprise Value:**
   - B2B companies spend $200-$500 per qualified lead (Gartner 2024)
   - A 50% cost reduction = massive ROI ($100-250 saved per lead)
   - At 10,000 leads/month, that's $1M-2.5M annual savings

2. **Clear Success Metrics:**
   - Cost per lead: $0.23 (Iron Cage optimized) vs $0.87 (baseline Python agent)
   - Leads per hour: 500 (with caching) vs 200 (without)
   - Error rate: 0.5% (with guardrails) vs 5% (without)

3. **Demonstrates All Three Value Props:**
   - **Cost Control:** Token counting shows exact cost per lead in real-time
   - **Safety:** PII filtering prevents leaking customer data, action whitelisting blocks unauthorized LinkedIn scraping
   - **Reliability:** safety cutoffs handle LinkedIn rate limits, fallback chains ensure 99.5% success rate

**Example Lead Generation Agent Flow:**

```
┌─────────────────────────────────────────────────────────┐
│  Step 1: Search LinkedIn                                │
│  Input: "VP of Engineering at Series B SaaS companies"  │
│  Output: 100 LinkedIn profile URLs                      │
│  Cost: $0.05 (LLM parses search results)               │
└─────────────────┬───────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────┐
│  Step 2: Scrape Profile (per lead)                      │
│  Input: https://linkedin.com/in/john-smith              │
│  Output: Name, Title, Company, Location                 │
│  Cost: $0.02 (LLM extracts structured data)            │
│  Safety Check: Rate limit (max 10 requests/second)     │
└─────────────────┬───────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────┐
│  Step 3: Enrich with Company Data                       │
│  Input: "Acme Corporation"                              │
│  Tool: Clearbit API (external enrichment service)       │
│  Output: Employee count, revenue, tech stack            │
│  Cost: $0.10 (Clearbit API fee)                        │
│  Safety Check: Action whitelist (Clearbit API allowed) │
└─────────────────┬───────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────┐
│  Step 4: Lead Scoring                                   │
│  Input: All collected data                              │
│  Output: Score 0-100, qualification decision            │
│  Cost: $0.03 (LLM-based scoring logic)                 │
│  Safety Check: Output filtering (no PII in logs)       │
└─────────────────┬───────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────┐
│  Step 5: CRM Integration                                │
│  Input: Structured lead data                            │
│  Tool: Salesforce API (create new lead record)          │
│  Output: Lead ID in CRM                                 │
│  Cost: $0.03 (LLM formats API payload)                 │
│  Safety Check: privacy protection (ensure proper storage)   │
└─────────────────────────────────────────────────────────┘

Total Cost Per Lead: $0.23 (5 LLM calls + 2 API calls)
Total Time Per Lead: 2.3 seconds (with caching)
Success Rate: 99.2% (with circuit breakers + fallbacks)
```

**Iron Cage Runtime Monitoring Control Panel (Live Demo):**

```
┌──────────────────────────────────────────────────────────┐
│  Lead Generation Agent - Real-Time Monitoring            │
├──────────────────────────────────────────────────────────┤
│  Status: ✅ Running (99.2% uptime)                       │
│  Leads Processed: 487 / 500 target (97.4%)              │
│  Current Rate: 211 leads/hour                            │
│                                                           │
│  💰 COST CONTROL                                         │
│  ├─ Cost per lead: $0.23 (target: <$0.30) ✅            │
│  ├─ Total cost today: $112.01 / $150 budget (74.7%)     │
│  ├─ Projected month: $3,360 (under $5K limit) ✅        │
│  └─ Savings vs baseline: $311.68 (73.5% reduction)      │
│                                                           │
│  🛡️ SAFETY VIOLATIONS                                   │
│  ├─ PII leaks blocked: 3 (email addresses in logs)      │
│  ├─ Unauthorized actions: 1 (attempted Facebook scrape) │
│  ├─ Rate limit hits: 12 (LinkedIn throttling)           │
│  └─ Prompt injections: 0                                 │
│                                                           │
│  ⚡ PERFORMANCE                                          │
│  ├─ Latency P50: 1.8s, P95: 3.2s, P99: 5.1s            │
│  ├─ Success rate: 99.2% (487 success / 491 attempts)    │
│  ├─ Cache hit rate: 34% (saved $38.47)                  │
│  └─ safety cutoff trips: 2 (LinkedIn API, recovered)  │
└──────────────────────────────────────────────────────────┘
```

**Business Impact (Shown in Talk):**

- **Before Iron Cage:** $0.87/lead × 10,000 leads/month = $8,700/month
- **After Iron Cage:** $0.23/lead × 10,000 leads/month = $2,300/month
- **Monthly Savings:** $6,400 (73.5% reduction)
- **Annual Savings:** $76,800

**Plus:**
- 3 data breaches prevented (PII leaks)
- 1 unauthorized data source blocked (compliance violation avoided)
- 99.2% uptime (vs 94% with Python-only agent)

---

## Reliability Design Patterns

### Safety Cutoff

**Definition:** A design pattern that prevents an application from repeatedly attempting an operation that's likely to fail, allowing it to continue operating instead of waiting for a timeout or failing catastrophically.

**The Problem: Cascade Failures**

Imagine your AI agent depends on 3 external services:
1. LinkedIn API (for profile scraping)
2. Clearbit API (for company enrichment)
3. Salesforce API (for CRM storage)

**Scenario: LinkedIn API Goes Down**

Without circuit breakers:
```
Agent makes request → LinkedIn API timeout (30 seconds)
Agent retries → Another timeout (30 seconds)
Agent retries again → Another timeout (30 seconds)

Result per lead: 90 seconds wasted, agent stuck, queue backs up
With 100 concurrent requests: All 100 agents frozen
System-wide impact: Complete system freeze, zero leads processed
```

With circuit breaker:
```
Agent makes request → LinkedIn API timeout (30 seconds)
Circuit opens → Future requests fail immediately (0 seconds)
Agent uses fallback → Switch to alternative data source

Result per lead: 30 seconds for first failure, then instant fallback
With 100 concurrent requests: 99 agents switch to fallback immediately
System-wide impact: Reduced throughput but system remains functional
```

**Safety Cutoff State Machine:**

```
┌─────────────────────────────────────────────────────────┐
│                        CLOSED                            │
│  (Normal operation, all requests sent to service)        │
│                                                           │
│  Success count: 147                                      │
│  Failure count: 2                                        │
│  Error rate: 1.3% (< 10% threshold)                     │
│                                                           │
│  Condition to OPEN:                                      │
│  - 5 consecutive failures OR                             │
│  - Error rate >10% over 60 seconds                       │
└────────────────┬────────────────────────────────────────┘
                 │
                 │ (Threshold breached)
                 ▼
┌─────────────────────────────────────────────────────────┐
│                         OPEN                             │
│  (Fail fast, reject all requests immediately)            │
│                                                           │
│  Requests rejected: 234                                  │
│  Time in OPEN state: 28 seconds                          │
│  Next retry attempt: 32 seconds                          │
│                                                           │
│  Condition to HALF-OPEN:                                 │
│  - Wait 60 seconds (cooldown period)                     │
└────────────────┬────────────────────────────────────────┘
                 │
                 │ (Cooldown complete)
                 ▼
┌─────────────────────────────────────────────────────────┐
│                      HALF-OPEN                           │
│  (Test if service recovered)                             │
│                                                           │
│  Test requests: 3 / 5                                    │
│  Success: 3, Failures: 0                                 │
│                                                           │
│  Condition to CLOSED:                                    │
│  - 5 consecutive successes                               │
│                                                           │
│  Condition to OPEN:                                      │
│  - Any single failure                                    │
└────────────────┬────────────────────────────────────────┘
                 │
                 │ (5 successes)
                 ▼
           (Back to CLOSED state)
```

**Implementation Example (Rust):**

```rust
pub struct CircuitBreaker
{
  state : Arc< RwLock< CircuitState > >,
  failure_threshold : usize,        // Open after N failures
  success_threshold : usize,        // Close after N successes
  timeout : Duration,               // Cooldown period
  error_rate_threshold : f64,       // Open if error rate >X%
  error_rate_window : Duration,     // Calculate rate over Y seconds
}

pub enum CircuitState
{
  Closed
  {
    failure_count : usize,
    success_count : usize,
    recent_errors : VecDeque< Instant >,
  },
  Open
  {
    opened_at : Instant,
  },
  HalfOpen
  {
    test_success_count : usize,
    test_failure_count : usize,
  },
}

impl CircuitBreaker
{
  pub async fn call< F, T >( &self, operation : F ) -> Result< T >
  where
    F : Future< Output = Result< T > >,
  {
    match self.state()
    {
      CircuitState::Closed =>
      {
        match operation.await
        {
          Ok( result ) =>
          {
            self.record_success();
            Ok( result )
          }
          Err( e ) =>
          {
            self.record_failure();
            Err( e )
          }
        }
      }
      CircuitState::Open =>
      {
        // Fail fast without calling operation
        Err( Error::CircuitBreakerOpen )
      }
      CircuitState::HalfOpen =>
      {
        match operation.await
        {
          Ok( result ) =>
          {
            self.record_test_success();
            Ok( result )
          }
          Err( e ) =>
          {
            self.record_test_failure();
            Err( e )
          }
        }
      }
    }
  }
}
```

**Real-World Example: LinkedIn API Rate Limiting**

LinkedIn limits API requests to:
- 100 requests per hour per user token
- 10 requests per second burst

**Without Safety Cutoff:**
```
Time 0:00 - Agent sends 50 requests in 5 seconds (10/sec burst)
Time 0:05 - LinkedIn returns 429 Too Many Requests
Time 0:05 - Agent waits 60 seconds (timeout)
Time 1:05 - Agent retries, still rate limited
Time 2:05 - Agent retries again, still rate limited
...
Time 60:00 - Rate limit resets, agent finally succeeds

Result: 1 hour of waiting for 50 leads (useless)
```

**With Safety Cutoff:**
```
Time 0:00 - Agent sends 50 requests in 5 seconds
Time 0:05 - LinkedIn returns 429 (rate limited)
Time 0:05 - Circuit opens immediately
Time 0:05 - Agent switches to fallback (cached data or alternative source)
Time 0:06 - Remaining 450 leads processed using fallback (no waiting)

Result: 450 leads processed in 10 minutes (acceptable degradation)
```

**Enterprise Value:**

- **Cost Savings:** Avoid $50-100 in wasted LLM API calls while waiting for timeouts
- **Throughput:** Process 90% of leads instead of 0% during outages
- **User Experience:** 2-second response time vs 60-second timeout
- **Observability:** Control Panel shows "LinkedIn circuit OPEN (recovered 450/500 leads via fallback)"

---

### Fallback Chain

**Definition:** A sequence of alternative strategies to accomplish a task, tried in priority order until one succeeds or all fail.

**The Hierarchy of Reliability:**

```
┌─────────────────────────────────────────────────────────┐
│  Tier 1: PRIMARY (Best quality, highest cost)            │
│  ├─ LLM: GPT-4-Turbo (OpenAI)                           │
│  ├─ Cost: $0.10 per 1K tokens                           │
│  ├─ Latency: 500ms average                              │
│  ├─ Success rate: 99.5%                                  │
│  └─ Use case: Complex reasoning, high-value tasks       │
└────────────────┬────────────────────────────────────────┘
                 │ (If OpenAI API fails or timeout >2s)
                 ▼
┌─────────────────────────────────────────────────────────┐
│  Tier 2: SECONDARY (Good quality, medium cost)          │
│  ├─ LLM: GPT-4 (Azure OpenAI)                           │
│  ├─ Cost: $0.08 per 1K tokens                           │
│  ├─ Latency: 700ms average                              │
│  ├─ Success rate: 99.3%                                  │
│  └─ Use case: Same quality, different provider          │
└────────────────┬────────────────────────────────────────┘
                 │ (If Azure OpenAI fails)
                 ▼
┌─────────────────────────────────────────────────────────┐
│  Tier 3: TERTIARY (Acceptable quality, low cost)        │
│  ├─ LLM: Claude 3 Sonnet (Anthropic)                    │
│  ├─ Cost: $0.06 per 1K tokens                           │
│  ├─ Latency: 600ms average                              │
│  ├─ Success rate: 99.4%                                  │
│  └─ Use case: Different architecture, good fallback     │
└────────────────┬────────────────────────────────────────┘
                 │ (If Anthropic fails)
                 ▼
┌─────────────────────────────────────────────────────────┐
│  Tier 4: EMERGENCY (Basic quality, minimal cost)        │
│  ├─ LLM: Llama 3.1 8B (Local, self-hosted)             │
│  ├─ Cost: $0.001 per 1K tokens (compute only)          │
│  ├─ Latency: 300ms average (local GPU)                  │
│  ├─ Success rate: 95% (less capable model)              │
│  └─ Use case: Always available, no external dependency  │
└────────────────┬────────────────────────────────────────┘
                 │ (If local model fails)
                 ▼
┌─────────────────────────────────────────────────────────┐
│  Tier 5: FINAL FALLBACK (Degraded experience)           │
│  ├─ Strategy: Return cached response from previous run  │
│  ├─ Cost: $0 (no API calls)                             │
│  ├─ Latency: 10ms (database lookup)                     │
│  ├─ Freshness: Data may be stale (minutes to hours old) │
│  └─ Use case: Better than complete failure              │
└─────────────────────────────────────────────────────────┘
```

**Example: Lead Enrichment Fallback Chain**

**Task:** Get company employee count for "Acme Corporation"

```rust
async fn get_employee_count( company_name : &str ) -> Result< u32 >
{
  // Tier 1: Clearbit API (most accurate, paid)
  match clearbit_api.get_company( company_name ).await
  {
    Ok( data ) => return Ok( data.employee_count ),
    Err( e ) =>
    {
      warn!( "Clearbit failed: {}, trying fallback", e );
    }
  }

  // Tier 2: LinkedIn scraping (free, less reliable)
  match scrape_linkedin_company_page( company_name ).await
  {
    Ok( data ) => return Ok( data.employee_count_estimate ),
    Err( e ) =>
    {
      warn!( "LinkedIn failed: {}, trying fallback", e );
    }
  }

  // Tier 3: LLM-based estimation (creative, may hallucinate)
  let prompt = format!(
    "Estimate employee count for {}. Return only a number.",
    company_name
  );
  match llm_call( prompt ).await
  {
    Ok( estimate ) => return Ok( estimate.parse()? ),
    Err( e ) =>
    {
      warn!( "LLM failed: {}, trying fallback", e );
    }
  }

  // Tier 4: Database cache (stale but better than nothing)
  match db.get_cached_employee_count( company_name ).await
  {
    Ok( cached ) =>
    {
      warn!( "Using cached data (age: {} days)", cached.age_days );
      return Ok( cached.value );
    }
    Err( e ) =>
    {
      warn!( "Cache failed: {}, no more fallbacks", e );
    }
  }

  // Tier 5: Hard-coded default (last resort)
  warn!( "All fallbacks exhausted, using industry average" );
  Ok( 250 ) // Default: Assume mid-size company
}
```

**Observability in Control Panel:**

```
┌──────────────────────────────────────────────────────────┐
│  Lead Enrichment - Fallback Chain Performance            │
├──────────────────────────────────────────────────────────┤
│  Total Enrichments: 487                                  │
│                                                           │
│  Tier 1 (Clearbit):     412 (84.6%) ✅ Primary worked   │
│  Tier 2 (LinkedIn):      58 (11.9%) ⚠️ Some failures    │
│  Tier 3 (LLM):           12 (2.5%)  ⚠️ Both APIs down   │
│  Tier 4 (Cache):          3 (0.6%)  🔴 Critical fallback│
│  Tier 5 (Default):        2 (0.4%)  🔴 Emergency mode   │
│                                                           │
│  Cost Breakdown:                                         │
│  ├─ Clearbit API: $41.20 (412 × $0.10)                  │
│  ├─ LinkedIn: $0 (free scraping)                         │
│  ├─ LLM calls: $0.36 (12 × $0.03)                       │
│  ├─ Cache: $0                                            │
│  └─ Total: $41.56 (avg $0.085/lead)                     │
│                                                           │
│  Savings vs Primary-Only:                                │
│  Without fallbacks: $48.70 (487 × $0.10, assumes 100%   │
│                     Clearbit success)                    │
│  With fallbacks: $41.56                                  │
│  Saved: $7.14 (14.7% reduction) ✅                       │
└──────────────────────────────────────────────────────────┘
```

**Why Fallback Chains Matter for Enterprises:**

1. **Cost Optimization:**
   - Try cheaper tiers first when quality acceptable
   - Example: Use GPT-3.5 ($0.002/1K tokens) for simple tasks, GPT-4 ($0.10/1K tokens) only when needed
   - Potential savings: 40-60% on LLM costs

2. **Availability:**
   - No single point of failure
   - Example: OpenAI outage (happened Dec 2024) → Automatically switch to Anthropic
   - Uptime improvement: 99.5% (single provider) → 99.95% (3-tier fallback)

3. **Geographic Compliance:**
   - Tier 1: US-hosted LLM (default)
   - Tier 2: EU-hosted LLM (GDPR requirement for EU customers)
   - Tier 3: On-premise model (air-gapped networks)

4. **Quality vs Speed Tradeoff:**
   - Tier 1: Best quality, 2s latency
   - Tier 2: Good quality, 500ms latency (for time-sensitive requests)
   - Tier 3: Basic quality, 100ms latency (for real-time responses)

**Advanced: Smart Fallback Selection**

Instead of always trying tiers in order, use ML to predict best tier:

```rust
pub struct SmartFallbackChain
{
  tiers : Vec< Tier >,
  success_history : HashMap< TierId, SuccessRate >,
  latency_history : HashMap< TierId, P95Latency >,
}

impl SmartFallbackChain
{
  pub async fn execute< T >( &self, task : Task ) -> Result< T >
  {
    // Predict best tier based on recent performance
    let ranked_tiers = self.rank_tiers_by_success_rate();

    for tier in ranked_tiers
    {
      // Skip tier if known to be down (circuit breaker open)
      if tier.circuit_breaker.is_open()
      {
        continue;
      }

      match tier.execute( &task ).await
      {
        Ok( result ) =>
        {
          self.record_success( tier.id );
          return Ok( result );
        }
        Err( e ) =>
        {
          self.record_failure( tier.id );
          warn!( "Tier {} failed: {}, trying next", tier.id, e );
        }
      }
    }

    Err( Error::AllTiersFailed )
  }

  fn rank_tiers_by_success_rate( &self ) -> Vec< Tier >
  {
    let mut tiers = self.tiers.clone();
    tiers.sort_by_key( | tier |
    {
      let success_rate = self.success_history
        .get( &tier.id )
        .map( | r | r.rate )
        .unwrap_or( 1.0 );

      // Sort by success rate DESC, then by cost ASC
      ( OrderedFloat( -success_rate ), tier.cost )
    } );
    tiers
  }
}
```

**Result:**

- If Azure OpenAI is 99.9% successful today but OpenAI is 97%, use Azure first
- If local model has 300ms latency vs 2s for cloud, prefer local for time-sensitive requests
- Automatically adapt to current conditions without manual intervention

---

## Agent Tool Execution Patterns

### Tool (Agent Tool)

**Definition:** A callable function or service that an AI agent can invoke to perform actions in the real world, such as reading files, calling APIs, querying databases, or sending emails.

**Why Tools Matter for AI Agents:**

AI agents are not just conversation models - they need to take actions to be useful. Tools are the bridge between LLM reasoning and real-world impact.

**Tool Lifecycle:**
```
Define Tool → Register with Iron Cage → Wrap in ToolProxy
→ Agent Calls Tool → Iron Cage Validates → Execute → Return Result
```

**Tool Categories:**

1. **File Operations**
   - `read_file(path)` - Read file contents
   - `write_file(path, content)` - Write to file
   - `delete_file(path)` - Delete file (high-risk)
   - `list_directory(path)` - List files in directory

2. **API Calls**
   - `http_request(url, method, payload)` - HTTP API calls
   - `graphql_query(endpoint, query)` - GraphQL queries
   - `websocket_connect(url)` - WebSocket connections

3. **Database Queries**
   - `execute_query(sql)` - Run SQL query
   - `execute_dml(table, operation)` - INSERT/UPDATE/DELETE
   - `execute_ddl(schema_change)` - CREATE/DROP/ALTER (high-risk)

4. **Communication**
   - `send_email(to, subject, body)` - Send email
   - `post_slack_message(channel, text)` - Slack notification
   - `send_sms(phone, message)` - SMS notification

5. **System Operations**
   - `run_command(cmd)` - Execute shell command (very high-risk)
   - `start_process(executable, args)` - Start background process
   - `kill_process(pid)` - Stop running process

**Example Tool Definition (LangChain):**

```python
from langchain.tools import Tool

# Define a simple tool
calculator = Tool(
  name="Calculator",
  func=lambda expr: eval(expr),  # ⚠️ Dangerous without validation!
  description="Evaluate mathematical expressions"
)

# Define a complex tool with error handling
def read_customer_data(customer_id: str) -> dict:
  if not customer_id.isdigit():
    raise ValueError("Invalid customer ID")

  conn = psycopg2.connect(DATABASE_URL)
  cursor = conn.execute(
    "SELECT * FROM customers WHERE id = %s",
    (customer_id,)
  )
  return cursor.fetchone()

customer_reader = Tool(
  name="ReadCustomer",
  func=read_customer_data,
  description="Fetch customer data by ID"
)
```

**Why Tools Need Oversight:**

Without Iron Cage:
```python
# ❌ DANGEROUS: Agent can call any tool without validation
agent = Agent(llm="gpt-4", tools=[delete_file, run_command])
agent.run("Clean up the database")
# Agent might execute: run_command("rm -rf /production/db")
```

With Iron Cage:
```python
# ✅ SAFE: Iron Cage validates all tool calls
cage = IronCageClient()
agent_id = cage.register_agent(
  tools=[read_file, send_email],
  authorization={
    "denied_tools": ["delete_file", "run_command"]  # Blocked
  }
)
# Agent CANNOT call delete_file - Iron Cage rejects it
```

---

### Action (Agent Action)

**Definition:** A specific invocation of a tool with concrete parameters, representing a discrete operation that an agent performs in the real world.

**Action vs Tool:**
- **Tool:** The capability (e.g., "send_email")
- **Action:** A specific use (e.g., "send_email(to='ceo@company.com', subject='Q4 Results')")

**Action Anatomy:**

```json
{
  "action_id": "action-12345",
  "tool_name": "send_email",
  "tool_args": {
    "to": "sales@acme-corp.com",
    "subject": "Follow-up: Iron Cage Demo",
    "body": "Hi team, here's the pricing..."
  },
  "agent_id": "agent-67890",
  "timestamp": "2025-01-17T10:34:52Z",
  "authorization_status": "pending_approval",  // human-in-loop
  "risk_level": "high"
}
```

**Action Lifecycle:**

```
Agent Decides Action → Iron Cage Validates → (Human Approval if needed)
→ Execute Tool → Log Result → Return to Agent
```

**Action Risk Levels:**

| Risk Level | Examples | Oversight |
|-----------|----------|-----------|
| **Low** | `read_file`, `http_get`, `SELECT query` | Automatic approval |
| **Medium** | `write_file`, `INSERT query`, `send_internal_email` | Rate limiting + audit |
| **High** | `delete_file`, `send_external_email`, `payment_api` | Human approval required |
| **Critical** | `run_command`, `DROP table`, `production_deploy` | Blocked by default |

**Example: Human-in-Loop Action:**

```python
# Agent wants to send email to external customer
action = {
  "tool_name": "send_email",
  "args": {
    "to": "customer@external.com",  # ← External domain
    "subject": "Special discount offer",
    "body": "Get 50% off today only!"
  }
}

# Iron Cage detects high-risk action
# Sends approval request to Slack:
"""
@sales-team: Agent wants to send email to customer@external.com
Subject: Special discount offer
Body: Get 50% off today only!

Approve? [✅ Yes] [❌ No]
Auto-denies in 5 minutes if no response.
"""

# Human approves → action executes
# Human denies → action blocked, agent informed
```

**Action Audit Trail:**

Every action is logged for compliance:

```json
{
  "event_type": "action_executed",
  "action_id": "action-12345",
  "agent_id": "agent-67890",
  "user_id": "user-54321",
  "tool_name": "send_email",
  "tool_args": { "to": "...", "subject": "..." },
  "authorization_decision": "approved",
  "approved_by": "user-99999",
  "approval_timestamp": "2025-01-17T10:35:12Z",
  "execution_timestamp": "2025-01-17T10:35:15Z",
  "execution_duration_ms": 142,
  "result": "success",
  "pii_detected": false
}
```

---

### Tool Proxy

**Definition:** A wrapper layer that Iron Cage places around agent tools to intercept, validate, and control tool executions before delegating to the original tool implementation.

**Problem Solved:**

Without Tool Proxy:
```
Agent → Tool → Direct Execution (no oversight, no audit, no safety)
```

With Tool Proxy:
```
Agent → Tool Proxy → Validation → Original Tool → Validation → Result
         ├─ Authorization check
         ├─ Parameter validation
         ├─ Rate limiting
         ├─ Audit logging
         └─ Output scanning
```

**Tool Proxy Architecture:**

```rust
pub struct ToolProxy {
  tool_id: String,                    // "langchain:file_ops:read_file"
  tool_name: String,                  // "Read File"
  original_tool: Box<dyn Tool>,       // LangChain/CrewAI tool
  authorization: AuthorizationPolicy, // Whitelist/blacklist
  rate_limiter: RateLimiter,          // Quotas (100 calls/hr)
  audit_logger: AuditLogger,          // Compliance logs
}

impl ToolProxy {
  pub async fn execute(&self, args: ToolArgs) -> Result<ToolResult> {
    // 1. BEFORE: Validate before execution
    self.validate_authorization(&args)?;
    self.validate_parameters(&args)?;
    self.check_rate_limit().await?;
    audit_log("tool_call_start", &self.tool_id, &args);

    // 2. EXECUTE: Delegate to original tool
    let result = self.original_tool.run(args).await?;

    // 3. AFTER: Validate result
    self.scan_for_pii(&result)?;
    self.scan_for_secrets(&result)?;
    audit_log("tool_call_success", &self.tool_id, &result);

    Ok(result)
  }
}
```

**Tool Registration Flow:**

```python
from langchain.tools import Tool
from iron_cage import IronCageClient

# 1. Define tool (standard LangChain)
file_reader = Tool(
  name="read_file",
  func=lambda path: open(path).read(),
  description="Read file contents"
)

# 2. Register with Iron Cage (wraps in ToolProxy)
cage = IronCageClient()
agent_id = cage.register_agent(
  name="my-agent",
  tools=[file_reader],  # ← Iron Cage wraps this in ToolProxy
  authorization={
    "allowed_tools": ["read_file"],
    "denied_tools": ["delete_file"]
  }
)

# 3. Agent calls tool (Iron Cage intercepts)
result = cage.run_tool(
  agent_id=agent_id,
  tool_name="read_file",
  args={"file_path": "/data/leads.csv"}
)
# Iron Cage ToolProxy:
# - Checks if "read_file" is in allowed_tools ✅
# - Validates path (no ".." traversal) ✅
# - Executes original tool ✅
# - Scans output for PII ✅
# - Logs to audit trail ✅
```

**Validation Layers in ToolProxy:**

```rust
fn validate_parameters(&self, args: &ToolArgs) -> Result<()> {
  match self.tool_id.as_str() {
    "file_ops:read_file" => {
      let path = args.get("file_path")?;

      // Prevent path traversal
      if path.contains("..") {
        return Err("Path traversal attack detected");
      }

      // Prevent restricted directories
      if path.starts_with("/etc") || path.starts_with("/root") {
        return Err("Access to restricted directory denied");
      }

      // Enforce file size limits
      let metadata = std::fs::metadata(path)?;
      if metadata.len() > 100 * 1024 * 1024 {  // 100 MB
        return Err("File too large (max 100 MB)");
      }
    }

    "api:http_request" => {
      let url = args.get("url")?;

      // Whitelist allowed domains
      let allowed_domains = ["api.openai.com", "api.anthropic.com"];
      if !allowed_domains.iter().any(|d| url.contains(d)) {
        return Err("Domain not whitelisted");
      }
    }

    _ => {}
  }
  Ok(())
}
```

**Benefits of Tool Proxy:**

1. **Zero Code Changes:** Agents keep using LangChain/CrewAI tools as-is
2. **Defense in Depth:** Every tool call validated (even if LLM bypasses prompt injection checks)
3. **Compliance:** 100% audit trail for SOC 2, HIPAA, GDPR
4. **Cost Control:** Rate limiting prevents runaway API costs
5. **Safety:** privacy protection on tool outputs (not just LLM outputs)

---

### Sandbox (Execution Sandbox)

**Definition:** An isolated execution environment where server-side agent tools run with strict resource limits, syscall restrictions, and network isolation to prevent malicious or runaway code from affecting the host system.

**Problem Solved:**

Without Sandbox (client-side execution):
```
Agent code runs on user's laptop → Trust user environment
Tool calls execute with user's permissions → Risk controlled by user
```

Without Sandbox (server-side execution):
```
❌ DANGEROUS: Agent code runs on Iron Cage servers with full permissions
Agent could:
- Fork bomb (kill server)
- Allocate 100 GB memory (OOM crash)
- Execute "rm -rf /" (delete all data)
- Exfiltrate secrets via network
```

With Sandbox (server-side execution):
```
✅ SAFE: Agent code runs in isolated container with:
- CPU quota (max 2 cores)
- Memory limit (max 1 GB)
- Disk quota (100 MB in /tmp only)
- Syscall whitelist (block fork, exec, chroot)
- Network isolation (no internet or whitelisted domains only)
```

**Sandbox Architecture (Linux):**

Iron Cage uses **cgroups + seccomp + network namespaces** for sandboxing:

```rust
pub struct SandboxedExecutor {
  resource_limits: ResourceLimits,
  allowed_syscalls: Vec<Syscall>,
  network_policy: NetworkPolicy,
}

impl SandboxedExecutor {
  pub async fn execute_tool(&self, code: &str, args: ToolArgs)
    -> Result<ToolResult>
  {
    // 1. CREATE CGROUP (resource limits)
    create_cgroup(
      cpu_quota: 200_000,      // 2 cores = 200% of 1 CPU
      memory_limit: 1_073_741_824,  // 1 GB
      pids_limit: 100,         // Max 100 processes
    )?;

    // 2. APPLY SECCOMP FILTER (syscall whitelist)
    let allowed = vec![
      Syscall::Read, Syscall::Write, Syscall::Open, Syscall::Close,
      Syscall::Mmap, Syscall::Brk, Syscall::Futex,
    ];
    apply_seccomp_filter(&allowed)?;  // Block all other syscalls

    // 3. NETWORK NAMESPACE (isolation)
    if !self.network_policy.allow_internet {
      create_network_namespace(NetworkNamespace::Isolated)?;
    }

    // 4. EXECUTE CODE (in sandbox)
    let result = execute_python(code, args).await?;

    // 5. CLEANUP (destroy cgroup, kill all processes)
    cleanup_cgroup()?;

    Ok(result)
  }
}
```

**Resource Limits (cgroups v2):**

| Resource | Limit | Violation Behavior |
|----------|-------|-------------------|
| **CPU** | 2 cores (200%) | Throttled to 2 cores max |
| **Memory** | 1 GB | OOMKilled after 1 GB |
| **Disk** | 100 MB in `/tmp` | Write fails after quota |
| **Processes** | 100 PIDs | Fork fails after 100 processes |
| **Execution Time** | 60 seconds | Killed after timeout |

**Syscall Whitelist (seccomp):**

```rust
// Allowed syscalls (safe operations)
const ALLOWED_SYSCALLS: &[Syscall] = &[
  Syscall::Read,      // Read file
  Syscall::Write,     // Write file
  Syscall::Open,      // Open file
  Syscall::Close,     // Close file
  Syscall::Mmap,      // Memory map
  Syscall::Brk,       // Heap allocation
  Syscall::Futex,     // Thread synchronization
];

// Blocked syscalls (dangerous operations)
const BLOCKED_SYSCALLS: &[Syscall] = &[
  Syscall::Exec,      // Execute new program (code injection)
  Syscall::Fork,      // Create process (fork bomb)
  Syscall::Chroot,    // Change root (escape sandbox)
  Syscall::Mount,     // Mount filesystem (privilege escalation)
  Syscall::Reboot,    // Reboot system (DoS)
  Syscall::Ptrace,    // Attach debugger (escape sandbox)
];
```

**Network Isolation:**

```rust
pub enum NetworkPolicy {
  Isolated,                    // No network access
  WhitelistDomains(Vec<String>), // Only allowed domains
  FullAccess,                  // All domains (development only)
}

// Example: Allow only OpenAI API
let policy = NetworkPolicy::WhitelistDomains(vec![
  "api.openai.com".to_string(),
  "api.anthropic.com".to_string(),
]);

// Implemented via:
// - Network namespace (new netns)
// - Firewall rules (iptables)
// - DNS filtering (only resolve whitelisted domains)
```

**Sandbox Violation Handling:**

```rust
match executor.execute_tool(code, args).await {
  Err(ExecutorError::MemoryLimitExceeded) => {
    audit_log("sandbox_violation", json!({
      "agent_id": agent_id,
      "violation": "memory_limit_exceeded",
      "limit": "1 GB",
      "action": "OOMKilled"
    }));
    // Agent suspended, admin alerted
  }

  Err(ExecutorError::CPUQuotaExceeded) => {
    audit_log("sandbox_violation", json!({
      "violation": "cpu_quota_exceeded",
      "action": "throttled"
    }));
    // Agent continues but throttled to 2 cores
  }

  Err(ExecutorError::ForbiddenSyscall(syscall)) => {
    audit_log("sandbox_violation", json!({
      "violation": "forbidden_syscall",
      "syscall": syscall,  // e.g., "exec"
      "action": "killed"
    }));
    // Agent killed immediately, marked as malicious
  }
}
```

**Sandbox Use Cases:**

| Deployment Mode | Sandboxing | Rationale |
|----------------|-----------|-----------|
| **Client-Side (Model A)** | No | Agent runs on user's machine, user controls environment |
| **Server-Side (Model B)** | Yes | Agent runs on Iron Cage servers, must protect infrastructure |

**Example: Server-Side Agent Upload:**

```python
# Upload agent to Iron Cage (runs 24/7 in K8s)
cage = IronCageClient(api_url="https://ironcage.company.com")

agent_id = cage.upload_agent(
  code=open("lead_gen_agent.py").read(),
  tools=[scrape_web, send_email],
  execution_mode="server",  # ← Runs in sandbox
  resource_limits={
    "cpu_cores": 1,
    "memory_gb": 0.5,
    "disk_mb": 50,
    "timeout_seconds": 30
  },
  network_policy={
    "allow_internet": True,
    "whitelisted_domains": ["linkedin.com", "clearbit.com"]
  }
)

# Iron Cage creates sandbox with:
# - 1 CPU core
# - 512 MB RAM
# - 50 MB disk quota
# - 30 second timeout per tool call
# - Network access only to linkedin.com and clearbit.com
```

**Security Benefits:**

1. **Prevents Resource Exhaustion:** Fork bombs, memory leaks can't crash server
2. **Prevents Privilege Escalation:** No `chroot`, `mount`, `ptrace` syscalls
3. **Prevents Data Exfiltration:** Network isolation blocks unauthorized API calls
4. **Prevents Code Injection:** No `exec`, `fork` means can't run arbitrary binaries
5. **Compliance:** Isolated execution required for SOC 2, HIPAA multi-tenancy

---

## Privacy and Compliance

### PII (Personally Identifiable Information)

**Definition:** Any data that can be used to identify, contact, or locate a specific individual, either alone or when combined with other information.

**Why PII Matters for AI Agents:**

AI agents process vast amounts of data, often including sensitive personal information. Leaking PII can result in:
1. **Legal Penalties:** GDPR fines up to €20M or 4% of global revenue (whichever is higher)
2. **Reputational Damage:** Customer trust destroyed, PR crisis
3. **Compliance Violations:** Loss of SOC 2, HIPAA, PCI-DSS certifications
4. **Customer Churn:** Enterprise customers terminate contracts immediately

**Categories of PII:**

```
┌─────────────────────────────────────────────────────────┐
│  TIER 1: DIRECT IDENTIFIERS (High Risk)                 │
│  Can identify individual without additional data         │
├─────────────────────────────────────────────────────────┤
│  ├─ Full name: "John Smith"                             │
│  ├─ Email: john.smith@example.com                       │
│  ├─ Phone: +1-555-0123, (555) 555-0123                  │
│  ├─ SSN: 123-45-6789                                    │
│  ├─ Passport number: US123456789                        │
│  ├─ Driver's license: CA-D1234567                       │
│  ├─ IP address: 192.168.1.100                           │
│  ├─ MAC address: 00:1B:44:11:3A:B7                      │
│  ├─ Device ID: UDID-1234-5678-ABCD                      │
│  └─ Biometrics: Fingerprints, facial recognition data   │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  TIER 2: QUASI-IDENTIFIERS (Medium Risk)                │
│  Can identify when combined with other data              │
├─────────────────────────────────────────────────────────┤
│  ├─ Date of birth: 1985-03-15                           │
│  ├─ Home address: 123 Main St, San Francisco, CA 94102  │
│  ├─ ZIP code: 94102 (especially in low-population areas)│
│  ├─ Age: 38 years old                                   │
│  ├─ Gender: Male                                        │
│  ├─ Race/Ethnicity: Asian                               │
│  ├─ Job title: "VP of Engineering"                      │
│  ├─ Employer: "Acme Corporation"                        │
│  ├─ Education: "Stanford University, Class of 2007"     │
│  └─ Salary: $250,000/year                               │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  TIER 3: SENSITIVE PERSONAL DATA (Highest Risk)         │
│  Special category data requiring extra protection        │
├─────────────────────────────────────────────────────────┤
│  ├─ Financial: Credit card (4532-1234-5678-9010),       │
│  │   Bank account (123456789), Investment portfolio     │
│  ├─ Health (PHI): Medical records, diagnoses,           │
│  │   prescriptions, insurance claims                    │
│  ├─ Biometric: DNA, fingerprints, iris scans            │
│  ├─ Political: Party affiliation, voting records        │
│  ├─ Religious: Beliefs, congregation membership         │
│  ├─ Sexual: Orientation, gender identity               │
│  ├─ Criminal: Arrest records, convictions               │
│  └─ Children: Any data from users <16 (GDPR) or <13    │
│      (COPPA in US)                                      │
└─────────────────────────────────────────────────────────┘
```

**Privacy Protection Techniques:**

**1. Regex Pattern Matching (Fast, 90% accuracy)**

```rust
pub struct PiiDetector
{
  patterns : Vec< PiiPattern >,
}

pub struct PiiPattern
{
  name : String,
  regex : Regex,
  risk_level : RiskLevel,
}

impl PiiDetector
{
  pub fn new() -> Self
  {
    Self
    {
      patterns : vec!
      [
        // Email addresses
        PiiPattern
        {
          name : "Email".into(),
          regex : Regex::new(
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"
          ).unwrap(),
          risk_level : RiskLevel::High,
        },

        // US Social Security Numbers
        PiiPattern
        {
          name : "SSN".into(),
          regex : Regex::new( r"\b\d{3}-\d{2}-\d{4}\b" ).unwrap(),
          risk_level : RiskLevel::Critical,
        },

        // Credit Card Numbers (Luhn algorithm validation needed)
        PiiPattern
        {
          name : "Credit Card".into(),
          regex : Regex::new(
            r"\b(?:\d{4}[-\s]?){3}\d{4}\b"
          ).unwrap(),
          risk_level : RiskLevel::Critical,
        },

        // Phone Numbers (US format)
        PiiPattern
        {
          name : "Phone".into(),
          regex : Regex::new(
            r"\b(?:\+1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b"
          ).unwrap(),
          risk_level : RiskLevel::High,
        },

        // IP Addresses
        PiiPattern
        {
          name : "IP Address".into(),
          regex : Regex::new(
            r"\b(?:\d{1,3}\.){3}\d{1,3}\b"
          ).unwrap(),
          risk_level : RiskLevel::Medium,
        },
      ],
    }
  }

  pub fn detect( &self, text : &str ) -> Vec< PiiMatch >
  {
    let mut matches = Vec::new();

    for pattern in &self.patterns
    {
      for m in pattern.regex.find_iter( text )
      {
        matches.push( PiiMatch
        {
          pii_type : pattern.name.clone(),
          value : m.as_str().to_string(),
          start : m.start(),
          end : m.end(),
          risk_level : pattern.risk_level,
        } );
      }
    }

    matches
  }
}
```

**2. Named Entity Recognition (NER) - ML-based (95% accuracy)**

```python
from transformers import pipeline

pii_detector = pipeline(
    "ner",
    model="StanfordAIMI/stanford-deidentifier-base",
    aggregation_strategy="simple"
)

text = """
Patient: John Smith
DOB: 03/15/1985
SSN: 123-45-6789
Address: 123 Main St, San Francisco, CA 94102
"""

entities = pii_detector(text)
# Output:
# [
#   {'entity': 'PERSON', 'word': 'John Smith', 'score': 0.99},
#   {'entity': 'DATE', 'word': '03/15/1985', 'score': 0.97},
#   {'entity': 'SSN', 'word': '123-45-6789', 'score': 0.98},
#   {'entity': 'ADDRESS', 'word': '123 Main St, ...', 'score': 0.96}
# ]
```

**3. Contextual Analysis (98% accuracy, slowest)**

Uses LLM to understand context:

```python
def detect_pii_with_context(text: str) -> List[PiiDetection]:
    prompt = f"""
Analyze the following text and identify all PII:

Text: {text}

For each PII found, return:
1. Type (email, SSN, phone, address, etc.)
2. Value (the actual PII)
3. Risk level (Low/Medium/High/Critical)
4. Reason (why this is PII)

Return as JSON array.
"""

    response = llm.call(prompt)
    return parse_json(response)
```

**PII Protection Strategies:**

**Strategy 1: Redaction (Most Common)**

Replace PII with placeholder:

```
Original: "Send invoice to john.smith@example.com"
Redacted: "Send invoice to [EMAIL_REDACTED]"

Original: "SSN: 123-45-6789"
Redacted: "SSN: [SSN_REDACTED]"
```

**Strategy 2: Masking (Partial Visibility)**

Show last 4 digits only:

```
Original: "4532-1234-5678-9010"
Masked:   "****-****-****-9010"

Original: "john.smith@example.com"
Masked:   "j***@example.com"
```

**Strategy 3: Tokenization (Reversible)**

Replace with token, store mapping in secure database:

```
Original: "john.smith@example.com"
Token:    "EMAIL_TOKEN_7A3F9C2D"

Mapping stored in secure DB:
{
  "EMAIL_TOKEN_7A3F9C2D": "john.smith@example.com",
  "encrypted": true,
  "access_requires_auth": true
}
```

**Strategy 4: Synthetic Data (Testing)**

Replace with realistic but fake data:

```
Original: "Patient: John Smith, SSN: 123-45-6789"
Synthetic: "Patient: Jane Doe, SSN: 987-65-4321"

Uses faker library to generate consistent fake data:
- Same name always maps to same fake name
- Preserves data relationships
- Safe for testing/development
```

**Example: Agent Output Filtering in Iron Cage**

```rust
pub async fn filter_agent_output( output : &str ) -> FilteredOutput
{
  let pii_matches = pii_detector.detect( output );

  if pii_matches.is_empty()
  {
    return FilteredOutput
    {
      text : output.to_string(),
      violations : vec![],
      action : Action::Allow,
    };
  }

  // Critical PII detected - block output
  if pii_matches.iter().any( | m | m.risk_level == RiskLevel::Critical )
  {
    return FilteredOutput
    {
      text : "[OUTPUT_BLOCKED_DUE_TO_PII]".to_string(),
      violations : pii_matches,
      action : Action::Block,
    };
  }

  // Medium/High PII - redact and warn
  let mut redacted_text = output.to_string();
  for m in pii_matches.iter().rev() // Reverse to preserve offsets
  {
    let replacement = format!( "[{}_REDACTED]", m.pii_type.to_uppercase() );
    redacted_text.replace_range( m.start..m.end, &replacement );
  }

  FilteredOutput
  {
    text : redacted_text,
    violations : pii_matches,
    action : Action::Redact,
  }
}
```

**Control Panel Alert Example:**

```
┌──────────────────────────────────────────────────────────┐
│  🔴 CRITICAL: PII LEAK PREVENTED                         │
├──────────────────────────────────────────────────────────┤
│  Agent: lead_gen_v2                                      │
│  Time: 2025-11-17 14:23:45 UTC                           │
│  Action: OUTPUT BLOCKED                                  │
│                                                           │
│  PII Detected:                                           │
│  ├─ Type: Email (3 instances)                           │
│  │   Values: john@example.com, jane@acme.com,           │
│  │           admin@company.org                           │
│  ├─ Type: Phone (1 instance)                            │
│  │   Value: +1-555-123-4567                             │
│  └─ Type: SSN (1 instance) ⚠️ CRITICAL                  │
│      Value: ***-**-**** (hidden for security)           │
│                                                           │
│  Original Output (sanitized):                            │
│  "Found 3 leads: [EMAIL], [EMAIL], [EMAIL]              │
│   Contact [PHONE] for verification.                      │
│   Tax ID: [SSN]"                                        │
│                                                           │
│  Action Taken:                                           │
│  ✅ Output blocked (not sent to user)                   │
│  ✅ Incident logged to SIEM                             │
│  ✅ Compliance team notified                            │
│  ✅ Agent auto-paused for review                        │
└──────────────────────────────────────────────────────────┘
```

**Compliance Implications:**

**GDPR (Europe):**
- PII = "personal data" under Article 4
- Must have legal basis for processing (consent, contract, legitimate interest)
- Right to erasure ("right to be forgotten") - must delete on request
- Data breach notification within 72 hours
- Fines: Up to €20M or 4% of global revenue

**HIPAA (US Healthcare):**
- PII in health context = PHI (Protected Health Information)
- Must encrypt PHI at rest and in transit
- Audit logs for all PHI access (who, what, when)
- Business Associate Agreement (BAA) required for vendors
- Penalties: $100-$50,000 per violation, criminal charges possible

**PCI-DSS (Payment Cards):**
- Never store CVV (3-digit security code) - forbidden
- Credit card numbers must be encrypted or tokenized
- Access to cardholder data requires justification and approval
- Quarterly security scans and annual audits
- Fines: $5,000-$100,000 per month of non-compliance

**Real-World Cost of PII Leak:**

**Case Study: Acme Corp (hypothetical)**
- AI agent leaked 10,000 customer emails in training logs
- Discovered during SOC 2 audit
- Impact:
  - GDPR fine: €500,000 (10K × €50 average fine per record)
  - Customer compensation: $250,000 (10K × $25 credit monitoring)
  - Lost business: $2M (20 enterprise customers churned)
  - Remediation: $150,000 (legal, PR, technical fixes)
  - **Total cost: $2.9M**

**With Iron Cage PII filtering:**
- PII detected in real-time, output blocked
- Zero customer impact
- Cost: $0 (plus $2K/month Iron Cage subscription)
- **ROI: 145,000% in avoided losses**

---

## Summary: Why These Concepts Matter for Iron Cage

### Lead Generation Demo
- **Business Context:** $200-500 value per lead in B2B sales
- **Cost Story:** $0.23/lead (Iron Cage) vs $0.87/lead (baseline) = 73% savings
- **Scale:** 10,000 leads/month = $76,800 annual savings

### Safety Cutoffs
- **Problem:** LinkedIn API fails → All 100 agents freeze
- **Solution:** Circuit opens → Agents switch to fallback in 1 second
- **Value:** 90% throughput maintained vs 0% without circuit breakers

### Fallback Chains
- **Problem:** Single point of failure (OpenAI API)
- **Solution:** 5-tier fallback (OpenAI → Azure → Anthropic → Local → Cache)
- **Value:** 99.95% uptime (vs 99.5% single provider)

### PII Protection
- **Problem:** Agent leaks customer emails → €500K GDPR fine
- **Solution:** Real-time privacy protection → Output blocked automatically
- **Value:** $2.9M in avoided losses per incident

**All three concepts work together:**

```
Agent generates output with PII
  ↓
PII detector catches email addresses
  ↓
Output filter blocks response
  ↓
Fallback chain activates: Try cached response
  ↓
safety cutoff checks: Is cache healthy?
  ↓
Cache available → Return safe cached data
  ↓
Result: Zero PII leaked, zero downtime, compliance maintained
```

This is the **Iron Cage value proposition** in action: Safety, cost control, and reliability working together to make AI agents enterprise-ready.
