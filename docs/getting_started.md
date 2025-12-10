# Getting Started with Iron Cage

Choose your path based on what you want to do:

---

## I Want to Protect My Python AI Agents

**You are:** Python developer building AI agents
**You need:** Iron SDK
**Time to start:** 2 minutes

### Step 1: Install

**Prerequisites:**
- Python 3.8+ (`python --version`)
- uv package manager (`curl -LsSf https://astral.sh/uv/install.sh | sh`)

```bash
uv pip install iron-sdk
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
**Time to start:** 5 minutes

### Step 1: Deploy Control Panel

```bash
docker pull iron-cage/control-panel:latest
docker run -d -p 8080:8080 \
  -e POSTGRES_URL="postgresql://..." \
  iron-cage/control-panel
```

### Step 2: Create Admin Account

```bash
docker exec -it control-panel \
  iron-admin create-user \
    --username admin \
    --role admin \
    --email admin@company.com
```

### Step 3: Access Dashboard

Open http://localhost:8080 and login.

**Next:**
- [Deployment Guide](deployment/readme.md)
- [User Management](features/006_user_management.md)
- [Control Panel API](protocol/readme.md)

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
- Python 3.11+ (`python --version`)
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

### Step 5: Install SDK in Dev Mode

```bash
cd module/iron_sdk
uv pip install -e .
```

**Next:**
- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [Architecture Docs](architecture/readme.md)
- [Module Specifications](../module/)

---

## Troubleshooting

### "ModuleNotFoundError: No module named 'iron_sdk'"

**Solution:** Install iron-sdk:
```bash
uv pip install iron-sdk
```

**Note:** Install uv first: `curl -LsSf https://astral.sh/uv/install.sh | sh`

### "Should I install iron-cage?"

**Answer:** No! iron-cage is automatically installed when you `uv pip install iron-sdk`. You never interact with it directly.

### "Do I need Rust installed?"

**Answer:** No! Only uv and Python. The Rust runtime is pre-compiled in the iron-cage package.

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
