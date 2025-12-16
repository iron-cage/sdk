# iron_cage_cost

LLM model pricing data and cost calculation.

[![Documentation](https://img.shields.io/badge/docs-ironcage.ai-blue.svg)](https://ironcage.ai/docs)

## Installation

```toml
[dependencies]
iron_cage_cost = { version = "0.1", features = ["full"] }
```


## Features

- `enabled` (default): Full pricing functionality
- `full`: All functionality (currently same as `enabled`)


## Quick Start

```rust
use iron_cost::pricing::PricingManager;

// Initialize pricing manager with embedded LiteLLM data
let pricing = PricingManager::new().expect("Failed to load pricing");

// Look up model pricing
if let Some(model) = pricing.get("gpt-4-turbo") {
    // Calculate actual cost for token usage
    let cost = model.calculate_cost(1000, 500); // 1000 input, 500 output tokens
    println!("Cost: ${:.6}", cost);

    // Calculate max cost for budget pre-reservation
    let max_cost = model.calculate_max_cost(1000, Some(4096));
    println!("Max possible cost: ${:.6}", max_cost);
}
```


## Documentation

- [API Reference](https://docs.rs/iron_cage_cost)


<details>
<summary>Scope & Boundaries</summary>

**Responsibilities:**
Provides pricing data for various LLM models (OpenAI, Anthropic, Google, etc.) loaded from LiteLLM pricing JSON. Supports cost calculation for budget enforcement and pre-reservation. Requires Rust 1.75+, all platforms supported.

**In Scope:**
- Model pricing lookup
- Cost calculation (input/output tokens)
- Max cost estimation for budget pre-reservation
- Thread-safe concurrent access

**Out of Scope:**
- Budget tracking (application-level concern)
- PII detection (see iron_cage_safety)
- Circuit breaker logic (see iron_cage_reliability)
- Agent lifecycle (see iron_cage_cli)
- LLM API integration (see iron_cage_cli)

</details>


<details>
<summary>Directory Structure</summary>

### Source Files

| File | Responsibility |
|------|----------------|
| lib.rs | LLM cost tracking and budget management for Iron Runtime. |
| budget_client.rs | Budget Client for Protocol 005: Budget Control Protocol |
| budget.rs | Budget control with atomic reservations |
| converter.rs | Currency conversion utilities for microdollar arithmetic. |
| error.rs | Error types for cost management |
| pricing.rs | LLM model pricing management. |
| token_estimation.rs | Helper utilities for cost estimation |

**Notes:**
- Entries marked 'TBD' require manual documentation
- Entries marked '⚠️ ANTI-PATTERN' should be renamed to specific responsibilities

</details>


## License

MIT
