# Pilot Platform Specification

**Version:** 1.4.0
**Created:** 2025-11-22
**Updated:** 2025-12-07
**Status:** Active (8-week implementation plan)

---

### Scope

**Responsibility:** Complete product specification for Warsaw pilot platform (40 features for conference demo including secrets management and Python SDK)

**In Scope:**
- All 40 pilot features (Features #1-40) with requirements and acceptance criteria
- Demo agent specification (lead generation with 100 synthetic leads)
- Demo triggers (PII at lead #67, budget warning at lead #85)
- Integration with conference presentation (Slide 18 demo script)
- Pilot customer deliverable ($10-25K pilot price point)
- Quick start subset (4 core features for 1-week implementation)

**Out of Scope:**
- Full production platform specifications (see `/spec/capability_*.md` for 8 full capabilities)
- Implementation details (see `/runtime/PILOT_GUIDE.md` for build instructions)
- Rust crate dependencies (see `crates.md` for dependency specifications)
- Technology stack setup (see `tech_stack.md` for installation guides)
- Execution planning (see `execution/` for 8-week plan, quick start, status tracking)
- Conference presentation materials (see `conferences/warsaw_2025/presentation/`)
- Business strategy and market analysis (see `/business/strategy/`)

---

## Executive Summary

**Purpose:** Minimal viable platform for conference demonstrations and early customer pilots. Implements the demo from [business/presentations/talk_outline.md](conferences/warsaw_2025/presentation/talk_outline.md) as a sellable product variant.

**Target Customers:** Conference leads, early adopters, pilot customers seeking AI safety + cost control

**Timeline:** 8 weeks, 1-2 engineers

**Investment:** $40-80K (engineering cost)

**Revenue Target:** 2-3 pilot contracts × $10-25K = $30-75K (Year 1)

**Scope:** Integrated platform combining:
- **Cap 2 (Safety Lite):** privacy protection and redaction
- **Cap 3 (Cost Lite):** Budget tracking and enforcement, circuit breakers
- **Cap 5 (Secrets Lite):** Encrypted secrets management with zero-downtime rotation
- **Cap 8 (Runtime Minimal):** Python-Rust gateway for LangChain/CrewAI agents

**Deliverable:** Working lead generation agent demo matching conference presentation exactly

---

## Product Overview

### Problem Statement

**Conference Challenge:**
- Speaker presents "Iron Cage Runtime" with safety + cost control features
- Audience asks: "When can I try this?"
- Need working demo within 30-60 days to convert leads

**Pilot Customer Challenge:**
- Enterprises want to test AI safety/cost control before $100K+ commitment
- Need production-ready pilot scope ($10-25K price point)
- Must deliver value in weeks, not months

### Solution Overview

**Pilot Platform:** Minimal viable integration of Safety + Cost + Runtime capabilities sufficient for:
1. Conference demo (30-minute live demonstration)
2. Customer pilots (4-8 hour integration for single agent)
3. Market validation (gather feedback to prioritize full capability builds)

**NOT a throwaway demo** - this is what you sell to pilot customers as "Iron Cage Pilot Platform"

### Target Customers

**Primary:**
- Conference leads from talk_outline.md (goal: 20+ qualified leads)
- Early adopters willing to pilot new technology
- CTOs/VP Engineering evaluating AI governance solutions

**Industries:**
- Financial services (GDPR compliance, cost control)
- Healthcare (HIPAA compliance, PII protection)
- B2B SaaS (multi-tenant cost attribution)

**Company Profile:**
- 50-500 employees
- Already using LangChain/CrewAI for AI agents
- Experiencing cost overruns OR compliance concerns

### Success Metrics

**Conference Demo:**
- ✅ Zero crashes during 30-minute demo run
- ✅ All 6 control panel panels match talk_outline.md
- ✅ privacy protection fires at lead #67 (planned trigger)
- ✅ Budget warning fires at 90% (planned trigger)
- ✅ safety cutoff fires at lead #34 (planned trigger)

**Pilot Sales:**
- ✅ 2-3 pilot contracts signed ($30-75K total revenue)
- ✅ Customer integration completed in <8 hours
- ✅ 90-day pilot retention (customers renew or upgrade)

**Market Validation:**
- ✅ Customer feedback identifies which capability to build next (Cap 1 vs Cap 2 full)
- ✅ Pricing validation ($10-25K pilot, $100-300K platform acceptable?)
- ✅ Feature prioritization (which 3 features do customers request most?)

---

## Features (Grouped by Capability)

### CAPABILITY 8: Production Agent Runtime (Minimal)

**Scope:** Minimal Rust runtime to demonstrate Python-Rust gateway, NOT full production deployment

#### Week 1-2: Core Runtime Features

**1. Agent Management**
- **Feature:** `iron_cage start <agent.py>` CLI command
- **Requirements:**
  - Agent process spawning and supervision
  - Graceful shutdown (SIGTERM/SIGINT handling)
  - Agent status monitoring (running/stopped/crashed)
  - Unique agent ID generation (format: `agent_id: lg-7a3f9c2d`)
- **Acceptance Criteria:** User runs `iron_cage start lead_gen_agent.py --budget $50`, agent starts successfully
- **Demo Usage:** Slide 18 Part 1 (Agent Startup)

**2. Python-Rust Integration (PyO3)**
- **Feature:** Python FFI bridge for LangChain/CrewAI agents
- **Requirements:**
  - Agent function interception (LLM calls, tool usage)
  - Async event loop integration (Tokio + Python asyncio)
  - Zero-copy data passing where possible
  - Error propagation (Python exceptions → Rust Result)
- **Acceptance Criteria:** Python agent makes LLM call → Rust runtime intercepts → tracks cost/safety
- **Demo Usage:** All demo functionality depends on this

**3. Configuration System**
- **Feature:** CLI argument parsing and runtime configuration
- **Requirements:**
  - CLI flags: `--budget`, `--verbose`, `--safety-mode`
  - Agent configuration loading (YAML/TOML)
  - Environment variable support (API keys, endpoints)
  - Runtime configuration validation
- **Acceptance Criteria:** Invalid config shows clear error message, valid config loads successfully
- **Demo Usage:** `--budget $50` flag sets budget limit

**4. Logging Infrastructure**
- **Feature:** Structured logging for debugging and compliance
- **Requirements:**
  - Structured logging (JSON format)
  - Log levels (INFO, WARN, ERROR, DEBUG)
  - Timestamped events (format: `[14:23:45]`)
  - Agent context in all logs (agent_id, user_id)
- **Acceptance Criteria:** Terminal output matches talk_outline.md format exactly
- **Demo Usage:** Terminal output during demo (Slide 18 Part 1)

### CAPABILITY 8.5: Python SDK Layer (Week 3)

**Scope:** Pythonic high-level API wrapping PyO3 bindings for improved developer experience

**36. Decorator API**
- **Feature:** `@protect_agent` decorator for simple agent protection
- **Requirements:**
  - Function decorator accepting budget, pii_detection, circuit_breaker parameters
  - Zero-configuration defaults (budget=100.0, pii_detection=True)
  - Works with both sync and async functions
  - Transparent to agent code (no modifications required)
  - Error messages in Pythonic exception format
- **Acceptance Criteria:** `@protect_agent(budget=50.0) def agent(): ...` runs with protections
- **Demo Usage:** Simplifies demo code for conference presentation

**37. Context Manager API**
- **Feature:** `with AgentRuntime(...)` context manager for resource management
- **Requirements:**
  - Proper __enter__ and __exit__ implementation
  - Automatic cleanup on context exit
  - Exception propagation with Pythonic errors
  - Works with Python's `with` statement
- **Acceptance Criteria:** `with AgentRuntime(config) as runtime:` manages resources properly
- **Demo Usage:** Clean resource management in examples

**38. LangChain Integration**
- **Feature:** IronCageChain for LangChain compatibility
- **Requirements:**
  - Extends LangChain Chain interface
  - Budget tracking for chain execution
  - PII detection in chain outputs
  - Compatible with existing LangChain tools, agents, memory
  - Callbacks for monitoring
- **Acceptance Criteria:** `IronCageChain(llm=OpenAI()).run(prompt)` tracks budget and PII
- **Demo Usage:** Demo agent uses LangChain patterns

**39. Testing Utilities**
- **Feature:** MockRuntime for agent testing without Rust compilation
- **Requirements:**
  - In-memory mock (no PyO3 compilation for unit tests)
  - pytest fixtures for common scenarios
  - Assertion helpers (assert_budget_exceeded, assert_pii_detected)
  - Test data generators (PII samples, API responses)
  - Deterministic behavior for CI
- **Acceptance Criteria:** Agent unit tests run without Rust runtime in CI
- **Demo Usage:** Validate demo agent behavior before conference

**40. Example Library**
- **Feature:** Comprehensive examples for LangChain, CrewAI, AutoGPT
- **Requirements:**
  - 20+ complete runnable examples
  - LangChain examples (5+ patterns: chains, agents, memory, tools, streaming)
  - CrewAI examples (3+ patterns: crews, agents, tasks)
  - AutoGPT examples (2+ patterns: goals, constraints)
  - Raw API examples (OpenAI, Anthropic, multi-provider)
  - Pattern examples (cost optimization, PII handling, circuit breakers)
- **Acceptance Criteria:** All examples run successfully with iron-cage[all] installed
- **Demo Usage:** Reference examples during Q&A session

---

### CAPABILITY 2: AI Safety Guardrails (Lite)

**Scope:** privacy protection and redaction only, NOT full prompt injection prevention or content filtering

#### Week 4-5: Privacy Protection & Redaction

**5. PII Pattern Detection**
- **Feature:** Real-time detection of emails, phones, SSNs in agent output
- **Requirements:**
  - Email detection (regex: `\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b`)
  - Phone number detection (US format: `\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}`)
  - SSN detection (format: `\d{3}-\d{2}-\d{4}`)
  - Risk classification (HIGH for email, MEDIUM for phone)
- **Acceptance Criteria:** Test string "Contact ceo@acme.com" → detects EMAIL with HIGH risk
- **Demo Usage:** Slide 18 Part 4 (Privacy Protection at lead #67)

**6. Real-Time Output Redaction**
- **Feature:** Automatic redaction of PII before logging or storage
- **Requirements:**
  - LLM output interception (before logging)
  - Automatic redaction (`[EMAIL_REDACTED]`, `[PHONE_REDACTED]`)
  - Original value encrypted storage (for audit)
  - Redacted output delivery to agent
- **Acceptance Criteria:** Agent output "Contact ceo@acme.com" → logged as "Contact [EMAIL_REDACTED]"
- **Demo Usage:** Slide 18 Part 4 (shows redacted output in control panel)

**7. PII Audit Logging**
- **Feature:** Compliance audit trail for all privacy protections
- **Requirements:**
  - privacy protection event logging (type, value hash, timestamp)
  - Incident severity tracking (CRITICAL for email, WARN for phone)
  - Compliance event stream (for SOC 2 audit trail)
  - Webhook notification support (Slack/email alerts)
- **Acceptance Criteria:** PII detected → event logged to SQLite with all required fields
- **Demo Usage:** Slide 18 Part 4 (audit log shown in control panel)

**8. Policy Enforcement**
- **Feature:** Configurable redact/block/warn modes
- **Requirements:**
  - Redact mode (continue agent execution with redacted output) - default
  - Block mode (halt agent on privacy protection) - config toggle
  - Warn mode (log only, no redaction) - for testing
- **Acceptance Criteria:** Config file sets `safety_mode: redact` → agent continues after PII
- **Demo Usage:** Demo uses redact mode (agent continues at lead #67)

---

### CAPABILITY 3: LLM Access Control & Cost (Lite)

**Scope:** Budget tracking and circuit breakers only, NOT full multi-LLM routing or governance

#### Week 6-7: Budget Tracking & Enforcement

**9. Real-Time Token Counting**
- **Feature:** Track OpenAI/Anthropic token usage and cost in real-time
- **Requirements:**
  - OpenAI token counting (tiktoken library via PyO3)
  - Anthropic token counting (approximate: `tokens ≈ chars / 4`)
  - Per-request cost calculation (model pricing table: GPT-4 $0.03/1K, GPT-3.5 $0.002/1K)
  - Running total tracking (format: `$5.29 / $50.00`)
- **Acceptance Criteria:** Agent makes GPT-4 call (1000 tokens) → cost increments by $0.03
- **Demo Usage:** Slide 18 Part 2 (Control Panel shows current cost)

**10. Budget Limits & Alerts**
- **Feature:** Hard budget enforcement with 90% warning threshold
- **Requirements:**
  - Hard budget limit enforcement (agent stops at 100%)
  - 90% threshold warning alert
  - Cost projection (format: `Projection: $23.00 total`)
  - Per-unit cost averaging (format: `$0.23 avg/lead`)
- **Acceptance Criteria:** Agent hits $45/$50 budget → warning alert fires, agent stops at $50
- **Demo Usage:** Slide 18 Part 5 (Budget Warning at lead #85)

**11. Alert System**
- **Feature:** Email/webhook notifications for budget thresholds
- **Requirements:**
  - Email notification at 90% budget threshold
  - Control Panel alert banner (red warning)
  - Recommended action calculation (format: `Approve $25 increase`)
  - Alert suppression (no spam, once per threshold)
- **Acceptance Criteria:** Budget hits 90% → email sent to ops@company.com within 5 seconds
- **Demo Usage:** Slide 18 Part 5 (shows email alert in terminal)

**12. Cost Attribution**
- **Feature:** Per-agent and per-request cost tracking
- **Requirements:**
  - Per-agent cost tracking
  - Per-request cost logging (LLM call → cost mapping)
  - Cost breakdown by model (GPT-4 vs GPT-3.5 split)
  - Baseline comparison (format: `vs Baseline: $64.86 saved`)
- **Acceptance Criteria:** Control Panel shows cost breakdown by model, matches actual API spend
- **Demo Usage:** Slide 18 Part 6 (Final Results cost summary)

#### Week 6-7: Safety Cutoff & Fallback

**13. Safety Cutoff State Machine**
- **Feature:** Automatic failure detection and circuit opening
- **Requirements:**
  - CLOSED state (normal operation)
  - OPEN state (failing, use fallback)
  - HALF_OPEN state (testing recovery)
  - Failure threshold detection (5 consecutive failures)
  - Cooldown timer (60 seconds)
- **Acceptance Criteria:** LinkedIn API fails 5 times → circuit opens, 60s cooldown starts
- **Demo Usage:** Slide 18 Part 3 (safety cutoff at lead #34)

**14. Fallback Chain**
- **Feature:** Multi-tier fallback for external API failures
- **Requirements:**
  - Tier 1 (primary): LinkedIn API
  - Tier 2 (fallback): Cached data
  - Tier 3 (final): Placeholder data or fail gracefully
  - Automatic tier switching on failure
  - Success rate tracking with fallback (format: `98% via fallback`)
- **Acceptance Criteria:** LinkedIn fails → cached data used, agent continues successfully
- **Demo Usage:** Slide 18 Part 3 (shows fallback activation)

**15. Safety Cutoff Metrics**
- **Feature:** Cost savings and reliability metrics from circuit breakers
- **Requirements:**
  - Requests blocked count (format: `66 leads`)
  - Cost savings from fallback (format: `$6.60 saved`)
  - Service-specific state tracking (per API endpoint)
  - Recovery detection (auto-close circuit after cooldown)
- **Acceptance Criteria:** Circuit opens → metrics show blocked requests and cost savings
- **Demo Usage:** Slide 18 Part 3 (shows cost saved by using cache)

---

### DEMO INFRASTRUCTURE

**Scope:** Lead generation agent and control panel for conference presentation

#### Week 8: Lead Generation Agent

**16. Sample Agent Implementation**
- **Feature:** Complete lead generation agent for demo
- **Requirements:**
  - LangChain-based lead gen agent (Python)
  - LinkedIn API integration (real or mocked)
  - Clearbit API integration (enrichment data)
  - 100-lead test dataset (CSV with company names)
  - Configurable processing rate (throttling)
- **Acceptance Criteria:** Agent processes 100 leads end-to-end without manual intervention
- **Demo Usage:** Entire demo runs this agent

**17. Agent Instrumentation**
- **Feature:** Progress and performance tracking
- **Requirements:**
  - Progress tracking (format: `23 / 100 leads`)
  - Success/failure counting per lead
  - Latency measurement (P50, P95, P99)
  - Throughput calculation (format: `212 leads/hour`)
- **Acceptance Criteria:** Control Panel shows all metrics, matches actual agent performance
- **Demo Usage:** Slide 18 Part 2 (Control Panel shows progress)

**18. Demo Triggers**
- **Feature:** Controlled failure injection for demonstration
- **Requirements:**
  - Simulated LinkedIn rate limit at lead #34 (for circuit breaker demo)
  - Injected PII in output at lead #67 (for redaction demo)
  - Budget threshold trigger at lead #85 (for alert demo)
  - Configurable failure injection (for testing)
- **Acceptance Criteria:** Demo runs deterministically, triggers fire at expected leads
- **Demo Usage:** All demo scenarios (Parts 3, 4, 5)

---

### DASHBOARD & MONITORING

**Scope:** Real-time web control panel matching talk_outline.md exactly

#### Week 8: Real-Time Control Panel

**19. Live Metrics Display**
- **Feature:** Real-time agent status monitoring
- **Requirements:**
  - Agent status indicator (format: `🟢 Running`)
  - Progress bar (format: `23 / 100 leads (23%)`)
  - Success rate percentage (format: `100% (23/23)`)
  - Auto-refresh (WebSocket or 1-second polling)
- **Acceptance Criteria:** Control Panel updates within 1 second of agent state change
- **Demo Usage:** Slide 18 Part 2 (entire control panel shown)

**20. Cost Control Panel**
- **Feature:** Budget tracking visualization
- **Requirements:**
  - Current spend display (format: `$5.29`)
  - Budget limit display (format: `$50.00`)
  - Usage percentage (format: `10.6% used`)
  - Cost per unit (format: `$0.23 avg/lead`)
  - Budget projection with status icon (✅ under budget, ⚠️ approaching limit)
- **Acceptance Criteria:** Cost updates in real-time, projection is accurate ±5%
- **Demo Usage:** Slide 18 Part 2 (Cost Control section)

**21. Protection Panel**
- **Feature:** Real-time safety violation tracking
- **Requirements:**
  - PII leaks blocked counter (real-time increment)
  - Unauthorized actions counter
  - Prompt injection attempts counter
  - Status icons (✅/⚠️/🔴)
- **Acceptance Criteria:** privacy protection at lead #67 → counter increments immediately
- **Demo Usage:** Slide 18 Part 2 (Safety section)

**22. Performance Panel**
- **Feature:** Agent performance metrics
- **Requirements:**
  - Throughput display (format: `212 leads/hour`)
  - Latency percentiles (format: `P95: 2.1s`)
  - Cache hit rate (format: `34%`)
  - Uptime percentage
- **Acceptance Criteria:** Metrics calculated from actual agent telemetry
- **Demo Usage:** Slide 18 Part 2 (Performance section)

**23. Event Log Stream**
- **Feature:** Scrolling event feed with severity colors
- **Requirements:**
  - Real-time event feed (scrolling log)
  - Color-coded severity (INFO blue, WARN yellow, ERROR red)
  - Timestamp for each event (format: `[14:23:45]`)
  - Expandable event details (click to see full context)
- **Acceptance Criteria:** Events appear within 500ms of occurrence
- **Demo Usage:** Slide 18 (shown throughout demo)

**24. Notifications**
- **Feature:** Pop-up alerts for critical events
- **Requirements:**
  - safety cutoff alert (detailed breakdown)
  - privacy protection alert (sanitized preview)
  - Budget warning alert (with action buttons)
  - Auto-dismiss or manual close
- **Acceptance Criteria:** Alert appears within 1 second of event, dismissible
- **Demo Usage:** Slide 18 Parts 3, 4, 5 (alerts shown)

---

### SUPPORTING INFRASTRUCTURE

**Scope:** Minimal data storage and API for control panel communication

#### Week 3: Data Storage

**25. State Management**
- **Feature:** In-memory and persistent state storage
- **Requirements:**
  - In-memory state (Rust HashMap or DashMap for agent state)
  - Redis for distributed state (optional, for multi-instance)
  - SQLite for audit logs (local persistence)
  - Log rotation (prevent disk fill, 7-day retention)
- **Acceptance Criteria:** Agent state persists across control panel refreshes
- **Demo Usage:** Control Panel queries agent state

**26. API & Communication**
- **Feature:** REST API and WebSocket for control panel
- **Requirements:**
  - REST API for control panel queries (GET `/agent/:id/status`)
  - WebSocket for real-time events (control panel live updates)
  - HTTP endpoints for control (POST `/agent/:id/stop`)
  - CORS configuration (allow control panel origin)
- **Acceptance Criteria:** Control Panel connects via WebSocket, receives events in <500ms
- **Demo Usage:** Control Panel communicates with runtime

---

### TESTING & QUALITY

**Scope:** Demo reliability and error handling

#### Week 8: Demo Testing

**27. Demo Testing**
- **Feature:** Automated testing of demo scenarios
- **Requirements:**
  - End-to-end demo script test (automated run)
  - privacy protection accuracy test (known patterns)
  - safety cutoff trigger test (forced failure)
  - Budget enforcement test (exceed limit)
  - Control Panel load test (100+ concurrent viewers)
- **Acceptance Criteria:** Demo runs 10 times successfully without manual intervention
- **Demo Usage:** Pre-conference validation

**28. Error Handling**
- **Feature:** Graceful degradation and recovery
- **Requirements:**
  - Agent crash recovery (restart with state preservation)
  - Network failure graceful degradation
  - Invalid agent configuration error messages
  - LLM API timeout handling (retry with exponential backoff)
- **Acceptance Criteria:** Agent crashes → auto-restarts within 5 seconds
- **Demo Usage:** Reliability during demo

---

### CAPABILITY 5: Secrets Management (NEW - Pilot Extension)

**Business Context:** Company requires centralized secrets management for agent credentials, API keys, and database passwords. Team has domain expertise in secrets management and can develop this efficiently.

**Pilot Scope:** Minimal secrets management for pilot agents (not full enterprise Vault integration)

**Architecture Decision:**
- **Pilot:** Custom implementation (iron_secrets) with AES-256-GCM encryption
- **Full Platform:** Thin wrapper around HashiCorp Vault
- **Rationale:** Pilot doesnt need enterprise Vault features (HSM, dynamic secrets, lease management). Simple encrypted storage sufficient for 28-day timeline.

---

#### Week 7.5 (Days 50-52): Secrets Management Sprint

**29. Secret Storage & Encryption**
- **Feature:** Encrypted storage of agent secrets in SQLite
- **Requirements:**
  - Secrets stored encrypted in SQLite (AES-256-GCM)
  - Each secret has unique salt (16 bytes) and nonce (12 bytes)
  - Master key derived from environment variable using Argon2id
  - Secrets never logged or exposed in plaintext
  - Encryption parameters follow OWASP recommendations (Argon2: m=19456 KiB, t=2, p=1)
- **Acceptance Criteria:** Secret created → encrypted blob in database, decryption returns original value
- **Demo Usage:** HIGH - Shows enterprise-grade security (differentiation from competitors)
- **Out of Scope (Full Platform):**
  - Hardware Security Modules (HSM)
  - Dynamic secret generation
  - Automatic rotation policies

**30. Secret CRUD API**
- **Feature:** REST API endpoints for secret management
- **Requirements:**
  - `POST /secrets` - Create new secret (returns secret ID)
  - `GET /secrets` - List all secrets (with masking: `sk-proj-abc...xyz`)
  - `GET /secrets/{id}` - Get specific secret (full value for agents, masked for viewers)
  - `PUT /secrets/{id}` - Update secret value
  - `DELETE /secrets/{id}` - Delete secret (soft delete with audit trail)
  - All endpoints require authentication (JWT tokens)
- **Acceptance Criteria:** Create secret via API → returns ID, subsequent GET returns masked value
- **Demo Usage:** HIGH - Live demo of adding OpenAI key via control panel
- **Example Request:**
```json
POST /secrets
{
  "name": "OPENAI_API_KEY",
  "value": "sk-proj-1234567890abcdef",
  "environment": "production"
}
```
- **Example Response:**
```json
{
  "id": "secret-abc123",
  "name": "OPENAI_API_KEY",
  "environment": "production",
  "created_at": "2025-01-17T10:34:52Z",
  "updated_at": "2025-01-17T10:34:52Z"
}
```

**31. Role-Based Access Control (RBAC)**
- **Feature:** Role-based access control for secrets
- **Requirements:**
  - Three roles implemented: Admin (full CRUD), Viewer (read masked), Agent (read full, runtime-only)
  - Role enforcement at API level (middleware checks JWT claims)
  - Audit log records role with every operation
- **Acceptance Criteria:** Viewer role requests secret → receives masked value (`sk-proj-abc...xyz`)
- **Demo Usage:** MEDIUM - Can show masked secrets in control panel
- **Out of Scope (Full Platform):**
  - Fine-grained policies (path-based, tag-based)
  - Multi-factor authentication
  - SSO integration

**32. Agent Secret Injection**
- **Feature:** Automatic secret injection into agent environment variables
- **Requirements:**
  - Secrets injected at agent spawn time (before Python process starts)
  - Secrets available as environment variables: `os.environ["OPENAI_API_KEY"]`
  - Secrets filtered by environment (dev agents get dev secrets, prod agents get prod secrets)
  - Secrets never logged during injection
  - Zero-downtime rotation: SIGUSR1 signal triggers agent reload with new secrets
- **Acceptance Criteria:** Agent starts → `os.environ["OPENAI_API_KEY"]` contains decrypted secret value
- **Demo Usage:** CRITICAL - Agents use secrets seamlessly (no hardcoded keys visible)
- **Integration Flow:**
```
iron_runtime starts agent
  ↓
Query iron_secrets for agent's required secrets
  ↓
Decrypt secrets (AES-256-GCM)
  ↓
Set environment variables
  ↓
Spawn Python agent process (secrets available in os.environ)
```

**33. Secrets Control Panel Panel**
- **Feature:** Control Panel panel for viewing and managing secrets
- **Requirements:**
  - New 7th control panel panel (after Performance panel)
  - Display list of all secrets (name, environment, created date, updated date)
  - Secrets masked by default: `OPENAI_API_KEY: sk-proj-abc...xyz`
  - "Reveal" button for admins (full value visible on click)
  - "Add Secret" button opens modal form
  - "Edit" button opens modal with current value pre-filled
  - "Delete" button requires confirmation
  - Control Panel polls `/secrets` endpoint every 5 seconds
- **Acceptance Criteria:** Control Panel shows secrets list, "Add Secret" form works, secrets created successfully
- **Demo Usage:** CRITICAL - Visual impact for conference demo (Slide 18 live demo)
- **UI Mockup:**
```
┌────────────────────────────────────────────────────┐
│ Secrets Management                       [+ Add]   │
├────────────────────────────────────────────────────┤
│                                                    │
│ Name                Environment    Value          │
│ ───────────────────────────────────────────────── │
│ OPENAI_API_KEY      Production     sk-proj-ab...z │ [👁️] [✏️] [🗑️]
│ ANTHROPIC_KEY       Production     sk-ant-12...89 │ [👁️] [✏️] [🗑️]
│ CLEARBIT_KEY        Development    pk_test_45...67│ [👁️] [✏️] [🗑️]
│                                                    │
└────────────────────────────────────────────────────┘
```

**34. Secret Audit Trail**
- **Feature:** Immutable audit log of all secret operations
- **Requirements:**
  - All secret operations logged to `secret_audit_log` table (SQLite)
  - Log fields: `id`, `secret_name`, `operation`, `requester_role`, `agent_id`, `success`, `error_message`, `timestamp`
  - Operations logged: CREATE, READ, UPDATE, DELETE
  - Logs immutable (append-only, no updates or deletes)
  - Logs queryable via API: `GET /secrets/audit?secret_name=OPENAI_API_KEY`
  - Logs redact secret values (never log plaintext secrets)
- **Acceptance Criteria:** Create secret → audit log entry created with all fields
- **Demo Usage:** LOW - Not visible in demo, but important for pitch deck
- **Example Audit Log Entry:**
```json
{
  "id": 1,
  "secret_name": "OPENAI_API_KEY",
  "operation": "READ",
  "requester_role": "agent",
  "agent_id": "lead-gen-bot-v2",
  "success": true,
  "error_message": null,
  "timestamp": "2025-01-17T10:34:52Z"
}
```

**35. Secret Rotation Workflow**
- **Feature:** Zero-downtime secret rotation
- **Requirements:**
  - Update secret via API: `PUT /secrets/{id}` with new value
  - Send SIGUSR1 signal to running agent process
  - Agent signal handler reloads secrets from iron_secrets
  - Agent continues running with new secret (no restart)
  - Zero downtime rotation (agent doesnt drop in-flight requests)
- **Acceptance Criteria:** Update secret → send SIGUSR1 → agent uses new secret without restart
- **Demo Usage:** CRITICAL - Live demo at lead #50 (rotate OpenAI key, agent continues)
- **Rotation Flow:**
```
Admin updates secret in control panel
  ↓
API updates encrypted value in SQLite
  ↓
API sends SIGUSR1 to agent process (via iron_runtime)
  ↓
Agent signal handler re-fetches secrets from iron_secrets
  ↓
Agent updates os.environ with new secret value
  ↓
Agent continues processing requests with new secret
```
- **Out of Scope (Full Platform):**
  - Automatic rotation policies (rotate every 90 days)
  - Secret versioning (rollback to previous value)
  - Multi-secret atomic rotation (update multiple secrets together)

---

## Security Architecture

**Scope:** Cryptographic requirements for authentication, authorization, and data protection

### Authentication & Authorization

**API Token Authentication:**
- API tokens for service-to-service authentication
- JWT tokens for user session management
- Role-based access control (Admin, Viewer, Agent)
- Token rotation support (zero-downtime secret updates)

### Hashing Requirements

**Critical Security Principle:** Hashing algorithm selection MUST be based on input entropy, not use case.

#### API Token Hashing (HIGH-ENTROPY)

**Requirements:**
- **Algorithm:** SHA-256 (fast, deterministic)
- **Input:** Cryptographically random tokens (256 bits entropy, generated via `rand::thread_rng()`)
- **Output:** 64-character hex-encoded hash
- **Storage:** Only hash stored in database, never plaintext token
- **Verification:** Deterministic comparison (`hash(token) == stored_hash`)

**Rationale:**
- API tokens have 256 bits of entropy (cryptographically random)
- SHA-256 provides collision resistance for high-entropy inputs
- Deterministic hashing enables fast database lookups via unique index
- No salt needed for cryptographically random values
- Brute-force attacks infeasible (2^256 search space)

**Implementation:** `iron_token_manager::TokenGenerator::hash_token()`

#### Password Hashing (LOW-ENTROPY)

**Requirements:**
- **Algorithm:** BCrypt with cost factor 12
- **Input:** User-chosen passwords (40-60 bits entropy typical)
- **Output:** BCrypt hash with embedded salt (60 characters, format: `$2b$12$...`)
- **Storage:** BCrypt hash in database
- **Verification:** BCrypt verification function (handles salt automatically)

**Rationale:**
- User passwords have low entropy (dictionary words, patterns, reuse)
- BCrypt's adaptive cost slows down brute-force attacks
- Random salt prevents rainbow table attacks
- Cost factor 12 balances security vs performance (250ms verification time)

**Implementation:** User authentication module (passwords only)

### Prohibited Patterns

**MUST NOT use BCrypt for API tokens:**
- BCrypt's random salt breaks database lookups (non-deterministic hashing)
- BCrypt's slow cost is unnecessary for high-entropy tokens
- BCrypt designed for LOW-ENTROPY passwords, not cryptographically random tokens

**MUST NOT use SHA-256 for passwords:**
- SHA-256 is too fast for low-entropy passwords (enables brute-force)
- SHA-256 has no salt (vulnerable to rainbow tables)
- Passwords require adaptive cost and salting (BCrypt/Argon2)

### Decision Criteria

**Use SHA-256 when:**
- Input has ≥128 bits of cryptographic entropy
- Examples: API tokens, session IDs, CSRF tokens, random keys
- Property: Cryptographically random, not user-chosen

**Use BCrypt/Argon2 when:**
- Input has <100 bits of entropy
- Examples: Passwords, PINs, security questions
- Property: User-chosen, low entropy, requires protection

### Encryption Requirements

**Secret Storage (AES-256-GCM):**
- Master key derived via Argon2id (m=19456 KiB, t=2, p=1)
- Unique salt (16 bytes) and nonce (12 bytes) per secret
- Authenticated encryption (prevents tampering)
- Implementation: `iron_secrets` crate

**See Also:**
- OWASP Password Storage Cheat Sheet
- NIST SP 800-63B (Digital Identity Guidelines)
- `iron_token_manager/src/token_generator.rs` (SHA-256 implementation with Fix comment)
- `iron_api/tests/tokens/corner_cases.rs` (hash format validation tests)

---

## Week-by-Week Implementation Plan

### Week 1-2: Core Runtime (Cap 8 Minimal)
- **Features:** #1-4 (Lifecycle, PyO3, Config, Logging)
- **Team:** 1 engineer (Rust + Python experience)
- **Deliverable:** `iron_cage start agent.py` works, agent runs to completion
- **Risk:** PyO3 async bridge complexity
- **Mitigation:** Use pyo3-asyncio examples, test with simple agent first

### Week 3: Infrastructure & Storage
- **Features:** #25-26 (State, API, WebSocket)
- **Team:** 1 engineer (backend, Rust + web)
- **Deliverable:** Control Panel can connect and show agent status
- **Risk:** WebSocket concurrency issues
- **Mitigation:** Use Axum WebSocket example, test with 100 concurrent connections

### Week 4-5: Safety (Cap 2 Lite)
- **Features:** #5-8 (privacy protection, redaction, audit, policy)
- **Team:** 1 engineer (Rust regex, security experience)
- **Deliverable:** PII email detected and redacted in real-time
- **Risk:** False positives (redacting non-PII emails)
- **Mitigation:** Whitelist domains (e.g., "@company.com" excluded), tune regex

### Week 6-7: Cost & Safety Cutoff (Cap 3 Lite)
- **Features:** #9-15 (Token counting, budgets, alerts, circuit breaker, fallback)
- **Team:** 1 engineer (Rust + LLM API experience)
- **Deliverable:** Budget warning at 90%, circuit breaker triggers on API failure
- **Risk:** Token counting accuracy (Anthropic approximation)
- **Mitigation:** Cross-check with actual API billing, adjust formula

### Week 8: Demo Integration & Polish
- **Features:** #16-24, #27-28 (Lead gen agent, control panel, testing)
- **Team:** 2 engineers (1 backend, 1 frontend)
- **Deliverable:** Full demo matches talk_outline.md exactly
- **Risk:** Demo timing (must complete in <30 minutes)
- **Mitigation:** Pre-process leads, cache API responses, tune throttling

---

## Success Criteria

### Conference Demo (Critical)
- ✅ **Zero crashes:** Demo runs 30 minutes without errors
- ✅ **Visual match:** Control Panel matches talk_outline.md screenshots
- ✅ **Trigger accuracy:** PII fires at lead #67, budget at #85, circuit breaker at #34
- ✅ **Timing:** Demo completes in 28-30 minutes (allows buffer for Q&A)
- ✅ **Audience experience:** Clear visuals, no lag, professional appearance

### Pilot Sales (High Priority)
- ✅ **Integration time:** Customer integrates single agent in <8 hours
- ✅ **Documentation:** README with quick start, API docs, troubleshooting
- ✅ **Support:** Response to customer questions within 4 hours (business hours)
- ✅ **Reliability:** 99% uptime during 90-day pilot
- ✅ **Value delivery:** Customer sees measurable cost savings OR PII protection

### Market Validation (Medium Priority)
- ✅ **Customer feedback:** 5+ customer interviews during pilots
- ✅ **Feature requests:** Top 3 requested features documented
- ✅ **Pricing validation:** 2+ customers willing to pay $100K+ for full platform
- ✅ **Capability priority:** Clear signal on Cap 1 vs Cap 2 vs Full Platform
- ✅ **Churn:** <20% churn after 90-day pilot

---

## Out of Scope (Explicitly NOT Included)

### Full Capability Features

**Cap 1 (Enterprise Data Access) - NOT INCLUDED:**
- ❌ No RAG platform
- ❌ No vector database management
- ❌ No ETL connectors (Salesforce, SharePoint, etc.)
- ❌ No real-time data sync

**Cap 2 (Full Safety) - SUBSET ONLY:**
- ❌ No prompt injection detection
- ❌ No content filtering (hate speech, violence)
- ❌ No jailbreak prevention
- ❌ No tool authorization engine

**Cap 3 (Full Cost Control) - SUBSET ONLY:**
- ❌ No multi-LLM routing
- ❌ No LLM fallback chains (Tier 1: OpenAI, Tier 2: Azure)
- ❌ No governance layer (RBAC, policies)

**Cap 4 (Safe Execution) - NOT INCLUDED:**
- ❌ No container isolation
- ❌ No filesystem controls
- ❌ No network isolation

**Cap 5 (Secrets) - SUBSET INCLUDED:**
- ✅ Basic encrypted storage (AES-256-GCM, SQLite)
- ✅ Secret CRUD API and control panel panel
- ✅ Agent secret injection (environment variables)
- ✅ Zero-downtime rotation (SIGUSR1 signal)
- ❌ No Vault integration (pilot uses custom crypto)
- ❌ No HSM support (master key from environment variable)
- ❌ No automatic rotation policies

**Cap 6 (Full Observability) - NOT INCLUDED:**
- ❌ No Prometheus integration
- ❌ No Grafana control panels
- ❌ No distributed tracing (Jaeger)

**Cap 7 (MCP) - NOT INCLUDED:**
- ❌ No Model Context Protocol integration

**Cap 8 (Full Runtime) - MINIMAL ONLY:**
- ❌ No Kubernetes deployment
- ❌ No multi-tenancy
- ❌ No RBAC
- ❌ No HA configuration

### Production Hardening

**NOT INCLUDED:**
- ❌ SOC 2 certification
- ❌ HIPAA compliance certification
- ❌ Multi-cloud deployment (AWS, Azure, GCP)
- ❌ Auto-scaling
- ❌ Disaster recovery
- ❌ 24/7 support
- ❌ SLA guarantees

---

## Evolution Path

### After 8-Week Pilot Platform

**Scenario A: Customer feedback prioritizes Safety (Cap 2)**
→ Build full Cap 2 (4-6 months, 2-3 engineers, $100-200K/year pricing)

**Scenario B: Customer feedback prioritizes Data Access (Cap 1)**
→ Build full Cap 1 (6-9 months, 3-4 engineers, $150-300K/year pricing)

**Scenario C: Customers want full integrated platform**
→ Build remaining capabilities (Cap 2+3 full, add Cap 4+5+6)
→ Platform pricing: $100-300K/year

**Scenario D: Insufficient traction (<2 pilots signed)**
→ Pivot to consulting services
→ Use pilot platform as proof-of-concept for consulting engagements

### Feature Addition Priority (Based on Customer Feedback)

**If customers request:**
1. **More PII types** (credit cards, passports) → Add to Cap 2 Lite (2 weeks)
2. **Multi-LLM support** (Azure, Anthropic) → Expand Cap 3 Lite (3 weeks)
3. **Advanced control panels** (custom queries, exports) → Add to Control Panel (2 weeks)
4. **Kubernetes deployment** → Build Cap 8 K8s wrapper (4 weeks)
5. **Data connectors** (Salesforce, Slack) → Start Cap 1 build (6-9 months)

---

## Module Architecture

### Overview

The pilot platform consists of **9 internal modules** (8 Rust crates + 1 React application) organized into 4 layers:

```
┌─────────────────────────────────────────────────────┐
│  Layer 4: Applications (2 modules)                  │
│  - iron_cli: CLI binary (Rust)                      │
│  - iron_control: Web UI (React/TypeScript)          │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│  Layer 3: Integration (2 modules)                   │
│  - iron_runtime: agent management + PyO3 bridge      │
│  - iron_api: REST + WebSocket server                │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│  Layer 2: Domain Logic (3 modules)                  │
│  - iron_safety: privacy protection + redaction      │
│  - iron_budget: Budget tracking + enforcement       │
│  - iron_reliability: safety cutoff + fallback       │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│  Layer 1: Foundation (3 modules)                    │
│  - iron_types: Shared types + error definitions     │
│  - iron_state: State management (in-memory + DB)    │
│  - iron_telemetry: Logging + tracing abstraction    │
└─────────────────────────────────────────────────────┘
```

**Total:** 9 modules (8 Rust crates + 1 React/TypeScript application)

---

### Layer 1: Foundation Modules

#### 1. `iron_types` (EXISTS - Expand)

**Responsibility:** Shared types, traits, and error definitions

**Features Mapped:**
- All features (foundational types used everywhere)

**Exports:**
```rust
// Core types
pub struct AgentId(String);
pub struct Budget { limit: f64, spent: f64 }
pub struct PiiDetection { pattern_type: PiiType, location: usize }

// Error types
pub enum Error {
  AgentError(String),
  BudgetExceeded { limit: f64, requested: f64 },
  PiiDetected { pii_type: PiiType },
  // ...
}

pub type Result<T> = std::result::Result<T, Error>;
```

**LOC:** ~800 lines (current ~200, expand to ~800)

**External Dependencies:** serde, thiserror

**Status:** ✅ Exists, needs expansion

---

#### 2. `iron_state` (NEW)

**Responsibility:** State management (in-memory + persistent storage)

**Features Mapped:**
- Feature #25: State Management (in-memory + SQLite + optional Redis)

**Exports:**
```rust
pub struct StateManager {
  memory: Arc<DashMap<AgentId, AgentState>>,
  db: Option<SqlitePool>,
}

impl StateManager {
  pub async fn get_agent_state(&self, id: &AgentId) -> Result<AgentState>;
  pub async fn save_agent_state(&self, id: &AgentId, state: AgentState) -> Result<()>;
  pub async fn save_audit_log(&self, event: AuditEvent) -> Result<()>;
}
```

**LOC:** ~600 lines

**External Dependencies:** dashmap, sqlx (sqlite), serde

**Status:** ❌ Not created

**File Structure:**
```
iron_state/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API
│   ├── memory.rs        # DashMap in-memory state
│   ├── sqlite.rs        # SQLite persistence
│   └── redis.rs         # Optional Redis (feature-gated)
└── tests/
    └── state_test.rs
```

---

#### 3. `iron_telemetry` (NEW)

**Responsibility:** Centralized logging and tracing abstraction

**Features Mapped:**
- Feature #4: Logging Infrastructure

**Exports:**
```rust
pub fn init_logging(level: LogLevel) -> Result<()>;

pub fn log_agent_event(agent_id: &AgentId, event: &str);
pub fn log_pii_detection(agent_id: &AgentId, detection: &PiiDetection);
pub fn log_budget_warning(agent_id: &AgentId, budget: &Budget);
```

**LOC:** ~400 lines

**External Dependencies:** tracing, tracing-subscriber

**Status:** ❌ Not created

**File Structure:**
```
iron_telemetry/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API
│   ├── init.rs          # Logger initialization
│   └── formatters.rs    # JSON/text formatters
└── tests/
    └── logging_test.rs
```

---

### Layer 2: Domain Logic Modules

#### 4. `iron_safety` (EXISTS - Expand)

**Responsibility:** privacy protection, redaction, and audit logging

**Features Mapped:**
- Feature #5: PII Pattern Detection
- Feature #6: Real-Time Output Redaction
- Feature #7: PII Audit Logging
- Feature #8: Policy Enforcement

**Current Exports:**
```rust
pub struct PiiDetector { /* existing */ }
```

**New Exports Needed:**
```rust
pub struct PiiRedactor {
  detector: PiiDetector,
  policy: RedactionPolicy,
}

pub enum RedactionPolicy {
  Redact,  // Replace with [EMAIL_REDACTED]
  Block,   // Stop agent execution
  Warn,    // Log only
}

impl PiiRedactor {
  pub fn scan_and_redact(&self, text: &str) -> (String, Vec<PiiDetection>);
  pub fn audit_detection(&self, detection: &PiiDetection) -> Result<()>;
}
```

**LOC:** ~1200 lines (current ~300, expand to ~1200)

**External Dependencies:** regex, serde

**Status:** ✅ Exists, needs expansion for Features #6-8

---

#### 5. `iron_budget` (EXISTS - Expand)

**Responsibility:** Budget tracking, enforcement, and cost attribution

**Features Mapped:**
- Feature #9: Real-Time Token Counting
- Feature #10: Budget Limits & Alerts
- Feature #11: Alert System
- Feature #12: Cost Attribution

**Current Exports:**
```rust
pub struct BudgetTracker { /* existing */ }
```

**New Exports Needed:**
```rust
pub struct CostTracker {
  budget: Budget,
  alert_threshold: f64,  // 0.9 for 90%
  per_request_costs: Vec<RequestCost>,
}

impl CostTracker {
  pub fn track_request(&mut self, tokens: usize, model: &str) -> Result<f64>;
  pub fn check_budget(&self) -> BudgetStatus;
  pub fn send_alert(&self, alert_type: AlertType) -> Result<()>;
}

pub enum BudgetStatus {
  Ok,
  Warning(f64),  // Percentage used
  Exceeded,
}
```

**LOC:** ~1000 lines (current ~250, expand to ~1000)

**External Dependencies:** serde, reqwest (for webhooks)

**Status:** ✅ Exists, needs expansion for Features #10-12

---

#### 6. `iron_reliability` (EXISTS)

**Responsibility:** safety cutoff, fallback chains, failure tracking

**Features Mapped:**
- Feature #13: Safety Cutoff State Machine
- Feature #14: Fallback Chain
- Feature #15: Safety Cutoff Metrics

**Current Exports:**
```rust
pub struct CircuitBreaker { /* complete */ }
```

**LOC:** ~600 lines (complete)

**External Dependencies:** tokio, serde

**Status:** ✅ Exists, complete

---

### Layer 3: Integration Modules

#### 7. `iron_runtime` (NEW - CRITICAL)

**Responsibility:** agent management management + PyO3 bridge + Python demo agent

**Features Mapped:**
- Feature #1: Agent Management
- Feature #2: Python-Rust Integration (PyO3)
- Feature #3: Configuration System (partial, CLI parsing in iron_cli)
- Feature #16: Sample Agent Implementation (Python demo in python/examples/)
- Feature #17: Agent Instrumentation (Python imports Rust via PyO3)
- Feature #18: Demo Triggers (PII, budget, circuit breaker triggers in demo)

**Exports:**
```rust
pub struct AgentRuntime {
  config: RuntimeConfig,
  state: Arc<StateManager>,
  safety: Arc<PiiRedactor>,
  cost: Arc<CostTracker>,
  reliability: Arc<CircuitBreaker>,
}

impl AgentRuntime {
  pub async fn start_agent(&self, script_path: &Path) -> Result<AgentHandle>;
  pub async fn stop_agent(&self, agent_id: &AgentId) -> Result<()>;
  pub async fn intercept_llm_call(&self, request: &LlmRequest) -> Result<LlmResponse>;
}

// PyO3 bridge
#[pyfunction]
fn register_runtime_hooks(py: Python) -> PyResult<()>;
```

**Python API (exposed via PyO3):**
```python
# python/__init__.py
from typing import Optional

class Runtime:
  """Iron Cage runtime for AI agent safety and cost control"""

  def __init__(self, budget: float, verbose: bool = False): ...
  def start_agent(self, script_path: str) -> str: ...  # Returns agent_id
  def stop_agent(self, agent_id: str) -> None: ...
  def get_metrics(self, agent_id: str) -> dict: ...

# Usage in lead_gen_agent.py:
# from iron_runtime import Runtime
# runtime = Runtime(budget=50.0)
# agent_id = runtime.start_agent("lead_gen_agent.py")
```

**LOC:** ~1500 lines Rust + ~500 lines Python

**External Dependencies:**
- Rust: tokio, pyo3, pyo3-asyncio, anyhow
- Python: langchain, openai, typing-extensions

**Status:** ❌ Not created (CRITICAL PATH)

**File Structure:**
```
iron_runtime/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API
│   ├── lifecycle.rs     # Agent spawning/monitoring
│   ├── pyo3_bridge.rs   # Python FFI (Feature #2)
│   ├── config.rs        # RuntimeConfig
│   └── integration.rs   # Integrate safety/cost/reliability
├── python/
│   ├── __init__.py      # Python module (PyO3 bindings)
│   ├── iron_runtime.pyi # Type stubs for IDE
│   ├── requirements.txt # Python dependencies (langchain, openai)
│   └── examples/
│       └── lead_gen_agent.py  # Demo agent (Features #16-18)
└── tests/
    ├── runtime_test.rs
    └── pyo3_integration_test.rs
```

---

#### 8. `iron_api` (NEW)

**Responsibility:** REST API + WebSocket server for control panel

**Features Mapped:**
- Feature #26: API & Communication (REST + WebSocket)

**Exports:**
```rust
pub struct ApiServer {
  state: Arc<StateManager>,
  addr: SocketAddr,
}

impl ApiServer {
  pub async fn start(self) -> Result<()>;
}

// REST endpoints
// GET /api/agents/:id/status
// POST /api/agents/:id/stop
// GET /api/agents/:id/metrics

// WebSocket: /ws (real-time event stream)
```

**LOC:** ~800 lines

**External Dependencies:** axum, tower, tower-http, tokio

**Status:** ❌ Not created

**File Structure:**
```
iron_api/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API (ApiServer)
│   ├── rest.rs          # REST endpoints
│   ├── websocket.rs     # WebSocket handler
│   └── handlers.rs      # Request handlers
└── tests/
    └── api_test.rs
```

---

### Layer 4: Application Modules

#### 9. `iron_cli` (EXISTS - Expand)

**Responsibility:** CLI binary, argument parsing, main entry point

**Features Mapped:**
- Feature #1: Agent Lifecycle (CLI command)
- Feature #3: Configuration System (CLI arguments)

**Current:**
```rust
// Basic CLI exists
fn main() { /* minimal */ }
```

**Needs:**
```rust
use clap::Parser;
use iron_runtime::AgentRuntime;

#[derive(Parser)]
struct Cli {
  #[arg(long)]
  budget: f64,

  #[arg(long)]
  verbose: bool,

  script: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
  let cli = Cli::parse();
  let runtime = AgentRuntime::new(RuntimeConfig {
    budget: cli.budget,
    log_level: if cli.verbose { LogLevel::Debug } else { LogLevel::Info },
  })?;

  runtime.start_agent(&cli.script).await?;
  Ok(())
}
```

**LOC:** ~500 lines (current ~100, expand to ~500)

**External Dependencies:** clap, tokio, anyhow

**Status:** ✅ Exists, needs integration with iron_runtime

---

#### 10. `iron_control` (NEW - React App, NOT Rust)

**Responsibility:** Web UI for real-time monitoring

**Features Mapped:**
- Feature #19: Live Metrics Display
- Feature #20: Cost Control Panel
- Feature #21: Protection Panel
- Feature #22: Performance Panel
- Feature #23: Event Log Stream
- Feature #24: Notifications

**Technology:** React 18 + TypeScript (NOT a Rust crate)

**Components:**
```typescript
// src/components/
BudgetPanel.tsx         // Feature #20
ProtectionPanel.tsx       // Feature #21
PerformancePanel.tsx  // Feature #22
ActivityLog.tsx          // Feature #23
Notification.tsx        // Feature #24
Control Panel.tsx         // Main layout

// src/api/
websocket.ts          // WebSocket client
rest.ts               // REST API client
```

**LOC:** ~2000 lines TypeScript

**External Dependencies:** react, recharts, websocket

**Status:** ❌ Not created

**File Structure:**
```
demo/control panel/
├── package.json
├── src/
│   ├── components/
│   │   ├── BudgetPanel.tsx
│   │   ├── ProtectionPanel.tsx
│   │   ├── PerformancePanel.tsx
│   │   ├── ActivityLog.tsx
│   │   └── Notification.tsx
│   ├── api/
│   │   ├── websocket.ts
│   │   └── rest.ts
│   └── App.tsx
└── public/
    └── index.html
```

---

## Module Dependency Graph

```
iron_cli ──────────────┐
                       ├──> iron_runtime ──┐
iron_api ──────────────┘                   │
                                           ├──> iron_safety ──┐
                                           ├──> iron_budget ────┤
                                           └──> iron_reliability ─┤
                                                                  │
                                                                  ├──> iron_types
                                                                  ├──> iron_state
                                                                  └──> iron_telemetry
```

**Clean DAG:** No circular dependencies, all flows from applications → integration → domain → foundation

---

## Feature-to-Module Mapping Table

| Feature # | Feature Name | Primary Module | Supporting Modules |
|-----------|--------------|---------------|-------------------|
| #1 | Agent Lifecycle | iron_runtime | iron_cli, iron_types |
| #2 | PyO3 Integration | iron_runtime | pyo3, pyo3-asyncio |
| #3 | Configuration | iron_cli | clap, serde |
| #4 | Logging | iron_telemetry | tracing |
| #5 | Privacy Protection | iron_safety | regex |
| #6 | Output Redaction | iron_safety | iron_types |
| #7 | PII Audit | iron_safety | iron_state |
| #8 | Policy Enforcement | iron_safety | iron_types |
| #9 | Token Counting | iron_budget | reqwest |
| #10 | Budget Limits | iron_budget | iron_types |
| #11 | Alert System | iron_budget | reqwest |
| #12 | Cost Attribution | iron_budget | iron_state |
| #13 | Safety Cutoff | iron_reliability | tokio |
| #14 | Fallback Chain | iron_reliability | iron_types |
| #15 | Circuit Metrics | iron_reliability | iron_state |
| #16 | Sample Agent | iron_runtime | langchain (Python) |
| #17 | Agent Instrumentation | iron_runtime | pyo3 bridge |
| #18 | Demo Triggers | iron_runtime | Python example |
| #19 | Live Metrics | iron_control | iron_api |
| #20 | Budget Panel | iron_control | iron_api |
| #21 | Protection Panel | iron_control | iron_api |
| #22 | Performance Panel | iron_control | iron_api |
| #23 | Event Log | iron_control | iron_api |
| #24 | Notifications | iron_control | iron_api |
| #25 | State Management | iron_state | dashmap, sqlx |
| #26 | API & WebSocket | iron_api | axum, tower |
| #27 | Demo Testing | (tests in all crates) | tokio-test |
| #28 | Error Handling | iron_types | thiserror, anyhow |

---

## Implementation Priority (Conference Focus)

**Critical Path (Must Have for Demo):**
1. ✅ iron_types (exists, minor expansion)
2. ❌ iron_telemetry (logging foundation)
3. ❌ iron_runtime (PyO3 bridge - BLOCKS EVERYTHING)
4. ✅ iron_safety (exists, expand redaction)
5. ✅ iron_budget (exists, expand alerts)
6. ✅ iron_reliability (exists, complete)

**High Priority (Demo Polish):**
7. ❌ iron_state (enables control panel)
8. ❌ iron_api (enables control panel)
9. ❌ iron_control (visual demo)
10. ❌ iron_runtime Python examples (demo agent - Features #16-18)

**Lower Priority (Can Skip for Conference):**
- Redis support in iron_state (use in-memory only)
- Advanced alerting in iron_budget (email can be mocked)
- Performance optimizations

---

## Workspace Structure

```
module/
├── iron_types/          # ✅ EXISTS
├── iron_safety/         # ✅ EXISTS (expand)
├── iron_budget/           # ✅ EXISTS (expand)
├── iron_reliability/    # ✅ EXISTS
├── iron_cli/            # ✅ EXISTS (expand)
├── iron_telemetry/      # ❌ NEW
├── iron_state/          # ❌ NEW
├── iron_runtime/        # ❌ NEW (CRITICAL)
│   └── python/
│       └── examples/
│           └── lead_gen_agent.py    # ❌ NEW (Features #16-18)
└── iron_api/            # ❌ NEW

pilot/
└── demo/
    └── control panel/
        └── src/                  # ❌ NEW (React)
```

**Total New Work:**
- 4 new Rust modules (iron_telemetry, iron_state, iron_runtime, iron_api)
  - iron_runtime includes Python module + demo agent
- 1 React/TypeScript application module (iron_control)
- Expand 3 existing Rust modules (iron_types, iron_safety, iron_budget)

**Total LOC:**
- New code: ~4,800 lines Rust + ~2,000 lines TypeScript + ~500 lines Python = ~7,300 lines
- Expanded code: ~1,500 lines Rust (in existing modules)
- **Grand total: ~8,800 lines**

---

## Next Steps

1. Create `iron_telemetry` (400 LOC Rust, 1-2 days)
2. Create `iron_state` (600 LOC Rust, 2-3 days)
3. Create `iron_runtime` (1500 LOC Rust + 500 LOC Python, 5-6 days) ← CRITICAL PATH
   - Includes PyO3 bridge
   - Includes Python module + demo agent (Features #16-18)
4. Expand `iron_safety` (+ 900 LOC, 2-3 days)
5. Expand `iron_budget` (+ 750 LOC, 2-3 days)
6. Create `iron_api` (800 LOC, 2-3 days)
7. Create `iron_control` (2000 LOC TypeScript, 4-5 days)

**With 3 developers:** Can parallelize Steps 1-6, then integrate in Step 7

---

## Technical Stack

### Runtime
- **Rust:** 1.75+ (Tokio async runtime, Axum web framework)
- **PyO3:** 0.20+ (Python FFI)
- **pyo3-asyncio:** 0.20+ (async bridge)

### Storage
- **SQLite:** 3.40+ (audit logs, local only)
- **Redis:** 7.0+ (optional, for distributed state)

### Control Panel
- **Frontend:** React 18 + TypeScript 5
- **Real-time:** WebSocket client
- **Visualization:** Recharts or Chart.js
- **Styling:** TailwindCSS

### Demo Agent
- **Python:** 3.11+
- **LangChain:** 0.1+
- **OpenAI SDK:** 1.0+

---

## Pricing & Packaging

### Pilot Platform Pricing

**Tier 1: Pilot (8-week trial)**
- **Price:** $10-25K one-time
- **Scope:** Single agent, single team
- **Support:** Email support (4-hour response time)
- **Features:** All 35+ features (with secrets management) included
- **Deliverables:** Integration assistance, 2 training sessions

**Tier 2: Production Lite (after successful pilot)**
- **Price:** $2-5K/month ($24-60K/year)
- **Scope:** Up to 5 agents, single organization
- **Support:** Email + Slack support (2-hour response time)
- **Features:** All pilot features + production hardening
- **SLA:** 99% uptime

**Upsell Path: Full Platform**
- **Price:** $100-300K/year
- **Scope:** Unlimited agents, full capabilities (Cap 1+2+3 complete)
- **Support:** 24/7 support, dedicated success manager
- **Features:** All 8 capabilities integrated
- **SLA:** 99.9% uptime

---

## Cross-References

### Implements
- **Demo script:** [business/presentations/talk_outline.md](conferences/warsaw_2025/presentation/talk_outline.md) - This spec implements the live demo from this presentation (Slide 18: Lead Generation Agent)
- **Conference strategy:** [business/presentations/talk_content_selector.md](conferences/warsaw_2025/presentation/talk_content_selector.md) - Pilot platform supports demo for all audience types

### Subset Of (Full Capabilities)
- **Safety:** [../spec/capability_2_ai_safety_guardrails.md](../spec/capability_2_ai_safety_guardrails.md) - Full Safety capability (this spec uses privacy protection subset)
- **Cost Control:** [../spec/capability_3_llm_access_control.md](../spec/capability_3_llm_access_control.md) - Full LLM Access Control capability (this spec uses budget tracking subset)
- **Runtime:** [../spec/capability_8_agent_runtime.md](../spec/capability_8_agent_runtime.md) - Full Runtime capability (this spec uses minimal Python-Rust gateway)

### Strategic Context
- **Build strategy:** [business/strategy/executive_summary.md](../business/strategy/executive_summary.md) - Pilot-first strategy as alternative to building Cap 1/2 first
- **Market positioning:** [business/strategy/capability_product_strategy.md](../business/strategy/capability_product_strategy.md) - Pilot platform as market validation vehicle

### Technical Architecture
- **System design:** [docs/architecture.md](../docs/architecture.md) - Full system architecture (this spec implements minimal subset)
- **Requirements:** [spec/requirements.md](../spec/requirements.md) - Functional requirements mapped to pilot features

---

### Deployment Architecture

**Pilot Platform (Current Implementation):**

The pilot platform uses **Pilot/Demo Mode** - a single-process architecture designed for conference demonstrations and localhost development.

**Architecture:**
```
Single Rust Process (localhost:8080)
├── iron_runtime (agent orchestration)
│   └── Arc<StateManager> (shared iron_state)
│         ├── DashMap (in-memory agent state)
│         └── SQLite (./iron_state.db audit logs)
│
├── iron_api (REST + WebSocket server)
│   ├── REST endpoints (GET /api/agents/:id)
│   └── WebSocket (/ws) for dashboard streaming
│       └── Subscribes to shared StateManager broadcasts
│
├── iron_safety (PII detection)
├── iron_cost (budget tracking)
├── iron_reliability (circuit breakers)
└── iron_secrets (secrets management)

Dashboard (localhost:5173)
└── Vite dev server (iron_dashboard Vue app)
    └── Connects via WebSocket to iron_api
```

**Key Characteristics:**
- **Single Process:** All modules run in same Rust binary
- **Shared State:** iron_state is shared Arc<StateManager> instance accessed by iron_runtime and iron_api
- **WebSocket Communication:** ws://localhost:8080/ws for real-time dashboard updates
- **Single Database:** Single SQLite file (./iron_state.db) for audit logs
- **Localhost Only:** No external network access, CORS allows localhost:5173

**Benefits for Conference Demo:**
- Zero deployment complexity (single `cargo run`)
- Sub-100ms dashboard latency (in-process broadcast channel)
- No database server required (SQLite file)
- Offline-capable (no cloud dependencies)

---

**Full Platform (Production Mode - Post-Pilot):**

After pilot validation, the full platform will use **Production Mode** - a distributed architecture for multi-user SaaS deployment.

**Architecture:**
```
Cloud: Control Panel (https://api.example.com)
├── iron_api (REST API server, Docker container)
│   ├── Token management endpoints
│   ├── Telemetry ingestion endpoints
│   └── iron_control_store (NEW module)
│       └── PostgreSQL
│           ├── users table
│           ├── api_tokens table
│           ├── secrets table
│           └── telemetry_events table
│
└── iron_dashboard (static UI, nginx)

Developer Machines (Local Agent Runtime)
├── Machine 1: Alice
│   ├── iron_runtime (PyPI package: iron-cage)
│   ├── iron_state (local SQLite: alice_state.db)
│   └── Optional telemetry → HTTPS POST to Control Panel
│
├── Machine 2: Bob
│   ├── iron_runtime (PyPI package: iron-cage)
│   ├── iron_state (local SQLite: bob_state.db)
│   └── Optional telemetry → HTTPS POST to Control Panel
```

**Key Differences from Pilot:**
- **Distributed:** Control Panel (cloud) + Agent Runtime (local machines)
- **Separate Databases:** PostgreSQL (Control Panel) + SQLite per agent (local)
- **No Shared State:** iron_state exists ONLY in Agent Runtime (not in Control Panel)
- **HTTPS Communication:** TLS-encrypted, not WebSocket
- **Multi-Tenant:** Control Panel serves multiple developer organizations
- **New Module:** iron_control_store replaces iron_state for Control Panel data

**Migration Path:**
1. **Pilot (Week 1-8):** Implement single-process architecture for conference demo
2. **Pilot Validation (Month 3-4):** Gather customer feedback, prove market fit
3. **Production Refactor (Month 5-6):** Extract iron_control_store, deploy distributed architecture
4. **Full Platform Launch (Month 7):** SaaS offering with cloud Control Panel

**See Also:**
- [docs/deployment_packages.md](../docs/deployment_packages.md) - Package definitions and deployment modes
- [docs/module_package_matrix.md](../docs/module_package_matrix.md) - Module-to-package mappings for both modes
- [docs/package_dependencies.md](../docs/package_dependencies.md) - Runtime dependencies between packages

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.4.0 | 2025-12-07 | [Team] | Added Deployment Architecture section explaining Pilot/Demo Mode (single process, shared iron_state) vs Production Mode (distributed, iron_control_store). Includes architecture diagrams, migration path, and cross-references to deployment docs. |
| 1.3.0 | 2025-12-06 | [Team] | Updated terminology: "Crate Architecture" → "Module Architecture" to reflect polyglot monorepo (Rust + TypeScript/Vue) |
| 1.2.0 | 2025-11-25 | [Team] | Added Capability 5 secrets management (Features #29-35, +iron_secrets module, +6 days, +$5.6K) |
| 1.1.1 | 2025-11-25 | [Team] | Moved demo_agent.py into iron_runtime module (9 modules total, better organization) |
| 1.1.0 | 2025-11-25 | [Team] | Added comprehensive Module Architecture section (9 modules: 8 Rust crates + 1 React app, feature mappings, ~650 lines) |
| 1.0.0 | 2025-11-22 | [Team] | Initial pilot platform specification - 28 features across 8 weeks |

---

**Status:** ✅ Ready for Implementation
**Next Steps:** Begin Week 1-2 implementation (Core Runtime)
**Questions:** Contact [email@company.com]
