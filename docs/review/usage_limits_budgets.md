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

However, in handshake() module/iron_control_api/src/routes/budget/handshake.rs:291-294 usage limit cost is read and in module/iron_control_api/src/routes/budget/handshake.rs:609 cost limit is updated, so actually cost limit is ensured moment of budget_leasing.

## Result

Usage Limits cost used in handshake process.

## How to introduce native mуthod from LimitEnforcer?

*LimitEnforcer* from *module/iron_token_manager/src/limit_enforcer.rs*

1) Add `LimitEnforcer` to `BudgetState`
2) Instead of calling row "SELECT ... from usage_limits" use LimitEnforcer method `check_cost_allowed` . If it returns `false`, throw error "budget exceeded"
3) Instead of calling row "UPDATE usage_limits ..." use `LimitEnforcer` method `increment_cost`.

## Bug found (module/iron_control_api/src/routes/budget/handshake.rs line 290-294)
Usage limit is set for user personally, but user can have many limits. In query 
```sql
SELECT max_cost_per_month_microdollars, current_cost_microdollars_this_month
       FROM usage_limits
       WHERE user_id = ?
       LIMIT 1
```
limit is retrieved specifically for user, but ignoring project, so actually this limit is first created limit.

