# Manual Testing Plan: iron-token CLI

Manual testing plan for iron-token CLI commands that require real API interaction.

## Prerequisites

Before running manual tests, ensure:

1. **API Access**: Iron runtime Token Manager API is running and accessible
2. **Authentication**: Valid API credentials configured in system keyring
3. **Binary Built**: `cargo build --bin iron-token` completed successfully
4. **Test Environment**: Dedicated test environment (not production)

## Environment Setup

### 1. Configure API Endpoint

```bash
# Verify API is accessible
curl -X GET https://api.example.com/health

# Expected: {"status": "healthy"}
```

### 2. Set Up Credentials

```bash
# Configure credentials in keyring (done once)
iron-token .auth.login username::<your-username> password::<your-password>

# Verify authentication
iron-token .auth.refresh

# Expected: {"access_token": "...", "expires_in": 3600}
```

### 3. Verify Binary

```bash
# Check iron-token is in PATH
which iron-token

# Verify version
iron-token .version

# Expected: Version information
```

## Test Cases

### TC-1: Health Check Commands

**Purpose**: Verify API connectivity and version information

#### TC-1.1: Health Check

```bash
iron-token .health
```

**Expected:**
- Exit code: 0
- Output: JSON with status "healthy"
- Response time: < 1 second

#### TC-1.2: Version Information

```bash
iron-token .version
```

**Expected:**
- Exit code: 0
- Output: Version number in JSON format
- Matches expected version

### TC-2: Authentication Flow

**Purpose**: Verify authentication commands work correctly

#### TC-2.1: Login

```bash
iron-token .auth.login username::<test-user> password::<test-pass>
```

**Expected:**
- Exit code: 0
- Access token stored in keyring
- Output contains "access_token" and "expires_in"

**Validation:**
```bash
# Verify token is stored
keyring get iron_token access_token
# Should return stored token
```

#### TC-2.2: Token Refresh

```bash
iron-token .auth.refresh
```

**Expected:**
- Exit code: 0
- New access token returned
- Token updated in keyring

#### TC-2.3: Logout

```bash
iron-token .auth.logout
```

**Expected:**
- Exit code: 0
- Access token removed from keyring
- Subsequent authenticated commands fail

#### TC-2.4: Login After Logout

```bash
iron-token .auth.login username::<test-user> password::<test-pass>
```

**Expected:**
- Exit code: 0
- Can re-authenticate successfully
- New token stored

### TC-3: Token Management Commands

**Purpose**: Verify token CRUD operations

#### TC-3.1: Generate Token

```bash
iron-token .tokens.generate name::"Test Token" scope::"read:write"
```

**Expected:**
- Exit code: 0
- Token ID returned in JSON
- Token visible in list

#### TC-3.2: List Tokens

```bash
iron-token .tokens.list format::json
```

**Expected:**
- Exit code: 0
- JSON array of tokens
- Contains token created in TC-3.1

#### TC-3.3: Get Token Details

```bash
# Use token ID from TC-3.1
iron-token .tokens.get id::<token-id>
```

**Expected:**
- Exit code: 0
- Token details in JSON
- Matches generated token

#### TC-3.4: Rotate Token

```bash
iron-token .tokens.rotate id::<token-id>
```

**Expected:**
- Exit code: 0
- New token value returned
- Old token invalidated

#### TC-3.5: Revoke Token

```bash
iron-token .tokens.revoke id::<token-id>
```

**Expected:**
- Exit code: 0
- Token no longer in list
- Token cannot be used

### TC-4: Usage Tracking Commands

**Purpose**: Verify usage data retrieval

#### TC-4.1: Show Usage

```bash
iron-token .usage.show
```

**Expected:**
- Exit code: 0
- Usage statistics in JSON
- Contains usage data for current period

#### TC-4.2: Usage by Project

```bash
iron-token .usage.by_project project_id::<test-project>
```

**Expected:**
- Exit code: 0
- Usage statistics for specified project
- Data matches API dashboard

#### TC-4.3: Usage by Provider

```bash
iron-token .usage.by_provider provider::anthropic
```

**Expected:**
- Exit code: 0
- Usage statistics for specified provider
- Data organized by provider

#### TC-4.4: Export Usage

```bash
iron-token .usage.export output_file::/tmp/usage.json export_format::json
```

**Expected:**
- Exit code: 0
- File created at /tmp/usage.json
- File contains valid JSON usage data

**Validation:**
```bash
# Verify file exists
test -f /tmp/usage.json && echo "File created"

# Verify JSON is valid
jq . /tmp/usage.json

# Clean up
rm /tmp/usage.json
```

### TC-5: Rate Limiting Commands

**Purpose**: Verify rate limit management

#### TC-5.1: List Limits

```bash
iron-token .limits.list format::json
```

**Expected:**
- Exit code: 0
- JSON array of rate limits
- Contains configured limits

#### TC-5.2: Get Limit

```bash
iron-token .limits.get id::<limit-id>
```

**Expected:**
- Exit code: 0
- Limit details in JSON
- Matches expected configuration

#### TC-5.3: Create Limit

```bash
iron-token .limits.create type::daily value::1000
```

**Expected:**
- Exit code: 0
- Limit ID returned
- Limit visible in list

#### TC-5.4: Update Limit

```bash
iron-token .limits.update id::<limit-id> value::2000
```

**Expected:**
- Exit code: 0
- Limit updated successfully
- New value reflected in get command

#### TC-5.5: Delete Limit

```bash
iron-token .limits.delete id::<limit-id>
```

**Expected:**
- Exit code: 0
- Limit removed from list
- Cannot retrieve deleted limit

### TC-6: Trace Management Commands

**Purpose**: Verify trace export functionality

#### TC-6.1: List Traces

```bash
iron-token .traces.list format::json
```

**Expected:**
- Exit code: 0
- JSON array of traces
- Contains recent API calls

#### TC-6.2: Get Trace Details

```bash
iron-token .traces.get id::<trace-id>
```

**Expected:**
- Exit code: 0
- Trace details in JSON
- Contains request/response data

#### TC-6.3: Export Traces

```bash
iron-token .traces.export output_file::/tmp/traces.json export_format::json
```

**Expected:**
- Exit code: 0
- File created at /tmp/traces.json
- File contains valid trace export

**Validation:**
```bash
# Verify file
test -f /tmp/traces.json && echo "File created"
jq . /tmp/traces.json

# Clean up
rm /tmp/traces.json
```

### TC-7: Error Handling

**Purpose**: Verify appropriate errors for invalid input

#### TC-7.1: Invalid Command

```bash
iron-token .invalid.command
```

**Expected:**
- Exit code: 1
- Error message: "Command '.invalid.command' not recognized"
- No stack trace

#### TC-7.2: Missing Authentication

```bash
# First logout
iron-token .auth.logout

# Then try authenticated command
iron-token .tokens.list
```

**Expected:**
- Exit code: 1
- Error message: "Not authenticated"
- Suggestion to run .auth.login

#### TC-7.3: Invalid Token ID

```bash
iron-token .tokens.get id::invalid-id
```

**Expected:**
- Exit code: 1
- Error message indicates invalid/not found
- Clear error description

#### TC-7.4: Missing Required Parameter

```bash
iron-token .tokens.generate name::"Test"
# Missing required 'scope' parameter
```

**Expected:**
- Exit code: 1
- Error message indicates missing parameter
- Shows required parameters

## Validation Criteria

For each test case, verify:

- ✅ Command executes without panic
- ✅ Exit code is appropriate (0=success, 1=error)
- ✅ Output format is valid (JSON/table as specified)
- ✅ Data matches API dashboard/expected values
- ✅ Error messages are user-friendly
- ✅ No sensitive data leaked in output

## Test Execution Checklist

Before declaring manual testing complete:

- [ ] All TC-1 (health) tests pass
- [ ] All TC-2 (authentication) tests pass
- [ ] All TC-3 (token management) tests pass
- [ ] All TC-4 (usage tracking) tests pass
- [ ] All TC-5 (rate limiting) tests pass
- [ ] All TC-6 (trace management) tests pass
- [ ] All TC-7 (error handling) tests pass
- [ ] No crashes or panics observed
- [ ] Performance is acceptable (< 2s response time)
- [ ] Clean up test data created during testing

## Notes

- **Test Data Cleanup**: Delete all test tokens and limits after testing
- **Real API Required**: These tests cannot run against mocked API
- **Environment Isolation**: Use dedicated test environment to avoid production impact
- **Credentials Security**: Never commit credentials or tokens to version control
- **Test Frequency**: Run after each release candidate build
- **Failure Reporting**: Document any failures with full error output

## Integration with Automated Tests

Manual tests complement automated integration tests by:

- **Real API Validation**: Automated tests use mocked API, manual tests use real API
- **End-to-End Flow**: Manual tests verify complete user workflow
- **Network Verification**: Manual tests catch network/connectivity issues
- **Keyring Integration**: Manual tests verify actual keyring storage

Run both automated (`cargo test`) and manual tests for complete coverage.
