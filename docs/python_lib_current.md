# Iron Cage Python Library (Current)

## Installation

```bash
pip install iron-cage
```

Requires compiled Rust extension (PyO3).

---

## API

| Method | Parameters | Returns |
|--------|------------|---------|
| `Runtime()` | `budget: float`, `verbose: bool` | Runtime instance |
| `start_agent()` | `path: str` | `agent_id: str` |
| `get_metrics()` | `agent_id: str` | JSON string |
| `stop_agent()` | `agent_id: str` | None |

---

## Usage

```python
from iron_runtime import Runtime

runtime = Runtime(budget=50.0, verbose=True)
agent_id = runtime.start_agent("/path/to/agent.py")
metrics = runtime.get_metrics(agent_id)
runtime.stop_agent(agent_id)
```

---

## Metrics Response

```json
{
  "agent_id": "agent-abc-123",
  "status": "Running",
  "budget_spent": 0.0,
  "pii_detections": 0
}
```

---

## Flow

```
Python                     Rust (PyO3)
   │                          │
   │  Runtime(budget=50)      │
   │─────────────────────────▶│ Create AgentRuntime
   │                          │ Create StateManager
   │                          │
   │  start_agent(path)       │
   │─────────────────────────▶│ Generate UUID
   │                          │ Save AgentState
   │  ◀─────────────────────  │
   │  "agent-{uuid}"          │
   │                          │
   │  get_metrics(agent_id)   │
   │─────────────────────────▶│ Query StateManager
   │  ◀─────────────────────  │
   │  JSON string             │
```

---

## Rust Modules Used

| Module | Purpose |
|--------|---------|
| `iron_state` | Agent state storage |
| `iron_telemetry` | Event logging |
| `iron_cost` | Budget tracking |
| `iron_safety` | PII detection |