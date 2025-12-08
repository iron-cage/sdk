# Technology Stack - Complete Pilot Platform

**Purpose:** Comprehensive technology inventory for entire pilot platform (Rust runtime + Python demo + Vue control panel + infrastructure)

**Last Updated:** 2025-11-23

---

### Scope

**Responsibility:** Complete technology inventory across all pilot platform components (Rust runtime, Python demo agent, Vue control panel, infrastructure, development tools)

**In Scope:**
- All technology versions and installation commands (Rust 1.75+, Python 3.11+, Node 18+, SQLite 3.40+, Redis 7.0+)
- System dependencies and setup (build tools, Python headers, database servers)
- Development tools and workflows (rustfmt, clippy, black, mypy, ESLint)
- Build commands for all three components (Rust runtime, Python agent, Vue control panel)
- Environment configuration (.env files, directory structure)
- Complete setup checklist from scratch (30-45 min estimated)
- Technology decision rationale (why Rust, why PyO3, why Vue, why SQLite, why Axum)
- Common issues and solutions (compile times, PyO3 errors, WebSocket failures, SQLite locks)
- OS compatibility matrix (Ubuntu 22.04+, macOS 13+, Windows 11 WSL2)
- Production upgrade path (SQLite→PostgreSQL, in-memory→Redis, single→Kubernetes)

**Out of Scope:**
- Detailed Rust crate explanations and rationale (see `crates.md` for WHY each crate needed)
- Step-by-step implementation instructions (see `/runtime/pilot_guide.md` for how to build)
- Feature specifications and acceptance criteria (see `spec.md` for all 35+ features)
- Python package details beyond versions (see `/pilot/demo/agent/requirements.txt`)
- Vue component specifications (see `/pilot/demo/control panel/package.json`)
- Execution planning and timeline (see `execution/` for 8-week plan, quick start)

---

## File Responsibility

**WHAT THIS FILE DOES:**
- ✅ Lists ALL technologies across entire pilot platform
- ✅ Covers Rust, Python, JavaScript/TypeScript, infrastructure
- ✅ Specifies versions, installation commands, and purposes
- ✅ Documents system dependencies and development tools
- ✅ Provides complete setup guide from scratch

**WHAT THIS FILE DOES NOT:**
- ❌ Detailed Rust crate explanations (see `crates.md` in this directory)
- ❌ Implementation instructions (see `/runtime/PILOT_GUIDE.md`)
- ❌ Feature specifications (see `spec.md` in this directory)

---

## Stack Overview

```
┌─────────────────────────────────────────────────────────┐
│  PILOT PLATFORM TECHNOLOGY STACK                        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌────────────┐  ┌──────────────┐  ┌────────────────┐ │
│  │  Frontend  │  │  Demo Agent  │  │ Rust Runtime   │ │
│  │            │  │              │  │                │ │
│  │  Vue 3.4   │  │  Python 3.11 │  │  Rust 1.75+    │ │
│  │  shadcn-vue│  │  LangChain   │  │  Tokio/Axum    │ │
│  │  Tailwind  │  │  OpenAI SDK  │  │  PyO3          │ │
│  └─────┬──────┘  └──────┬───────┘  └────────┬───────┘ │
│        │                │                   │         │
│        └────────────────┴───────────────────┘         │
│                         │                             │
│                  ┌──────▼──────┐                      │
│                  │ Infrastructure│                     │
│                  │  SQLite 3.40+ │                     │
│                  │  Redis 7.0+   │                     │
│                  └───────────────┘                     │
└─────────────────────────────────────────────────────────┘
```

---

## 1. Rust Runtime

**Location:** `/runtime/`

### Core Technologies

| Technology | Version | Purpose | Features Used |
|------------|---------|---------|---------------|
| **Rust** | 1.75+ | Runtime implementation language | #1-35 (all features) |
| **Cryptography Stack** | Latest | Secrets management | #29-35 (secrets management) |
| **Tokio** | 1.40+ | Async runtime | #1-4, #25-26, #32, #35 |
| **PyO3** | 0.22+ | Python FFI bridge | #2 (core integration) |
| **Axum** | 0.7 | Web framework | #26, #30 (REST API + WebSocket) |
| **SQLx** | 0.7 | Async database driver | #7, #25, #29, #34 (audit logs, secrets) |
| **Regex** | 1.10 | Pattern matching | #5-8 (privacy protection) |

**Complete crate list:** See `crates.md` in this directory (23-24 production dependencies with cryptography)

**Cryptography Dependencies (iron_secrets):**
- `aes-gcm = "0.10"` - AES-256-GCM authenticated encryption (AEAD)
- `argon2 = "0.5"` - Key derivation function (OWASP recommended)
- `rand = "0.8"` - Cryptographically secure random number generation (nonces, salts)
- `getrandom = "0.2"` - OS entropy source (via rand)
- `zeroize = "1.5"` - Secure memory clearing (zero secrets after use)

**Why These Choices:**
- **AES-256-GCM:** Industry standard, hardware-accelerated (AES-NI), authenticated encryption
- **Argon2id:** OWASP recommended KDF (password hashing competition winner)
- **rand + getrandom:** Best practice for cryptographic random values (no thread_rng())
- **zeroize:** Defense in depth (clear secrets from memory after use)

### Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
rustup default stable

# Verify
rustc --version  # Should be 1.75+
cargo --version
```

### Build Commands

```bash
cd /home/user1/pro/lib/willbe/module/iron_cage/runtime

# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 2. Python Demo Agent

**Location:** `/pilot/demo/agent/`

### Core Technologies

| Technology | Version | Purpose | Features Used |
|------------|---------|---------|---------------|
| **Python** | 3.11+ | Demo agent language | #16-18 (demo infrastructure) |
| **LangChain** | 0.1+ | Agent framework | #16 (lead gen agent) |
| **OpenAI SDK** | 1.0+ | LLM API client | #9 (token counting), demo |
| **Requests** | 2.31+ | HTTP client | #16 (LinkedIn/Clearbit APIs) |
| **Pydantic** | 2.0+ | Data validation | #16 (agent config) |

### Python Dependencies (requirements.txt)

```txt
# Agent framework
langchain==0.1.0
langchain-openai==0.0.2

# LLM APIs
openai==1.0.0
anthropic==0.8.0  # Optional

# HTTP client
requests==2.31.0
httpx==0.25.0  # Async HTTP

# Data validation
pydantic==2.5.0
pydantic-settings==2.1.0

# Data processing
pandas==2.1.0  # For CSV test data
```

### Installation

```bash
# Install Python 3.11+
sudo apt install python3.11 python3.11-venv  # Ubuntu
brew install python@3.11                      # macOS

# Create virtual environment
cd /home/user1/pro/lib/willbe/module/iron_cage/pilot/demo/agent
python3.11 -m venv .venv
source .venv/bin/activate  # Linux/macOS
# .venv\Scripts\activate   # Windows

# Install dependencies
uv pip install -r requirements.txt

# Verify
python --version  # Should be 3.11+
uv pip list
```

### Demo Agent Structure

```python
# lead_gen_agent.py (simplified)
from langchain.agents import AgentExecutor, create_openai_functions_agent
from langchain_openai import ChatOpenAI
from langchain.tools import tool

@tool
def search_linkedin(company: str) -> dict:
    """Search LinkedIn for company contacts."""
    # Demo implementation
    pass

llm = ChatOpenAI(model="gpt-4", temperature=0)
agent = create_openai_functions_agent(llm, [search_linkedin], prompt)
executor = AgentExecutor(agent=agent, tools=[search_linkedin])

# Process 100 leads
for lead in leads:
    result = executor.invoke({"company": lead.company})
```

---

## 3. Vue Control Panel

**Location:** `/pilot/demo/control panel/`

### Core Technologies

| Technology | Version | Purpose | Features Used |
|------------|---------|---------|---------------|
| **Vue** | 3.4+ | UI framework | #19-24 (all control panel features) |
| **TypeScript** | 5.0+ | Type safety | #19-24 (all control panel) |
| **Vite** | 5.0+ | Build tool | Development + production builds |
| **shadcn-vue** | Latest | Component library | #19-24 (UI components) |
| **Tailwind CSS** | 3.4+ | Styling framework | #19-24 (UI styling, basis for shadcn-vue) |
| **Recharts** | 2.10+ | Data visualization | #20, #22 (charts) |
| **Socket.io-client** | 4.6+ | WebSocket client | #19, #23 (real-time updates) |

### Frontend Dependencies (package.json)

```json
{
  "name": "iron-cage-control panel",
  "version": "0.1.0",
  "type": "module",
  "dependencies": {
    "vue": "^3.4.0",
    "shadcn-vue": "latest",
    "socket.io-client": "^4.6.0",
    "recharts": "^2.10.0",
    "axios": "^1.6.0",
    "date-fns": "^2.30.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "eslint": "^8.55.0",
    "eslint-plugin-vue": "^9.0.0"
  }
}
```

### Installation

```bash
# Install Node.js 18+
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install nodejs  # Ubuntu

brew install node@18     # macOS

# Verify
node --version  # Should be 18+
npm --version

# Install dependencies
cd /home/user1/pro/lib/willbe/module/iron_cage/pilot/demo/control panel
npm install

# Development server
npm run dev
# Open http://localhost:5173

# Production build
npm run build
npm run preview
```

### Control Panel Structure

```
control panel/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
├── index.html
├── src/
│   ├── main.ts               # Entry point
│   ├── App.vue               # Main app component
│   ├── components/
│   │   ├── LiveMetrics.vue   # Feature #19
│   │   ├── BudgetPanel.vue     # Feature #20
│   │   ├── ProtectionPanel.vue   # Feature #21
│   │   ├── PerfPanel.vue     # Feature #22
│   │   ├── ActivityLog.vue      # Feature #23
│   │   └── Notification.vue    # Feature #24
│   ├── composables/
│   │   └── useWebSocket.ts   # WebSocket connection
│   └── types/
│       └── agent.ts          # TypeScript types
└── public/
    └── favicon.ico
```

---

## 4. Infrastructure & Storage

### Database (Feature #7, #25)

**SQLite 3.40+**
- **Purpose:** Audit log storage (PII events, cost tracking)
- **Features:** #7 (PII Audit Logging), #25 (State Management)
- **Installation:**
  ```bash
  sudo apt install sqlite3 libsqlite3-dev  # Ubuntu
  brew install sqlite                       # macOS

  # Verify
  sqlite3 --version  # Should be 3.40+
  ```

**Redis 7.0+** (Optional)
- **Purpose:** Distributed state for multi-instance runtime
- **Features:** #25 (optional distributed state)
- **Installation:**
  ```bash
  sudo apt install redis-server  # Ubuntu
  brew install redis             # macOS

  # Start server
  redis-server

  # Verify
  redis-cli ping  # Should return PONG
  ```

### System Dependencies

**Python development headers** (for PyO3)
```bash
sudo apt install python3-dev python3.11-dev  # Ubuntu
xcode-select --install                        # macOS
```

**Build tools** (for compiling Rust)
```bash
sudo apt install build-essential pkg-config libssl-dev  # Ubuntu
xcode-select --install                                   # macOS
```

**Git** (for version control)
```bash
sudo apt install git  # Ubuntu
brew install git      # macOS

git --version  # Should be 2.0+
```

**uv** (Python package manager - REQUIRED)
```bash
# Linux/macOS
curl -LsSf https://astral.sh/uv/install.sh | sh

# macOS (Homebrew)
brew install uv

# Windows (PowerShell)
powershell -c "irm https://astral.sh/uv/install.ps1 | iex"

# Verify installation
uv --version  # Should show 0.1.0 or higher
```

**Why uv over pip:**
- **Performance:** 10-100x faster than pip (parallel downloads, better caching)
- **Reliability:** Better dependency resolution (like Poetry), reproducible installs
- **Compatibility:** Drop-in replacement for pip, works with requirements.txt
- **Modern:** Built in Rust, actively maintained by Astral (creators of Ruff)

**Basic Usage:**
```bash
# Install from requirements.txt
uv pip install -r requirements.txt

# Install package
uv pip install langchain
# OR add to project (creates/updates pyproject.toml)
uv add langchain

# Install dev dependencies
uv pip install black mypy pytest
# OR
uv add --dev black mypy pytest

# List installed packages
uv pip list

# Freeze dependencies
uv pip freeze > requirements.txt

# Create virtual environment (optional - faster than python -m venv)
uv venv
source .venv/bin/activate  # Linux/macOS
.venv\Scripts\activate     # Windows
```

**Migration from pip:**
- `pip install <package>` → `uv pip install <package>` or `uv add <package>`
- `pip install -r requirements.txt` → `uv pip install -r requirements.txt`
- `pip list` → `uv pip list`
- `pip freeze` → `uv pip freeze`
- `python -m venv .venv` → `uv venv` (optional, both work with uv)

**Documentation:** https://github.com/astral-sh/uv

---

## 5. Development Tools

### Code Quality

**Rust tooling:**
```bash
# Formatter
rustfmt --version
cargo fmt

# Linter
clippy --version
cargo clippy

# Test runner
cargo install cargo-nextest
cargo nextest run
```

**Python tooling:**
```bash
uv pip install black mypy pytest

# Formatter
black *.py

# Type checker
mypy lead_gen_agent.py

# Test runner
pytest tests/
```

**TypeScript/Vue tooling:**
```bash
# Comes with npm install
npm run lint      # ESLint + eslint-plugin-vue
npm run type-check  # TypeScript
```

### Documentation

**Rust docs:**
```bash
cargo doc --open
```

**Python docs:**
```bash
uv pip install sphinx
sphinx-quickstart
sphinx-build -b html docs/ docs/_build/
```

---

## 6. CI/CD & Deployment

### GitHub Actions (Optional)

```yaml
# .github/workflows/rust.yml
name: Rust CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
```

### Docker (Optional)

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY runtime/ .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y python3.11
COPY --from=builder /app/target/release/iron_cage_runtime /usr/local/bin/
CMD ["iron_cage_runtime"]
```

**Build:**
```bash
docker build -t iron-cage-runtime .
docker run -p 8080:8080 iron-cage-runtime
```

---

## 7. Environment Setup

### Environment Variables

```bash
# .env file
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
IRON_SECRETS_MASTER_KEY=<64-char-hex-string>  # For secrets encryption
REDIS_URL=redis://localhost:6379
DATABASE_URL=sqlite://pilot_audit.db
RUST_LOG=debug
```

### Directory Structure

```
/home/user1/pro/lib/willbe/module/iron_cage/
├── runtime/                 # Rust runtime implementation
│   ├── Cargo.toml
│   ├── PILOT_GUIDE.md      # Implementation guide
│   ├── src/
│   └── tests/
├── iron_secrets/           # Secrets management crate (NEW)
│   ├── Cargo.toml
│   ├── spec.md             # Complete specification
│   ├── src/
│   │   ├── lib.rs          # Public API
│   │   ├── crypto.rs       # AES-256-GCM encryption
│   │   ├── storage.rs      # SQLite storage
│   │   ├── access_control.rs # RBAC
│   │   └── audit.rs        # Audit trail
│   └── tests/
├── pilot/                   # Pilot project documentation
│   ├── spec.md             # Feature specification
│   ├── tech_stack.md       # This file (complete technology stack)
│   ├── crates.md           # Rust crate list (WHY each crate needed)
│   ├── demo/
│   │   ├── agent/          # Python demo agent
│   │   │   ├── requirements.txt
│   │   │   ├── lead_gen_agent.py
│   │   │   └── .venv/
│   │   └── control panel/      # Vue control panel
│   │       ├── package.json
│   │       ├── src/
│   │       └── node_modules/
│   └── execution/
└── spec/                    # Full capability specs
```

---

## 8. Version Matrix

### Minimum Versions

| Component | Minimum | Recommended | Notes |
|-----------|---------|-------------|-------|
| **Rust** | 1.75 | Latest stable | Use rustup for updates |
| **Python** | 3.11 | 3.11 | PyO3 requires 3.11+ |
| **Node.js** | 18.0 | 20 LTS | For Vue control panel |
| **SQLite** | 3.40 | Latest | System package |
| **Redis** | 7.0 | Latest | Optional for pilot |
| **Git** | 2.0 | Latest | Version control |

### OS Compatibility

| OS | Support | Notes |
|----|---------|-------|
| **Ubuntu 22.04+** | ✅ Full | Primary development OS |
| **macOS 13+** | ✅ Full | Apple Silicon + Intel |
| **Windows 11** | ⚠️ Partial | WSL2 recommended |
| **Docker** | ✅ Full | Cross-platform |

---

## 9. Installation Checklist

**Complete setup from scratch (Ubuntu 22.04):**

```bash
# 1. System dependencies
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev \
  python3.11 python3.11-dev python3.11-venv \
  sqlite3 libsqlite3-dev \
  git curl

# 2. Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup update stable

# 3. Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# 4. Clone repository
cd /home/user1/pro/lib/willbe/module/iron_cage

# 5. Rust runtime
cd runtime
cargo build --release

# 6. Python demo agent
cd ../pilot/demo/agent
python3.11 -m venv .venv
source .venv/bin/activate
uv pip install -r requirements.txt

# 7. Vue control panel
cd ../control panel
npm install

# 8. Verify installation
cd /home/user1/pro/lib/willbe/module/iron_cage
./runtime/target/release/iron_cage_runtime --version
```

**Estimated setup time:** 30-45 minutes (including compile times)

---

## 10. Technology Decision Rationale

### Why Rust?
- ✅ Performance (near-C++ speed, <1ms LLM call overhead)
- ✅ Memory safety (no segfaults, no data races)
- ✅ PyO3 maturity (production-ready Python FFI)
- ✅ Async ecosystem (Tokio is industry standard)

### Why PyO3 over alternatives?
- ✅ Zero-copy data sharing (vs subprocess communication)
- ✅ Native async support (pyo3-asyncio)
- ❌ Alternative: gRPC (higher latency, more complex)
- ❌ Alternative: Subprocess (cannot intercept LLM calls)

### Why Vue over alternatives?
- ✅ Modern composition API (excellent TypeScript support)
- ✅ shadcn-vue component library (production-ready UI components)
- ✅ Tailwind CSS integration (utility-first styling)
- ✅ Smaller bundle size than React (better performance)
- ❌ Alternative: React (larger bundle, more boilerplate)
- ❌ Alternative: Svelte (smaller ecosystem, fewer component libraries)

### Why SQLite over PostgreSQL?
- ✅ Zero-config (no server setup)
- ✅ Pilot scope (single-instance, low volume)
- ✅ Fast reads/writes for audit logs
- ❌ PostgreSQL: Overkill for pilot, adds deployment complexity

### Why Axum over Actix-web?
- ✅ Built on Tower (composable middleware)
- ✅ Better async ergonomics
- ✅ Excellent WebSocket support
- ❌ Actix: More complex, macro-heavy

---

## 11. Technology Upgrade Path

**After pilot (if scaling to production):**

| Component | Pilot | Production |
|-----------|-------|------------|
| **Database** | SQLite | PostgreSQL 15+ |
| **Cache** | In-memory (DashMap) | Redis Cluster |
| **Deployment** | Single binary | Kubernetes |
| **Observability** | tracing logs | Prometheus + Grafana |
| **Authentication** | None | OAuth2 + JWT |
| **Load balancing** | None | Nginx/Traefik |

---

## Common Issues & Solutions

### Rust compile times too slow
```bash
# Use mold linker (5x faster linking)
cargo install mold
echo '[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["-C", "link-arg=-fuse-ld=mold"]' >> ~/.cargo/config.toml
```

### PyO3 import errors
```bash
# Ensure Python version matches
python --version  # Must be 3.11+
rustc --print cfg | grep python  # Check PyO3 Python version
```

### Control Panel WebSocket connection fails
```bash
# Check CORS configuration in Axum
# Ensure control panel origin is allowed
# Default: http://localhost:5173 (Vite dev server)
```

### SQLite locked errors
```bash
# Increase timeout in sqlx
sqlx::sqlite::SqliteConnectOptions::new()
    .busy_timeout(Duration::from_secs(5))
```

---

**Next Steps:**
1. Read `crates.md` for Rust crate details (in this directory)
2. Read `/runtime/PILOT_GUIDE.md` for implementation guide
3. Read `spec.md` for feature specifications (in this directory)
4. Follow installation checklist above

**Questions:** See `readme.md` for project overview (in this directory)
