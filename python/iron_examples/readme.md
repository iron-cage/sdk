# Iron Examples

Example library for Iron Cage with LangChain, CrewAI, and AutoGPT integrations.

**Status:** Initial scaffolding (v0.1.0)
**Layer:** 6 (Application)
**License:** Apache-2.0

---

## Overview

Iron Examples provides 20+ production-ready examples demonstrating Iron Cage protection patterns with popular AI agent frameworks. Each example is self-contained, runnable, and demonstrates specific protection features.

**Example Categories:**
- **LangChain** (10+ examples) - Chat agents, RAG pipelines, multi-step agents, async agents
- **CrewAI** (5+ examples) - Protected crews, multi-agent collaboration, task delegation
- **AutoGPT** (5+ examples) - Autonomous agents, plugin integration, command sandboxing
- **Patterns** - Budget enforcement, PII redaction, circuit breakers, sandbox isolation

---

## Quick Start

```bash
# Install with framework dependencies
pip install iron-examples[langchain]

# Run an example
python -m iron_examples.langchain.simple_chat
```

---

## Installation

```bash
# Base package
pip install iron-examples

# With specific framework
pip install iron-examples[langchain]
pip install iron-examples[crewai]
pip install iron-examples[autogpt]

# All frameworks
pip install iron-examples[all]
```

**Requirements:**
- Python 3.8+
- iron-sdk >=0.1.0 (automatically installed)

---

## Available Examples

### LangChain Examples

- `simple_chat.py` - Basic chat agent with budget tracking
- `rag_pipeline.py` - RAG pipeline with PII detection
- `multi_step_agent.py` - Complex agent with circuit breakers
- `async_agent.py` - Async agent with concurrent LLM calls
- `tool_agent.py` - Tool-using agent with sandboxing
- More examples coming soon...

### CrewAI Examples

- `simple_crew.py` - Basic crew with shared budget
- `multi_agent.py` - Multi-agent collaboration with cost tracking
- `task_delegation.py` - Task delegation with failure recovery
- More examples coming soon...

### AutoGPT Examples

- `autonomous_agent.py` - Protected autonomous agent
- `plugin_integration.py` - Plugin system with Iron Cage
- `command_sandbox.py` - Command execution with sandboxing
- More examples coming soon...

---

## Running Examples

```bash
# Set API keys
export OPENAI_API_KEY=sk-...

# Run example
python -m iron_examples.langchain.simple_chat

# View example source
python -m iron_examples.langchain.simple_chat --help
```

---

## Documentation

- **Specification:** See `spec.md` for complete requirements
- **Example Index:** Coming soon
- **API Key Setup:** See examples for required environment variables

---

## Development Status

**Current Phase:** Initial scaffolding

**Pending Implementation:**
- LangChain examples (10+)
- CrewAI examples (5+)
- AutoGPT examples (5+)
- Pattern examples
- Example index documentation

---

## License

Apache-2.0 - See `license` file for details
