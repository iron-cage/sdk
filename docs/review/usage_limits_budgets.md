# Cost/Month Limit + Budget connection

## Main question

 - How setting up limit reflects on agent budget?
 
## Searching Evidences

### Limit endpoints

Limits are connected to dashboard using simple CRUD endpoints. Limits don't affect budgets in endpoints.

Evidences:
 - module/iron_dashboard/src/composables/useApi.ts:262-289
 - module/iron_control_api/src/routes/limits.rs:275-519
 - module/iron_token_manager/src/limit_enforcer.rs:445-609, 109-150

### Cost ensurement logic

Budget is validated in check_budget() function: module/iron_runtime/src/llm_router/proxy.rs:166-216 using CostController. 

Budget reservation logic in: module/iron_token_manager/src/agent_budget.rs:260 uses agent_budget table.

Limit for cost is ensured using check_cost_allowed() module/iron_token_manager/src/limit_enforcer.rs:263 

*NO USAGES FOUND, PRESENTS ONLY 1 TEST*

### Result

Usage Limits currently not used to limit requests in production code. 
