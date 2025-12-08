# Rust Crates - Pilot Platform Dependencies

**Purpose:** Specification of Rust crates required for Warsaw pilot platform (28 days, 35+ features with secrets management)

**Last Updated:** 2025-11-24

---

### Scope

**Responsibility:** Rust dependency specifications for Warsaw pilot implementation (WHY each crate needed, feature mappings, version requirements)

**In Scope:**
- Complete crate list for all 35+ pilot features (including secrets management) with rationale
- Feature-to-crate mapping (which features need which dependencies)
- Three implementation options (slides-only, minimal CLI, full pilot) with respective crate lists
- Version specifications, feature flags, and compilation requirements
- Quick start minimal dependencies (4 core features, 10 crates)
- Full pilot dependencies (35+ features with secrets, 23-24 crates)
- Installation commands and common compilation issues
- Alternative crates considered and rejection rationale

**Out of Scope:**
- Actual Cargo.toml implementation (see `/runtime/Cargo.toml` for current state)
- Full production platform crates beyond pilot (see `/spec/capability_*.md`)
- Python dependencies for demo agent (see `/pilot/demo/agent/requirements.txt`)
- React/TypeScript dependencies for control panel (see `/pilot/demo/control panel/package.json`)
- System-level dependencies and setup (see `tech_stack.md` in this directory)
- Build instructions and implementation guide (see `/runtime/PILOT_GUIDE.md`)
- Feature specifications and acceptance criteria (see `spec.md` in this directory)

---

## Document Responsibility

### WHAT THIS FILE IS

**Scope:** Warsaw pilot project ONLY (not full production platform)
- **Timeline:** 28 days until conference (Dec 16-17, 2025)
- **Features:** 35+ pilot features (4 core + 24 enhanced + 7 secrets management)
- **Goal:** Working demo for conference + pilot customer sales ($10-25K)
- **Deliverable:** CLI-based runtime (control panel optional)

**Responsibility:** Define WHAT Rust crates are needed and WHY
- Specification document (not implementation)
- Maps each crate to specific pilot features (#1-28)
- Explains rationale for crate selection
- Provides version requirements and feature flags
- Offers both minimal (quick start) and full (8-week) configurations

### WHAT THIS FILE DOES

**Contains:**
- ✅ Complete crate list for all 35+ pilot features (including secrets management)
- ✅ Quick start minimal crates (4 core features only)
- ✅ Feature-to-crate mapping table (which crates needed for which features)
- ✅ Version specifications and feature flags
- ✅ Rationale for each crate selection
- ✅ Alternative crates considered and rejected
- ✅ Complete Cargo.toml template
- ✅ Installation commands
- ✅ Common compilation issues

**Excludes:**
- ❌ Full production platform crates (see `/spec/capability_*.md` for full platform)
- ❌ Python dependencies (see `/pilot/demo/agent/requirements.txt`)
- ❌ Frontend dependencies (see `/pilot/demo/control panel/package.json`)
- ❌ System dependencies (see `tech_stack.md` in this directory)
- ❌ Implementation details (see `/runtime/PILOT_GUIDE.md`)
- ❌ Actual Cargo.toml file (see `/runtime/Cargo.toml` for current state)

### Relationship to Other Files

**This file (pilot/crates.md):** WHY each crate is needed (specification)
**runtime/Cargo.toml:** WHAT crates to actually use (implementation)

**Separation:**
- pilot/crates.md = Product specification (documents requirements)
- runtime/Cargo.toml = Implementation (actual dependencies)
- Changes flow: spec → implementation (not the reverse)

---

## Implementation Options Summary

| | **Option 1: Slides-Only** | **Option 2: Minimal CLI** | **Option 3: Full Pilot** |
|---|---|---|---|
| **Timeline** | 1 week (80-120h) | 2-3 weeks | 8 weeks (580h) |
| **Team** | Solo or +Dev1 | Solo +Dev1 (both Rust exp) | Solo +Dev1 +Dev2 |
| **Features** | 0 (slides only) | 4 core features | 35+ features (with secrets) |
| **Crates** | 0 | 10 | 23-24 (with cryptography) |
| **Deliverable** | Conference slides + rehearsal | CLI runtime (no control panel) | Full runtime + control panel |
| **Recommendation** | ✅ **Best for 23 days** | ⚠️ Risky for timeline | ❌ Infeasible for 23 days |
| **See** | `execution/quick_start.md` | Section below | `execution/8_week_plan.md` |

**Recommendation:** With 23 days remaining, focus on **Option 1 (Slides-Only)** for Warsaw conference. Build implementation afterward for pilot customers.

---

## Quick Start vs Full Pilot

### Option 1: Slides-Only Quick Start (Recommended for 23 days)

**Timeline:** 1 week (80-120 hours)
**Team:** Solo founder or + Dev1
**Deliverable:** Conference slides + rehearsal (NO code implementation)

**Crates needed:** NONE (slides only, no runtime implementation)

**See:** `execution/quick_start.md` for slides-only approach

---

### Option 2: Minimal CLI Demo (4 Core Features)

**Timeline:** 2-3 weeks if starting implementation
**Team:** Solo + Dev1 (both experienced Rust developers)
**Deliverable:** CLI runtime with 4 core features (no control panel)

**Features implemented:**
- Feature #1: Agent Lifecycle (spawn/monitor Python process)
- Feature #2: PyO3 Integration (Python-Rust FFI bridge) ← CRITICAL
- Feature #3: Configuration (CLI args, --budget flag)
- Feature #4: Logging (structured terminal output)

**Crates needed (9 total):**
```toml
tokio = { version = "1.40", features = ["full"] }
pyo3 = { version = "0.22", features = ["extension-module", "auto-initialize"] }
pyo3-asyncio = { version = "0.22", features = ["tokio-runtime"] }
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
regex = "1.10"  # For basic privacy protection
thiserror = "1.0"
anyhow = "1.0"
```

**See:** "Quick Start Minimal Dependencies" section below

---

### Option 3: Full Pilot Platform (All 35+ Features)

**Timeline:** 8 weeks (580 hours, 3-person team)
**Team:** Solo + Dev1 + Dev2
**Deliverable:** Full runtime + control panel + all 35+ features (including secrets management)

**Features implemented:** All 35+ features across 5 capabilities
- Capability 8: Runtime (4 features)
- Capability 2: Safety (4 features)
- Capability 3: Cost Control (7 features)
- Capability 5: Secrets Management (7 features)
- Demo Infrastructure (3 features)
- Control Panel (6 features)
- Supporting Infrastructure (2 features)
- Testing & Quality (2 features)

**Crates needed (23-24 total):** See "Complete Cargo.toml" section below

**See:** `execution/8_week_plan.md` for full timeline

---

## Current Status: What's Actually Implemented?

**Check runtime/Cargo.toml** to see current state.

**As of 2025-11-24:**
- runtime/Cargo.toml has 9 production dependencies (basics only)
- Missing: pyo3-asyncio, axum, tower, tower-http, toml, sqlx, dashmap, anyhow
- Status: Minimal skeleton, not yet demo-ready

**To implement quick start (4 features):** Add pyo3-asyncio, anyhow
**To implement full pilot (35+ features with secrets):** Add all crates listed in "Complete Cargo.toml" section

---

## Core Runtime Crates

### Async Runtime (Features #1-4, #25-26)

**tokio** = `{ version = "1.40", features = ["full"] }`
- **Why:** Async runtime for all I/O operations
- **Features:** #1 (agent lifecycle), #2 (PyO3 async bridge), #26 (WebSocket server)
- **Critical:** Yes - foundation of entire runtime
- **Alternatives:** async-std (not recommended, less ecosystem support)

**pyo3** = `{ version = "0.22", features = ["extension-module", "auto-initialize"] }`
- **Why:** Python FFI bridge for LangChain/CrewAI agents
- **Features:** #2 (Python-Rust Integration)
- **Critical:** Yes - core feature for pilot
- **Alternatives:** None (PyO3 is the standard)

**pyo3-asyncio** = `{ version = "0.22", features = ["tokio-runtime"] }`
- **Why:** Bridges Python asyncio with Tokio
- **Features:** #2 (async event loop integration)
- **Critical:** Yes - required for async agent calls
- **Alternatives:** None

---

## Web Framework (Feature #26)

**axum** = `"0.7"`
- **Why:** REST API + WebSocket server for control panel
- **Features:** #26 (API & Communication)
- **Critical:** Yes (for control panel), No (for CLI-only quick start)
- **Alternatives:** actix-web (more complex), warp (less ergonomic)

**tower** = `"0.4"`
- **Why:** Middleware layer (CORS, logging, rate limiting)
- **Features:** #26 (CORS configuration, request logging)
- **Critical:** Yes (if using axum)
- **Alternatives:** None (tower is axum's middleware layer)

**tower-http** = `{ version = "0.5", features = ["cors", "trace"] }`
- **Why:** HTTP-specific middleware (CORS headers)
- **Features:** #26 (control panel CORS)
- **Critical:** Yes (if control panel on different origin)

---

## Configuration & Serialization (Features #3, #25)

**clap** = `{ version = "4.5", features = ["derive"] }`
- **Why:** CLI argument parsing
- **Features:** #3 (--budget, --verbose, --safety-mode flags)
- **Critical:** Yes - user interface for runtime
- **Alternatives:** structopt (deprecated, merged into clap v3+)

**serde** = `{ version = "1.0", features = ["derive"] }`
- **Why:** Serialization/deserialization framework
- **Features:** #3 (config), #25 (state), #26 (API JSON)
- **Critical:** Yes - used throughout codebase
- **Alternatives:** None (industry standard)

**serde_json** = `"1.0"`
- **Why:** JSON parsing for API responses and logs
- **Features:** #4 (structured logging), #26 (REST API)
- **Critical:** Yes

**toml** = `"0.8"`
- **Why:** TOML config file parsing
- **Features:** #3 (agent configuration loading)
- **Critical:** No (can use YAML or env vars only)
- **Alternatives:** config crate (supports multiple formats)

---

## Logging (Feature #4)

**tracing** = `"0.1"`
- **Why:** Structured logging framework
- **Features:** #4 (Logging Infrastructure)
- **Critical:** Yes - required for demo output
- **Alternatives:** log crate (less structured), slog (more complex)

**tracing-subscriber** = `{ version = "0.3", features = ["env-filter", "json"] }`
- **Why:** Logging backend and formatters
- **Features:** #4 (JSON format, log levels, RUST_LOG env)
- **Critical:** Yes
- **Feature flags:**
  - `env-filter`: RUST_LOG environment variable support
  - `json`: JSON-formatted logs for production

---

## Safety Features (Features #5-8)

**regex** = `"1.10"`
- **Why:** PII pattern detection (email, phone, SSN)
- **Features:** #5 (PII Pattern Detection)
- **Critical:** Yes - core safety feature
- **Alternatives:** fancy-regex (supports lookahead/lookbehind, slower)

**aes-gcm** = `"0.10"`
- **Why:** AES-256-GCM authenticated encryption
- **Features:** #6 (PII encrypted storage), #29 (Secrets encryption)
- **Critical:** Yes (for secrets management), Optional (for PII storage)
- **Alternatives:** chacha20poly1305 (faster on some CPUs, but AES-GCM has hardware acceleration)

**argon2** = `"0.5"`
- **Why:** Key derivation function (OWASP recommended)
- **Features:** #29 (Secrets encryption - derive encryption key from master key)
- **Critical:** Yes (for secrets management)
- **Alternatives:** scrypt (older), bcrypt (password hashing only)

**rand** = `"0.8"`
- **Why:** Cryptographically secure random number generation
- **Features:** #29 (Secrets encryption - generate nonces and salts)
- **Critical:** Yes (for secrets management)
- **Alternatives:** getrandom (lower-level, rand uses it internally)

**zeroize** = `"1.5"` (Optional)
- **Why:** Secure memory clearing (zero secrets after use)
- **Features:** #29 (Defense in depth - clear decrypted secrets from memory)
- **Critical:** No (nice-to-have security hardening)
- **Alternatives:** Manual zeroing (less safe, easy to miss)

---

## Secrets Management (Features #29-35) - NEW

### iron_secrets Crate (Separate Domain Logic Crate)

**Purpose:** Centralized secrets management for pilot platform (agent credentials, API keys, database passwords).

**Business Context:** Company requires secrets management feature, team has domain expertise (can develop efficiently).

**Architecture Decision:**
- **Pilot:** Custom implementation (iron_secrets) with AES-256-GCM encryption
- **Full Platform:** Thin wrapper around HashiCorp Vault
- **Rationale:** Pilot doesnt need enterprise Vault features (HSM, dynamic secrets, lease management). Simple encrypted storage sufficient for 28-day timeline.

**Key Dependencies:**
- `aes-gcm = "0.10"` - AES-256-GCM authenticated encryption
- `argon2 = "0.5"` - Key derivation function (OWASP recommended)
- `sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-native-tls"] }` - Database access
- `rand = "0.8"` - Cryptographically secure random number generation
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `tokio = { version = "1.40", features = ["full"] }` - Async runtime

**Functionality:**
- Secret CRUD operations (create, read, update, delete)
- AES-256-GCM encryption at rest (secrets stored encrypted in SQLite)
- Argon2id key derivation (m=19456 KiB, t=2, p=1)
- Role-based access control (Admin, Viewer, Agent)
- Secret masking in control panel (`sk-proj-abc...xyz` format)
- Audit trail (immutable log of all secret operations)
- Environment-based secrets (Development, Staging, Production)

**Integration Points:**
- **iron_runtime:** Secret injection at agent spawn (environment variables)
- **iron_api:** REST endpoints for secrets CRUD (`/secrets/...`)
- **iron_control:** New 7th panel for secrets management
- **iron_state:** Two new SQLite tables (`secrets`, `secret_audit_log`)
- **iron_telemetry:** Secret redaction in logs (treat like PII)

**Security Properties:**
- Master key from environment variable (`IRON_SECRETS_MASTER_KEY`)
- Unique nonces per encryption (12-byte random)
- Unique salts per secret (16-byte random)
- Secrets never logged in plaintext
- Zero-downtime rotation via SIGUSR1 signal

**Pilot Scope:**
- SQLite storage (pilot-appropriate)
- Environment variable master key (full platform uses AWS KMS)
- Single-instance deployment (no distributed secret coordination)

**Migration to Full Platform:**
- Pilot: AES-256-GCM + SQLite (no external dependencies)
- Full: Thin wrapper around HashiCorp Vault (production-grade)
- Migration: Export/import tool to migrate secrets from pilot to Vault

**Specification:** `/home/user1/pro/lib/willbe/module/iron_secrets/spec.md` (1,000+ lines)

**Effort Estimate:**
- Implementation: 4 days (800 LOC Rust + tests + integration)
- Cost Impact: +$5,600 (8.7% increase)
- Timeline Impact: +6 calendar days (Days 23-28)

---

## Storage (Features #7, #25, #29, #34)

**sqlx** = `{ version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }`
- **Why:** Async SQLite driver for audit logs and secrets storage
- **Features:** #7 (PII Audit Logging), #25 (State Management), #29 (Secret Storage), #34 (Secret Audit Trail)
- **Critical:** Yes (for audit trail and secrets), No (for quick start)
- **Feature flags:**
  - `sqlite`: SQLite database support
  - `runtime-tokio-rustls`: Async with Tokio + TLS
- **Alternatives:** rusqlite (sync only), diesel (heavier ORM)

**redis** = `{ version = "0.24", features = ["tokio-comp", "connection-manager"] }` (Optional)
- **Why:** Distributed state for multi-instance runtime
- **Features:** #25 (Redis for distributed state)
- **Critical:** No (optional for pilot, in-memory is fine)

**dashmap** = `"5.5"`
- **Why:** Concurrent in-memory HashMap
- **Features:** #25 (in-memory state storage)
- **Critical:** Yes (if not using Redis)
- **Alternatives:** std::sync::RwLock<HashMap> (less ergonomic)

---

## Error Handling

**thiserror** = `"1.0"`
- **Why:** Derive Error trait for custom errors
- **Features:** All features (consistent error types)
- **Critical:** Yes - Rust best practice
- **Alternatives:** anyhow (for applications, not libraries)

**anyhow** = `"1.0"`
- **Why:** Error propagation with context
- **Features:** All features (? operator with context)
- **Critical:** Yes (for main.rs, not lib.rs)
- **Alternatives:** eyre (similar, more features)

---

## Cost Control (Features #9-15)

**tiktoken-rs** = `"0.5"` (Optional)
- **Why:** OpenAI token counting (exact)
- **Features:** #9 (Real-Time Token Counting)
- **Critical:** No (can approximate with char count)
- **Alternatives:** Manual approximation (tokens ≈ chars / 4)
- **Note:** Large dependency (~50MB), consider approximation for pilot

**reqwest** = `{ version = "0.12", features = ["json"] }`
- **Why:** HTTP client for LLM API calls (cost tracking)
- **Features:** #9 (intercept LLM calls), #13 (circuit breaker on API failures)
- **Critical:** Yes (if runtime intercepts LLM calls)
- **Alternatives:** hyper (lower-level), ureq (blocking)

---

## Safety Cutoff (Features #13-15)

**tokio::time** (part of tokio)
- **Why:** Cooldown timers for circuit breaker
- **Features:** #13 (60-second cooldown timer)
- **Critical:** Yes (if implementing circuit breaker)

**governor** = `"0.6"` (Optional)
- **Why:** Rate limiting (can be used for circuit breaker logic)
- **Features:** #13 (failure threshold detection)
- **Critical:** No (can implement manually)
- **Alternatives:** Manual implementation with atomic counters

---

## Testing (Features #27-28)

**tokio-test** = `"0.4"` (dev-dependency)
- **Why:** Testing utilities for async code
- **Features:** #27 (Demo Testing)
- **Critical:** Yes (for tests)

**mockall** = `"0.12"` (dev-dependency)
- **Why:** Mock generation for testing
- **Features:** #27 (mocking LLM APIs in tests)
- **Critical:** No (can use real test APIs)
- **Alternatives:** Manual mocks

**criterion** = `"0.5"` (dev-dependency, Optional)
- **Why:** Benchmarking tool
- **Features:** #22 (Performance Panel validation)
- **Critical:** No (for optimization only)

---

## Complete Cargo.toml

```toml
[package]
name = "iron_cage_runtime"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[dependencies]
# Async runtime (Features #1-4, #25-26)
tokio = { version = "1.40", features = ["full"] }

# Python FFI (Feature #2)
pyo3 = { version = "0.22", features = ["extension-module", "auto-initialize"] }
pyo3-asyncio = { version = "0.22", features = ["tokio-runtime"] }

# Web framework (Feature #26)
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Configuration (Feature #3)
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Logging (Feature #4)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Safety (Features #5-8)
regex = "1.10"

# Storage (Features #7, #25)
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }
dashmap = "5.5"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# HTTP client (Features #9, #13)
reqwest = { version = "0.12", features = ["json"] }

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"

[features]
default = ["control panel"]
control panel = ["axum", "tower", "tower-http"]  # Optional for CLI-only quick start
```

---

## Quick Start Minimal Dependencies (Option 2)

**For:** CLI-only demo with 4 core features (no control panel, no database, no circuit breaker)

**Features covered:**
- Feature #1: Agent Management
- Feature #2: PyO3 Integration (CRITICAL)
- Feature #3: Configuration System
- Feature #4: Logging Infrastructure
- Feature #5: Basic Privacy Protection (regex only)

**Crates (10 total):**

```toml
[package]
name = "iron_cage_runtime"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[dependencies]
# Async runtime (Features #1, #2)
tokio = { version = "1.40", features = ["full"] }

# Python FFI (Feature #2 - CRITICAL)
pyo3 = { version = "0.22", features = ["extension-module", "auto-initialize"] }
pyo3-asyncio = { version = "0.22", features = ["tokio-runtime"] }

# Configuration (Feature #3)
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }

# Logging (Feature #4)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Safety - privacy protection (Feature #5)
regex = "1.10"

# Error handling (All features)
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
tokio-test = "0.4"
```

**What this excludes vs full pilot:**
- ❌ No axum/tower (no control panel, saves ~8 crates)
- ❌ No sqlx (no database audit logs, saves ~5 crates)
- ❌ No dashmap (no in-memory state management)
- ❌ No reqwest (no circuit breaker, no LLM interception)
- ❌ No toml (config via CLI args only)

**Savings:**
- ~8 fewer production dependencies
- ~50MB smaller compiled binary
- ~3-5 minutes faster compile time

**Trade-offs:**
- ✅ Can demo core functionality (agent lifecycle, PyO3 bridge, logging)
- ✅ Can show basic privacy protection
- ❌ Cannot show control panel UI
- ❌ Cannot show circuit breaker
- ❌ Cannot show budget enforcement (no LLM interception)
- ❌ Cannot show audit logs (no database)

**Recommendation for 23 days:** Implement this minimal set ONLY if you can't do slides-only approach

---

## Installation Commands

```bash
# Add all dependencies at once
cd /home/user1/pro/lib/willbe/module/iron_cage/runtime

# Copy Cargo.toml content above, then:
cargo build --release

# Or add individually:
cargo add tokio --features full
cargo add pyo3 --features extension-module,auto-initialize
cargo add pyo3-asyncio --features tokio-runtime
cargo add axum@0.7
cargo add tower@0.4
cargo add tower-http --features cors,trace
cargo add clap --features derive
cargo add serde --features derive
cargo add serde_json
cargo add toml
cargo add tracing
cargo add tracing-subscriber --features env-filter,json
cargo add regex
cargo add sqlx --features sqlite,runtime-tokio-rustls
cargo add dashmap
cargo add thiserror
cargo add anyhow
cargo add reqwest --features json

# Dev dependencies
cargo add --dev tokio-test
cargo add --dev mockall
```

---

## Crate Summary by Feature

| Feature # | Feature Name | Required Crates |
|-----------|--------------|-----------------|
| #1 | Agent Lifecycle | tokio, clap, tracing, anyhow |
| #2 | PyO3 Integration | pyo3, pyo3-asyncio, tokio |
| #3 | Configuration | clap, serde, toml, anyhow |
| #4 | Logging | tracing, tracing-subscriber |
| #5 | Privacy Protection | regex |
| #6 | Output Redaction | regex, aes-gcm |
| #7 | PII Audit | sqlx, serde_json |
| #8 | Policy Enforcement | (logic only, no new crates) |
| #9 | Token Counting | reqwest (optional: tiktoken-rs) |
| #10 | Budget Limits | (logic only, no new crates) |
| #11 | Alert System | reqwest (for webhooks) |
| #12 | Cost Attribution | (logic only, no new crates) |
| #13 | Safety Cutoff | tokio::time, reqwest |
| #14 | Fallback Chain | (logic only, no new crates) |
| #15 | Circuit Metrics | (logic only, no new crates) |
| #25 | State Management | dashmap, sqlx |
| #26 | API & WebSocket | axum, tower, tower-http |
| #27 | Testing | tokio-test, mockall |
| #28 | Error Handling | anyhow, thiserror |
| **#29** | **Secret Storage & Encryption** | **aes-gcm, argon2, rand, sqlx, serde** |
| **#30** | **Secret CRUD API** | **axum, serde, serde_json** |
| **#31** | **RBAC for Secrets** | **(logic only, no new crates)** |
| **#32** | **Agent Secret Injection** | **tokio** |
| **#33** | **Secrets Control Panel Panel** | **(frontend only, no Rust crates)** |
| **#34** | **Secret Audit Trail** | **sqlx, serde_json** |
| **#35** | **Secret Rotation Workflow** | **tokio (SIGUSR1 signal handling)** |

---

## Total Dependency Count

**Production dependencies:** 23-24 crates (with secrets management cryptography)
**Dev dependencies:** 2 crates
**Optional dependencies:** 2 crates (tiktoken-rs, redis)

**Compile time (release):** ~5-10 minutes (first build)
**Binary size (release):** ~15-25MB

---

## Version Management

**Recommendation:** Pin major versions, allow minor/patch updates

```toml
tokio = "1.40"        # Allows 1.40.x, 1.41.x, etc. (not 2.0)
axum = "0.7"          # Allows 0.7.x (not 0.8)
pyo3 = "0.22"         # Allows 0.22.x (not 0.23)
```

**Update strategy:**
```bash
# Check for updates
cargo outdated

# Update to latest compatible versions
cargo update

# Update to latest versions (may break)
cargo upgrade  # requires cargo-edit
```

---

## Common Issues

### PyO3 compilation fails
**Error:** `Python.h not found`
**Fix:**
```bash
sudo apt install python3-dev  # Ubuntu
brew install python@3.11       # macOS
```

### SQLx compile-time verification fails
**Error:** `DATABASE_URL not set`
**Fix:**
```bash
# Create database first
touch pilot_audit.db

# Or disable compile-time checks
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"], default-features = false }
```

### Large compile times
**Issue:** First build takes 10+ minutes
**Fix:**
```bash
# Use mold linker (Linux)
cargo install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# Use lld linker (macOS/Linux)
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
```

---

## Next Steps

**Decision tree:**
1. **Have 23 days or less?** → Use Option 1 (Slides-Only), read `execution/quick_start.md`
2. **Have 2-3 weeks + Rust team?** → Use Option 2 (Minimal CLI), read section above
3. **Have 8 weeks + 3 devs?** → Use Option 3 (Full Pilot), read `execution/8_week_plan.md`

**Related files:**
- **Implementation guide:** `/runtime/PILOT_GUIDE.md` (step-by-step build instructions)
- **Technology stack:** `tech_stack.md` (Python, React, system deps)
- **Feature specifications:** `spec.md` (all 35+ features detailed)
- **Execution plans:** `execution/` (quick start, 8-week, status)

**Current implementation status:** Check `/runtime/Cargo.toml` for what's actually built
