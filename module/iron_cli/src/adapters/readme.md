# Directory: adapters

## Responsibility Table

| File | Responsibility |
|------|----------------|
| auth_adapters.rs | Implements Token Manager API authentication adapters |
| auth.rs | Bridges unilang CLI to authentication handlers |
| error.rs | Defines adapter layer error types |
| health_adapters.rs | Implements Token Manager API health endpoints |
| health.rs | Bridges unilang CLI to health handlers |
| keyring.rs | Provides secure credential storage via system keyring |
| limits_adapters.rs | Implements Token Manager API limit management |
| limits.rs | Bridges unilang CLI to limits handlers |
| mod.rs | Exports adapter layer modules and architecture |
| services.rs | Defines service trait interfaces for async operations |
| token_adapters.rs | Implements Token Manager API token CRUD operations |
| tokens.rs | Bridges unilang CLI to token handlers |
| traces_adapters.rs | Implements Token Manager API trace querying |
| traces.rs | Bridges unilang CLI to traces handlers |
| usage_adapters.rs | Implements Token Manager API usage tracking |
| usage.rs | Bridges unilang CLI to usage handlers |

## Validation

One-Second Test: Scan the Responsibility column - if any two entries sound similar, they overlap and must be reconsolidated.
