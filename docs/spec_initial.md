# Project Specification: Iron Cage (MVP)

## 1. Overall Architecture

The system is built on the **Sidecar / Control Plane** principle. We are not a proxy server for LLM traffic (to avoid slowing down operations), but we serve as the central authority for authorization and environment configuration.

### Key Components:

1. **Python SDK (`The Client`)**: Installed by the developer. Responsible for injecting keys into memory, local token counting, and PII blocking.

2. **Rust Backend with Embedded UI (`The Core`)**: Single deployment that includes:
    - REST/gRPC API for SDK communication
    - Built-in Admin Dashboard (served from `/dashboard` route)
    - All business logic: license validation, encrypted secrets, billing, analytics
    - Web interface for managers/team leads: project creation, card linking, statistics viewing

---

## 2. Component Details

### A. Client-Side (Python SDK)

A wrapper library that integrates with LangChain/LlamaIndex. Runs in the user's execution environment.

**Modules:**

- **Bootstrap / Secrets Injector:**
    - Connects to the server at startup using `Iron Cage Key`
    - Receives encrypted payload (OpenAI/Anthropic Keys)
    - Decrypts and loads keys into process environment variables (`os.environ`). **Never saves them to disk.**

- **Local Accountant:**
    - Uses tokenizers (e.g., `tiktoken`) to estimate request cost before sending
    - Manages local cache of "leased budget"
    - **POST Request Price Calculation:**
      - Cost formula: `(input_tokens / 1,000,000) × input_rate + (output_tokens / 1,000,000) × output_rate`
      - Pricing Table (USD per 1M tokens, as of Dec 2025):
        | Provider | Model | Input | Output |
        |----------|-------|-------|--------|
        | OpenAI | gpt-4-turbo | $10.00 | $30.00 |
        | OpenAI | gpt-3.5-turbo | $0.50 | $1.50 |
        | Anthropic | claude-3-5-sonnet | $3.00 | $15.00 |
        | Anthropic | claude-3-opus | $15.00 | $75.00 |
        | Anthropic | claude-3-haiku | $0.25 | $1.25 |
        | Google | gemini-1.5-pro | $1.25 | $5.00 |
        | Google | gemini-1.5-flash | $0.075 | $0.30 |
      - Returns cost in cents (USD) for budget tracking

- **Lease Manager:**
    - Background process. Communicates with the server
    - Requests micro-tranches of budget (e.g., $0.50 at a time)
    - Sends reports on actual usage (Telemetry)

- **Safety Filter:**
    - Regex engine for detecting PII (email, credit cards) in prompts before sending

---

### B. Server-Side (Rust Backend with Embedded UI)

Single binary deployment: high-load API server (REST/gRPC) + embedded web dashboard.

**API Modules:**

- **Auth Gateway:**
    - Validates `Iron Cage Keys`
    - IP address verification (Whitelisting)
    - Rate Limiting (protection against DDoS and request spam)

- **Secrets Vault:**
    - Stores provider keys (OpenAI, AWS) in encrypted form (AES-256-GCM)
    - Issues keys to client only after successful policy verification

- **Billing Engine:**
    - Implements "Pessimistic Leasing" logic (advance deduction)
    - Manages project and user balances
    - Handles session expiration (TTL)

- **Analytics Ingest:**
    - Receives asynchronous usage logs (token count, model, time)
    - Aggregates data for the dashboard

**Embedded Dashboard Modules (served at `/dashboard`):**

- **Project Management:** Create/edit projects, generate Iron Cage Keys
- **Budget Controls:** Set limits per project/user, pause/resume access
- **Usage Analytics:** Real-time graphs, cost breakdown by model/user
- **Team Management:** Invite members, assign roles (Admin/Developer)

**Tech Stack:**
- Framework: Axum or Actix-web
- Frontend: Embedded SPA (e.g., Leptos, Yew, or pre-built React bundle)
- Auth: JWT for dashboard, API Keys for SDK

---

### C. Database and Infrastructure

**1. PostgreSQL (Primary Storage / Cold Storage):**

| Table | Purpose |
|-------|---------|
| Users/Orgs | Accounts, roles |
| Projects | Budget settings, Iron Cage keys linked to project |
| Vault | Encrypted third-party API keys (OpenAI Key) |
| Ledger | Transaction history (deposits, withdrawals) |

**2. Redis (Operational Storage / Hot Storage):**

| Key Space | Purpose |
|-----------|---------|
| Active Leases | Current reserved amounts for active sessions |
| Rate Limits | Request counters for API protection |
| Key Cache | Caching key validity to minimize Postgres queries |

---

## 3. Logical Flows (Workflows)

### Scenario 1: "Injection" (Obtaining Keys)

*When a developer runs a script.*

1. **SDK → Server:** "Hello, I am key `ic_dev_123`. Give me environment configuration."
2. **Server:** Verifies key, IP, project activity.
3. **Server → DB:** Retrieves encrypted `OPENAI_API_KEY`.
4. **Server → SDK:** Returns key through secure channel.
5. **SDK:** Injects key into RAM. Now the agent can operate.

### Scenario 2: "Leasing" (Operation and Billing)

*During agent operation.*

1. **SDK:** "I need to start generation. Reserve $0.50 for me."
2. **Server:** Deducts $0.50 from project balance → status `PENDING`.
3. **SDK:** Works locally, subtracting token costs from this $0.50.
4. **SDK (after 1 min):** "I spent $0.20, returning the rest and give me a new tranche."
5. **Server:** Records $0.20 expense, returns $0.30 to balance, issues new $0.50.

### Scenario 3: "Kill Switch" (Overspending)

*When funds are exhausted.*

1. **SDK:** Requests new tranche.
2. **Server:** "Denied. Project budget exhausted / Project on pause."
3. **SDK:** Throws `BudgetExceededError` exception, stopping code execution before making a request to OpenAI.

---

## 4. Security Model

1. **Ephemerality:** Real OpenAI keys exist on the developer's machine only in RAM during process execution.

2. **Scope Keys (Limited Keys):** The `Iron Cage` key (which the developer has) does not grant access to admin panel or billing. It only allows "borrowing" budget and receiving configuration.

3. **Anomalies:** The server automatically blocks an Iron Cage Key if:
    - Requests from one key come from different IPs simultaneously
    - Expenses exceed $X per minute (protection against infinite loops)

---

## 5. MVP Roadmap

### Stage 1: Core API

- Rust server with basic authorization and Leasing logic
- Python SDK that can count tokens locally and request budget
- Command-line interface (CLI) for key generation

### Stage 2: Vault + Security

- Adding key encryption in DB (AES-256-GCM)
- Implementation of `inject_secrets()` logic in SDK
- IP whitelisting and anomaly detection

### Stage 3: Embedded Dashboard

- Web UI built into the same Rust binary
- Project creation and expense viewing at `/dashboard`
- Usage graph visualization
- Team invite and role management

---

This structure enables the creation of a scalable system that can easily be sold as a B2B solution for AI development management.

---

## 6. First Milestone: Server Control Panel

### Objective

Minimal viable control panel that allows users to securely store and manage LLM provider API keys.

### Scope

**In Scope:**
- Web-based control panel served at `/dashboard`
- Save and retrieve Anthropic API keys
- Save and retrieve OpenAI API keys
- Basic key management (add, view masked, delete)
- Encrypted storage (AES-256-GCM)
- Key balance display (fetch remaining credits from OpenAI/Anthropic APIs)

**Out of Scope (Future Milestones):**
- Budget management and leasing
- Token counting and cost tracking
- Team/user management
- Analytics and usage graphs
- Python SDK integration

### Technical Requirements

1. **Backend (Rust):**
   - REST API endpoints:
     - `POST /api/keys` - Save a new API key
     - `GET /api/keys` - List all keys (masked) with balances
     - `GET /api/keys/:id/balance` - Fetch balance from provider API
     - `DELETE /api/keys/:id` - Remove a key
   - Key encryption before database storage
   - Key types: `openai`, `anthropic`
   - Provider API integration:
     - OpenAI: `GET https://api.openai.com/v1/dashboard/billing/credit_grants`
     - Anthropic: `GET https://api.anthropic.com/v1/usage` (check available credits)

2. **Frontend (Vue.js):**
   - Vue 3 with Composition API
   - Simple form to add new keys (provider dropdown + key input)
   - Table displaying saved keys (masked: `sk-...xxxx`) with balance column
   - Refresh balance button per key
   - Delete button per key
   - Served as static assets from Rust backend

3. **Database Schema:**
   ```sql
   CREATE TABLE api_keys (
     id SERIAL PRIMARY KEY,
     provider VARCHAR(50) NOT NULL,  -- 'openai' or 'anthropic'
     key_name VARCHAR(255),          -- user-friendly label
     encrypted_key BYTEA NOT NULL,   -- AES-256-GCM encrypted
     cached_balance_cents INTEGER,   -- cached balance (updated on fetch)
     balance_updated_at TIMESTAMP,   -- last balance check
     created_at TIMESTAMP DEFAULT NOW()
   );
   ```

### Success Criteria

- [ ] User can add an OpenAI API key via web UI
- [ ] User can add an Anthropic API key via web UI
- [ ] Keys are stored encrypted in database
- [ ] Keys display masked in the UI (never show full key after save)
- [ ] Balance is fetched from OpenAI/Anthropic APIs and displayed
- [ ] User can delete keys
