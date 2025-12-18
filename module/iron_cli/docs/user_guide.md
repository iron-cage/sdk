# Iron CLI User Guide

**Version:** 1.0.0
**Last Updated:** 2025-12-12

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Concepts](#basic-concepts)
3. [Configuration](#configuration)
4. [Command Categories](#command-categories)
5. [Common Workflows](#common-workflows)
6. [Output Formats](#output-formats)
7. [Dry-Run Mode](#dry-run-mode)
8. [Examples](#examples)
9. [Troubleshooting](#troubleshooting)

---

## Getting Started

### Installation

The Iron CLI is built as part of the iron_cli crate:

```bash
cd iron_runtime/dev/module/iron_cli
cargo build --release --bin iron
```

The binary will be located at `target/release/iron`.

### First Command

Test your installation:

```bash
iron .agent.list
```

If the server is running and you have valid authentication, you'll see a table of agents.

### Quick Reference

```bash
# General syntax
iron .resource.action param1::value1 param2::value2

# Get help (any command)
iron .agent.create --help

# Use dry-run mode
iron .agent.create name::test budget::1000 dry::1

# Change output format
iron .agent.list format::json
iron .agent.list format::yaml
iron .agent.list format::table  # default
```

---

## Basic Concepts

### Command Naming Convention

All commands follow the **dot-prefix** format:

```
.resource.action
```

**Examples:**
- `.agent.list` - List agents
- `.provider.create` - Create provider
- `.user.delete` - Delete user

This naming ensures commands don't conflict with system commands.

### Parameter Format

Parameters use the **double-colon** format:

```
parameter::value
```

**Examples:**
- `name::my-agent`
- `budget::5000`
- `dry::1`
- `format::json`

### Resource Categories

The CLI manages 8 resource types:

1. **Agents** - AI agents that consume provider services
2. **Providers** - AI provider configurations (OpenAI, Anthropic, etc.)
3. **Analytics** - Usage and spending data
4. **Budget Limits** - Global budget constraints
5. **API Tokens** - Authentication tokens
6. **Projects** - Project viewing
7. **Budget Requests** - Budget increase requests
8. **Users** - User management

---

## Configuration

### Environment Variables

The CLI uses environment variables for configuration:

```bash
# Required: Control API base URL
export IRON_CONTROL_API_URL="http://localhost:3001"

# Optional: API authentication token
export IRON_API_TOKEN="your-token-here"
```

### Config File (Planned)

Future versions will support `~/.iron/config.toml`:

```toml
[control_api]
base_url = "http://localhost:3001"
api_token = "your-token-here"
timeout_seconds = 30
```

---

## Command Categories

### Agent Management (8 commands)

Manage AI agents with budgets and provider assignments.

| Command | Description | Parameters |
|---------|-------------|------------|
| `.agent.list` | List all agents | `format` |
| `.agent.create` | Create new agent | `name`, `budget`, `provider_ids` (optional), `dry` |
| `.agent.get` | Get agent details | `id`, `format` |
| `.agent.update` | Update agent | `id`, `name` (optional), `dry` |
| `.agent.delete` | Delete agent | `id`, `dry` |
| `.agent.assign_providers` | Assign providers | `id`, `provider_ids`, `dry` |
| `.agent.list_providers` | List agent's providers | `id`, `format` |
| `.agent.remove_provider` | Remove provider | `id`, `provider_id`, `dry` |

### Provider Management (8 commands)

Manage AI provider configurations.

| Command | Description | Parameters |
|---------|-------------|------------|
| `.provider.list` | List all providers | `format` |
| `.provider.create` | Create provider | `name`, `api_key`, `endpoint`, `dry` |
| `.provider.get` | Get provider details | `id`, `format` |
| `.provider.update` | Update provider | `id`, `name` (optional), `api_key` (optional), `dry` |
| `.provider.delete` | Delete provider | `id`, `dry` |
| `.provider.assign_agents` | Assign agents | `id`, `agent_ids`, `dry` |
| `.provider.list_agents` | List provider's agents | `id`, `format` |
| `.provider.remove_agent` | Remove agent | `id`, `agent_id`, `dry` |

### Analytics (8 commands)

View usage statistics and spending data.

| Command | Description | Parameters |
|---------|-------------|------------|
| `.analytics.usage` | Get usage stats | `start_date` (optional), `end_date` (optional), `format` |
| `.analytics.spending` | Get spending data | `start_date` (optional), `end_date` (optional), `format` |
| `.analytics.metrics` | Get system metrics | `format` |
| `.analytics.usage_by_agent` | Usage by agent | `agent_id` (optional), `format` |
| `.analytics.usage_by_provider` | Usage by provider | `provider_id` (optional), `format` |
| `.analytics.spending_by_period` | Spending by period | `period`, `format` |
| `.analytics.export_usage` | Export usage data | `format_type`, `start_date` (optional), `end_date` (optional) |
| `.analytics.export_spending` | Export spending data | `format_type`, `start_date` (optional), `end_date` (optional) |

### Budget Limits (2 commands)

Manage global budget constraints.

| Command | Description | Parameters |
|---------|-------------|------------|
| `.budget_limit.get` | Get current limit | `format` |
| `.budget_limit.set` | Set new limit | `limit`, `dry` |

### API Tokens (4 commands)

Manage API authentication tokens.

| Command | Description | Parameters |
|---------|-------------|------------|
| `.api_token.list` | List all tokens | `format` |
| `.api_token.create` | Create new token | `name`, `expires_at` (optional), `dry` |
| `.api_token.get` | Get token details | `id`, `format` |
| `.api_token.revoke` | Revoke token | `id`, `dry` |

### Projects (2 commands)

View project information.

| Command | Description | Parameters |
|---------|-------------|------------|
| `.project.list` | List all projects | `format` |
| `.project.get` | Get project details | `id`, `format` |

### Budget (1 command)

View budget analytics across agents.

| Command | Description | Parameters |
|---------|-------------|------------|
| `.budget.status` | Get budget status | `agent_id` (optional), `threshold` (optional), `status` (optional), `format` |

### Users (8 commands)

Manage users and permissions.

| Command | Description | Parameters |
|---------|-------------|------------|
| `.user.list` | List all users | `format` |
| `.user.create` | Create user | `username`, `email`, `password`, `role` (optional), `dry` |
| `.user.get` | Get user details | `id`, `format` |
| `.user.update` | Update user | `id`, `email` (optional), `username` (optional), `dry` |
| `.user.delete` | Delete user | `id`, `dry` |
| `.user.set_role` | Set user role | `id`, `role`, `dry` |
| `.user.reset_password` | Reset password | `id`, `new_password`, `dry` |
| `.user.get_permissions` | Get permissions | `id`, `format` |

---

## Common Workflows

### Workflow 1: Create and Configure an Agent

```bash
# Step 1: Create agent
iron .agent.create name::production-agent budget::10000 dry::1

# Step 2: Verify parameters look correct, then create for real
iron .agent.create name::production-agent budget::10000

# Step 3: Get agent ID from response, assign providers
iron .agent.assign_providers id::agent_abc123 provider_ids::provider_1,provider_2

# Step 4: Verify configuration
iron .agent.get id::agent_abc123 format::table
```

### Workflow 2: Monitor Usage and Spending

```bash
# Check overall usage
iron .analytics.usage start_date::2025-01-01 end_date::2025-12-31

# Check spending by month
iron .analytics.spending_by_period period::month

# Check specific agent usage
iron .analytics.usage_by_agent agent_id::agent_abc123

# Export data for analysis
iron .analytics.export_usage format_type::csv start_date::2025-01-01
```

### Workflow 3: Budget Monitoring

```bash
# Check budget status across all agents
iron .budget.status

# Filter by specific agent
iron .budget.status agent_id::1

# Find agents using more than 80% of budget
iron .budget.status threshold::80

# Find exhausted budgets
iron .budget.status status::exhausted
```

### Workflow 4: User Management

```bash
# Create new developer user
iron .user.create \
  username::johndoe \
  email::john@example.com \
  password::SecurePass123 \
  role::developer

# Grant admin role
iron .user.set_role id::user_123 role::admin

# Check user permissions
iron .user.get_permissions id::user_123
```

---

## Output Formats

The CLI supports three output formats via the `format` parameter:

### Table Format (Default)

Human-readable ASCII tables.

```bash
iron .agent.list format::table
```

**Example output:**
```
ID          | NAME              | BUDGET | STATUS
------------|-------------------|--------|-------
agent_001   | Production Agent  | 10000  | active
agent_002   | Test Agent        | 5000   | active
```

### JSON Format

Machine-readable JSON for scripting.

```bash
iron .agent.list format::json
```

**Example output:**
```json
[
  {
    "id": "agent_001",
    "name": "Production Agent",
    "budget": 10000,
    "status": "active"
  }
]
```

### YAML Format

Human-readable YAML for configuration.

```bash
iron .agent.list format::yaml
```

**Example output:**
```yaml
- id: agent_001
  name: Production Agent
  budget: 10000
  status: active
```

---

## Dry-Run Mode

### What is Dry-Run?

Dry-run mode validates parameters WITHOUT making actual changes. Perfect for:
- Testing commands before execution
- Verifying parameter syntax
- Learning the CLI

### How to Use

Add `dry::1` to any mutation command:

```bash
# Dry-run mode
iron .agent.create name::test budget::1000 dry::1
# Output: [DRY RUN] Agent would be created (no HTTP request made)

# Real execution
iron .agent.create name::test budget::1000
# Output: (actual API response)
```

### Which Commands Support Dry-Run?

All **mutation** commands support dry-run:
- Create operations
- Update operations
- Delete operations
- Approve/reject operations

**Read operations** don't support dry-run (they don't modify data):
- List operations
- Get operations

---

## Examples

### Agent Examples

```bash
# List all agents (table format)
iron .agent.list

# List agents (JSON format)
iron .agent.list format::json

# Create agent with single provider
iron .agent.create name::my-agent budget::5000 provider_ids::provider_123

# Create agent with multiple providers
iron .agent.create \
  name::multi-provider-agent \
  budget::15000 \
  provider_ids::provider_1,provider_2,provider_3

# Get specific agent details
iron .agent.get id::agent_abc123

# Update agent name
iron .agent.update id::agent_abc123 name::renamed-agent

# Delete agent (dry-run first)
iron .agent.delete id::agent_abc123 dry::1
iron .agent.delete id::agent_abc123
```

### Provider Examples

```bash
# List providers
iron .provider.list

# Create OpenAI provider
iron .provider.create \
  name::openai-prod \
  api_key::sk-abc123xyz \
  endpoint::https://api.openai.com/v1

# Create Anthropic provider
iron .provider.create \
  name::anthropic-prod \
  api_key::sk-ant-xyz789 \
  endpoint::https://api.anthropic.com/v1

# Update provider API key
iron .provider.update id::provider_123 api_key::new-key-here
```

### Analytics Examples

```bash
# Get usage for date range
iron .analytics.usage \
  start_date::2025-01-01 \
  end_date::2025-01-31

# Get spending data
iron .analytics.spending \
  start_date::2025-01-01 \
  end_date::2025-01-31

# Get metrics
iron .analytics.metrics

# Export usage as CSV
iron .analytics.export_usage \
  format_type::csv \
  start_date::2025-01-01 \
  end_date::2025-12-31
```

### Budget Status Examples

```bash
# Get budget status for all agents
iron .budget.status

# Get budget status for specific agent
iron .budget.status agent_id::1

# Find agents at risk (>80% budget used)
iron .budget.status threshold::80

# Find agents with exhausted budgets
iron .budget.status status::exhausted

# Export as JSON
iron .budget.status format::json
```

---

## Troubleshooting

### Common Errors

#### 1. "Missing authentication token"

**Error:**
```
Error: HTTP request failed: API error (401):
{"error":{"code":"AUTH_MISSING_TOKEN","message":"Missing authentication token"}}
```

**Solution:**
Set the `IRON_API_TOKEN` environment variable:
```bash
export IRON_API_TOKEN="your-token-here"
```

#### 2. "Connection refused"

**Error:**
```
Error: HTTP request failed: Connection refused
```

**Solution:**
- Check if the Control API server is running
- Verify the `IRON_CONTROL_API_URL` is correct
```bash
export IRON_CONTROL_API_URL="http://localhost:3001"
```

#### 3. "Command not recognized"

**Error:**
```
Error: Command '.agent.lst' not found
```

**Solution:**
Check command spelling. Use `.agent.list` (not `.agent.lst`).

#### 4. "Missing required parameter"

**Error:**
```
Error: Handler error: Missing required parameter: name
```

**Solution:**
Add the required parameter:
```bash
iron .agent.create name::my-agent budget::1000
```

#### 5. "Invalid parameter format"

**Error:**
```
Error: Handler error: Budget must be a positive integer
```

**Solution:**
Check parameter values match expected types:
```bash
# Wrong
iron .agent.create name::test budget::abc

# Correct
iron .agent.create name::test budget::1000
```

### Debug Mode

For detailed debugging, check the environment variables:

```bash
# Verify configuration
echo $IRON_CONTROL_API_URL
echo $IRON_API_TOKEN

# Test server connectivity
curl -v http://localhost:3001/api/health
```

### Getting Help

For each command, detailed help is available:

```bash
iron .agent.create --help
```

---

## Advanced Usage

### Scripting with JSON Output

```bash
# Get all agent IDs
iron .agent.list format::json | jq -r '.[].id'

# Count active agents
iron .agent.list format::json | jq '[.[] | select(.status=="active")] | length'

# Create agents from CSV
while IFS=, read -r name budget; do
  iron .agent.create name::$name budget::$budget
done < agents.csv
```

### Automation

```bash
#!/bin/bash
# Daily usage report script

TODAY=$(date +%Y-%m-%d)
YESTERDAY=$(date -d "yesterday" +%Y-%m-%d)

echo "Daily Usage Report: $TODAY"
echo "================================"

iron .analytics.usage \
  start_date::$YESTERDAY \
  end_date::$TODAY \
  format::table

iron .analytics.spending \
  start_date::$YESTERDAY \
  end_date::$TODAY \
  format::table
```

### Batch Operations

```bash
# Delete multiple agents
for agent_id in agent_1 agent_2 agent_3; do
  iron .agent.delete id::$agent_id dry::1
done

# If dry-run looks good, run for real
for agent_id in agent_1 agent_2 agent_3; do
  iron .agent.delete id::$agent_id
done
```

---

## Best Practices

### 1. Always Dry-Run First

```bash
# ✅ Good: Test with dry-run
iron .agent.delete id::important-agent dry::1
iron .agent.delete id::important-agent

# ❌ Bad: Delete immediately
iron .agent.delete id::important-agent
```

### 2. Use Descriptive Names

```bash
# ✅ Good: Clear, descriptive names
iron .agent.create name::production-chatbot budget::10000
iron .provider.create name::openai-production api_key::...

# ❌ Bad: Vague names
iron .agent.create name::agent1 budget::10000
iron .provider.create name::provider api_key::...
```

### 3. Store Tokens Securely

```bash
# ✅ Good: Use environment variables, not command line
export IRON_API_TOKEN="secret-token"
iron .agent.list

# ❌ Bad: Token in command history
iron .agent.list token::secret-token  # DON'T DO THIS
```

### 4. Use JSON for Automation

```bash
# ✅ Good: Parse JSON in scripts
AGENT_ID=$(iron .agent.create name::temp budget::1000 format::json | jq -r '.id')

# ❌ Bad: Parse table output
AGENT_ID=$(iron .agent.create name::temp budget::1000 | grep ID | cut -d: -f2)
```

### 5. Check Before Bulk Operations

```bash
# ✅ Good: Verify count before mass delete
COUNT=$(iron .agent.list format::json | jq 'length')
echo "About to delete $COUNT agents"
read -p "Continue? (y/n) " -n 1 -r
```

---

## Quick Reference Card

### Syntax
```
iron .resource.action param::value param2::value2
```

### Essential Parameters
- `format::json|yaml|table` - Output format (default: table)
- `dry::1` - Dry-run mode (test without executing)

### Environment Variables
```bash
export IRON_CONTROL_API_URL="http://localhost:3001"
export IRON_API_TOKEN="your-token-here"
```

### Most Common Commands
```bash
iron .agent.list                              # List agents
iron .agent.create name::NAME budget::AMOUNT # Create agent
iron .analytics.usage                         # View usage
iron .budget.status                           # View budget status
iron .user.list                               # List users
```

---

## Next Steps

- Read the [API Reference](api_reference.md) for complete command details
- See [Troubleshooting Guide](troubleshooting_guide.md) for common issues
- Check [Configuration Guide](configuration_guide.md) for advanced setup

