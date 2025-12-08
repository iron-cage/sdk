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
