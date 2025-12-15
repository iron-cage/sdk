# crewai/

CrewAI framework examples with Iron Cage protection.

---

## Responsibility Table

| Entity | Responsibility | Input ’ Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| `simple_crew.py` | Demonstrate basic crew with shared budget | Simple crew use case ’ Protected crew example | Crew setup, @protect_agent decorator, shared budget pool, task delegation basics | NOT multi-stage workflows (’ multi_stage_crew.py), NOT research crews (’ research_crew.py), NOT testing patterns (’ ../testing/) |
| `research_crew.py` | Show research crew with data access controls | Research scenario ’ Research crew example | Research tasks, data access policies, web search integration, budget allocation per researcher | NOT simple crews (’ simple_crew.py), NOT multi-stage (’ multi_stage_crew.py), NOT testing (’ ../testing/) |
| `multi_stage_crew.py` | Illustrate complex multi-stage crew workflow | Multi-stage need ’ Workflow example | Sequential stages, stage-specific budgets, intermediate result validation, failure recovery | NOT simple crews (’ simple_crew.py), NOT research crews (’ research_crew.py), NOT testing (’ ../testing/) |

---

## Running Examples

```bash
# Set API key
export OPENAI_API_KEY=sk-...

# Run simple crew example
python simple_crew.py

# Run research crew example
python research_crew.py

# Run multi-stage crew example
python multi_stage_crew.py
```

---

**Status:** Scaffolding (files created, implementation pending)
