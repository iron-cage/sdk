# Iron Examples Specification

**Version:** 0.2.0
**Status:** Initial scaffolding
**Layer:** 6 (Application)
**Date:** 2025-12-07

---

## Scope

**Responsibility:**
Provides runnable example library demonstrating Iron Cage protection patterns with popular AI agent frameworks (LangChain, CrewAI, AutoGPT). Contains 20+ production-ready examples covering budget tracking, PII detection, circuit breakers, sandboxing, and multi-agent orchestration. Serves as reference implementations and starting templates for developers building protected agents.

**In Scope:**
- LangChain examples (10+ examples)
  - Simple chat agent with budget
  - RAG pipeline with PII detection
  - Multi-step agent with circuit breakers
  - Async agent with concurrent calls
  - Tool-using agent with sandboxing
- CrewAI examples (5+ examples)
  - Protected crew with shared budget
  - Multi-agent collaboration with cost tracking
  - Task delegation with failure recovery
- AutoGPT examples (5+ examples)
  - Protected autonomous agent
  - Plugin integration with Iron Cage
  - Command execution with sandboxing
- Pattern examples
  - Budget enforcement patterns
  - PII redaction patterns
  - Circuit breaker patterns
  - Sandbox isolation patterns
- Runnable scripts with clear output
- Detailed docstrings explaining each example

**Out of Scope:**
- Framework implementations (use iron_sdk)
- Testing utilities (use iron_testing)
- Production application code
- Custom agent business logic
- Framework documentation (link to official docs)

## Deployment Context

This module operates identically in both Pilot and Production modes as example code for developers.

**See:** [docs/deployment_packages.md](../../docs/deployment_packages.md) § Deployment Modes for deployment architecture.

**This Module (iron_examples):**
- Example code for developers learning Iron Cage
- Not deployed in any package (development/documentation only)
- Examples demonstrate Agent Runtime usage patterns

---

## Dependencies

**Required:**
- iron-sdk >=0.1.0 (Pythonic SDK layer)
- Python 3.8+

**Optional (per example type):**
- langchain >=0.1.0, langchain-openai >=0.1.0 (for LangChain examples)
- crewai >=0.1.0 (for CrewAI examples)
- autogpt >=0.1.0 (for AutoGPT examples)

---

## Example Structure

### Module Organization

```
iron_examples/
├── __init__.py                 # Package exports
├── langchain/                  # LangChain examples (10+)
│   ├── __init__.py
│   ├── simple_chat.py         # Basic chat with budget
│   ├── rag_pipeline.py        # RAG with PII detection
│   ├── multi_step_agent.py    # Complex agent with circuit breakers
│   ├── async_agent.py         # Async agent with concurrent calls
│   ├── tool_agent.py          # Tool-using agent with sandbox
│   └── ...                    # 5+ more examples
├── crewai/                    # CrewAI examples (5+)
│   ├── __init__.py
│   ├── simple_crew.py         # Basic crew with budget
│   ├── multi_agent.py         # Multi-agent collaboration
│   ├── task_delegation.py     # Task delegation with recovery
│   └── ...                    # 2+ more examples
├── autogpt/                   # AutoGPT examples (5+)
│   ├── __init__.py
│   ├── autonomous_agent.py    # Protected autonomous agent
│   ├── plugin_integration.py  # Plugin with Iron Cage
│   ├── command_sandbox.py     # Command execution with sandbox
│   └── ...                    # 2+ more examples
└── patterns/                  # Pattern examples
    ├── __init__.py
    ├── budget_patterns.py     # Budget enforcement patterns
    ├── pii_patterns.py        # PII redaction patterns
    ├── circuit_breaker_patterns.py
    └── sandbox_patterns.py    # Sandbox isolation patterns
```

---

## Example Template

Each example follows this structure:

```python
"""
Example: [Title]

Description: [1-2 sentences explaining what this demonstrates]

Requirements:
- API keys: [List required API keys]
- Dependencies: [List required packages]

Usage:
    python -m iron_examples.langchain.simple_chat

Expected output:
    [Brief description of what user should see]
"""

from iron_sdk import protect_agent, BudgetConfig
import os

# Configuration
API_KEY = os.getenv("OPENAI_API_KEY")
BUDGET_USD = 1.0

@protect_agent(budget=BudgetConfig(max_usd=BUDGET_USD))
def main():
  """Example implementation with clear comments."""
  # Step 1: Setup
  # Step 2: Execute
  # Step 3: Display results
  pass

if __name__ == "__main__":
  main()
```

---

## Development Status

**Current Phase:** Initial scaffolding (v0.1.0)

**Completed:**
- ✅ Project structure created
- ✅ pyproject.toml configured with optional dependencies
- ✅ Package scaffolding (framework subdirectories)

**Pending:**
- 📋 LangChain examples (10+ examples)
  - Simple chat agent
  - RAG pipeline
  - Multi-step agent
  - Async agent
  - Tool-using agent
  - 5+ additional patterns
- 📋 CrewAI examples (5+ examples)
  - Simple crew
  - Multi-agent collaboration
  - Task delegation
  - 2+ additional patterns
- 📋 AutoGPT examples (5+ examples)
  - Autonomous agent
  - Plugin integration
  - Command sandboxing
  - 2+ additional patterns
- 📋 Pattern examples
  - Budget enforcement
  - PII redaction
  - Circuit breakers
  - Sandbox isolation
- 📋 README with example index
- 📋 API key management guide
- 📋 Troubleshooting guide

---

## Non-Functional Requirements

### NFR1: Clarity
- **NFR1.1:** Each example must be self-contained and runnable
- **NFR1.2:** Code must have clear comments explaining each step
- **NFR1.3:** Output must demonstrate protection features working

### NFR2: Coverage
- **NFR2.1:** At least 10 LangChain examples covering common patterns
- **NFR2.2:** At least 5 CrewAI examples
- **NFR2.3:** At least 5 AutoGPT examples
- **NFR2.4:** All protection features must be demonstrated (budget, PII, circuit breakers, sandbox)

### NFR3: Maintenance
- **NFR3.1:** Examples must work with latest framework versions
- **NFR3.2:** Examples must be tested in CI/CD
- **NFR3.3:** Broken examples must be fixed or removed within 1 week

---

## Revision History

- **2025-12-07 (v0.2.0):** Added Deployment Context - clarify development/documentation-only module
- **2025-12-07 (v0.1.0):** Initial scaffolding specification

**Next Milestone:** Implement 3 LangChain examples (simple chat, RAG, multi-step)
