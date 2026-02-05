# Iron CLI Reference

**Version**: 0.4.0

## Commands

### Token API Commands (9 commands)

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

### Control API Commands (33 commands)

| Command | Description | Params |
|---------|-------------|--------|
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
| `.auth.login` | Authenticate (Control API) | 3 |
| `.auth.refresh` | Refresh token (Control API) | 1 |
| `.auth.logout` | Logout (Control API) | 1 |

See [parameter_groups.md](../parameter_groups.md) for shared parameters (output, pagination, dry_run).

**Namespaces**: [.auth](auth.md) | [.tokens](tokens.md) | [.analytics](analytics.md) | [.limits](limits.md) | [.agent](agent.md) | [.provider](provider.md) | [.budget](budget.md) | [.project](project.md) | [.user](user.md)

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
