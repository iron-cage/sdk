# iron_cage_types

Shared types and traits for the Iron Cage multi-agent safety framework.

[![Documentation](https://img.shields.io/badge/docs-ironcage.ai-E5E7EB.svg)](https://ironcage.ai/docs)

## Installation

```toml
[dependencies]
iron_cage_types = { version = "0.1", features = ["full"] }
```


## Features

- `enabled` (default): Core functionality with all required dependencies
- `full`: All functionality (currently same as `enabled`)


## Quick Start

```rust
use iron_cage_types::{Config, SafetyConfig, CostConfig};

// Create configuration for multi-agent system
let config = Config {
    safety: SafetyConfig {
        pii_detection_enabled: true,
        audit_log_path: Some("/var/log/safety.log".into()),
    },
    cost: CostConfig {
        budget_usd: 100.0,
        alert_threshold: 0.8,
    },
    reliability: Default::default(),
};

// Configuration is serializable when `enabled` feature is active
let json = serde_json::to_string(&config)?;
```


## Documentation

- [API Reference](https://docs.rs/iron_cage_types)
- [Architecture Guide](docs/architecture.md)


<details>
<summary>Scope & Boundaries</summary>

**Responsibilities:**
Defines shared types, traits, and error types used across all iron_cage crates as foundation layer. Provides type-safe builders for configuration, comprehensive error handling with error_tools integration, and Serde serialization support. Requires Rust 1.75+, all platforms supported, serves as dependency for all other workspace crates.

**In Scope:**
- Common configuration types (SafetyConfig, CostConfig, ReliabilityConfig)
- Shared error types and Result aliases
- Core trait definitions (Agent, SafetyGuard, CostTracker, CircuitBreaker)
- Type-safe builders for configuration
- Serde serialization support (behind `enabled` feature)

**Out of Scope:**
- Implementation logic (see iron_cage_safety, iron_cage_cost, iron_cage_reliability)
- CLI interface (see iron_cage_cli)
- Python bindings (see iron_cage_cli)
- Business logic or orchestration

</details>


<details>
<summary>Directory Structure</summary>

### Source Files

| File | Responsibility |
|------|----------------|
| lib.rs | Foundational types and type-safe identifiers for Iron Runtime |
| ids.rs | Type-safe entity identifiers with validation |

**Notes:**
- Entries marked 'TBD' require manual documentation
- Entries marked '⚠️ ANTI-PATTERN' should be renamed to specific responsibilities

</details>


## License

MIT
