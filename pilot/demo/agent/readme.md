# Python Demo Agent

**Purpose:** LangChain-based lead generation agent for Warsaw conference demo

**Last Updated:** 2025-11-24

### Scope

**Responsibility:** Python agent implementation for Warsaw demo (processes 100 leads to trigger safety/cost events).

**In Scope:**
- Agent code (lead_gen_agent.py, Python 3.11 + LangChain)
- Dependencies (requirements.txt)
- Test data (100 synthetic leads)

**Out of Scope:**
- Production agent implementation
- Rust runtime (see `/runtime/`)
- Vue control panel (see `../control panel/`)

---

## Directory Contents & Responsibilities

| File/Directory | Responsibility | In Scope | Out of Scope (See) |
|----------------|----------------|----------|-------------------|
| **lead_gen_agent.py** | Main agent implementation | LangChain agent, tool definitions, lead processing logic | Rust runtime (→ /runtime/), Control Panel (→ ../control panel/) |
| **requirements.txt** | Python dependencies | LangChain, OpenAI SDK, requests, pydantic | Rust crates (→ /pilot/crates.md), npm packages (→ ../control panel/package.json) |
| **test_data/** | Synthetic lead data | 100 leads CSV, test data documentation | Real customer data (NOT included in demo) |
| **.venv/** | Python virtual environment | Isolated Python dependencies | System-wide Python packages |

---

## Implementation Status

**As of 2025-11-24:**
- ❌ lead_gen_agent.py: NOT IMPLEMENTED
- ❌ requirements.txt: NOT CREATED
- ❌ test_data/leads.csv: NOT CREATED

**Priority:** Low (slides-only approach recommended for 23-day timeline)

---

## Specification

**See:** `/pilot/spec.md` lines 300-350 (Features #16-18)

### Feature #16: Demo Agent Implementation

**Requirements:**
- LangChain agent with 2 tools (LinkedIn search, Clearbit enrich)
- Process 100 leads from test_data/leads.csv
- Make ~200-300 LLM calls (GPT-4 for leads #1-50, GPT-3.5 for #51-100)
- Total cost: ~$30-40 for full run

### Feature #17: Demo Data & Triggers

**Lead #67 trigger:** Contact info contains CEO email `ceo@acme.com` → PII detected
**Lead #85 trigger:** Budget reaches $45/$50 (90% threshold) → Budget warning alert

### Feature #18: Demo Monitoring Hooks

**Hooks for runtime interception:**
- Before LLM call → runtime intercepts, tracks cost
- After LLM response → runtime scans for PII, redacts if found
- On tool usage → runtime logs tool invocation

---

## Python Dependencies

**When implemented, requirements.txt should include:**

```txt
# Agent framework
langchain==0.1.0
langchain-openai==0.0.2

# LLM APIs
openai==1.0.0
anthropic==0.8.0  # Optional for fallback

# HTTP client
requests==2.31.0
httpx==0.25.0  # Async HTTP

# Data validation
pydantic==2.5.0
pydantic-settings==2.1.0

# Data processing
pandas==2.1.0  # For CSV test data

# Environment variables
python-dotenv==1.0.0
```

**See:** `/pilot/tech_stack.md` lines 116-137 for complete dependency list

---

## Agent Structure (When Implemented)

```python
# lead_gen_agent.py (simplified spec)

from langchain.agents import AgentExecutor, create_openai_functions_agent
from langchain_openai import ChatOpenAI
from langchain.tools import tool
import pandas as pd

# Tool 1: LinkedIn search
@tool
def search_linkedin(company: str) -> dict:
    """Search LinkedIn for company decision makers."""
    # Demo: Return mock data from test_data/leads.csv
    pass

# Tool 2: Clearbit enrichment
@tool
def enrich_clearbit(company: str) -> dict:
    """Enrich company data via Clearbit API."""
    # Demo: Return mock enrichment data
    pass

# Load leads
leads_df = pd.read_csv("test_data/leads.csv")

# Create agent
llm = ChatOpenAI(model="gpt-4", temperature=0)
tools = [search_linkedin, enrich_clearbit]
agent = create_openai_functions_agent(llm, tools, prompt)
executor = AgentExecutor(agent=agent, tools=tools)

# Process leads
for idx, lead in leads_df.iterrows():
    # Lead #67: Trigger privacy protection
    # Lead #85: Trigger budget warning
    result = executor.invoke({
        "company": lead.company,
        "industry": lead.industry
    })
    print(f"Lead {idx+1}: {result}")
```

**Lines of code:** ~300 LOC

---

## Test Data Structure

**test_data/leads.csv format:**

```csv
company,industry,website,trigger
Acme Corp,SaaS,acme.com,none
TechStart Inc,AI/ML,techstart.io,none
...
MegaCorp LLC,Enterprise,megacorp.com,pii_at_67
DataViz Co,Analytics,dataviz.com,budget_at_85
...
```

**100 rows total:**
- Leads #1-66: Normal processing
- Lead #67: Contains PII trigger (CEO email in response)
- Leads #68-84: Normal processing
- Lead #85: Budget reaches 90% threshold
- Leads #86-100: Optional (may halt if budget exceeded)

---

## Usage (When Implemented)

### Standalone Mode (No Runtime)

```bash
cd /home/user1/pro/lib/willbe/module/iron_cage/pilot/demo/agent

# Setup environment
python3.11 -m venv .venv
source .venv/bin/activate
uv pip install -r requirements.txt

# Set API keys
export OPENAI_API_KEY="sk-..."
export CLEARBIT_API_KEY="sk_..."  # Optional

# Run agent
python lead_gen_agent.py
```

### With Rust Runtime (Demo Mode)

```bash
cd /home/user1/pro/lib/willbe/module/iron_cage/runtime

# Runtime intercepts LLM calls automatically
cargo run --release -- ../pilot/demo/agent/lead_gen_agent.py --budget 50
```

---

## Demo Script Integration

**Warsaw conference demo (Slide 18):**

1. **Part 1 (0:00-2:00):** Agent startup
   - Show terminal command
   - Agent initializes, loads 100 leads
   - First 5 leads process successfully

2. **Part 2 (2:00-4:00):** Cost tracking
   - Control Panel shows real-time cost: $5.29 / $50.00
   - GPT-4 vs GPT-3.5 split visible

3. **Part 3 (4:00-6:00):** Lead processing
   - Leads #10-60 flash by quickly
   - Cost increments smoothly

4. **Part 4 (6:00-8:00):** privacy protection (lead #67)
   - Agent finds CEO contact: "ceo@acme.com"
   - Runtime DETECTS → control panel shows red alert
   - Output REDACTED: "Contact [EMAIL_REDACTED]"
   - Agent continues processing

5. **Part 5 (8:00-10:00):** Budget warning (lead #85)
   - Cost reaches $45/$50 (90% threshold)
   - Alert fires: "Budget Warning: 90% reached"
   - Email alert shown in terminal
   - Recommended action: "Approve $25 increase"

6. **Part 6 (10:00-12:00):** Final results
   - 100 leads processed
   - Final cost: $38.47 / $50.00
   - 1 PII incident (email redacted)
   - 0 budget overruns
   - vs Baseline: $64.86 saved (no guardrails)

**See:** `/conferences/warsaw_2025/presentation/talk_slides.md` lines 450-600

---

## Related Documentation

**Specifications:**
- **Agent spec:** `/pilot/spec.md` lines 300-350 (Features #16-18)
- **Technology stack:** `/pilot/tech_stack.md` lines 102-183 (Python section)

**Demo materials:**
- **Conference slides:** `/conferences/warsaw_2025/presentation/talk_slides.md`
- **Control Panel spec:** `/pilot/spec.md` lines 350-432 (Features #19-24)

**Implementation:**
- **Rust runtime:** `/runtime/` (intercepts LLM calls)
- **Control Panel:** `../control panel/` (visualizes metrics)

---

**Last Updated:** 2025-11-24
**Status:** Specification complete, implementation not started
**Priority:** Low (slides-only approach for 23-day timeline)
