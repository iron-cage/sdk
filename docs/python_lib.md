# Iron Cage Python Library (PROPOSED)

> **PROPOSED API** - This document describes a planned future API design.
> The API described here (`iron_cage.SafetyRuntime`) does not yet exist.
> For the current implementation, see [python_lib_current.md](python_lib_current.md).

```
pip install iron-cage
```

---

## Usage

```python
from iron_cage import SafetyRuntime

runtime = SafetyRuntime()
runtime.run(agent)
```

That's it. No configuration in code.

---

## Setup

1. Get IC Key from admin (Dashboard → Project → IC Keys → Generate)
2. Set environment variables:

```bash
export IC_KEY="ic_a1b2c3d4..."
export IC_SERVER="http://your-server:3000"
```

3. Use in code:

```python
from iron_cage import SafetyRuntime

runtime = SafetyRuntime()
runtime.run(agent)
```

---

## Examples

### LangChain

```python
from langchain.agents import create_openai_agent
from iron_cage import SafetyRuntime

agent = create_openai_agent(llm, tools, prompt)

runtime = SafetyRuntime()
runtime.run(agent, input="Analyze this data...")
```

### LlamaIndex

```python
from llama_index import VectorStoreIndex
from iron_cage import SafetyRuntime

index = VectorStoreIndex.from_documents(documents)
query_engine = index.as_query_engine()

runtime = SafetyRuntime()
runtime.run(query_engine, query="What is...")
```

### Context Manager

```python
from iron_cage import SafetyRuntime

with SafetyRuntime():
    from openai import OpenAI
    client = OpenAI()  # Key auto-injected
    client.chat.completions.create(...)
```

### Direct OpenAI

```python
from iron_cage import SafetyRuntime

runtime = SafetyRuntime()

with runtime:
    import openai
    response = openai.chat.completions.create(
        model="gpt-4",
        messages=[{"role": "user", "content": "Hello"}]
    )
```

---

## What Happens

```
runtime.run(agent)
       │
       ├──► POST /lib/keys      → Get OpenAI/Claude keys + settings
       │                          (budget, PII rules from Dashboard)
       │
       ├──► Inject keys         → os.environ (RAM only)
       │
       ├──► Apply settings      → Budget limit, PII filter (from server)
       │
       ├──► Execute agent       → Run actual LLM calls
       │
       ├──► POST /lib/usage     → Report tokens & cost
       │
       └──► Cleanup             → Remove keys from RAM
```

---

## All Settings From Dashboard

| Setting | Where | Library |
|---------|-------|---------|
| Budget | Dashboard → Project | Enforced |
| PII Detection | Dashboard → Project | Enforced |
| Rate Limits | Dashboard → Project | Enforced |
| Provider Keys | Dashboard → Vault | Injected |

**Developer controls nothing. Admin controls everything.**

---

## Exceptions

```python
from iron_cage import (
    SafetyRuntime,
    BudgetExceededError,
    InvalidKeyError,
    KeyRevokedError,
    ConnectionError,
)

try:
    runtime = SafetyRuntime()
    runtime.run(agent)
except BudgetExceededError as e:
    print(f"Budget exceeded: ${e.spent_usd:.2f} / ${e.budget_usd:.2f}")
except KeyRevokedError:
    print("Key revoked by admin")
except InvalidKeyError:
    print("Invalid IC key")
except ConnectionError:
    print("Cannot reach server")
```

---

## Compatibility

| Framework | Status |
|-----------|--------|
| LangChain | ✅ |
| LlamaIndex | ✅ |
| OpenAI SDK | ✅ |
| Anthropic SDK | ✅ |
| CrewAI | ✅ |
| AutoGen | ✅ |

---

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `IC_KEY` | Yes | IC key from Dashboard |
| `IC_SERVER` | Yes | Iron Cage server URL |

```bash
# .env
IC_KEY=ic_a1b2c3d4e5f6...
IC_SERVER=http://localhost:3000
```
