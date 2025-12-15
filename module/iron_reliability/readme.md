# iron_cage_reliability

Circuit breakers and fault tolerance for multi-agent systems.

### Scope

**Responsibilities:**
Implements circuit breaker pattern (Closed/Open/HalfOpen states) for agent reliability with automatic failure detection and recovery. Prevents cascading failures by temporarily disabling failing agents while allowing periodic retry attempts. Requires Rust 1.75+, all platforms supported, uses exponential backoff for recovery.

**In Scope:**
- Circuit breaker pattern implementation
- Automatic fallback mechanisms
- Retry logic with exponential backoff
- Service health monitoring
- Failure threshold tracking

**Out of Scope:**
- Cost tracking (see iron_cage_cost)
- PII detection (see iron_cage_safety)
- Agent orchestration (see iron_cage_cli)
- Configuration management (see iron_cage_types)

## Installation

```toml
[dependencies]
iron_cage_reliability = { version = "0.1", features = ["full"] }
```

## Features

- `enabled` (default): Full circuit breaker functionality
- `full`: All functionality (currently same as `enabled`)

## Example

```rust
use iron_cage_reliability::CircuitBreaker;

// Create circuit breaker with 5-failure threshold
let cb = CircuitBreaker::new(5, 60);

// Perform operation with circuit breaker protection
match cb.call("external_api", || {
    call_external_service()
}) {
    Ok(result) => println!("Success: {:?}", result),
    Err(e) if cb.is_open("external_api") => {
        println!("Circuit breaker open, using fallback");
    },
    Err(e) => println!("Call failed: {}", e),
}
```

## Documentation

- [API Reference](https://docs.rs/iron_cage_reliability)
- [Reliability Patterns](docs/patterns.md)

## License

MIT

## Directory Structure

### Source Files

| File | Responsibility |
|------|----------------|
| lib.rs | Circuit breaker pattern for preventing cascading failures. |

**Notes:**
- Entries marked 'TBD' require manual documentation
- Entries marked '⚠️ ANTI-PATTERN' should be renamed to specific responsibilities

