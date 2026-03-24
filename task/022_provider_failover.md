# Task 022: Provider failover

## Goal
Extend the LLM router to automatically fail over from a primary provider to secondary providers when the circuit breaker opens, ensuring agents with multiple assigned providers experience minimal disruption. The failover must be transparent to calling agents and produce observable analytics showing which provider served each request.

## Dependencies
- Task 002

## In Scope
- Priority-ordered failover configuration per agent
- Automatic routing to the next provider when the circuit breaker opens
- Request-level tracking of which provider served the request
- Failover statistics in the analytics module
- Configurable retry with exponential backoff before triggering failover

## Out of Scope
- Weighted load balancing across providers (round-robin, least-connections)
- Automatic provider health recovery detection and failback
- Cross-region provider routing

## Description
The existing circuit breaker mechanism in `iron_reliability` correctly stops retrying a failed provider, but it leaves the request in a failed state rather than routing it to an alternative. For agents that have multiple providers configured, this means an outage at a single provider causes agent failure even when healthy alternatives exist.

This task adds a failover layer to the LLM router that maintains a priority-ordered list of providers per agent. When the primary provider's circuit breaker opens, the router transparently routes the request to the next provider in the priority list. Each request records which provider actually served it, feeding into the analytics module so operators can see failover frequency and patterns. A configurable retry-with-backoff step runs before failover to handle transient errors without unnecessary provider switching.

## Context
Circuit breaker stops retrying a failed provider but does not automatically route to an alternative. Agents with multiple assigned providers need automatic failover.

Critical areas:
- `module/iron_runtime/src/llm_router/router.rs`
- `module/iron_reliability/src/lib.rs`
- `module/iron_runtime_analytics/src/`

## Work Procedure
1. Define a `FailoverConfig` struct in the router module with a priority-ordered list of provider identifiers per agent.
2. Extend the agent configuration schema to accept a `providers` list with priority ordering.
3. Modify `router.rs` to check the circuit breaker state before routing and select the next available provider on open circuit.
4. Add retry-with-backoff logic (configurable attempts and base delay) before triggering failover.
5. Annotate each routed request with the provider that actually served it.
6. Extend the analytics module to track failover events (source provider, target provider, timestamp).
7. Add failover statistics aggregation (failover count, frequency, per-provider breakdown).
8. Write tests covering: single provider (no failover), primary failure with secondary available, all providers failed.
9. Verify that agents with a single provider preserve existing behavior (no failover attempted).

## Implementation plan
1. Add failover configuration: priority ordering of providers per agent.
2. On circuit breaker open, route to next provider in priority list.
3. Track which provider served each request (for analytics).
4. Add fallback statistics to analytics.
5. Configurable retry with backoff before failover.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Primary provider healthy | Requests route to primary | All responses from primary provider |
| Primary circuit open, secondary available | Requests route to secondary | Responses from secondary, analytics record failover |
| Primary circuit open, no secondary configured | Request fails with existing error behavior | Error returned, no panic or hang |
| All providers circuit open | Request fails after exhausting providers | Error returned listing all providers attempted |
| Transient error with retry success | Retry succeeds before failover | Request served by primary after retry |
| Failover and primary recovers | New requests continue to secondary until circuit resets | No premature failback |

## Validation List
- [ ] `FailoverConfig` struct accepts priority-ordered provider lists
- [ ] Router selects next provider when circuit breaker is open
- [ ] Retry with backoff executes before failover
- [ ] Each request records the serving provider in analytics
- [ ] Failover statistics are accessible via analytics API
- [ ] Single-provider agents preserve existing behavior exactly
- [ ] No panics or deadlocks under concurrent failover scenarios
- [ ] Agent-facing API is unchanged (failover is transparent)

## Validation Procedure
1. Run `cargo test -p iron_runtime -p iron_reliability` and confirm all tests pass.
2. Configure an agent with two providers and `failure_threshold=1`, send one failed request to trip the circuit breaker naturally, and verify requests route to the secondary.
3. Query analytics to confirm failover events are recorded with correct source and target providers.
4. Configure an agent with a single provider, simulate failure, and verify the existing error behavior is preserved.
5. Run a concurrent test with multiple agents failing over simultaneously and verify no deadlocks or dropped requests.
6. Review analytics output for failover frequency statistics.

## Acceptance criteria
- When primary provider circuit opens, requests route to secondary.
- Analytics record which provider served each request.
- Failover is transparent to the calling agent.
- No failover if no secondary provider configured (existing behavior preserved).
