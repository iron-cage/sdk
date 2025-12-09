# Iron SDK Examples

Example library for Iron Cage SDK with LangChain, CrewAI, and AutoGPT integrations.

**Note:** These examples are part of the iron_sdk module (located in `module/iron_sdk/examples/`)

---

## Directory Responsibilities

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| **patterns/** | Demonstrate Iron Cage protection patterns | Pattern question → Pattern examples | Circuit breaker, cost optimization, PII handling, rate limiting (4 examples) | NOT framework-specific (→ langchain/, crewai/, autogpt/), NOT raw API (→ raw_api/), NOT testing utils (→ testing/) |
| **langchain/** | Show LangChain framework integration | LangChain use case → LangChain examples | Simple chain, memory chain, agent with tools, multi-agent, streaming (5 examples) | NOT CrewAI (→ crewai/), NOT AutoGPT (→ autogpt/), NOT raw API (→ raw_api/) |
| **crewai/** | Illustrate CrewAI framework integration | CrewAI use case → CrewAI examples | Simple crew, research crew, multi-stage crew (3 examples) | NOT LangChain (→ langchain/), NOT AutoGPT (→ autogpt/), NOT patterns (→ patterns/) |
| **autogpt/** | Demonstrate AutoGPT framework integration | AutoGPT use case → AutoGPT examples | Autonomous agent examples (2 examples, below table threshold) | NOT LangChain (→ langchain/), NOT CrewAI (→ crewai/), NOT testing (→ testing/) |
| **raw_api/** | Show direct provider API usage | Raw API need → Provider examples | OpenAI direct, Anthropic direct, multi-provider fallback (3 examples) | NOT framework wrappers (→ langchain/, crewai/, autogpt/), NOT patterns (→ patterns/), NOT testing (→ testing/) |
| **testing/** | Provide testing and simulation utilities | Testing need → Testing examples | Integration tests, budget simulation, mock runtime (3 examples) | NOT production examples (→ langchain/, crewai/), NOT patterns (→ patterns/), NOT raw API (→ raw_api/) |

---

### Scope

**Responsibilities:**
Provides production-ready example code demonstrating Iron Cage protection patterns with popular AI agent frameworks. Each example is self-contained, runnable, and demonstrates specific protection features like budget tracking, PII detection, and circuit breakers.

**In Scope:**
- LangChain examples (chat agents, RAG pipelines, multi-step agents)
- CrewAI examples (protected crews, multi-agent collaboration)
- AutoGPT examples (autonomous agents, plugin integration)
- Pattern examples (budget enforcement, PII redaction, circuit breakers)
- Runnable example scripts with documentation
- Framework-specific best practices

**Out of Scope:**
- SDK implementation (see iron_sdk)
- Testing utilities (see iron_testing)
- CLI tools (see iron_cli, iron_cli_py)
- Production deployment guides (see iron_api documentation)
- Framework source code modifications

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
# Install SDK with examples and framework dependencies
pip install iron-sdk[examples,langchain]

# Run an example (from module/iron_sdk/ directory)
python examples/langchain/simple_chat.py
```

---

## Installation

```bash
# SDK with examples
pip install iron-sdk[examples]

# With specific framework
pip install iron-sdk[examples,langchain]
pip install iron-sdk[examples,crewai]
pip install iron-sdk[examples,autogpt]

# All frameworks
pip install iron-sdk[examples,all]
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

# Run example (from module/iron_sdk/ directory)
python examples/langchain/simple_chat.py

# View example source
cat examples/langchain/simple_chat.py
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
