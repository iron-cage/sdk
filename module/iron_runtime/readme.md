# iron_runtime

Agent orchestration and Python bridge for AI agent execution.

### Scope

**Responsibilities:**
Bridges Python AI agents with Rust-based safety, cost, and reliability infrastructure via PyO3. Manages agent lifecycle (spawn, monitor, shutdown), intercepts LLM calls for policy enforcement, coordinates tokio async runtime, and provides WebSocket server for real-time dashboard updates.

**In Scope:**
- Python-Rust FFI via PyO3 (agent execution bridge)
- Agent lifecycle management (spawn, monitor, shutdown)
- LLM call interception and policy enforcement
- Tokio async runtime coordination
- WebSocket server for dashboard real-time updates
- Configuration management (CLI args to RuntimeConfig)
- Single-agent execution model

**Out of Scope:**
- REST API endpoints (see iron_control_api)
- PII detection logic (see iron_safety)
- Cost calculation (see iron_cost)
- Circuit breaker patterns (see iron_reliability)
- Token management (see iron_token_manager)
- State persistence (see iron_runtime_state)
- Multi-agent orchestration (future)
- Distributed runtime (future)

## Installation

```toml
[dependencies]
iron_runtime = { path = "../iron_runtime" }
```

## Example

```rust
use iron_runtime::Runtime;

// Create runtime with configuration
let runtime = Runtime::builder()
  .budget_usd(50.0)
  .safety_enabled(true)
  .build()?;

// Execute Python agent
runtime.execute("agent.py")?;
```

## License

Apache-2.0
