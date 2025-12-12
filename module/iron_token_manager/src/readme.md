# Directory: src

## Responsibility Table

| File | Responsibility |
|------|----------------|
| agent_budget.rs | Manages per-agent budget allocations |
| budget_request.rs | Stores budget change requests and history |
| config.rs | Loads configuration from TOML files |
| cost_calculator.rs | Converts token usage to USD costs |
| error.rs | Defines token management error types |
| lease_manager.rs | Manages budget leases for agent sessions |
| lib.rs | Exports token manager module public API |
| limit_enforcer.rs | Enforces usage limits and quotas |
| migrations.rs | Applies database migrations with guards |
| provider_adapter.rs | Wraps LLM provider clients with tracking |
| provider_key_storage.rs | Stores encrypted AI provider API keys |
| rate_limiter.rs | Implements token bucket rate limiting |
| seed.rs | Seeds database with sample development data |
| storage.rs | Persists tokens and metadata to database |
| token_generator.rs | Generates cryptographically secure API tokens |
| trace_storage.rs | Stores API call traces for analysis |
| usage_tracker.rs | Tracks LLM API usage per user |
| user_service.rs | Manages user lifecycle and administration |

## Validation

One-Second Test: Scan the Responsibility column - if any two entries sound similar, they overlap and must be reconsolidated.
