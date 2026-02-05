# Iron CLI Reference

**Version**: 0.4.0

## Commands

| Command | Description | Params |
|---------|-------------|--------|
| `.auth.login` | Authenticate user and obtain JWT tokens | 3 |
| `.auth.refresh` | Refresh access token | 1 |
| `.auth.logout` | Invalidate tokens and clear keyring | 1 |
| `.tokens` | Unified token management (api, ic types) | 12 |
| `.limits.list` | List all limits | 1 |
| `.limits.get` | Get limit details | 2 |
| `.limits.create` | Create new limit | 5 |
| `.limits.update` | Update existing limit | 5 |
| `.limits.delete` | Delete limit | 2 |
| `.agent.list` | List all agents | 4 |
| `.agent.create` | Create new agent | 6 |
| `.agent.get` | Get agent details | 2 |
| `.agent.update` | Update agent | 5 |
| `.agent.delete` | Delete agent | 3 |
| `.agent.assign_providers` | Assign providers to agent | 4 |
| `.agent.list_providers` | List agent's providers | 2 |
| `.agent.remove_provider` | Remove provider from agent | 4 |
| `.provider.list` | List all providers | 2 |
| `.provider.create` | Create new provider | 6 |
| `.provider.get` | Get provider details | 2 |
| `.provider.update` | Update provider | 6 |
| `.provider.delete` | Delete provider | 3 |
| `.provider.assign_agents` | Assign agents to provider | 4 |
| `.provider.list_agents` | List provider's agents | 2 |
| `.provider.remove_agent` | Remove agent from provider | 4 |
| `.analytics` | Unified analytics: metrics, usage, spending, traces | 11 |
| `.budget_limit.get` | Get global budget limit | 1 |
| `.budget_limit.set` | Set global budget limit | 3 |
| `.budget.status` | Get budget status | 6 |
| `.project.list` | List all projects | 1 |
| `.project.get` | Get project details | 2 |
| `.user.list` | List all users | 1 |
| `.user.create` | Create new user | 6 |
| `.user.get` | Get user details | 2 |
| `.user.update` | Update user | 4 |
| `.user.delete` | Delete user | 3 |
| `.user.set_role` | Set user role | 4 |
| `.user.reset_password` | Reset user password | 4 |
| `.user.get_permissions` | Get user permissions | 2 |

See [parameter_groups.md](../parameter_groups.md) for shared parameters (output, pagination, dry_run).

**Namespaces**: [.auth](auth.md) | [.tokens](tokens.md) | [.analytics](analytics.md) | [.limits](limits.md) | [.agent](agent.md) | [.provider](provider.md) | [.budget](budget.md) | [.project](project.md) | [.user](user.md)

## Priority

| Priority | Command |
|----------|---------|
| 1 | `.auth.login` |
| 1 | `.auth.refresh` |
| 1 | `.auth.logout` |
| 2 | `.tokens` |
| 3 | `.provider.list` |
| 3 | `.provider.create` |
| 3 | `.provider.get` |
| 3 | `.provider.update` |
| 3 | `.provider.delete` |
| 4 | `.agent.list` |
| 4 | `.agent.create` |
| 4 | `.agent.get` |
| 4 | `.agent.update` |
| 4 | `.agent.delete` |
| 4 | `.agent.assign_providers` |
| 4 | `.agent.list_providers` |
| 4 | `.agent.remove_provider` |
| 4 | `.provider.assign_agents` |
| 4 | `.provider.list_agents` |
| 4 | `.provider.remove_agent` |
| 5 | `.analytics` |
| 6 | `.budget_limit.get` |
| 6 | `.budget_limit.set` |
| 6 | `.budget.status` |
| 7 | `.limits.list` |
| 7 | `.limits.get` |
| 7 | `.limits.create` |
| 7 | `.limits.update` |
| 7 | `.limits.delete` |
| 8 | `.user.list` |
| 8 | `.user.create` |
| 8 | `.user.get` |
| 8 | `.user.update` |
| 8 | `.user.delete` |
| 8 | `.user.set_role` |
| 8 | `.user.reset_password` |
| 8 | `.user.get_permissions` |
| 9 | `.project.list` |
| 9 | `.project.get` |

## Files

| File | Responsibility |
|------|----------------|
| auth.md | Authentication commands (`.auth.*`) |
| tokens.md | Unified token management: api, ic types (`.tokens`) |
| analytics.md | Unified analytics: metrics, usage, spending, traces (`.analytics`) |
| limits.md | Limit management (`.limits.*`) |
| agent.md | Agent management (`.agent.*`) |
| provider.md | Provider management (`.provider.*`) |
| budget.md | Budget management (`.budget.*`, `.budget_limit.*`) |
| project.md | Project management (`.project.*`) |
| user.md | User management (`.user.*`) |
| readme.md | Command index |
