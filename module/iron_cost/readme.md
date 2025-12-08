# iron_cage_cost

Budget tracking and cost enforcement for LLM-powered agents.

### Scope

**Responsibilities:**
Tracks and enforces per-agent budget limits for LLM API costs (OpenAI, Anthropic, etc.) preventing runaway spending through real-time cost accumulation and budget exhaustion detection. Provides configurable cost limits per agent with automatic enforcement before API calls. Requires Rust 1.75+, all platforms supported, integrates with iron_runtime for enforcement.

**In Scope:**
- Per-agent cost tracking
- Budget enforcement and alerts
- Token counting for LLM calls
- Real-time spending metrics
- Threshold-based notifications

**Out of Scope:**
- PII detection (see iron_cage_safety)
- Circuit breaker logic (see iron_cage_reliability)
- Agent lifecycle (see iron_cage_cli)
- LLM API integration (see iron_cage_cli)

## Installation

```toml
[dependencies]
iron_cage_cost = { version = "0.1", features = ["full"] }
```

## Features

- `enabled` (default): Full budget tracking functionality
- `full`: All functionality (currently same as `enabled`)

## Example

```rust
use iron_cage_cost::BudgetTracker;

// Initialize tracker with $100 budget
let tracker = BudgetTracker::new(100.0);

// Record agent API call cost
tracker.record_cost("agent_1", 2.50)?;

// Check remaining budget
println!("Remaining: ${:.2}", tracker.remaining());
// Output: "Remaining: $97.50"

// Tracker automatically enforces budget limits
match tracker.record_cost("agent_2", 150.0) {
    Err(e) => println!("Budget exceeded: {}", e),
    Ok(_) => unreachable!(),
}
```

## Documentation

- [API Reference](https://docs.rs/iron_cage_cost)
- [Budget Strategies](docs/strategies.md)

## License

MIT
