# Iron Cage CLI - Command Reference

**Version:** 1.0.0
**Last Updated:** 2025-12-10

---

## Table of Contents

1. [Authentication Commands](#authentication-commands)
2. [Token Management Commands](#token-management-commands)
3. [User Management Commands](#user-management-commands)
4. [Usage Commands](#usage-commands)
5. [Limits Commands](#limits-commands)
6. [Traces Commands](#traces-commands)
7. [Health Commands](#health-commands)

---

## Authentication Commands

### `iron login`

Authenticate with Iron Cage Control Panel.

**Usage:**
```bash
iron login --username <username> --password <password>
```

**Parameters:**
- `--username` (required): Your username (3-255 characters)
- `--password` (required): Your password (1-1000 characters)
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron login --username admin --password SecurePass123
```

**Output:**
```
Login successful
Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Expires: 2025-01-09 12:00:00
```

---

### `iron logout`

Invalidate current authentication token.

**Usage:**
```bash
iron logout
```

**Parameters:**
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron logout
```

**Output:**
```
Logged out successfully
```

---

### `iron refresh`

Refresh authentication token (extends expiration).

**Usage:**
```bash
iron refresh
```

**Parameters:**
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron refresh
```

**Output:**
```
Token refreshed successfully
New expiration: 2025-01-09 12:00:00
```

---

## Token Management Commands

### `iron tokens generate`

Generate a new IC Token for agent authentication.

**Usage:**
```bash
iron tokens generate --name <name> --scope <scope> [--ttl <seconds>]
```

**Parameters:**
- `--name` (required): Token name (non-empty)
- `--scope` (required): Token scope in format `action:resource` (e.g., `read:tokens`)
- `--ttl` (optional): Time-to-live in seconds (60-31536000)
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron tokens generate --name "Production Agent" --scope "execute:agents" --ttl 86400
```

**Output:**
```
Token generated successfully
ID: tok_abc123def456
Name: Production Agent
Scope: execute:agents
Expires: 2025-12-11 12:00:00
```

---

### `iron tokens list`

List all IC Tokens.

**Usage:**
```bash
iron tokens list [--filter <status>] [--sort <field>]
```

**Parameters:**
- `--filter` (optional): Filter by status (active, revoked, expired)
- `--sort` (optional): Sort field (created_at, name)
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron tokens list --filter active --format table
```

**Output:**
```
ID              Name              Scope            Created
tok_abc123      Production Agent  execute:agents   2025-12-10
tok_def456      Dev Agent         read:tokens      2025-12-09
```

---

### `iron tokens get`

Get details for a specific IC Token.

**Usage:**
```bash
iron tokens get <token_id>
```

**Parameters:**
- `<token_id>` (required): Token ID (must start with `tok_`)
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron tokens get tok_abc123def456
```

**Output:**
```
ID: tok_abc123def456
Name: Production Agent
Scope: execute:agents
Created: 2025-12-10 12:00:00
Last Used: 2025-12-10 14:30:00
Status: Active
```

---

### `iron tokens rotate`

Rotate an IC Token (generates new token, revokes old one).

**Usage:**
```bash
iron tokens rotate <token_id> [--ttl <seconds>]
```

**Parameters:**
- `<token_id>` (required): Token ID to rotate
- `--ttl` (optional): New time-to-live in seconds
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron tokens rotate tok_abc123def456 --ttl 86400
```

**Output:**
```
Token rotated successfully
New ID: tok_xyz789ghi012
Old token revoked: tok_abc123def456
```

---

### `iron tokens revoke`

Revoke an IC Token.

**Usage:**
```bash
iron tokens revoke <token_id> [--reason <text>]
```

**Parameters:**
- `<token_id>` (required): Token ID to revoke
- `--reason` (optional): Reason for revocation
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron tokens revoke tok_abc123def456 --reason "Security incident"
```

**Output:**
```
Token revoked successfully
ID: tok_abc123def456
Reason: Security incident
```

---

## User Management Commands

**Note:** All user management commands require Admin role.

---

### `iron users create`

Create a new user account.

**Usage:**
```bash
iron users create --username <name> --password <pass> --email <email> --role <role>
```

**Parameters:**
- `--username` (required): Username (non-empty, max 255 chars)
- `--password` (required): Password (min 8 chars, max 1000 chars)
- `--email` (required): Email address (must contain @, max 255 chars)
- `--role` (required): User role (viewer, user, or admin)
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron users create \
  --username john_doe \
  --password SecurePass123! \
  --email john.doe@example.com \
  --role user
```

**Output:**
```
User created successfully
ID: 1001
Username: john_doe
Email: john.doe@example.com
Role: user
Status: Active
```

**Validation Rules:**
- Username: 1-255 characters, unique
- Password: 8-1000 characters
- Email: Must contain @, 1-255 characters, unique
- Role: Must be viewer, user, or admin

---

### `iron users list`

List all users with optional filters.

**Usage:**
```bash
iron users list [--role <role>] [--is-active <bool>] [--search <term>] [--page <num>] [--page-size <size>]
```

**Parameters:**
- `--role` (optional): Filter by role (viewer, user, admin)
- `--is-active` (optional): Filter by status (true, false)
- `--search` (optional): Search in username or email
- `--page` (optional): Page number (default: 1)
- `--page-size` (optional): Results per page (1-100, default: 20)
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron users list --role admin --is-active true --format table
```

**Output:**
```
ID    Username  Email                 Role   Status
1     admin     admin@example.com     admin  Active
42    jane_doe  jane.doe@example.com  admin  Active
```

---

### `iron users get`

Get detailed information for a specific user.

**Usage:**
```bash
iron users get <user_id>
```

**Parameters:**
- `<user_id>` (required): User ID (integer)
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron users get 1001
```

**Output:**
```
ID: 1001
Username: john_doe
Email: john.doe@example.com
Role: user
Status: Active
Created: 2025-12-10 12:00:00
Last Login: 2025-12-10 14:30:00
```

---

### `iron users suspend`

Suspend a user account (prevents login).

**Usage:**
```bash
iron users suspend <user_id> [--reason <text>]
```

**Parameters:**
- `<user_id>` (required): User ID to suspend
- `--reason` (optional): Reason for suspension (stored in audit log)
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron users suspend 1001 --reason "Policy violation - multiple failed login attempts"
```

**Output:**
```
User suspended successfully
ID: 1001
Username: john_doe
Status: Suspended
Reason: Policy violation - multiple failed login attempts
```

**Notes:**
- Suspended users cannot login
- Admins cannot suspend their own account
- Creates audit log entry

---

### `iron users activate`

Activate a suspended user account.

**Usage:**
```bash
iron users activate <user_id>
```

**Parameters:**
- `<user_id>` (required): User ID to activate
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron users activate 1001
```

**Output:**
```
User activated successfully
ID: 1001
Username: john_doe
Status: Active
```

**Notes:**
- Activated users can login again
- Creates audit log entry

---

### `iron users delete`

Soft-delete a user account (preserves audit trail).

**Usage:**
```bash
iron users delete <user_id>
```

**Parameters:**
- `<user_id>` (required): User ID to delete
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron users delete 1001
```

**Output:**
```
User deleted successfully
ID: 1001
Username: john_doe
Status: Deleted
```

**Notes:**
- Soft delete preserves user record and audit log
- Deleted users cannot login
- Admins cannot delete their own account
- Creates audit log entry

---

### `iron users change-role`

Change a user's role.

**Usage:**
```bash
iron users change-role <user_id> --role <role>
```

**Parameters:**
- `<user_id>` (required): User ID to modify
- `--role` (required): New role (viewer, user, or admin)
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron users change-role 1001 --role admin
```

**Output:**
```
User role changed successfully
ID: 1001
Username: john_doe
Old Role: user
New Role: admin
```

**Notes:**
- Admins cannot change their own role
- Creates audit log entry with old and new values

---

### `iron users reset-password`

Reset a user's password (admin-initiated).

**Usage:**
```bash
iron users reset-password <user_id> --new-password <pass> --force-change <bool>
```

**Parameters:**
- `<user_id>` (required): User ID to modify
- `--new-password` (required): New password (min 8 chars, max 1000 chars)
- `--force-change` (required): Force password change on next login (true or false)
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron users reset-password 1001 --new-password TempPass456! --force-change true
```

**Output:**
```
Password reset successfully
ID: 1001
Username: john_doe
Force Change: Yes
User must change password on next login
```

**Notes:**
- Admin can reset any user's password (including their own)
- If force-change is true, user must change password on next login
- Creates audit log entry (password itself not logged)

---

## Usage Commands

### `iron usage show`

Display usage statistics for current user.

**Usage:**
```bash
iron usage show [--from <date>] [--to <date>]
```

**Parameters:**
- `--from` (optional): Start date (YYYY-MM-DD)
- `--to` (optional): End date (YYYY-MM-DD)
- `--format` (optional): Output format (table, json, yaml, expanded)

**Example:**
```bash
iron usage show --from 2025-12-01 --to 2025-12-10 --format table
```

**Output:**
```
Period: 2025-12-01 to 2025-12-10
Requests: 1,234
Tokens Consumed: 456,789
Total Cost: $12.34
```

---

### `iron usage export`

Export usage data to file.

**Usage:**
```bash
iron usage export --output <path> [--format <format>]
```

**Parameters:**
- `--output` (required): Output file path
- `--format` (optional): Export format (csv, json)
- `--from` (optional): Start date
- `--to` (optional): End date

**Example:**
```bash
iron usage export --output usage_report.csv --format csv --from 2025-12-01
```

**Output:**
```
Usage data exported successfully
File: usage_report.csv
Records: 1,234
```

---

### `iron usage by-project`

View usage statistics by project.

**Usage:**
```bash
iron usage by-project <project_id> [--from <date>] [--to <date>]
```

**Parameters:**
- `<project_id>` (required): Project ID
- `--from` (optional): Start date
- `--to` (optional): End date
- `--format` (optional): Output format

**Example:**
```bash
iron usage by-project proj_123 --from 2025-12-01
```

---

### `iron usage by-provider`

View usage statistics by provider.

**Usage:**
```bash
iron usage by-provider <provider> [--aggregation <level>]
```

**Parameters:**
- `<provider>` (required): Provider name (openai, anthropic, gemini)
- `--aggregation` (optional): Aggregation level (day, week, month)
- `--format` (optional): Output format

**Example:**
```bash
iron usage by-provider anthropic --aggregation day
```

---

## Limits Commands

### `iron limits create`

Create usage limits.

**Usage:**
```bash
iron limits create --requests-per-minute <num> --tokens-per-day <num> --cost-per-month <amount>
```

**Parameters:**
- `--requests-per-minute` (optional): Rate limit
- `--tokens-per-day` (optional): Daily token limit
- `--cost-per-month` (optional): Monthly cost limit in USD
- `--format` (optional): Output format

**Example:**
```bash
iron limits create --requests-per-minute 60 --tokens-per-day 100000 --cost-per-month 500
```

**Output:**
```
Limits created successfully
Requests/Minute: 60
Tokens/Day: 100,000
Cost/Month: $500.00
```

---

### `iron limits list`

List all configured limits.

**Usage:**
```bash
iron limits list
```

**Parameters:**
- `--format` (optional): Output format

**Example:**
```bash
iron limits list --format table
```

**Output:**
```
ID   Type                 Value      Status
1    requests_per_minute  60         Active
2    tokens_per_day       100000     Active
3    cost_per_month       500.00     Active
```

---

### `iron limits get`

Get specific limit details.

**Usage:**
```bash
iron limits get <limit_id>
```

**Parameters:**
- `<limit_id>` (required): Limit ID
- `--format` (optional): Output format

**Example:**
```bash
iron limits get 1
```

---

### `iron limits update`

Update existing limit.

**Usage:**
```bash
iron limits update <limit_id> --limit-value <value>
```

**Parameters:**
- `<limit_id>` (required): Limit ID to update
- `--limit-value` (required): New limit value
- `--format` (optional): Output format

**Example:**
```bash
iron limits update 1 --limit-value 120
```

**Output:**
```
Limit updated successfully
ID: 1
Old Value: 60
New Value: 120
```

---

### `iron limits delete`

Delete a limit.

**Usage:**
```bash
iron limits delete <limit_id>
```

**Parameters:**
- `<limit_id>` (required): Limit ID to delete
- `--format` (optional): Output format

**Example:**
```bash
iron limits delete 1
```

**Output:**
```
Limit deleted successfully
ID: 1
```

---

## Traces Commands

### `iron traces list`

List execution traces.

**Usage:**
```bash
iron traces list [--limit <num>] [--filter <status>]
```

**Parameters:**
- `--limit` (optional): Number of traces to return
- `--filter` (optional): Filter by status (success, error, pending)
- `--format` (optional): Output format

**Example:**
```bash
iron traces list --limit 10 --filter success
```

**Output:**
```
ID              Status    Duration    Started
trace_abc123    success   2.5s        2025-12-10 12:00:00
trace_def456    success   1.2s        2025-12-10 12:05:00
```

---

### `iron traces get`

Get detailed trace information.

**Usage:**
```bash
iron traces get <trace_id>
```

**Parameters:**
- `<trace_id>` (required): Trace ID
- `--format` (optional): Output format

**Example:**
```bash
iron traces get trace_abc123
```

---

### `iron traces export`

Export traces to file.

**Usage:**
```bash
iron traces export --output <path> [--format <format>]
```

**Parameters:**
- `--output` (required): Output file path
- `--format` (optional): Export format (json, yaml)

**Example:**
```bash
iron traces export --output traces_report.json --format json
```

---

## Health Commands

### `iron health`

Check Iron Cage Control Panel health.

**Usage:**
```bash
iron health [--details]
```

**Parameters:**
- `--details` (optional): Show detailed health information
- `--format` (optional): Output format

**Example:**
```bash
iron health --details
```

**Output:**
```
Status: Healthy
Uptime: 45 days, 3 hours
Version: 1.0.0
Database: Connected
Redis: Connected
```

---

### `iron version`

Display CLI and API versions.

**Usage:**
```bash
iron version
```

**Parameters:**
- `--format` (optional): Output format

**Example:**
```bash
iron version
```

**Output:**
```
CLI Version: 1.0.0
API Version: 1.0.0
Rust Version: 1.75.0
```

---

## Global Options

All commands support these global options:

- `--help, -h`: Show command help
- `--format <format>`: Output format (table, json, yaml, expanded)
- `--verbose, -v`: Enable verbose output
- `--quiet, -q`: Suppress non-error output
- `--config <path>`: Path to configuration file

---

## Exit Codes

- `0`: Success
- `1`: General error
- `2`: Missing required parameter
- `3`: Invalid parameter value
- `4`: Authentication error
- `5`: Authorization error (insufficient permissions)
- `6`: Resource not found
- `7`: Network error
- `8`: Server error

---

## Environment Variables

- `IRON_CAGE_BASE_URL`: Base URL for Control Panel API
- `IRON_CAGE_TOKEN`: Authentication token (alternative to login)
- `IRON_CAGE_FORMAT`: Default output format
- `IRON_CAGE_CONFIG`: Path to configuration file

---

*Related: [features/006_user_management.md](../../docs/features/006_user_management.md) (user management details) | [protocol/008_user_management_api.md](../../docs/protocol/008_user_management_api.md) (API spec)*

**Last Updated:** 2025-12-10
