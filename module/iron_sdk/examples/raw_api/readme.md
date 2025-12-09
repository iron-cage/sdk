# raw_api/

Raw API usage examples without framework wrappers.

---

## Responsibility Table

| Entity | Responsibility | Input ’ Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| `openai_direct.py` | Demonstrate direct OpenAI API integration | OpenAI use case ’ Protected API example | Direct OpenAI API calls, iron_sdk wrapper, budget tracking, request/response validation | NOT Anthropic (’ anthropic_direct.py), NOT multi-provider (’ multi_provider.py), NOT framework examples (’ ../langchain/, ../crewai/) |
| `anthropic_direct.py` | Show direct Anthropic API integration | Anthropic use case ’ Protected API example | Direct Anthropic API calls, iron_sdk wrapper, streaming responses, cost tracking | NOT OpenAI (’ openai_direct.py), NOT multi-provider (’ multi_provider.py), NOT framework examples (’ ../langchain/, ../crewai/) |
| `multi_provider.py` | Illustrate multi-provider fallback pattern | Provider failover need ’ Fallback example | Primary/fallback provider chain, automatic switchover, provider health checking, unified interface | NOT single provider (’ openai_direct.py, anthropic_direct.py), NOT framework wrappers (’ ../langchain/, ../crewai/) |

---

## Running Examples

```bash
# Set API keys
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...

# Run OpenAI direct example
python openai_direct.py

# Run Anthropic direct example
python anthropic_direct.py

# Run multi-provider example
python multi_provider.py
```

---

**Status:** Scaffolding (files created, implementation pending)
