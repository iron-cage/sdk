# Getting Started with Iron Cage

Choose your path based on what you want to do:

---

## I Want to Protect My Python AI Agents

**You are:** Python developer building AI agents
**You need:** Iron SDK
**Time to start:** 2 minutes

### Step 1: Install

**Prerequisites:**
- Python 3.9+ (`python --version`)

```bash
pip install iron-sdk
```

### Step 2: Get IC Token

Contact your admin to receive an IC Token (budget credential).

### Step 3: Protect Your Agent

```python
import os
from iron_sdk import protect_agent, BudgetConfig, SafetyConfig

@protect_agent(
  ic_token=os.getenv("IC_TOKEN"),
  budget=BudgetConfig(max_usd=50.0),
  safety=SafetyConfig(pii_detection=True)
)
def my_agent(prompt: str) -> str:
  return llm.chat(prompt)  # Your existing agent code unchanged
```

### Step 4: Run

```bash
export IC_TOKEN="your-token-here"
python my_agent.py
```

**That's it!** No Rust, no cargo, no building required.

**Next:**
- [SDK Documentation](../module/iron_sdk/readme.md)
- [Examples](../module/iron_sdk/examples/readme.md)
- [Troubleshooting](#troubleshooting)

---

## I Want to Deploy Control Panel for My Team

**You are:** Platform admin / DevOps engineer
**You need:** Control Panel server
**Time to start:** 2 minutes (dev mode) or 5 minutes (production)

### Quick Start: Development Mode (Recommended for Testing)

**Best for:** Testing, debugging, or first-time exploration with automatic database reset.

```bash
# Clone repository
git clone https://github.com/iron-cage/iron_runtime.git
cd iron_runtime/dev

# Start development mode (single command, no configuration needed)
docker compose -f docker-compose.dev.yml up

# Access:
# - Frontend: http://localhost:5173
# - Backend: http://localhost:3000/api/health
```

**What happens:**
- Database automatically wiped on every restart (fresh state)
- Hot reload enabled (edit code, see changes instantly)
- No .env file required (development secrets hardcoded)
- First startup: 2-5 min (compiles from source)

**⚠️ WARNING:** Development mode **deletes all data on startup**. Use only for testing.

---

### Production Deployment (Data Persistence)

**Best for:** Production deployment where data must persist across restarts.

#### Step 1: Clone Repository

```bash
git clone https://github.com/iron-cage/iron_runtime.git
cd iron_runtime/dev
```

#### Step 2: Configure Secrets

```bash
# Copy environment template
cp .env.example .env

# Generate secrets (required for production)
openssl rand -hex 32    # Copy output for JWT_SECRET
openssl rand -base64 32 # Copy output for IRON_SECRETS_MASTER_KEY

# Edit .env and paste the generated secrets
nano .env
# Or use your preferred editor (vim, code, etc.)
```

#### Step 3: Start Services

```bash
# Start all services (Backend + Frontend with SQLite database)
docker compose up -d

# View logs to verify startup
docker compose logs -f

# Check service status (all should show "healthy")
docker compose ps
```

#### Step 4: Access Dashboard

Open http://localhost:8080

Default credentials are created on first startup (check logs for details).

**Next:**
- [Full Deployment Guide](deployment_guide.md) - Production deployment, troubleshooting, security
- [Docker Compose Architecture](deployment/006_docker_compose_deployment.md) - Design details and trade-offs
- [User Management](features/006_user_management.md) - Creating users and managing roles
- [Control Panel API](protocol/readme.md) - API documentation

---

## I Want to Contribute to Iron Cage Platform

**You are:** Platform contributor / open source developer
**You need:** Full source build
**Time to start:** 15 minutes

### Step 1: Clone Repository

```bash
git clone https://github.com/iron-cage/iron_runtime.git
cd iron_runtime
```

### Step 2: Install Prerequisites

- Rust 1.75+ (`rustup update`)
- Python 3.9+ (`python --version`)
- uv package manager (`curl -LsSf https://astral.sh/uv/install.sh | sh`)
- Node.js 18+ (`node --version`)

### Step 3: Build All Modules

```bash
cargo build --release --workspace
```

### Step 4: Run Tests

```bash
cargo nextest run --all-features
```

### Step 5: Setup Python Modules for Development

```bash
cd module/iron_sdk
uv sync  # Installs dependencies and creates .venv
```

**Next:**
- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [Architecture Docs](architecture/readme.md)

---

## Troubleshooting

### "ModuleNotFoundError: No module named 'iron_sdk'"

**Solution:** Install iron-sdk:
```bash
pip install iron-sdk
```

### "Should I install iron-cage?"

**Answer:** No! iron-cage is automatically installed when you `pip install iron-sdk`. You never interact with it directly.

### "Do I need Rust installed?"

**Answer:** No! Only Python. The Rust runtime is pre-compiled in the iron-cage package.

### "I see cargo build commands in the docs"

**Answer:** Those are for platform contributors developing Iron Cage itself. If you're using Iron Cage (not developing it), ignore cargo entirely.

### "Budget exceeded errors"

**Solution:** Contact your admin to increase your agent's budget allocation.

### "IC Token invalid"

**Solution:**
1. Verify token hasn't expired
2. Check token format (should be JWT)
3. Regenerate token via Control Panel dashboard or CLI

---

*Last Updated: 2025-12-10*
