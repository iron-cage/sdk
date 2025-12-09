# langchain/

LangChain framework examples with Iron Cage protection.

---

## Responsibility Table

| Entity | Responsibility | Input ’ Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| `simple_chain.py` | Demonstrate basic LangChain agent with budget tracking | Simple use case ’ Protected chain example | Basic chain setup, @protect_agent decorator, budget context, single LLM call | NOT multi-agent (’ multi_agent.py), NOT complex workflows (’ agent_with_tools.py), NOT streaming (’ streaming_response.py) |
| `memory_chain.py` | Show conversation memory with safety guardrails | Memory requirement ’ Memory chain example | Conversation history, memory management, PII detection in history, context window limits | NOT simple chains (’ simple_chain.py), NOT tool usage (’ agent_with_tools.py), NOT streaming (’ streaming_response.py) |
| `agent_with_tools.py` | Illustrate tool-using agent with sandboxing | Tool usage need ’ Protected agent example | Tool integration, parameter validation, sandbox execution, tool authorization | NOT simple chains (’ simple_chain.py), NOT memory (’ memory_chain.py), NOT multi-agent (’ multi_agent.py) |
| `multi_agent.py` | Demonstrate multi-agent collaboration with shared budget | Multi-agent scenario ’ Collaboration example | Multiple agents, shared budget pool, cost attribution, agent coordination | NOT simple chains (’ simple_chain.py), NOT tools (’ agent_with_tools.py), NOT streaming (’ streaming_response.py) |
| `streaming_response.py` | Show streaming LLM responses with real-time cost tracking | Streaming need ’ Streaming example | Streaming API, real-time token counting, progressive cost updates, async iteration | NOT simple chains (’ simple_chain.py), NOT memory (’ memory_chain.py), NOT tools (’ agent_with_tools.py) |

---

## Running Examples

```bash
# Set API key
export OPENAI_API_KEY=sk-...

# Run simple chain example
python simple_chain.py

# Run memory chain example
python memory_chain.py

# Run agent with tools example
python agent_with_tools.py

# Run multi-agent example
python multi_agent.py

# Run streaming response example
python streaming_response.py
```

---

**Status:** Scaffolding (files created, implementation pending)
