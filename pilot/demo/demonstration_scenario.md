# Demonstration Scenario - Lead Generation Agent

**Version:** 1.0.0
**Created:** 2025-11-25
**Conference:** Warsaw EXPO XXI, Dec 16-17, 2025
**Duration:** 5 minutes (live demo within 25-minute talk)
**Status:** Specification Complete, Implementation Pending

---

## Deployment Mode

**Pilot/Demo Mode:** This demonstration uses Pilot Mode deployment (single process, localhost-only architecture).

**Architecture:**
- All components run on presenter laptop (offline capable)
- Control Panel: localhost:3000 (React dev server, WebSocket connection)
- Runtime API: localhost:3001 (Rust backend)
- No cloud services required for demo

**Production Mode:** For production deployment architecture (distributed, cloud-based Control Panel), see [docs/deployment_packages.md](../../docs/deployment_packages.md).

---

## Executive Summary

**Purpose:** Live demonstration of Iron Cage Runtime processing 100 synthetic leads with real-time safety, cost, and reliability controls.

**Demo Thesis:** "Production AI requires infrastructure. Watch how Iron Cage prevents $47K cost spirals, GDPR violations, and cascade failures—automatically."

**Success Criteria:**
- ✅ Zero crashes during 5-minute demo
- ✅ All 3 triggers fire correctly (circuit breaker #34, PII #67, budget #85)
- ✅ Control Panel updates in real-time (<100ms latency)
- ✅ Final metrics match spec (98% success rate, $23.14 total cost)

---

## Demo Architecture

### System Components

```
┌────────────────────────────────────────────────────────┐
│                    PRESENTER LAPTOP                    │
├────────────────────────────────────────────────────────┤
│                                                        │
│  Terminal (left half)          Control Panel (right half) │
│  ┌─────────────────────┐       ┌──────────────────┐  │
│  │ iron_cage CLI       │◄──────┤ React + WebSocket│  │
│  │ stdout: logs        │       │ localhost:3000   │  │
│  └──────────┬──────────┘       └────────▲─────────┘  │
│             │                            │             │
│             ▼                            │             │
│  ┌─────────────────────────────────────────────────┐  │
│  │  Iron Cage Runtime (Rust)                       │  │
│  │  ├─ PyO3 bridge                                 │  │
│  │  ├─ iron_safety (privacy protection)                 │  │
│  │  ├─ iron_budget (budget tracking)                 │  │
│  │  ├─ iron_reliability (circuit breaker)          │  │
│  │  └─ iron_control_api (WebSocket server :3001)          │  │
│  └──────────┬──────────────────────────────────────┘  │
│             │                                          │
│             ▼                                          │
│  ┌─────────────────────────┐                          │
│  │ lead_gen_agent.py       │                          │
│  │ (LangChain + GPT-4)     │                          │
│  └──────────┬──────────────┘                          │
│             │                                          │
│             ▼                                          │
│  ┌─────────────────────────┐                          │
│  │ Test Data (CSV)         │                          │
│  │ leads.csv (100 records) │                          │
│  └─────────────────────────┘                          │
└────────────────────────────────────────────────────────┘
```

### Network Topology

```
Presenter Laptop (Offline)
├─ iron_cage runtime: localhost:3001 (API server)
├─ Control Panel: localhost:3000 (React dev server)
├─ Python agent: subprocess (no network)
└─ Test data: local CSV file

NO EXTERNAL DEPENDENCIES:
✅ No OpenAI API calls (mocked responses)
✅ No LinkedIn API calls (synthetic data)
✅ No Clearbit API calls (synthetic data)
✅ Demo works 100% offline
```

---

## Demo Flow - Detailed Timeline

### Pre-Demo Setup (Done Before Talk Starts)

**30 minutes before presentation:**

1. **System Check**
   ```bash
   # Verify all components installed
   cd /home/user1/pro/lib/willbe
   cargo build --release -p iron_cli -p iron_runtime -p iron_control_api
   cd module/iron_cage/pilot/demo/control panel
   npm run build
   cd ../agent
   uv venv && source .venv/bin/activate
   uv sync --group examples
   ```

2. **Data Preparation**
   ```bash
   # Verify test data exists
   cat agent/test_data/leads.csv | wc -l  # Should be 101 (100 + header)

   # Verify triggers injected
   grep "lead_34" agent/test_data/leads.csv  # LinkedIn failure trigger
   grep "lead_67" agent/test_data/leads.csv  # PII trigger (has email)
   grep "lead_85" agent/test_data/leads.csv  # Budget warning trigger
   ```

3. **Rehearsal Dry Run**
   ```bash
   # Full demo run (5 minutes)
   iron_cage start agent/lead_gen_agent.py --budget 50 --verbose
   # Watch for 3 triggers, verify control panel updates
   # Ctrl+C to stop after completion
   ```

4. **Screen Layout**
   - **Left 50%:** Terminal (80x40, font size 16pt, dark theme)
   - **Right 50%:** Browser (Chrome, localhost:3000/control panel)
   - **Hide:** Desktop icons, notifications, Slack, email

5. **Backup Plan**
   - **Plan A:** Live demo (preferred)
   - **Plan B:** Pre-recorded video (if live fails)
   - **Plan C:** Slide-based walkthrough (if video fails)

---

### Part 1: Agent Startup (30 seconds)

**Slide Transition:** Slide 18 → Screen share → Terminal

**Action: Execute startup command**

```bash
$ iron_cage start lead_gen_agent.py --budget $50 --verbose
```

**Expected Output (3 seconds):**

```
[14:23:31] INFO  Iron Cage Runtime v1.0.0
[14:23:31] INFO  Loading agent: lead_gen_agent.py
[14:23:31] INFO  Budget: $50.00 (hard limit)
[14:23:31] INFO  Safety guardrails: ENABLED
[14:23:31] INFO    ├─ privacy protection: ACTIVE (email, phone, SSN)
[14:23:31] INFO    ├─ Prompt injection: ACTIVE
[14:23:31] INFO    └─ Action policy: redact-and-log
[14:23:31] INFO  Reliability:
[14:23:31] INFO    ├─ safety cutoffs: ENABLED
[14:23:31] INFO    └─ Fallback chains: 3 tiers configured
[14:23:32] INFO  Starting agent...
[14:23:33] OK    Agent running (agent_id: lg-7a3f9c2d)
[14:23:33] INFO  WebSocket server: ws://localhost:3001
[14:23:33] INFO  Control Panel: http://localhost:3000
```

**Presenter Narration (while output appears):**

> "Agent is starting up. Notice the safety guardrails—privacy protection active, circuit breakers enabled. Budget hard limit set to $50. Let's open the control panel..."

**Action: Switch to browser (right half of screen)**

**Expected: Control Panel loads immediately (already open)**

---

### Part 2: Control Panel Overview (1 minute)

**Presenter Points Out 6 Panels:**

#### Panel 1: Agent Status (top-left)

```
┌─────────────────────────────────┐
│ Lead Generation Agent           │
├─────────────────────────────────┤
│ Status: 🟢 Running              │
│ Agent ID: lg-7a3f9c2d           │
│ Uptime: 14 seconds              │
│ Processed: 23 / 100 leads (23%) │
│ Success Rate: 100% (23/23)      │
│ Failed: 0                        │
└─────────────────────────────────┘
```

**Narration:**
> "Agent is running. 23 leads processed so far. 100% success rate. Now watch the cost panel..."

#### Panel 2: Cost Control (top-middle)

```
┌─────────────────────────────────┐
│ 💰 COST CONTROL                 │
├─────────────────────────────────┤
│ Current: $5.29                   │
│ Budget: $50.00                   │
│ Used: 10.6%  [▓▓░░░░░░░░]      │
│                                  │
│ Cost/lead: $0.23 avg             │
│ Projection: $23.00 total ✅     │
│                                  │
│ Breakdown:                       │
│ ├─ OpenAI GPT-4: $4.60 (87%)   │
│ ├─ Clearbit API: $0.46 (9%)    │
│ └─ LinkedIn API: $0.23 (4%)    │
└─────────────────────────────────┘
```

**Narration:**
> "We're at $5.29 so far, tracking to finish around $23 total. Cost per lead is $0.23—well under our $0.50 target. System is projecting we'll stay under budget."

#### Panel 3: Safety (top-right)

```
┌─────────────────────────────────┐
│ 🛡️ SAFETY (Real-time)          │
├─────────────────────────────────┤
│ PII leaks blocked: 0             │
│ Prompt injections: 0             │
│ Unauthorized actions: 0          │
│                                  │
│ Last check: 2 seconds ago        │
│ Status: ✅ COMPLIANT            │
└─────────────────────────────────┘
```

**Narration:**
> "No safety violations yet. All outputs clean. Now watch what happens at lead #34 when LinkedIn fails..."

---

### Part 3: Safety Cutoff Activation (1 minute)

**At Lead #34 (approx 14:23:45):**

**Expected: LinkedIn API returns 429 (rate limit) - INJECTED TRIGGER**

#### Terminal Output:

```
[14:23:45] WARN  Lead #34: LinkedIn API failed (429 Too Many Requests)
[14:23:45] WARN    Request: GET /v2/people/search?company=Acme
[14:23:45] WARN    Response: 429 (rate limit exceeded, retry after 300s)
[14:23:45] ERROR safety cutoff threshold reached (5 consecutive failures)
[14:23:45] CRIT  🔴 safety cutoff OPENED for linkedin_api
[14:23:46] INFO  ⚡ Fallback activated: Tier 2 (cached_data)
[14:23:46] OK    ✅ Lead #34 processed via fallback (degraded: true)
```

#### Control Panel Alert (modal popup):

```
┌─────────────────────────────────────────────────┐
│  ⚡ CIRCUIT BREAKER ALERT                       │
├─────────────────────────────────────────────────┤
│  Service: linkedin_api                          │
│  State: OPEN 🔴                                 │
│  Reason: 5 consecutive failures (rate limit)    │
│  Cooldown: 58 seconds remaining                 │
│                                                  │
│  Fallback: Tier 2 (cached_data) ✅ ACTIVE      │
│                                                  │
│  Impact:                                         │
│  ├─ Requests blocked: 66 (leads 34-100)        │
│  ├─ Success rate: 98% (via fallback)           │
│  └─ Cost saved: $6.60 (avoided Clearbit calls) │
│                                                  │
│  [Acknowledge]  [View Details]  [Force Reset]   │
└─────────────────────────────────────────────────┘
```

#### Updated Performance Panel:

```
┌─────────────────────────────────┐
│ ⚡ PERFORMANCE                  │
├─────────────────────────────────┤
│ Throughput: 212 leads/hour       │
│ Latency P95: 2.1s                │
│ Cache hit rate: 66% (tier 2)     │
│                                  │
│ Safety Cutoffs:                │
│ ├─ linkedin_api: 🔴 OPEN        │
│ ├─ clearbit_api: 🟢 CLOSED     │
│ └─ openai_api: 🟢 CLOSED       │
└─────────────────────────────────┘
```

**Presenter Narration:**

> "There it is. LinkedIn hit rate limit. safety cutoff opened immediately. Agent didn't freeze. Didn't timeout. It switched to cached data automatically. System kept running."

> "And notice the cost panel—we saved $6.60 by using cached data instead of hitting Clearbit's paid API. safety cutoffs aren't just for reliability. They save money too."

**Key Teaching Moment:**
> "This is the difference between prototype and production. In Python alone, your agent would be stuck retrying LinkedIn for 5 minutes. With Iron Cage, it failed fast and used a fallback. Zero downtime."

---

### Part 3.5: Secret Rotation (1 minute) - NEW TRIGGER #4

**At Lead #50 (approx 14:23:58):**

**Expected: Live secret rotation demonstration - INJECTED TRIGGER**

#### Presenter Action:

**Presenter switches to control panel and navigates to Secrets panel (7th panel)**

```
[Presenter clicks "Secrets Management" tab in control panel]
```

**Expected: Secrets panel displays current secrets**

```
┌────────────────────────────────────────────────────┐
│ Secrets Management                       [+ Add]   │
├────────────────────────────────────────────────────┤
│                                                    │
│ Name                Environment    Value          │
│ ───────────────────────────────────────────────── │
│ OPENAI_API_KEY      Production     sk-proj-ab...z │ [👁️] [✏️] [🗑️]
│ CLEARBIT_KEY        Production     pk_live_45...67│ [👁️] [✏️] [🗑️]
│                                                    │
└────────────────────────────────────────────────────┘
```

**Presenter Narration:**

> "While the agent is running—processing lead #50 right now—I'm going to rotate the OpenAI API key. Watch what happens..."

**Action: Click Edit button on OPENAI_API_KEY**

**Expected: Edit modal appears**

```
┌──────────────────────────────────────────────────┐
│  Edit Secret                                     │
├──────────────────────────────────────────────────┤
│  Name: OPENAI_API_KEY                            │
│  Environment: Production                         │
│                                                   │
│  New Value:                                       │
│  ┌──────────────────────────────────────────┐   │
│  │ sk-proj-NEW_KEY_1234567890abcdef        │   │
│  └──────────────────────────────────────────┘   │
│                                                   │
│  ⚠️  Agent will reload this secret immediately   │
│      without restart (SIGUSR1 signal)            │
│                                                   │
│  [Cancel]                           [Update] ✅   │
└──────────────────────────────────────────────────┘
```

**Action: Enter new API key value and click Update**

#### Terminal Output (during rotation):

```
[14:23:59] INFO  🔄 Secret rotation requested: OPENAI_API_KEY
[14:23:59] INFO    ├─ Encrypted new value with AES-256-GCM
[14:23:59] INFO    ├─ Stored in secrets table (id: secret-abc123)
[14:23:59] INFO    └─ Audit log entry created
[14:23:59] INFO  Sending SIGUSR1 to agent process (PID: 42891)
[14:23:59] INFO  Agent signal handler: Reloading secrets...
[14:23:59] INFO  ✅ Secret reloaded: OPENAI_API_KEY
[14:24:00] INFO    └─ Updated os.environ["OPENAI_API_KEY"]
[14:24:00] OK    Lead #51: Processing (using new API key) ✅
```

#### Control Panel Notification (toast):

```
┌─────────────────────────────────────────┐
│ ✅ Secret rotated successfully          │
│ OPENAI_API_KEY updated                  │
│ Agent reloaded without restart          │
│ Lead #51 using new key                  │
└─────────────────────────────────────────┘
```

#### Updated Credentials Panel:

```
┌────────────────────────────────────────────────────┐
│ Secrets Management                       [+ Add]   │
├────────────────────────────────────────────────────┤
│                                                    │
│ Name                Environment    Value          │
│ ───────────────────────────────────────────────── │
│ OPENAI_API_KEY      Production     sk-proj-NE...ef│ [👁️] [✏️] [🗑️]  ⬅️ Updated
│ CLEARBIT_KEY        Production     pk_live_45...67│ [👁️] [✏️] [🗑️]
│                                                    │
│ Last updated: 2 seconds ago                        │
└────────────────────────────────────────────────────┘
```

**Presenter Narration:**

> "Secret rotated. Agent got the signal—SIGUSR1. Reloaded the secret from encrypted storage. Lead #51 is processing right now using the new API key. Agent never stopped. Zero downtime."

> "Switch back to terminal—see lead #51? That's using the new key. The agent has no idea it was rotated. Just works."

**Key Teaching Moment:**

> "This is enterprise secrets management. You rotate credentials—maybe because of a security audit, maybe because someone left the company—and your agents keep running. No restarts. No deployments. Just signal the process and it reloads."

> "In production, you'd automate this. Set a policy: rotate every 90 days. System does it automatically while agents are running. That's how you pass SOC 2 audit requirement 3.6: 'Demonstrate credential rotation capability.'"

---

### Part 4: Privacy Protection (1 minute)

**At Lead #67 (approx 14:24:12):**

**Expected: Agent output contains email address - INJECTED TRIGGER**

#### Terminal Output:

```
[14:24:12] WARN  Lead #67: Processing Acme Corporation...
[14:24:12] WARN  LLM output: "Contact CEO at ceo@acme-corp.com for partnership"
[14:24:12] CRIT  🔴 CRITICAL: PII DETECTED IN OUTPUT
[14:24:12] CRIT    Type: EMAIL (high risk)
[14:24:12] CRIT    Pattern: ceo@acme-corp.com
[14:24:12] CRIT    Location: char 15-35
[14:24:12] INFO  Action: OUTPUT_REDACTED
[14:24:12] INFO  Redacted: "Contact CEO at [EMAIL_REDACTED] for partnership"
[14:24:12] INFO  Compliance: GDPR violation prevented ✅
[14:24:12] INFO  Audit log: Saved to sqlite (encrypted original)
[14:24:12] INFO  Webhook: Sent to compliance@company.com
```

#### Control Panel Alert (modal popup):

```
┌──────────────────────────────────────────────────┐
│  🔒 PII LEAK PREVENTED                           │
├──────────────────────────────────────────────────┤
│  Lead: #67 (Acme Corporation)                    │
│  Time: 2025-11-17 14:24:12 UTC                   │
│  Severity: HIGH                                   │
│                                                   │
│  PII Detected:                                    │
│  ├─ Type: EMAIL                                  │
│  ├─ Value: [REDACTED] (stored encrypted)        │
│  ├─ Risk: HIGH (GDPR Article 32 violation)      │
│  └─ Context: LLM output (char 15-35)            │
│                                                   │
│  Original Output (sanitized):                    │
│  "Contact CEO at [EMAIL_REDACTED] for..."       │
│                                                   │
│  Actions Taken:                                   │
│  ✅ Output redacted automatically                 │
│  ✅ Incident logged to audit system              │
│  ✅ Compliance team notified (webhook)           │
│  ✅ Agent continues (warn mode)                  │
│                                                   │
│  [Download Audit Report]  [View Policy]  [OK]    │
└──────────────────────────────────────────────────┘
```

#### Updated Protection Panel:

```
┌─────────────────────────────────┐
│ 🛡️ SAFETY (Real-time)          │
├─────────────────────────────────┤
│ PII leaks blocked: 1 🔴          │
│ ├─ Type: EMAIL (high risk)      │
│ ├─ Lead: #67                    │
│ └─ Action: REDACTED             │
│                                  │
│ Prompt injections: 0             │
│ Unauthorized actions: 0          │
│                                  │
│ Compliance: ✅ COMPLIANT        │
│ Audit trail: 1 event logged      │
└─────────────────────────────────┘
```

**Presenter Narration:**

> "PII detected. Email address in agent output. Immediately redacted. Original value encrypted and stored for audit. Compliance team got a Slack notification."

> "Agent kept running because we're in 'redact' mode, not 'block' mode. In block mode, this would have stopped the agent immediately."

**Key Teaching Moment:**
> "In a SOC 2 audit, auditor asks: 'Show me your PII protection.' We pull up this log. Every instance detected, redacted, and audited. That's compliance. Without this, you're logging customer emails in plaintext—instant GDPR violation."

---

### Part 5: Budget Warning (1 minute)

**At Lead #85 (approx 14:24:38):**

**Expected: Budget hits 90% threshold - INJECTED TRIGGER**

#### Terminal Output:

```
[14:24:38] WARN  ⚠️ BUDGET WARNING: 90% threshold reached
[14:24:38] WARN    Current: $45.12 / $50.00 budget
[14:24:38] WARN    Remaining: $4.88 (estimated 21 more leads)
[14:24:38] WARN    Projection: Will hit limit at lead #106
[14:24:38] WARN    Action: ALERT sent to ops@company.com
[14:24:38] INFO  Agent will auto-stop at 100% budget (hard limit)
```

#### Control Panel Alert (modal popup):

```
┌──────────────────────────────────────────────────┐
│  💰 BUDGET WARNING                               │
├──────────────────────────────────────────────────┤
│  Current: $45.12 / $50.00 (90.2%)                │
│  Remaining: $4.88                                 │
│                                                   │
│  Projection:                                      │
│  ├─ At current rate: $0.53/lead (avg)           │
│  ├─ Budget exhausted at: Lead #106 (est.)       │
│  ├─ Time remaining: ~3 minutes                   │
│  └─ Action: Auto-stop at 100% (hard limit)      │
│                                                   │
│  Options:                                         │
│  ┌────────────┬──────────┬────────────────────┐ │
│  │ Increase   │ Pause    │ Continue           │ │
│  │ Budget     │ Agent    │ (auto-stop)        │ │
│  │ [+$25]     │ [Pause]  │ [Continue] ✅      │ │
│  └────────────┴──────────┴────────────────────┘ │
│                                                   │
│  [Send Alert to CFO]  [View Breakdown]  [OK]     │
└──────────────────────────────────────────────────┘
```

#### Updated Budget Panel:

```
┌─────────────────────────────────┐
│ 💰 COST CONTROL                 │
├─────────────────────────────────┤
│ Current: $45.12  ⚠️              │
│ Budget: $50.00                   │
│ Used: 90.2%  [▓▓▓▓▓▓▓▓▓░] 🔴   │
│                                  │
│ Cost/lead: $0.53 avg (↑ 130%)   │
│ Projection: $50.00 at lead #95  │
│ Warning: Budget will be exceeded │
│                                  │
│ Alert sent: ops@company.com ✅   │
│ Auto-stop: ENABLED at 100%       │
└─────────────────────────────────┘
```

**Presenter Narration:**

> "Budget warning at 90%. System projected we'll hit the limit around lead #106. Alert sent via email. Now someone can approve a budget increase, pause the agent, or let it auto-stop at 100%."

> "This is the control that prevents $47K surprises. Remember that war story from Act 1? CFO sets budget. System enforces it automatically. No more Monday morning panic."

**Key Teaching Moment:**
> "Notice the cost per lead jumped from $0.23 to $0.53? That's because LinkedIn circuit breaker forced us to use more expensive Clearbit API calls. System adapts in real-time. Warns you before you blow past budget."

---

### Part 6: Completion & Final Metrics (30 seconds)

**At Lead #100 (approx 14:25:06):**

#### Terminal Output:

```
[14:25:06] INFO  Lead #100: Processing final lead...
[14:25:07] OK    ✅ Lead #100 completed successfully
[14:25:07] INFO  Agent run complete
[14:25:07] INFO  Generating summary report...

════════════════════════════════════════════════
  LEAD GENERATION AGENT - RUN COMPLETE ✅
════════════════════════════════════════════════

Duration: 28 minutes 14 seconds
Total Leads: 100
Success Rate: 98% (98 successful, 2 failed)

💰 COST SUMMARY:
├─ Total spent: $23.14
├─ Budget: $50.00 (46.3% used)
├─ Avg cost/lead: $0.23
├─ vs Baseline: $64.86 saved (73.7% reduction)
└─ Breakdown:
   ├─ OpenAI GPT-4: $20.12 (87%)
   ├─ Clearbit API: $2.31 (10%)
   └─ LinkedIn API: $0.71 (3%)

🛡️ SAFETY SUMMARY:
├─ PII leaks blocked: 3 (all redacted)
│  └─ Types: 2 emails, 1 phone number
├─ safety cutoff trips: 1 (LinkedIn API)
├─ Fallback activations: 66 (cached data)
└─ Compliance: ✅ PASS (0 violations)

⚡ PERFORMANCE SUMMARY:
├─ Throughput: 212 leads/hour
├─ Latency: P50=1.8s, P95=3.2s, P99=5.1s
├─ Cache hit rate: 34% (saved $8.47)
└─ Uptime: 100% (0 crashes, 0 restarts)

Audit log: /var/log/iron_cage/lg-7a3f9c2d.jsonl
Control Panel: http://localhost:3000/reports/lg-7a3f9c2d

[14:25:08] INFO  Agent stopped gracefully
```

#### Control Panel Final State (all 6 panels):

```
┌──────────────────────────────────────────────────────────┐
│  Lead Generation Agent - COMPLETED ✅                    │
├──────────────────────────────────────────────────────────┤
│                                                           │
│  ┌────────────┬────────────┬────────────┐              │
│  │ STATUS     │ COST       │ SAFETY     │              │
│  ├────────────┼────────────┼────────────┤              │
│  │ 🟢 Done    │ $23.14     │ 3 blocked  │              │
│  │ 98/100     │ 46% budget │ ✅ Pass    │              │
│  └────────────┴────────────┴────────────┘              │
│                                                           │
│  ┌────────────────────────────────────────────────────┐ │
│  │ COST BREAKDOWN                                     │ │
│  ├────────────────────────────────────────────────────┤ │
│  │ [████████████████████░░░░░░░░░] OpenAI: $20.12   │ │
│  │ [██░░░░░░░░░░░░░░░░░░░░░░░░░░] Clearbit: $2.31  │ │
│  │ [█░░░░░░░░░░░░░░░░░░░░░░░░░░░] LinkedIn: $0.71  │ │
│  │                                                     │ │
│  │ Baseline (no Iron Cage): $87.00                    │ │
│  │ With Iron Cage: $23.14                             │ │
│  │ Savings: $64.86 (73.7%)                            │ │
│  └────────────────────────────────────────────────────┘ │
│                                                           │
│  ┌────────────────────────────────────────────────────┐ │
│  │ EVENT LOG (last 10 events)                         │ │
│  ├────────────────────────────────────────────────────┤ │
│  │ 14:25:07  ✅  Agent completed (98/100 success)    │ │
│  │ 14:24:38  ⚠️  Budget warning (90% threshold)      │ │
│  │ 14:24:12  🔒  PII blocked: EMAIL (#67)            │ │
│  │ 14:23:45  🔴  safety cutoff: linkedin_api       │ │
│  │ 14:23:33  🟢  Agent started (lg-7a3f9c2d)         │ │
│  └────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

**Presenter Narration:**

> "Done. 100 leads processed in 28 minutes. Total cost: $23.14. That's 74% cheaper than baseline without Iron Cage."

> "Three PII leaks caught and redacted. One circuit breaker activation saved us from a LinkedIn outage. Zero crashes. Zero downtime. 100% compliance."

> "This is what production AI looks like. Controlled. Observable. Reliable."

**Final Teaching Moment:**
> "Every metric you see here—cost, safety, reliability—is impossible with Python agents alone. You need infrastructure. And that's what Iron Cage provides."

---

### Demo Wrap-Up (10 seconds)

**Action: Return to slides**

**Presenter:**
> "Let's go back to slides for the business case..." → **Transition to Slide 19**

---

## Test Data Specification

### File: `agent/test_data/leads.csv`

**Format:** CSV with 101 rows (1 header + 100 data rows)

**Schema:**

```csv
lead_id,company_name,domain,industry,employee_count,linkedin_url,contact_email,trigger_type,expected_behavior
1,Acme Corp,acme.com,Software,250,https://linkedin.com/company/acme,NULL,normal,success
2,GlobalTech,globaltech.io,AI/ML,1200,https://linkedin.com/company/globaltech,NULL,normal,success
...
34,FailCorp,failcorp.com,FinTech,450,https://linkedin.com/company/failcorp,NULL,circuit_breaker,"linkedin_api returns 429, fallback to cached"
...
67,PII Leaker Inc,piileaker.com,Healthcare,890,https://linkedin.com/company/piileaker,ceo@piileaker.com,pii_detection,"LLM output contains email, redact and log"
...
85,Budget Trigger Co,budgettrigger.com,Marketing,320,https://linkedin.com/company/budgettrigger,NULL,budget_warning,"Cost reaches 90%, send alert"
...
100,Final Lead LLC,finallead.com,Consulting,75,https://linkedin.com/company/finallead,NULL,normal,success
```

**Key Records:**

#### Lead #34: Safety Cutoff Trigger

```csv
34,FailCorp,failcorp.com,FinTech,450,https://linkedin.com/company/failcorp,NULL,circuit_breaker,linkedin_429
```

**Expected Behavior:**
- LinkedIn API mock returns: `HTTP 429 Too Many Requests`
- safety cutoff opens after 5 consecutive failures (leads 30-34)
- Fallback activates: Use cached company data instead of live API
- Cost saved: $0.10 per lead × 66 remaining leads = $6.60

#### Lead #67: Privacy Protection Trigger

```csv
67,PII Leaker Inc,piileaker.com,Healthcare,890,https://linkedin.com/company/piileaker,ceo@piileaker.com,pii_detection,email_in_output
```

**Expected Behavior:**
- Agent enrichment step includes `contact_email` in LLM output
- LLM generates: `"Contact CEO at ceo@piileaker.com for partnership opportunities"`
- PII detector regex matches email pattern
- Output redacted to: `"Contact CEO at [EMAIL_REDACTED] for partnership opportunities"`
- Incident logged with:
  - Type: EMAIL
  - Risk: HIGH
  - Original value: encrypted in SQLite
  - Webhook: POST to compliance@company.com

#### Lead #85: Budget Warning Trigger

```csv
85,Budget Trigger Co,budgettrigger.com,Marketing,320,https://linkedin.com/company/budgettrigger,NULL,budget_warning,90_percent
```

**Expected Behavior:**
- Cumulative cost at lead #85: $45.12
- Budget: $50.00
- Usage: 90.24% (exceeds 90% threshold)
- Alert triggered: Email to ops@company.com
- Control Panel modal: "Budget Warning" displayed
- Projection: Budget will be exhausted at lead #106 (if continuing)
- Agent continues running (hard stop at 100%, not 90%)

---

## Technical Requirements

### Hardware Requirements

**Minimum:**
- CPU: 4 cores (Intel i5 or equivalent)
- RAM: 8 GB
- Disk: 2 GB free (for logs and SQLite)
- Display: 1920×1080 (for side-by-side layout)

**Recommended:**
- CPU: 8 cores (Intel i7 or equivalent)
- RAM: 16 GB
- Disk: 10 GB SSD (for faster SQLite writes)
- Display: 2560×1440 or dual monitors

### Software Requirements

```toml
[rust]
version = "1.75+"
components = ["cargo", "rustc", "clippy"]

[python]
version = "3.11+"
packages = ["langchain==0.1.0", "openai==1.0.0", "pandas==2.1.0"]

[nodejs]
version = "20+"
packages = ["react==18.2.0", "typescript==5.0.0", "vite==5.0.0"]

[system]
os = "Linux or macOS"
shell = "bash or zsh"
browser = "Chrome 120+ or Firefox 120+"
```

### Network Requirements

**CRITICAL: Demo must work 100% offline**

```yaml
external_apis:
  openai:
    mode: MOCKED
    responses: Pre-generated in test_data/mock_responses.json
  linkedin:
    mode: MOCKED
    failure_trigger: lead_id == 34
  clearbit:
    mode: MOCKED
    responses: Synthetic enrichment data

local_services:
  iron_cage_runtime: localhost:3001 (API server)
  react_control panel: localhost:3000 (dev server)
  websocket: ws://localhost:3001/ws
```

**Fallback Strategy:**
- **If conference WiFi is required:** Pre-download all dependencies (no live `cargo build` or `npm install`)
- **If screen share fails:** Have backup recording ready (720p, 5min, <50MB)

---

## Success Metrics

### Functional Metrics (Must Pass)

```yaml
agent_completion:
  - total_leads: 100
  - success_rate: >= 98%
  - failed_leads: <= 2
  - duration: 25-30 minutes

cost_control:
  - total_cost: $20-25
  - budget_used: 40-50%
  - cost_per_lead: $0.20-$0.30
  - baseline_savings: >= 70%

safety:
  - pii_detected: >= 3
  - pii_blocked: 100%
  - compliance_violations: 0
  - audit_log_entries: >= 3

reliability:
  - circuit_breaker_trips: 1
  - fallback_activations: 66
  - crashes: 0
  - restarts: 0
  - uptime: 100%
```

### Performance Metrics (Nice to Have)

```yaml
latency:
  - p50: <= 2.0s
  - p95: <= 3.5s
  - p99: <= 6.0s

throughput:
  - leads_per_hour: >= 200
  - requests_per_second: >= 5

control panel:
  - websocket_latency: <= 100ms
  - ui_update_lag: <= 50ms
  - chart_render_time: <= 200ms
```

---

## Troubleshooting Guide

### Problem: safety cutoff doesn't fire at lead #34

**Symptoms:**
- Lead #34 processes successfully
- No "safety cutoff OPENED" message
- Control Panel shows 0 circuit breaker trips

**Diagnosis:**
```bash
# Check test data trigger
grep "lead_34" agent/test_data/leads.csv
# Should show: 34,...,circuit_breaker,...

# Check mock API configuration
grep "linkedin_failure_trigger" agent/mock_api.py
# Should show: if lead_id == 34: return 429
```

**Fix:**
1. Verify `trigger_type` column in CSV has `circuit_breaker` for lead 34
2. Ensure mock API code checks `lead_id == 34` condition
3. Re-run: `iron_cage start lead_gen_agent.py --budget 50`

---

### Problem: privacy protection doesn't trigger at lead #67

**Symptoms:**
- Lead #67 processes without PII alert
- Safety panel shows "0 PII leaks blocked"
- No redaction in logs

**Diagnosis:**
```bash
# Check test data has email
grep "lead_67" agent/test_data/leads.csv | grep "@"
# Should show: 67,...,ceo@piileaker.com,...

# Check PII detector regex
iron_cage test pii "Contact CEO at ceo@example.com"
# Should output: DETECTED: EMAIL at position 15-31
```

**Fix:**
1. Verify lead #67 has non-NULL `contact_email` column
2. Ensure agent code includes email in LLM output for lead #67
3. Test PII regex: `echo "ceo@test.com" | iron_cage detect-pii`
4. Re-run demo

---

### Problem: Budget warning doesn't fire at lead #85

**Symptoms:**
- Lead #85 processes without alert
- Cost panel doesn't show warning
- No budget warning modal

**Diagnosis:**
```bash
# Check cumulative cost at lead 85
iron_cage replay --stop-at 85 | grep "Current:"
# Should show: Current: $45-46

# Check budget threshold config
iron_cage config show | grep warning_threshold
# Should show: warning_threshold: 0.90 (90%)
```

**Fix:**
1. Verify mock API costs sum to ~$45 by lead #85
2. Adjust per-lead costs in mock_api.py:
   - GPT-4: $0.45/lead
   - Clearbit: $0.05/lead
   - LinkedIn: $0.03/lead (when working)
3. Re-run demo

---

### Problem: Control Panel doesn't update in real-time

**Symptoms:**
- Terminal shows logs, but control panel is frozen
- Metrics don't change
- WebSocket disconnected

**Diagnosis:**
```bash
# Check WebSocket connection
wscat -c ws://localhost:3001/ws
# Should connect without errors

# Check API server logs
tail -f /var/log/iron_cage/api_server.log
# Look for WebSocket connection messages
```

**Fix:**
1. Restart API server: `killall iron_control_api && iron_cage start --api-only`
2. Reload control panel: Ctrl+R in browser
3. Check firewall: `sudo ufw allow 3000 && sudo ufw allow 3001`
4. Use fallback: Pre-recorded video

---

### Problem: Demo completes too fast or too slow

**Symptoms:**
- Target: 5 minutes
- Actual: 2 minutes (too fast) or 10 minutes (too slow)

**Fix (Too Fast):**
```python
# In lead_gen_agent.py, add artificial delay
import time
time.sleep(2)  # 2 seconds per lead = ~3.5 min total
```

**Fix (Too Slow):**
```python
# In lead_gen_agent.py, reduce processing time
# Option 1: Skip non-critical steps
# Option 2: Use smaller LLM (gpt-3.5-turbo instead of gpt-4)
# Option 3: Reduce leads: 50 instead of 100
```

---

## Rehearsal Checklist

### 1 Week Before Conference

- [ ] Run full demo 3 times, verify all 3 triggers fire
- [ ] Record backup video (720p, H.264, <50MB)
- [ ] Test on conference laptop (not development machine)
- [ ] Verify offline mode works (disconnect WiFi, run demo)
- [ ] Print troubleshooting guide (physical backup)

### 1 Day Before Conference

- [ ] Dry run on conference projector/screen
- [ ] Test screen share via Zoom/Teams (if hybrid conference)
- [ ] Verify font sizes readable from back of room (16pt+ terminal)
- [ ] Charge laptop to 100% (bring charger anyway)
- [ ] Download all dependencies (no live `cargo build`)

### 1 Hour Before Presentation

- [ ] Close all unnecessary apps (Slack, email, notifications)
- [ ] Set display to "Do Not Disturb" mode
- [ ] Open terminal (left half) and browser (right half)
- [ ] Navigate to demo directory: `cd agent/`
- [ ] Activate Python venv: `source .venv/bin/activate`
- [ ] Test WebSocket: `wscat -c ws://localhost:3001/ws` (then close)
- [ ] Clear terminal history: `clear && history -c`

### 5 Minutes Before Demo Slide

- [ ] Verify iron_cage runtime is NOT already running
- [ ] Control Panel loaded in browser (localhost:3000)
- [ ] Terminal prompt ready: cursor at `$ iron_cage start ...`
- [ ] Backup video file open (in case live demo fails)
- [ ] Water bottle nearby (5-minute narration is long)

---

## Contingency Plans

### Plan A: Live Demo (Preferred)

**Prerequisites:**
- ✅ All systems operational
- ✅ Terminal + Control Panel visible
- ✅ Audience can see screen clearly

**Go/No-Go Decision:** 30 seconds before demo
- If everything works in pre-demo test → **GO**
- If any component fails → **NO-GO, switch to Plan B**

---

### Plan B: Pre-Recorded Video (Backup)

**Trigger Conditions:**
- Live demo crashes during rehearsal
- WebSocket connection unstable
- Laptop performance issues
- Screen share fails

**Execution:**
1. Say: *"Let me show you a recorded demo..."*
2. Play video file: `demo_recording.mp4` (5 min, 720p)
3. Narrate over video (same script as live)
4. Advantage: Perfect timing, no technical risk

**Video Requirements:**
- Resolution: 1280×720 (readable on projector)
- Codec: H.264 (universal compatibility)
- Audio: No audio track (presenter narrates live)
- Duration: Exactly 5 minutes
- File size: <50 MB (fast to copy to backup laptop)

---

### Plan C: Slide-Based Walkthrough (Emergency)

**Trigger Conditions:**
- Both Plan A and Plan B fail
- Video file corrupted
- Complete technical meltdown

**Execution:**
1. Skip to Slide 19 (Demo Recap)
2. Say: *"Technical difficulties. Let me walk you through what you would have seen..."*
3. Use screenshots embedded in slides
4. Narrate the 3 trigger events (circuit breaker, PII, budget)
5. Show final metrics slide
6. Advantage: Always works, no dependencies

**Slide Requirements:**
- Include 6 high-res screenshots of control panel
- Add annotations showing key events
- Embed terminal output as code blocks
- Highlight 3 triggers with red circles

---

## Post-Demo Analysis

### Metrics to Capture

**During Demo:**
- Actual duration: ______ minutes (target: 5 min)
- Audience engagement: ______ / 10 (based on body language)
- Trigger #1 (circuit breaker): ✅ / ❌
- Trigger #2 (PII): ✅ / ❌
- Trigger #3 (budget): ✅ / ❌
- Technical issues: ______ (describe any failures)

**After Demo:**
- Questions asked: ______ (count, record topics)
- Leads generated: ______ (signup form submissions)
- Demo requests: ______ (attendees asking for private demo)

### Lessons Learned Template

```markdown
## Warsaw Conference Demo - Retrospective

**Date:** 2025-12-16
**Audience Size:** _____
**Plan Used:** A / B / C (circle one)

### What Went Well
1.
2.
3.

### What Went Wrong
1.
2.
3.

### Improvements for Next Demo
1.
2.
3.

### Questions We Couldn't Answer
1.
2.
3.

### Follow-Up Actions
- [ ] Update demo script based on feedback
- [ ] Fix identified bugs
- [ ] Improve backup video quality
- [ ] Add missing features to roadmap
```

---

## References

### Primary Documents

- **Talk Script:** `/conferences/warsaw_2025/presentation/talk_outline.md` (lines 754-962)
- **Pilot Spec:** `/pilot/spec.md` (Features #16-18, demo agent specification)
- **Demo Components:** `/pilot/demo/readme.md` (agent + control panel structure)

### Implementation Dependencies

- **iron_runtime:** `/module/iron_runtime/` (PyO3 bridge, agent lifecycle)
- **iron_safety:** `/module/iron_safety/` (privacy protection, redaction)
- **iron_budget:** `/module/iron_budget/` (budget tracking, alerts)
- **iron_reliability:** `/module/iron_reliability/` (circuit breakers, fallbacks)
- **iron_control_api:** `/module/iron_control_api/` (WebSocket server, REST API)
- **iron_control:** `/pilot/demo/control panel/` (React + TypeScript UI)

### Test Data

- **Leads CSV:** `/pilot/demo/agent/test_data/leads.csv` (100 synthetic records)
- **Mock Responses:** `/pilot/demo/agent/test_data/mock_responses.json` (OpenAI completions)
- **Trigger Configuration:** `/pilot/demo/agent/config/triggers.yaml` (lead #34, #67, #85)

---

## Appendix: Terminal Color Scheme

**For maximum readability on projector:**

```bash
# ~/.bashrc or ~/.zshrc
export PS1="\[\033[1;36m\]\u@\h\[\033[0m\]:\[\033[1;34m\]\w\[\033[0m\]$ "

# Log level colors (configure in iron_telemetry)
INFO:  \033[0;32m (green)
WARN:  \033[0;33m (yellow)
ERROR: \033[0;31m (red)
CRIT:  \033[1;31m (bold red)
OK:    \033[1;32m (bold green)
```

**Font Settings:**
- Font: Menlo, Monaco, or Source Code Pro
- Size: 16pt (minimum for projector visibility)
- Theme: Dark background (Solarized Dark or Dracula)

---

**Document Version:** 1.0.0
**Last Updated:** 2025-11-25
**Next Review:** After Warsaw conference (Dec 17, 2025)
**Status:** ✅ Specification Complete, Implementation Pending
