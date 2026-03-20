# Task 022: Provider failover

## Dependencies
- None

## Context
Circuit breaker stops retrying a failed provider but does not automatically route to an alternative. Agents with multiple assigned providers need automatic failover.

Critical areas:
- `module/iron_runtime/src/llm_router/router.rs`
- `module/iron_runtime/src/llm_router/circuit_breaker.rs`
- `module/iron_runtime_analytics/src/`

## Implementation plan
1. Add failover configuration: priority ordering of providers per agent.
2. On circuit breaker open, route to next provider in priority list.
3. Track which provider served each request (for analytics).
4. Add fallback statistics to analytics.
5. Configurable retry with backoff before failover.

## Acceptance criteria
- When primary provider circuit opens, requests route to secondary.
- Analytics record which provider served each request.
- Failover is transparent to the calling agent.
- No failover if no secondary provider configured (existing behavior preserved).
