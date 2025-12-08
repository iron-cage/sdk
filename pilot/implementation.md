# Implementation Location

The Rust implementation for the pilot platform is located in the **runtime** crate.

**Location:** `/home/user1/pro/lib/willbe/module/iron_cage/runtime/`

---

## Quick Links

- **Crate README:** [`/runtime/readme.md`](../runtime/readme.md)
- **Pilot Implementation Guide:** [`/runtime/PILOT_GUIDE.md`](../runtime/PILOT_GUIDE.md)
- **Cargo.toml:** [`/runtime/Cargo.toml`](../runtime/Cargo.toml)
- **Source Code:** [`/runtime/src/`](../runtime/src/)

---

## Pilot Features (from spec.md)

| Feature Set | Status | Implementation Location |
|-------------|--------|------------------------|
| Features #1-4 (Runtime) | ⬜ Not started | `runtime/src/runtime/` |
| Features #5-8 (Safety) | ⬜ Not started | `runtime/src/safety/` |
| Features #9-15 (Cost) | ⬜ Not started | `runtime/src/cost/` |
| Features #25-26 (API) | ⬜ Not started | `runtime/src/api/` |
| Features #29-35 (Secrets) | ⬜ Not started | `/iron_secrets/` (separate crate) |

---

## Build Commands

```bash
cd /home/user1/pro/lib/willbe/module/iron_cage/runtime

# Build runtime
cargo build --release

# Run CLI
./target/release/iron_cage_runtime --help

# Run tests
cargo test
```

---

## Demo Usage

```bash
# Start runtime with demo agent
cd /home/user1/pro/lib/willbe/module/iron_cage

./runtime/target/release/iron_cage_runtime pilot/demo/agent/lead_gen_agent.py --budget 50
```

---

## Directory Structure

```
runtime/
├── Cargo.toml              # Workspace manifest
├── readme.md               # Crate overview
├── PILOT_GUIDE.md         # Step-by-step pilot implementation guide
├── src/
│   ├── lib.rs              # Library exports
│   ├── runtime/            # Features #1-4 (Agent Lifecycle, PyO3, Config, Logging)
│   ├── safety/             # Features #5-8 (Privacy Protection, Redaction, Audit, Policy)
│   ├── cost/               # Features #9-15 (Token Counting, Budget, Safety Cutoff)
│   ├── api/                # Features #25-26 (State Management, WebSocket API)
│   └── secrets/            # Integration layer for iron_secrets crate
└── tests/
    ├── runtime_tests.rs    # Runtime feature tests
    ├── safety_tests.rs     # Safety feature tests
    ├── cost_tests.rs       # Cost control tests
    └── secrets_tests.rs    # Secrets management integration tests

iron_secrets/               # Separate crate for secrets management
├── Cargo.toml              # Crate manifest (cryptography dependencies)
├── spec.md                 # Complete specification (1,000+ lines)
├── src/
│   ├── lib.rs              # Public API (SecretsManager, Role, Environment)
│   ├── crypto.rs           # AES-256-GCM encryption/decryption
│   ├── storage.rs          # SQLite storage (secrets, secret_audit_log tables)
│   ├── access_control.rs   # RBAC (Admin, Viewer, Agent roles)
│   └── audit.rs            # Audit trail logging
└── tests/
    ├── crypto_tests.rs     # Encryption/decryption tests
    ├── storage_tests.rs    # Database tests
    └── integration_tests.rs # End-to-end tests
```

---

## Why Implementation Lives in runtime/

**Separation of Concerns:**
- **pilot/** = Documentation (spec, execution plans, conference materials, demo scripts)
- **runtime/** = Implementation (Rust crate with actual code)

**Benefits:**
- Pilot docs remain clean and focused
- Runtime crate can be published independently
- Clear boundary between WHAT (spec) and HOW (code)
- Runtime can support full platform beyond pilot features

---

**For detailed implementation guidance, see:** [`/runtime/PILOT_GUIDE.md`](../runtime/PILOT_GUIDE.md)
