# Iron Cage Runtime: Product Requirements Specification

**Version:** 1.0.0
**Date:** 2025-11-17
**Status:** Initial Draft
**Target Release:** MVP (3 weeks)

---

### Scope

**Responsibility:** Complete functional and non-functional requirements for full production Iron Cage Runtime (WHAT the system must do)

**In Scope:**
- Functional requirements across all capabilities (Core Runtime, Safety, Cost Control, Reliability, Credential Management, Data Access, MCP Integration, Observability)
- Non-functional requirements (performance, scalability, security, compliance, reliability)
- Acceptance criteria for each requirement (measurable success metrics)
- Requirement priorities (MUST/SHOULD/COULD) and MVP scope
- Integration requirements (Python FFI, LLM providers, enterprise systems)
- agent management management requirements (start, pause, resume, stop, restart, health checks)
- Multi-agent orchestration requirements (100-1000+ concurrent agents)
- Safety requirements (privacy protection, prompt injection prevention, action authorization)
- Cost control requirements (budget enforcement, token tracking, circuit breaker)
- Compliance requirements (SOC2/HIPAA audit logs, data retention, encryption)
- Performance requirements (99.9% uptime, <1ms FFI overhead, 1000+ calls/sec)

**Out of Scope:**
- System architecture and component design (see `architecture.md` for HOW components interact)
- Warsaw pilot specifications (see `../pilot/spec.md` for 28 pilot features, $10-25K pilot scope)
- Implementation guide (see `/runtime/PILOT_GUIDE.md` for HOW to build)
- Rust crate dependencies (see `../pilot/crates.md` for dependency specifications)
- Technology stack installation (see `../pilot/tech_stack.md` for setup guides)
- Capability strategic planning (see `/spec/capability_*.md` for detailed capability specs)
- Business strategy and market analysis (see `/business/strategy/` for GTM, pricing)
- Competitor research (see `/research/competitors/` for competitive analysis)
- Deployment procedures (see `deployment_guide.md` for operational steps)

---

## Executive Summary

Iron Cage Runtime is a production-grade Rust-based infrastructure layer for deploying autonomous AI agents with enterprise guarantees around cost control, safety, and reliability. The system solves the critical gap between AI prototypes (90% of enterprises) and production deployment (15% success rate) by providing memory-safe execution, real-time cost tracking, multi-layer safety guardrails, and compliance-ready observability.

**Key Value Propositions:**
- **Cost Control:** Reduce AI infrastructure spending by 40-60% through intelligent caching, token optimization, and automatic budget enforcement
- **Safety Guarantees:** Zero memory vulnerabilities (Rust), privacy protection, prompt injection prevention, action authorization
- **Production Readiness:** 99.9% uptime, horizontal scaling, enterprise compliance (SOC2/HIPAA audit logs)

**Target Market:** Fortune 500 enterprises with AI Centers of Excellence, regulated industries (finance, healthcare, government), AI-first startups scaling to enterprise (Series B+)

---

## 1. Functional Requirements

### 1.1 Core Runtime (MUST)

### FR-1.1.1: Python FFI Integration
- **Description:** Seamless integration with Python-based AI agent frameworks (LangChain, CrewAI, AutoGPT, custom)
- **Requirements:**
  - Support PyO3 or ctypes bindings for zero-copy data transfer
  - Handle Python GIL efficiently for concurrent agents
  - Marshal complex Python objects (dict, list, custom classes) to Rust
  - Provide Python SDK with type hints and async/await support
  - Error propagation from Rust to Python with full stack traces
- **Acceptance Criteria:**
  - Python agent can make 1000+ calls/sec with <1ms FFI overhead
  - Memory usage remains constant over 24-hour continuous operation
  - Automatic cleanup of Python references (no memory leaks)

### FR-1.1.2: Agent Management
- **Description:** Full lifecycle control over AI agent execution
- **Requirements:**
  - **Start:** Initialize agent with configuration (model, tools, memory, budget)
  - **Pause/Resume:** Suspend agent execution and restore state
  - **Stop:** Graceful shutdown with cleanup (close connections, flush logs)
  - **Restart:** Recover from crashes with last known state
  - **Health Checks:** Periodic liveness and readiness probes
- **State Persistence:**
  - Save agent state to durable storage (Redis/PostgreSQL)
  - Support checkpointing every N operations or M seconds
  - Restore from checkpoint within 500ms
- **Acceptance Criteria:**
  - Agent survives runtime restart without data loss
  - Pause→Resume completes in <100ms
  - Failed agents restart automatically within 5 seconds

### FR-1.1.3: Multi-Agent Orchestration
- **Description:** Coordinate multiple agents running concurrently
- **Requirements:**
  - Support 100-1000+ concurrent agents per runtime instance
  - Agent-to-agent communication (message passing, shared memory)
  - Resource isolation per agent (CPU quotas, memory limits)
  - Priority scheduling (critical agents get resources first)
  - Deadlock detection and prevention
- **Acceptance Criteria:**
  - 1000 concurrent agents with <5% CPU overhead per agent
  - Zero deadlocks over 72-hour stress test
  - Agent isolation prevents cascade failures

### FR-1.1.4: Hot Reload and Updates
- **Description:** Update agent configuration or code without downtime
- **Requirements:**
  - Reload agent logic from Python without runtime restart
  - Update safety policies dynamically (add/remove guardrails)
  - A/B testing: route 10% of requests to new agent version
  - Gradual rollout: 10% → 25% → 50% → 100% traffic shift
  - Automatic rollback on error rate spike
- **Acceptance Criteria:**
  - Hot reload completes in <1 second
  - Zero request failures during reload
  - Rollback triggers automatically if error rate >5%

---

### 1.2 Safety Guardrails (MUST)

### FR-1.2.1: Input Validation
- **Description:** Detect and block malicious or unsafe inputs
- **Requirements:**
  - **Prompt Injection Detection:**
    - Pattern matching against known injection templates
    - ML-based classifier (fine-tuned BERT model, 95%+ accuracy)
    - Severity scoring: Low (warn) / Medium (flag) / High (block)
  - **Input Sanitization:**
    - Strip HTML/JavaScript tags from user input
    - Normalize Unicode to prevent homograph attacks
    - Limit input length (configurable, default 10K chars)
  - **Allowlist/Blocklist:**
    - Support regex patterns for allowed/blocked content
    - Per-agent custom rules
- **Acceptance Criteria:**
  - Detect 95%+ of OWASP Top 10 prompt injection patterns
  - False positive rate <2%
  - Processing latency <10ms per input

### FR-1.2.2: Output Filtering
- **Description:** Prevent sensitive data leakage in agent responses
- **Requirements:**
  - **Privacy Protection:**
    - Identify SSN, credit cards, phone numbers, emails (regex + ML)
    - Named entity recognition for names, addresses
    - Custom patterns per organization (e.g., employee IDs)
  - **Secret Scanning:**
    - API keys, passwords, tokens (using truffleHog-style rules)
    - Database connection strings
    - Private keys (PGP, SSH)
  - **Actions:**
    - Redact: Replace with `[REDACTED]`
    - Block: Reject response entirely
    - Alert: Log to SIEM but allow (audit mode)
  - **Compliance:**
    - GDPR: Detect EU citizen PII
    - HIPAA: Detect PHI (protected health information)
    - PCI-DSS: Detect payment card data
- **Acceptance Criteria:**
  - Detect 98%+ of common PII patterns
  - Configurable sensitivity levels (strict/moderate/permissive)
  - <20ms latency per output scan

### FR-1.2.3: Action Authorization & Tool Execution Control
- **Description:** Intercept and validate all agent tool/action calls before execution

- **Critical Requirement:** Iron Cage MUST intercept both LLM calls AND tool calls. Without tool interception, agents could bypass safety by directly calling `delete_file()` or `run_shell_command()` without oversight.

- **Dual Interception Architecture:**
  - **Layer 1: LLM Call Interception**
    - Intercept all calls to OpenAI, Anthropic, Azure OpenAI, etc.
    - Apply privacy protection, cost tracking, prompt injection checks
    - Enforce circuit breakers and fallback chains
  - **Layer 2: Tool Call Interception (NEW)**
    - Intercept all calls to agent tools (file_ops, API calls, database queries)
    - Apply authorization policies before execution
    - Validate parameters to prevent injection attacks
    - Scan outputs for PII/secrets
    - Enforce per-tool rate limits
    - Log all tool calls for compliance

- **Tool Proxy Pattern:**
  - **Requirement:** All agent tools must be wrapped in `ToolProxy` during registration
  - **Implementation:**
    - Agent registers tools via `cage.register_agent(tools=[...])`
    - Iron Cage wraps each tool in `ToolProxy`
    - `ToolProxy.execute()` validates before delegating to original tool
    - Framework-agnostic: Works with LangChain, CrewAI, custom tools
  - **Validation Steps (per tool call):**
    1. Authorization check (whitelist/blacklist)
    2. Parameter validation (prevent path traversal, SQL injection, etc.)
    3. Rate limit check (per-tool quotas)
    4. Audit log (start of execution)
    5. Delegate to original tool
    6. Output scanning (privacy protection, secret scanning)
    7. Audit log (completion with metadata)

- **Authorization Policy Configuration:**
  - **Whitelist/Blacklist:**
    - Define allowed tools per agent: `["read_file", "scrape_url"]`
    - Define denied tools: `["delete_file", "run_command", "write_file"]`
    - Support glob patterns: `api.openai.com/*` allowed
  - **Conditional Policies:**
    - File operations: Restrict to specific directories (`/data/`, `/tmp/`)
    - API calls: Whitelist domains, enforce rate limits (1000 req/hr)
    - Database queries: Allow SELECT/INSERT, deny DELETE/DROP/TRUNCATE
  - **Permission Models:**
    - Role-based: Agent types (read-only, read-write, admin)
    - Resource-based: Per-database, per-API, per-file-system
    - Time-based: Allow `send_email` only 9am-5pm weekdays
    - Attribute-based: Combine role + resource + time + context

- **Parameter Validation (Injection Prevention):**
  - **File Operations:**
    - Prevent path traversal: Block `..` in file paths
    - Prevent restricted directories: Block `/etc`, `/root`, `/var/log`
    - Enforce file size limits: Max 100 MB per file
  - **API Calls:**
    - Validate URLs: Only whitelisted domains
    - Validate headers: Block auth token manipulation
    - Validate payloads: Max size limits, schema validation
  - **Database Queries:**
    - Prevent SQL injection: Parameterized queries only
    - Prevent destructive operations: Block `DELETE`, `DROP`, `TRUNCATE`
    - Enforce row limits: Max 10K rows per query

- **Tool Execution Modes:**
  - **Client-Side Execution (Model A):**
    - Agent runs on user's laptop/server
    - Tools execute on user's infrastructure
    - Iron Cage validates via API call (authorization check + audit log)
    - No sandboxing (trust user environment)
    - Use case: Development, testing, batch jobs
  - **Server-Side Execution (Model B):**
    - Agent uploaded to Iron Cage (runs in K8s cluster)
    - Tools execute in Iron Cage infrastructure
    - Iron Cage validates in-process (no API overhead)
    - Full sandboxing (cgroups + seccomp + network isolation)
    - Use case: Production 24/7 agents, SaaS customers

- **Sandboxed Execution (Server-Side Only):**
  - **Resource Limits (cgroups):**
    - CPU: Max 2 cores per tool execution
    - Memory: Max 1 GB per tool execution
    - Disk: Read-only except `/tmp` (100 MB quota)
    - Processes: Max 100 child processes
    - Execution time: 60 second timeout (configurable)
  - **Syscall Whitelist (seccomp):**
    - Allow: `read`, `write`, `open`, `close`, `mmap`, `brk`, `futex`
    - Block: `exec`, `fork`, `chroot`, `mount`, `reboot`, `ptrace`
  - **Network Isolation:**
    - Default: No internet access (isolated network namespace)
    - Configurable: Whitelist specific domains (e.g., `api.openai.com`)
    - Block: All traffic except whitelisted endpoints
  - **Violation Handling:**
    - If resource limit exceeded: Kill process immediately
    - Log violation: Audit log with agent ID, tool name, limit exceeded
    - Optionally: Suspend agent, alert admin, auto-disable

- **Audit Trail:**
  - **Log Every Tool Call:**
    - Timestamp, agent ID, user ID, tool name, tool args
    - Authorization decision (allowed/denied)
    - Execution duration, output size
    - PII detected (yes/no), secrets detected (yes/no)
    - Rate limit consumed/remaining
    - Result (success/error)
  - **Compliance Requirements:**
    - SOC 2: 100% of actions logged with user attribution
    - HIPAA: PHI access logged with IP, timestamp, user ID
    - GDPR: Data access logged for audit trail (Article 30)
    - PCI-DSS: Payment actions logged with approval chain
  - **Log Retention:**
    - Hot storage (PostgreSQL): 90 days
    - Warm storage (S3/GCS): 7 years
    - Immutable: Write-once, no deletion

- **Human-in-the-Loop:**
  - Require approval for high-risk actions (delete, payment, external email)
  - Approval via webhook, Slack, or control panel UI
  - Timeout: Auto-deny if no response in 5 minutes (configurable)
  - Approval context: Show tool name, args, risk level, requester

- **Framework Integration:**
  - **LangChain:** Wrap tools with `IronCageToolWrapper(tool)`
  - **CrewAI:** Wrap crew with `IronCageCrewAI.wrap_crew([agent])`
  - **Custom:** Register tools via `cage.register_agent(tools=[...])`
  - **Transparency:** No agent code changes required (zero-rewrite promise)

- **Acceptance Criteria:**
  - Zero unauthorized tool executions in production over 30 days
  - 100% of tool calls logged with full context (8 metadata fields minimum)
  - Human approval workflow completes in <30 seconds
  - Tool interception overhead <5ms per call (client-side) or <1ms (server-side)
  - Sandbox violations detected and killed within 100ms
  - Support 1000+ concurrent tool executions per runtime instance
  - Parameter validation prevents 100% of path traversal attacks in testing
  - privacy protection on tool outputs: 98%+ accuracy, <2% false positives

### FR-1.2.4: Rate Limiting
- **Description:** Prevent resource exhaustion and abuse
- **Requirements:**
  - **Per-Agent Limits:**
    - Max LLM calls per minute/hour/day
    - Max tokens consumed per period
    - Max tool invocations per period
  - **Per-User/Tenant Limits:**
    - Aggregate across all agents for a customer
    - Different tiers: Free (100 calls/day), Pro (10K calls/day), Enterprise (unlimited)
  - **Adaptive Limits:**
    - Increase limit if agent consistently stays under budget
    - Decrease limit if agent shows abusive patterns
  - **Backpressure:**
    - Return 429 Too Many Requests with Retry-After header
    - Queue requests instead of rejecting (configurable)
- **Acceptance Criteria:**
  - Rate limits enforced with <1ms latency overhead
  - Distributed rate limiting across multiple runtime instances
  - Graceful degradation: Slow down before hard cutoff

### FR-1.2.5: Safety Cutoffs
- **Description:** Fail fast when downstream dependencies are unhealthy
- **Requirements:**
  - **Per-Dependency Safety Cutoffs:**
    - Monitor LLM API (OpenAI, Anthropic, Azure OpenAI)
    - Monitor external tools (databases, APIs, filesystems)
  - **States:**
    - Closed: Normal operation
    - Open: Reject requests immediately (fail fast)
    - Half-Open: Test if dependency recovered
  - **Thresholds:**
    - Open after N consecutive failures or X% error rate in Y seconds
    - Reset after Z successful requests in half-open state
  - **Fallbacks:**
    - Use cached responses
    - Degrade to simpler model (GPT-4 → GPT-3.5)
    - Return static response or error message
- **Acceptance Criteria:**
  - Circuit opens within 1 second of dependency failure
  - Prevents cascade failures across agent fleet
  - Automatic recovery when dependency restores

### FR-1.2.6: Fallback Chains
- **Description:** Define graceful degradation strategies
- **Requirements:**
  - **Multi-Tier Fallbacks:**
    - Primary: GPT-4 via OpenAI
    - Secondary: GPT-4 via Azure OpenAI
    - Tertiary: Claude via Anthropic
    - Final: Local Llama 3.1 model
  - **Automatic Failover:**
    - Try primary, if timeout/error try secondary
    - Track success rate per tier
    - Prefer tier with best recent performance
  - **Cost Optimization:**
    - Use cheaper tiers when quality acceptable
    - A/B test: 10% traffic to cheaper model, compare metrics
- **Acceptance Criteria:**
  - <500ms total latency for full fallback chain
  - 99.5% success rate with fallbacks enabled
  - Cost reduction of 30%+ by using tiered models

---

### 1.3 Cost Control (MUST)

### FR-1.3.1: Real-Time Token Counting
- **Description:** Accurate tracking of LLM token consumption
- **Requirements:**
  - **Token Accounting:**
    - Count input tokens (prompt + context)
    - Count output tokens (completion)
    - Count cached tokens (when using prompt caching)
  - **Per-Model Tokenization:**
    - Use model-specific tokenizers (tiktoken for OpenAI, custom for others)
    - Handle special tokens correctly (<|im_start|>, etc.)
  - **Granularity:**
    - Per-agent token usage
    - Per-user/tenant aggregation
    - Per-conversation thread tracking
  - **Real-Time Updates:**
    - Update token counts within 100ms of LLM response
    - Stream updates to control panel via WebSocket
- **Acceptance Criteria:**
  - Token count accuracy within 1% of actual API billing
  - <10ms overhead per token count operation
  - Handle 10K+ concurrent token tracking sessions

### FR-1.3.2: Budget Enforcement
- **Description:** Automatically stop agents that exceed budget
- **Requirements:**
  - **Budget Types:**
    - Token budget: Max 100K tokens per agent per day
    - Cost budget: Max $50 per agent per month
    - Time budget: Max 1 hour of execution per agent per day
  - **Enforcement Actions:**
    - Soft limit (90% of budget): Send warning alert
    - Hard limit (100% of budget): Pause agent, require approval to continue
    - Emergency stop: Kill agent immediately if runaway detected
  - **Budget Pools:**
    - Shared budget across agent team (10 agents share $500/month)
    - Priority allocation: Critical agents get larger share
  - **Reset Schedule:**
    - Daily reset at midnight UTC
    - Monthly reset on 1st of month
    - Custom schedules (weekly, quarterly)
- **Acceptance Criteria:**
  - Budget enforcement within 1 second of limit breach
  - Zero overages: No agent exceeds budget by >1%
  - Alerts delivered via email, Slack, webhook within 5 seconds

### FR-1.3.3: Cost Projection
- **Description:** Predict future costs using ML-based forecasting
- **Requirements:**
  - **Historical Analysis:**
    - Analyze past 30 days of token usage
    - Identify trends (daily patterns, weekly spikes)
    - Detect anomalies (sudden 10x usage increase)
  - **Forecasting Models:**
    - ARIMA time series model for stable agents
    - Prophet for seasonal patterns
    - Simple moving average for new agents
  - **Projection Outputs:**
    - "At current rate, will cost $X this month"
    - "Budget will be exhausted in Y days"
    - "Confidence interval: $X - $Y (90% confidence)"
  - **What-If Analysis:**
    - "If we 2x traffic, cost becomes $X"
    - "If we switch to GPT-3.5, save $Y per month"
- **Acceptance Criteria:**
  - Projection accuracy within 15% of actual costs
  - Updated projections every 15 minutes
  - Predictions available via API and control panel

### FR-1.3.4: Cost Attribution
- **Description:** Track costs by agent, user, tenant, project
- **Requirements:**
  - **Multi-Dimensional Tagging:**
    - Agent: Which agent consumed tokens
    - User: Which end-user triggered the agent
    - Tenant: Which customer (for multi-tenant SaaS)
    - Project: Which business unit or cost center
    - Environment: prod/staging/dev
  - **Cost Breakdown:**
    - By model: 60% GPT-4, 30% Claude, 10% Llama
    - By operation: 50% chat, 30% embeddings, 20% function calls
    - By time: Hourly/daily/monthly aggregation
  - **Billing Integration:**
    - Export to CSV for finance team
    - API for chargeback to departments
    - Integration with Stripe/Zuora for customer billing
- **Acceptance Criteria:**
  - 100% of costs attributed to at least one dimension
  - <1 second latency for cost queries
  - Audit trail: Match cost attribution to LLM provider invoices within 2%

### FR-1.3.5: Optimization Recommendations
- **Description:** Suggest cost-saving opportunities
- **Requirements:**
  - **Caching Analysis:**
    - Identify repeated prompts (>10% duplicate)
    - Estimate savings: "Caching could save $X/month"
  - **Model Selection:**
    - Benchmark quality vs cost for agent use case
    - Recommend: "Switch 30% of requests to GPT-3.5 (same quality, 50% cheaper)"
  - **Batching Opportunities:**
    - Detect agents making sequential calls that could batch
    - Estimate: "Batching reduces latency by 40%, cost by 15%"
  - **Prompt Engineering:**
    - Detect verbose prompts (>5K tokens when 1K sufficient)
    - Suggest: "Reduce system prompt from 3K to 800 tokens (70% saving)"
- **Acceptance Criteria:**
  - Generate at least 5 actionable recommendations per week
  - Recommendations lead to 20%+ cost reduction when adopted
  - Presented in control panel with one-click implementation

---

### 1.4 Observability (MUST)

### FR-1.4.1: OpenTelemetry Integration
- **Description:** Standardized distributed tracing and metrics
- **Requirements:**
  - **Trace Spans:**
    - `agent.execute`: Top-level agent execution
    - `llm.call`: Each LLM API request (include model, tokens, latency)
    - `tool.invoke`: Each tool call (include tool name, parameters, result)
    - `guardrail.check`: Each safety validation
  - **Span Attributes:**
    - agent_id, user_id, tenant_id, conversation_id
    - model_name, input_tokens, output_tokens, cost
    - error flag, error message, stack trace
  - **Context Propagation:**
    - Trace IDs flow from Python → Rust → external APIs
    - Baggage for passing metadata (user tier, region)
  - **Exporters:**
    - OTLP (OpenTelemetry Protocol) to any backend
    - Support Jaeger, Zipkin, Honeycomb, Datadog, New Relic
- **Acceptance Criteria:**
  - 100% of agent operations traced
  - <2ms tracing overhead per operation
  - Traces available in backend within 10 seconds

### FR-1.4.2: Audit Logging
- **Description:** Compliance-ready immutable logs
- **Requirements:**
  - **Log Events:**
    - Agent started/stopped
    - Action authorized/denied
    - Budget limit warning/exceeded
    - Safety violation detected
    - Configuration change
  - **Log Format:**
    - Structured JSON (ISO 8601 timestamps, UTC)
    - Include correlation IDs for trace linking
    - Cryptographically signed (HMAC-SHA256) for tamper-proofing
  - **Log Storage:**
    - Write-ahead log to durable storage (S3, GCS, Azure Blob)
    - Retention: 90 days (configurable, up to 7 years for compliance)
    - Encryption at rest (AES-256)
  - **Log Analysis:**
    - Ship to SIEM (Splunk, Elastic, Sumo Logic)
    - Support SQL queries (DuckDB for local analysis)
    - Alerting rules: Trigger on N violations in M minutes
- **Acceptance Criteria:**
  - Zero log loss over 72-hour chaos test
  - Logs immutable (detect tampering via signature verification)
  - Query response time <1 second for 1M log entries

### FR-1.4.3: Performance Metrics
- **Description:** Real-time system and agent performance monitoring
- **Requirements:**
  - **System Metrics:**
    - CPU usage per agent
    - Memory usage (RSS, heap allocation)
    - Network I/O (bytes sent/received)
    - Disk I/O (for checkpointing)
  - **Agent Metrics:**
    - Throughput: Requests per second per agent
    - Latency: P50, P95, P99 for agent operations
    - Error rate: % of failed requests
    - Success rate: % of successful completions
  - **LLM Metrics:**
    - Time to first token (TTFT)
    - Tokens per second (throughput)
    - Cache hit rate (for prompt caching)
    - Model availability (uptime %)
  - **Custom Metrics:**
    - Application-level KPIs (leads generated, emails sent, tasks completed)
    - Business metrics (revenue per agent, customer satisfaction)
  - **Metrics Exporters:**
    - Prometheus scrape endpoint
    - StatsD/DogStatsD for push-based
    - CloudWatch/Stackdriver native integration
- **Acceptance Criteria:**
  - Metrics updated every 10 seconds (configurable)
  - Support 10K+ unique metric series
  - <1% CPU overhead for metrics collection

### FR-1.4.4: Error Tracking
- **Description:** Capture and diagnose errors and exceptions
- **Requirements:**
  - **Error Capture:**
    - Rust panics and recoverable errors
    - Python exceptions propagated from agents
    - External API errors (HTTP 4xx/5xx)
    - Timeout errors, circuit breaker trips
  - **Error Context:**
    - Full stack trace (Rust + Python)
    - Request/response payloads (sanitized for PII)
    - Agent state snapshot at time of error
    - Recent logs (last 100 log lines)
  - **Error Aggregation:**
    - Group similar errors (same stack trace signature)
    - Track frequency, first seen, last seen
    - Assign severity: Critical, High, Medium, Low
  - **Error Notification:**
    - Integrate with Sentry, Rollbar, Bugsnag
    - Alerting via PagerDuty, Opsgenie for critical errors
    - Slack notifications for high-severity errors
  - **Error Remediation:**
    - Link errors to runbooks (auto-generated or manual)
    - Track fix status: Open, In Progress, Resolved, Ignored
    - Auto-close if error not seen in 7 days
- **Acceptance Criteria:**
  - 100% of errors captured (zero silent failures)
  - Errors appear in tracking system within 30 seconds
  - <5% false positive rate for error grouping

### FR-1.4.5: Compliance Reporting
- **Description:** Generate reports for audits and certifications
- **Requirements:**
  - **SOC 2 Type II:**
    - Access logs: Who accessed which agent data
    - Change logs: All configuration changes with approval trail
    - Security incidents: Detected threats and response actions
  - **HIPAA:**
    - PHI access logs: Every time agent touched health data
    - Encryption evidence: Proof of data-at-rest/in-transit encryption
    - Business Associate Agreements: Track subprocessor compliance
  - **GDPR:**
    - Data subject requests: Locate all data for user ID
    - Right to be forgotten: Proof of data deletion
    - Data breach notification: Timeline of detection→disclosure
  - **PCI-DSS:**
    - Cardholder data access: Audit trail for payment info
    - Network segmentation: Proof agents can't access payment systems
  - **Report Generation:**
    - Automated reports (weekly, monthly, quarterly)
    - Export to PDF, Excel, JSON
    - Custom report templates per compliance framework
- **Acceptance Criteria:**
  - Reports pass external auditor review (zero findings)
  - Report generation completes in <60 seconds for 1 year of data
  - Automated delivery to compliance team every month

---

### 1.5 Developer Experience (MUST)

**Goal:** Enable platform engineers to deploy Iron Cage in <1 day with zero LLM infrastructure expertise

**Why Critical:**
- Operational buyers (Platform Engineers, DevOps) are key decision-makers
- Deployment complexity is #3 pain point (after cost and compliance)
- Fast time-to-value (1 day vs 2 weeks) drives Team Edition sales

### FR-1.5.1: Preconfigured MCP Servers

**Requirement:** Provide pre-built, production-ready MCP server configurations for common tools

**Details:**
- **Pre-built servers:**
  - `filesystem` - Safe file operations (read-only by default, whitelist paths)
  - `github` - Repository operations (PRs, issues, code search)
  - `slack` - Team communication (send messages, read channels)
  - `web_search` - Internet search (Google, Bing, DuckDuckGo)
  - `postgres` - Database queries (read-only by default)
  - `redis` - Cache operations
- **One-command install:** `iron_cage mcp install <server-name>`
- **Automatic safety:** Pre-configured with sane defaults (rate limits, sandboxing, auth)
- **Customization:** YAML config for advanced users

**Acceptance Criteria:**
- 6 MCP servers available out-of-box
- Install command works in <30 seconds
- Each server has safety guardrails enabled by default
- Documentation for each server (usage, config, examples)

**Priority:** P0 (Team Edition blocker)

---

### FR-1.5.2: Quick Start Templates

**Requirement:** Provide `docker-compose.yml` that deploys Iron Cage in <5 minutes with zero configuration

**Details:**
- **What's included:**
  - `docker-compose.yml` - All services (runtime, API gateway, Redis, PostgreSQL)
  - `.env.example` - Environment variables with comments
  - `README.quickstart.md` - Step-by-step setup guide
  - `examples/` - Sample agents (lead gen, support, data analysis)
- **Setup process:**
  1. `git clone` repository
  2. `cp .env.example .env` (add OpenAI API key)
  3. `docker-compose up -d` (starts all services)
  4. Open http://localhost:8080 (web control panel)
  5. Run example agent in 1 command
- **Pre-configured defaults:**
  - Budget limits: $100/month per user
  - Rate limits: 100 req/hour per agent
  - privacy protection: Enabled (basic regex)
  - Audit logs: 30-day retention
  - Sandboxing: Docker-based

**Acceptance Criteria:**
- Fresh Ubuntu VM → running Iron Cage in <5 minutes
- All services start with zero manual configuration
- Example agent runs successfully
- Web control panel accessible
- README has <10 steps

**Priority:** P0 (Team Edition blocker)

---

### FR-1.5.3: Self-Service Access Portal

**Requirement:** Web UI where engineers request LLM access and platform engineers approve

**Details:**
- **User flow:**
  1. Engineer visits portal (SSO login)
  2. Clicks "Request LLM Access"
  3. Fills form: Team (dropdown), Use Case (text), Budget Request ($/month)
  4. Submits request
  5. Platform engineer gets email/Slack notification
  6. Approves/denies via web UI
  7. Engineer gets API key + usage instructions
- **Admin features:**
  - View all access requests (pending, approved, denied)
  - Set default budgets per team
  - Bulk approve/deny
  - Audit log of all approvals
- **Security:**
  - API keys auto-rotate every 90 days
  - Keys never shown in UI (copy to clipboard once)
  - Revoke access instantly

**Acceptance Criteria:**
- Engineer can request access in <2 minutes
- Admin can approve in <30 seconds
- API key delivered securely (no email)
- Audit log tracks all approvals
- Integrates with Slack for notifications

**Priority:** P1 (Team Edition nice-to-have)

---

### FR-1.5.4: Team-Based Multi-Tenancy

**Requirement:** Separate budgets, control panels, and policies per team (e.g., engineering vs marketing vs sales)

**Details:**
- **Team isolation:**
  - Each team has separate budget pool
  - Spending tracked per team
  - Control Panels filtered by team
  - Policies customizable per team (e.g., marketing = GPT-3.5 only, engineering = GPT-4 allowed)
- **Admin controls:**
  - Create teams (name, budget, members)
  - Assign users to teams
  - Set per-team policies (model whitelist, rate limits, privacy protection level)
  - Transfer budget between teams
- **Reporting:**
  - Top spenders by team
  - Cost trends per team (this month vs last month)
  - Budget utilization (80% used → alert)

**Acceptance Criteria:**
- Admin can create teams
- Budgets enforced per team (team A can't use team B's budget)
- Control Panels show per-team breakdown
- Policies customizable per team
- Alerts when team budget 80% consumed

**Priority:** P1 (Team Edition differentiator)

---

### FR-1.5.5: Usage Control Panels

**Requirement:** Real-time control panels showing spending, top users, and cost trends per team

**Details:**
- **Metrics:**
  - Total spending (today, this week, this month)
  - Top 10 users by spend
  - Top 10 expensive queries
  - Cost by model (GPT-4 vs GPT-3.5 vs Claude)
  - Cost by team (engineering, marketing, sales)
  - Trend: cost over time (line chart)
- **Drill-down:**
  - Click team → see users in team
  - Click user → see queries
  - Click query → see full prompt + response + cost
- **Export:**
  - CSV export for billing
  - PDF report for finance team

**Acceptance Criteria:**
- Control Panel loads in <2 seconds
- Real-time updates (1-minute refresh)
- All metrics accurate vs database
- Export works (CSV + PDF)
- Mobile-responsive

**Priority:** P2 (Team Edition nice-to-have)

---

### FR-1.5.6: API Key Auto-Rotation

**Requirement:** Automatically rotate LLM provider API keys (OpenAI, Anthropic) on schedule

**Details:**
- **Rotation schedule:**
  - Default: Every 90 days
  - Configurable: 30/60/90/180 days
  - Manual: Rotate now (1-click)
- **Process:**
  1. Generate new key via provider API
  2. Update Iron Cage config (both old + new keys active for 24h grace period)
  3. Alert users: "Keys rotated, update .env by tomorrow"
  4. After 24h: Deactivate old key
- **Leak detection:**
  - Monitor GitHub API for leaked keys (GitHub Secret Scanning API)
  - If detected: Auto-rotate immediately + alert admin
  - Revoke leaked key instantly

**Acceptance Criteria:**
- Keys rotate automatically on schedule
- 24-hour grace period (no downtime)
- Leak detection works (test with fake key)
- Admin gets alert (Slack/email) on rotation
- Audit log tracks all rotations

**Priority:** P2 (Security best practice)

---

### 1.7 Enterprise Data Integration (MUST)

### FR-1.7.1: Vector Database Management

**Description:** Manage vector databases for semantic search and RAG

**Requirements:**
- **Supported Vector Databases:**
  - Pinecone (managed cloud)
  - Weaviate (self-hosted or cloud)
  - ChromaDB (embedded or server)
  - Qdrant (self-hosted or cloud)
  - Milvus (self-hosted)
  - pgvector (PostgreSQL extension)

- **Automatic Embedding Generation:**
  - Support OpenAI embeddings (text-embedding-3-small, text-embedding-3-large)
  - Support open-source models (BGE, E5, instructor)
  - Batch embedding API calls (reduce cost)
  - Incremental updates (only embed changed chunks)

- **Namespace Isolation:**
  - Per-tenant namespaces (prevent data leakage)
  - Per-agent namespaces (isolated RAG stores)
  - Per-project namespaces (dev/staging/prod)

- **Chunking Strategies:**
  - Fixed-size chunking (512/1024/2048 tokens)
  - Semantic chunking (paragraph/section boundaries)
  - Recursive chunking (preserve hierarchy)
  - Configurable overlap (128 tokens default)

**Acceptance Criteria:**
- ✅ Support 3+ vector databases out of box
- ✅ Embedding generation <5s for 1000 chunks
- ✅ Namespace isolation prevents cross-tenant queries
- ✅ Chunking preserves document structure

---

### FR-1.7.2: Enterprise Data Connectors

**Description:** Connect to 20+ enterprise data sources

**Requirements:**
- **Document Repositories:**
  - SharePoint (Office 365, on-premise)
  - Google Drive (personal, Workspace)
  - Confluence (Cloud, Data Center)
  - Notion (personal, team)
  - Dropbox Business
  - Box Enterprise

- **Databases:**
  - SQL: MySQL, PostgreSQL, SQL Server, Oracle
  - NoSQL: MongoDB, DynamoDB, Firestore
  - Data warehouses: Snowflake, BigQuery, Redshift

- **CRM/Sales:**
  - Salesforce (Sales Cloud, Service Cloud)
  - HubSpot (CRM, Marketing)
  - Zendesk (Support tickets)

- **Communication:**
  - Slack (messages, threads, channels)
  - Microsoft Teams (chats, files)
  - Gmail (emails with filters)

- **Code/Dev:**
  - GitHub (repos, issues, PRs, wiki)
  - GitLab (repos, issues, wiki)
  - Jira (issues, epics, sprints)

- **Web:**
  - Website crawler (sitemap-based)
  - RSS/Atom feeds
  - REST APIs (generic connector)

**Connector Features:**
- OAuth 2.0 authentication
- Incremental sync (only fetch changes)
- Metadata extraction (author, date, tags)
- Permission mapping (inherit source ACLs)

**Acceptance Criteria:**
- ✅ 20+ connectors available
- ✅ OAuth flow works for all supported platforms
- ✅ Incremental sync reduces API calls by 90%+
- ✅ Permissions correctly mapped from source

---

### FR-1.7.3: Real-Time Sync Pipeline

**Description:** Keep vector stores synchronized with source data

**Requirements:**
- **Sync Modes:**
  - Real-time: Webhooks from source (SharePoint, Salesforce, GitHub)
  - Near real-time: Polling every 5 minutes
  - Scheduled: Hourly, daily, weekly batch updates
  - Manual: On-demand sync trigger

- **Change Detection:**
  - Modified timestamps (if-modified-since)
  - Checksums/ETags (content-based)
  - Change logs (audit trails from source)
  - Delta APIs (Microsoft Graph, Salesforce)

- **Sync Processing:**
  - Deduplication (same document from multiple sources)
  - Conflict resolution (last-write-wins, manual review)
  - Tombstones (soft delete for removed docs)
  - Versioning (keep last N versions)

- **Error Handling:**
  - Retry with exponential backoff
  - Dead letter queue for failed syncs
  - Alert on sustained failures (>10 retries)
  - Manual reconciliation UI

**Acceptance Criteria:**
- ✅ Webhook latency <10 seconds (real-time mode)
- ✅ Polling detects changes within 5 minutes
- ✅ Deduplication catches 95%+ duplicates
- ✅ Failed syncs retry automatically

---

### FR-1.7.4: Unified Query Layer

**Description:** Single API for semantic, keyword, and SQL queries

**Requirements:**
- **Query Types:**
  - Semantic search (vector similarity, cosine/euclidean/dot product)
  - Keyword search (BM25, TF-IDF via Elasticsearch)
  - SQL queries (structured data from databases)
  - Hybrid search (combine semantic + keyword)
  - Graph queries (relationships, knowledge graphs)

- **Query Routing:**
  - Intent detection (classify query type)
  - Multi-source queries (fan out to vector DB + SQL)
  - Result merging (rank aggregation)
  - Result deduplication (same doc from multiple sources)

- **Query Optimization:**
  - Query rewriting (expand acronyms, fix typos)
  - Re-ranking (cross-encoder for top results)
  - Caching (identical queries return cached results)
  - Explain mode (show query plan, sources)

- **Response Format:**
  - Document chunks with metadata (source, author, date)
  - Relevance scores (0-1 confidence)
  - Citations (source URLs, page numbers)
  - Highlighted snippets (matched text)

**Acceptance Criteria:**
- ✅ Query latency <500ms (p95)
- ✅ Hybrid search improves accuracy by 20%+ vs semantic alone
- ✅ Cache hit rate >40% for common queries
- ✅ Citations link to exact source location

---

### FR-1.7.5: Data Access Control

**Description:** Enforce fine-grained access policies

**Requirements:**
- **Row-Level Security (RLS):**
  - Filter results by user/tenant/agent
  - Inherit permissions from source (SharePoint ACLs → vector DB)
  - Group-based access (AD groups, RBAC roles)
  - Time-based access (expire after N days)

- **Column Masking:**
  - Redact PII in query results (emails, SSNs, credit cards)
  - Partial masking (show first 4 digits of card)
  - Dynamic masking (based on user role)

- **Tenant Isolation:**
  - Multi-tenant vector stores (namespace per tenant)
  - Query cannot cross tenant boundaries
  - Admin can view all tenants (with audit)

- **SSO Integration:**
  - SAML 2.0 (Okta, Auth0, Azure AD)
  - OAuth 2.0 (Google, Microsoft)
  - LDAP/Active Directory
  - API key authentication (for service accounts)

- **Audit Logging:**
  - Log all data access (who, what, when)
  - Compliance reports (GDPR Article 30, HIPAA)
  - Data lineage (trace data from source → query)

**Acceptance Criteria:**
- ✅ RLS filters 100% of unauthorized results
- ✅ PII redaction catches 98%+ sensitive data
- ✅ Tenant isolation verified via penetration test
- ✅ Audit logs capture all access events

---

## 2. Non-Functional Requirements

### 2.1 Performance (MUST)

### NFR-2.1.1: Low Overhead
- **Requirement:** Runtime overhead must be <1ms per LLM API call
- **Measurement:** Median overhead for agent.execute() wrapper
- **Rationale:** LLM calls already take 500ms-5s; <1ms overhead is negligible (<0.2%)
- **Validation:** Benchmark with 10K agents, measure P50/P95/P99 overhead

### NFR-2.1.2: High Throughput
- **Requirement:** Support 1000+ concurrent agents per runtime instance
- **Measurement:** Run 1000 agents continuously for 1 hour, measure success rate
- **Rationale:** Enterprise customers deploy agent fleets (10-100 agents per use case)
- **Validation:** Load test with 1000 synthetic agents, verify <5% error rate

### NFR-2.1.3: Low Latency
- **Requirement:** P99 latency for guardrail checks <50ms
- **Measurement:** End-to-end latency from input validation to output filtering
- **Rationale:** Latency budget: 50ms runtime + 500ms LLM = 550ms total (acceptable)
- **Validation:** Profile guardrail pipeline, optimize hot paths

### NFR-2.1.4: Memory Efficiency
- **Requirement:** <100MB memory per idle agent, <500MB per active agent
- **Measurement:** RSS (Resident Set Size) measured via `ps` or container stats
- **Rationale:** 1000 agents × 500MB = 500GB (fits in single 1TB server)
- **Validation:** Run 1000 agents, measure peak memory, verify no leaks over 24 hours

---

### 2.2 Scalability (SHOULD)

### NFR-2.2.1: Horizontal Scaling
- **Requirement:** Scale to 10,000+ agents by adding runtime instances
- **Approach:**
  - Stateless runtime (state in Redis/PostgreSQL)
  - Load balancer distributes agents across instances
  - Auto-scaling based on CPU/memory utilization
- **Validation:** Deploy to Kubernetes, scale from 1 → 10 instances, verify linear throughput

### NFR-2.2.2: Multi-Region Deployment
- **Requirement:** Support active-active deployment across 3+ regions
- **Approach:**
  - Regional runtime clusters (US-East, EU-West, Asia-Pacific)
  - Global load balancer (Cloudflare, AWS Global Accelerator)
  - Regional data residency (EU data stays in EU)
- **Validation:** Deploy to 3 regions, measure cross-region latency <200ms

### NFR-2.2.3: Auto-Scaling
- **Requirement:** Automatically scale runtime instances based on load
- **Triggers:**
  - CPU >70% for 5 minutes → scale up
  - CPU <30% for 10 minutes → scale down
  - Agent queue depth >100 → scale up
- **Validation:** Simulate load spike (100 → 1000 agents in 1 minute), verify auto-scale

---

### 2.3 Reliability (MUST)

### NFR-2.3.1: High Availability
- **Requirement:** 99.9% uptime SLA (max 43 minutes downtime per month)
- **Approach:**
  - Redundant runtime instances (N+1 or N+2)
  - Health checks every 10 seconds
  - Automatic failover to healthy instances
- **Validation:** Kill primary instance, verify failover in <30 seconds, zero request loss

### NFR-2.3.2: Disaster Recovery
- **Requirement:** RPO (Recovery Point Objective) <1 minute, RTO (Recovery Time Objective) <5 minutes
- **Approach:**
  - Continuous replication to standby region
  - Hourly backups to S3/GCS
  - Automated disaster recovery drills monthly
- **Validation:** Simulate region failure, restore from backup, verify RTO <5 minutes

### NFR-2.3.3: Graceful Degradation
- **Requirement:** System remains partially functional even if dependencies fail
- **Scenarios:**
  - LLM API down → Use cached responses or local model
  - Database down → Fall back to in-memory cache
  - External tool down → Return cached data or error message
- **Validation:** Chaos engineering (inject failures), verify >80% requests succeed

---

### 2.4 Security (MUST)

### NFR-2.4.1: Memory Safety
- **Requirement:** Zero memory vulnerabilities (buffer overflows, use-after-free, data races)
- **Approach:**
  - 100% Rust code for runtime (no unsafe blocks unless audited)
  - Deny unsafe code in workspace Cargo.toml
  - Automated memory safety checks (Miri, Valgrind)
- **Validation:** Run Miri on full test suite, zero errors

### NFR-2.4.2: Zero Trust Architecture
- **Requirement:** All communication authenticated and encrypted
- **Approach:**
  - mTLS (mutual TLS) between runtime instances
  - OAuth 2.0 / JWT for API authentication
  - API keys rotated every 90 days
- **Validation:** Penetration test, verify zero unauthenticated access

### NFR-2.4.3: Secrets Management
- **Requirement:** Never store secrets in plaintext
- **Approach:**
  - Integration with HashiCorp Vault, AWS Secrets Manager, Azure Key Vault
  - Encrypted at rest (AES-256-GCM)
  - Encrypted in transit (TLS 1.3)
- **Validation:** Scan codebase for hardcoded secrets, zero findings

### NFR-2.4.4: Compliance Certifications
- **Requirement:** Achieve SOC 2 Type II within 6 months of launch
- **Scope:**
  - Security: Access controls, encryption, incident response
  - Availability: Uptime monitoring, disaster recovery
  - Confidentiality: Data isolation, secure deletion
- **Validation:** Pass external SOC 2 audit with zero findings

---

### 2.5 Maintainability (SHOULD)

### NFR-2.5.1: Code Quality
- **Requirement:** 90%+ code coverage, zero clippy warnings
- **Enforcement:**
  - CI pipeline fails if coverage <90%
  - `cargo clippy -- -D warnings` in CI
  - Deny missing docs, missing debug implementations
- **Validation:** Run `cargo tarpaulin`, `cargo clippy`, verify thresholds met

### NFR-2.5.2: Documentation
- **Requirement:** Every public API documented with examples
- **Coverage:**
  - Module-level docs (purpose, architecture)
  - Function-level docs (parameters, return values, errors, examples)
  - User guides (getting started, tutorials, recipes)
- **Validation:** Run `cargo doc`, verify zero missing docs warnings

### NFR-2.5.3: Dependency Management
- **Requirement:** All dependencies declared in workspace Cargo.toml
- **Constraints:**
  - Use wTools absorption crates (error_tools, unilang, macro_tools)
  - Pin major versions, allow minor/patch updates
  - Security audits via `cargo audit` weekly
- **Validation:** Run `cargo deny check`, verify zero vulnerabilities

---

## 3. Enterprise Requirements

### 3.1 Deployment (MUST)

### ER-3.1.1: Container Support
- **Requirement:** Provide official Docker images
- **Images:**
  - `iron_cage:latest` (latest stable release)
  - `iron_cage:v1.2.3` (specific version)
  - `iron_cage:nightly` (bleeding edge, unstable)
- **Platforms:** linux/amd64, linux/arm64 (for AWS Graviton)
- **Validation:** Deploy to Docker, Docker Compose, Kubernetes, verify startup <10 seconds

### ER-3.1.2: Kubernetes Native
- **Requirement:** First-class Kubernetes support
- **Artifacts:**
  - Helm chart for easy deployment
  - Kubernetes Operator for lifecycle management
  - Custom Resource Definitions (CRDs) for agents
- **Features:**
  - Pod auto-scaling (HPA based on custom metrics)
  - Rolling updates with zero downtime
  - Integration with Kubernetes secrets/config maps
- **Validation:** Deploy to GKE/EKS/AKS, run `helm test`, verify all tests pass

### ER-3.1.3: On-Premise Deployment
- **Requirement:** Support air-gapped enterprise environments
- **Requirements:**
  - No internet connectivity required (offline model downloads)
  - Local license server (no phone-home)
  - Installation from tarball or ISO
- **Validation:** Deploy to air-gapped VM, verify full functionality

### ER-3.1.4: Cloud-Agnostic
- **Requirement:** Run on any cloud provider or bare metal
- **Tested Platforms:**
  - AWS (EC2, ECS, EKS, Lambda)
  - Google Cloud (Compute Engine, GKE, Cloud Run)
  - Azure (VMs, AKS, Container Instances)
  - On-premise (bare metal, VMware)
- **Validation:** Deploy to 3+ clouds, verify feature parity

---

### 3.2 Integration (MUST)

### ER-3.2.1: REST API
- **Requirement:** Comprehensive REST API for all operations
- **Endpoints:**
  - `POST /agents` → Create agent
  - `GET /agents/{id}` → Get agent status
  - `POST /agents/{id}/execute` → Run agent
  - `DELETE /agents/{id}` → Stop and delete agent
  - `GET /metrics` → Prometheus metrics
- **Features:**
  - OpenAPI 3.0 specification
  - Rate limiting (per API key)
  - Pagination for list endpoints
- **Validation:** Generate client SDKs (Python, JavaScript, Go), verify all operations

### ER-3.2.2: gRPC Support
- **Requirement:** High-performance gRPC API for latency-sensitive clients
- **Services:**
  - AgentService: Lifecycle management
  - MetricsService: Real-time metrics streaming
  - LogsService: Tail agent logs
- **Features:**
  - Bi-directional streaming for real-time updates
  - Protocol Buffers for efficient serialization
  - Load balancing via gRPC LB
- **Validation:** Benchmark gRPC vs REST, verify 50%+ latency reduction

### ER-3.2.3: WebSocket Streaming
- **Requirement:** Real-time updates for control panels
- **Channels:**
  - `/ws/agents/{id}/logs` → Stream agent logs
  - `/ws/agents/{id}/metrics` → Stream performance metrics
  - `/ws/agents/{id}/events` → Stream lifecycle events
- **Protocol:** JSON messages, heartbeat every 30 seconds
- **Validation:** Connect 100 WebSocket clients, verify <100ms message latency

### ER-3.2.4: Webhook Notifications
- **Requirement:** Push notifications for important events
- **Events:**
  - Agent started/stopped
  - Budget exceeded
  - Safety violation detected
  - Error threshold breached
- **Delivery:**
  - HTTP POST to customer-provided URL
  - Retry with exponential backoff (3 attempts)
  - Signature verification (HMAC-SHA256)
- **Validation:** Trigger events, verify webhook delivery within 5 seconds

---

### 3.3 Management (MUST)

### ER-3.3.1: Multi-Tenancy
- **Requirement:** Isolate multiple customers on shared infrastructure
- **Isolation:**
  - Separate database schemas per tenant
  - Resource quotas per tenant (CPU, memory, budget)
  - Network isolation (VPC per tenant or namespace isolation)
- **Features:**
  - Tenant provisioning API
  - Billing per tenant
  - Per-tenant configuration (custom safety policies)
- **Validation:** Deploy 100 tenants, verify zero data leakage

### ER-3.3.2: Role-Based Access Control (RBAC)
- **Requirement:** Fine-grained permissions for users
- **Roles:**
  - Admin: Full control (create/delete agents, modify config)
  - Operator: Run agents, view metrics (no config changes)
  - Viewer: Read-only access (control panels, logs)
  - Auditor: Access audit logs only
- **Permissions:**
  - agent.create, agent.read, agent.update, agent.delete
  - metrics.read, logs.read, config.write
- **Validation:** Create users with each role, verify permissions enforced

### ER-3.3.3: SSO/SAML Integration
- **Requirement:** Enterprise single sign-on
- **Protocols:**
  - SAML 2.0 (Okta, Azure AD, Google Workspace)
  - OAuth 2.0 / OpenID Connect
  - LDAP (for on-premise Active Directory)
- **Features:**
  - Just-in-time (JIT) user provisioning
  - Group-based role mapping
  - Session timeout (configurable, default 8 hours)
- **Validation:** Integrate with Okta test tenant, verify SSO login

### ER-3.3.4: Configuration Management
- **Requirement:** Centralized configuration with version control
- **Approach:**
  - YAML configuration files (checked into Git)
  - Environment variables for secrets
  - Configuration API for runtime changes
- **Features:**
  - Configuration validation (schema checks)
  - Rollback to previous config version
  - Change audit trail (who changed what when)
- **Validation:** Update config, verify applied without restart, rollback works

---

## 4. Architecture Documentation

For detailed technical architecture, see separate architecture documents:

- **[architecture.md](architecture.md)** - Iron Cage Gateway detailed architecture (runtime, safety layer, cost control, reliability, observability)
- **[technical_architecture.md](technical_architecture.md)** - Platform-level architecture covering all 8 capabilities as an integrated system

**Note:** Requirements define WHAT the system must do (functional requirements, non-functional requirements). Architecture documents define HOW the system implements those requirements (component design, data flows, technology stack).

---

## 5. Success Metrics

### 5.1 Adoption Metrics
- **Target:** 10 enterprise pilot customers within 3 months
- **Measurement:** Signed pilot agreements (POC or paid trial)

### 5.2 Technical Metrics
- **Reliability:** 99.9% uptime in production
- **Performance:** <1ms overhead per LLM call (P99)
- **Cost Reduction:** 40-60% savings vs baseline

### 5.3 Business Metrics
- **Revenue:** $500K ARR within 12 months
  - $50K from SaaS control panel subscriptions
  - $300K from enterprise support contracts
  - $150K from consulting engagements
- **Customer Retention:** 90%+ renewal rate after pilot

### 5.4 Security Metrics
- **Vulnerabilities:** Zero critical CVEs in production
- **Compliance:** SOC 2 Type II certification within 6 months
- **Incidents:** Zero data breaches

---

## 6. Out of Scope (v1.0)

**Features explicitly NOT included in initial release:**

1. **Multi-LLM Orchestration:**
   - No automatic model selection across providers
   - Users must explicitly choose model per agent
   - Rationale: Complex, low ROI for MVP

2. **Agent Marketplace:**
   - No pre-built agent templates
   - No community sharing of agent configurations
   - Rationale: Need user base first

3. **Advanced Analytics:**
   - No ML-powered cost anomaly detection
   - No automated performance optimization
   - Rationale: Requires historical data (6+ months)

4. **Multi-Cloud Active-Active:**
   - Support single-region or active-passive multi-region
   - No automatic failover across clouds
   - Rationale: Complex, edge case for most customers

5. **Custom LLM Hosting:**
   - No bundled LLM inference (users bring API keys)
   - No on-premise model serving
   - Rationale: Focus on runtime, not inference

---

## 7. Release Roadmap

### Phase 1: MVP (Weeks 1-3)
**Goal:** Validate core value proposition with demo

**Deliverables:**
- Core Rust runtime with Python FFI
- Basic safety guardrails (input validation, output filtering)
- Real-time token counting and budget enforcement
- Simple control panel (cost, safety violations, performance)
- Demo agent: Lead generation with live monitoring

**Success Criteria:**
- Demo runs without crashes for 1-hour presentation
- Audience feedback: 80%+ say "I would use this"

### Phase 2: Alpha (Weeks 4-8)
**Goal:** Pilot with 3-5 early customers

**Deliverables:**
- Multi-agent orchestration
- Advanced guardrails (action authorization, rate limiting, circuit breakers)
- OpenTelemetry integration
- REST API with authentication
- Docker deployment

**Success Criteria:**
- 3 pilot customers running in production
- Zero critical bugs for 2 weeks
- <1ms P99 overhead validated

### Phase 3: Beta (Weeks 9-16)
**Goal:** Production-ready for 20+ customers

**Deliverables:**
- gRPC API, WebSocket streaming
- Kubernetes support (Helm chart, Operator)
- Multi-tenancy and RBAC
- SOC 2 audit prep (security controls, audit logs)
- Cost optimization recommendations

**Success Criteria:**
- 20 paying customers
- 99.9% uptime over 30 days
- SOC 2 Type I report completed

### Phase 4: GA (Weeks 17-24)
**Goal:** Scale to 100+ customers

**Deliverables:**
- SSO/SAML integration
- Advanced compliance reporting (HIPAA, GDPR, PCI-DSS)
- Multi-region deployment support
- Enterprise support SLAs (24/7, 4-hour response)

**Success Criteria:**
- 100 customers, $500K ARR
- SOC 2 Type II certification
- Customer satisfaction: NPS >50

---

## 8. Dependencies and Assumptions

### 8.1 External Dependencies
- **LLM APIs:** Requires stable OpenAI/Anthropic/Azure APIs (99.9% SLA)
- **Cloud Infrastructure:** AWS/GCP/Azure availability
- **Open Source Crates:** Tokio, Axum, PyO3, Tonic (assume maintained)

### 8.2 Assumptions
- **Market Demand:** Enterprises willing to pay $2-5K/month for AI agent safety
- **Python Ecosystem:** LangChain/CrewAI remain dominant agent frameworks
- **Regulatory Pressure:** Increased AI regulations drive compliance demand
- **Rust Talent:** Can hire 2-3 Rust engineers for core team

### 8.3 Risks
- **Competition:** OpenAI/Anthropic build native guardrails (mitigate: faster iteration)
- **Technical:** PyO3 FFI overhead >1ms (mitigate: optimize, use C FFI)
- **Adoption:** Enterprises slow to adopt Rust (mitigate: provide Python SDK)
- **Compliance:** SOC 2 audit fails (mitigate: hire compliance consultant)

---

## 9. Appendix

### 9.1 Glossary
- **Agent:** Autonomous AI system that performs tasks (e.g., lead generation, customer support)
- **Guardrail:** Safety constraint that limits agent behavior (e.g., PII filtering, action whitelist)
- **Token:** Unit of LLM input/output (roughly 0.75 words)
- **FFI:** Foreign Function Interface (Rust calling Python or vice versa)
- **Safety Cutoff:** Design pattern for failing fast when dependency is unhealthy
- **TTFT:** Time To First Token (latency until LLM starts responding)
- **RPO/RTO:** Recovery Point/Time Objective (disaster recovery metrics)

### 9.2 References
- OWASP LLM Top 10: https://owasp.org/www-project-top-10-for-large-language-model-applications/
- NIST AI Risk Management Framework: https://www.nist.gov/itl/ai-risk-management-framework
- SOC 2 Compliance Guide: https://www.aicpa.org/soc2
- OpenTelemetry Specification: https://opentelemetry.io/docs/specs/otel/

### 9.3 Document History
- **v1.0.0** (2025-11-17): Initial requirements for MVP
- **v1.1.0** (TBD): Updated after customer interviews
- **v2.0.0** (TBD): Requirements for Beta release

---

**END OF REQUIREMENTS SPECIFICATION**
