# Usage limits and agent budget (2025-12-16)

## Usage limits

Current usage limits status:
 - Usage limits on startup are seeded for each different user (impossible to understand from dashboard as user name not shown in table row) (module/iron_token_manager/src/seed.rs line 468-534)
 - When new limit is created, it uses user_id from request, just creating new limit without overriding existing.
 - It's possible to create limit for non-existing project 
 - List_all_limits returns all limits without filtration by user


## Agent Budgets

Current agent budget status:
 - Agent budget is initialized with agent creation automatically.
 - In table row budget_status is shown.