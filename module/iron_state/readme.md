# iron_state

Agent execution state management and audit logging.

### Scope

**Responsibilities:**
Manages agent execution state with fast in-memory access (DashMap) for real-time dashboard updates and persistent SQLite storage for compliance audit trails. Provides thread-safe concurrent access for multiple crates reading and writing state simultaneously.

**In Scope:**
- In-memory state storage (DashMap for concurrent access)
- Agent state CRUD operations (create, read, update, list)
- Audit event logging to SQLite
- SQLite persistence (single-file database)
- Thread-safe state access (Arc-based sharing)
- State change broadcasting for dashboard updates
- Agent lifecycle state tracking (Starting, Running, Stopped, Failed)

**Out of Scope:**
- Redis distributed state (future)
- State replication and consensus (future)
- Historical state queries and time-series (future)
- State migrations and schema versioning (future)
- State backup and recovery automation (future)
- REST API endpoints (see iron_api)
- Dashboard UI (see iron_dashboard)
- WebSocket broadcasting (see iron_api)

## Installation

```toml
[dependencies]
iron_state = { path = "../iron_state" }
```

## Example

```rust
use iron_state::StateManager;
use std::sync::Arc;

// Create shared state manager
let state = Arc::new(StateManager::new("./state.db")?);

// Update agent state (thread-safe)
state.update_agent("agent-001", AgentState::Running)?;

// Retrieve current state
let agent = state.get_agent("agent-001")?;

// Log audit event
state.log_event("agent-001", "PII detected and redacted")?;
```

## License

Apache-2.0
