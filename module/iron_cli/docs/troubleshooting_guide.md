# Iron CLI Troubleshooting Guide

**Version:** 1.0.0
**Last Updated:** 2025-12-12

---

## Table of Contents

1. [Authentication Errors](#authentication-errors)
2. [Connection Errors](#connection-errors)
3. [Command Errors](#command-errors)
4. [Parameter Errors](#parameter-errors)
5. [Server Errors](#server-errors)
6. [Performance Issues](#performance-issues)
7. [Debug Tools](#debug-tools)
8. [Common Solutions](#common-solutions)

---

## Authentication Errors

### Error: AUTH_MISSING_TOKEN

**Symptom:**
```
Error: HTTP request failed: API error (401):
{"error":{"code":"AUTH_MISSING_TOKEN","message":"Missing authentication token"}}
```

**Cause:** The CLI cannot find your API token.

**Solutions:**

1. Set environment variable:
   ```bash
   export IRON_API_TOKEN="your-token-here"
   ```

2. Verify token is set:
   ```bash
   echo $IRON_API_TOKEN
   ```

3. Create token if you don't have one:
   ```bash
   # Log in to get a token (requires username/password)
   curl -X POST http://localhost:3001/api/v1/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username":"admin","password":"your-password"}'
   ```

4. Check token expiration:
   ```bash
   iron .api_token.list format::json | jq '.[] | select(.name=="my-token")'
   ```

### Error: AUTH_INVALID_TOKEN

**Symptom:**
```
Error: HTTP request failed: API error (401):
{"error":{"code":"AUTH_INVALID_TOKEN","message":"Invalid or expired token"}}
```

**Cause:** Token is malformed or has expired.

**Solutions:**

1. Verify token format (should be long alphanumeric string)
2. Create new token
3. Check system time (tokens are time-sensitive)
4. Ensure no extra spaces in token value:
   ```bash
   # Wrong (has space)
   export IRON_API_TOKEN=" sk-abc123"

   # Correct
   export IRON_API_TOKEN="sk-abc123"
   ```

### Error: PERMISSION_DENIED

**Symptom:**
```
Error: HTTP request failed: API error (403):
{"error":{"code":"PERMISSION_DENIED","message":"Insufficient permissions"}}
```

**Cause:** Your user role lacks permission for this operation.

**Solutions:**

1. Check your permissions:
   ```bash
   iron .user.get_permissions id::your-user-id
   ```

2. Request elevated permissions from admin

3. Use appropriate commands for your role:
   - **Developer**: Can create budget requests, view analytics
   - **Admin**: Can approve requests, manage users, set limits

---

## Connection Errors

### Error: Connection Refused

**Symptom:**
```
Error: HTTP request failed: Connection refused
```

**Cause:** Control API server is not running or wrong URL.

**Solutions:**

1. Check if server is running:
   ```bash
   curl http://localhost:3001/api/health
   ```

2. Start the server:
   ```bash
   cd iron_runtime/dev/module/iron_control_api
   cargo run --bin iron_control_api_server
   ```

3. Verify URL configuration:
   ```bash
   echo $IRON_CONTROL_API_URL
   # Should be: http://localhost:3001
   ```

4. Set correct URL:
   ```bash
   export IRON_CONTROL_API_URL="http://localhost:3001"
   ```

5. Check server port:
   ```bash
   lsof -i :3001
   # Should show iron_control_api_server process
   ```

### Error: Network Timeout

**Symptom:**
```
Error: HTTP request failed: operation timed out
```

**Cause:** Server not responding, network issues, or slow operation.

**Solutions:**

1. Check server logs for errors
2. Verify network connectivity
3. Increase timeout (future feature)
4. Try simpler command first:
   ```bash
   iron .agent.list
   ```

### Error: DNS Resolution Failed

**Symptom:**
```
Error: HTTP request failed: failed to lookup address information
```

**Cause:** Invalid hostname in URL.

**Solutions:**

1. Use IP address instead of hostname:
   ```bash
   export IRON_CONTROL_API_URL="http://127.0.0.1:3001"
   ```

2. Check `/etc/hosts` if using custom hostname

---

## Command Errors

### Error: Command Not Found

**Symptom:**
```
Error: Command '.agent.lst' not found
```

**Cause:** Typo in command name.

**Solutions:**

1. Check spelling - command names use full words:
   ```bash
   # Wrong
   iron .agent.lst

   # Correct
   iron .agent.list
   ```

2. List available commands:
   ```bash
   iron --help
   ```

3. Common typos:
   - `.agent.lst` → `.agent.list`
   - `.provider.creat` → `.provider.create`
   - `.analytic.usage` → `.analytics.usage`

### Error: Command Recognized but Not Implemented

**Symptom:**
```
Command '.agent.list' recognized but handler not yet implemented
```

**Cause:** Command definition exists but routing/handler missing.

**Solutions:**

1. This should not happen in production
2. Check if using development version
3. Rebuild CLI:
   ```bash
   cargo build --bin iron
   ```

---

## Parameter Errors

### Error: Missing Required Parameter

**Symptom:**
```
Error: Handler error: Missing required parameter: name
```

**Cause:** Required parameter not provided.

**Solutions:**

1. Add missing parameter:
   ```bash
   # Wrong
   iron .agent.create budget::1000

   # Correct
   iron .agent.create name::my-agent budget::1000
   ```

2. Check command help:
   ```bash
   iron .agent.create --help
   ```

3. Use dry-run to test:
   ```bash
   iron .agent.create name::test budget::1000 dry::1
   ```

### Error: Invalid Parameter Value

**Symptom:**
```
Error: Handler error: Budget must be a positive integer
```

**Cause:** Parameter value doesn't meet validation rules.

**Solutions:**

1. Check value type:
   ```bash
   # Wrong (budget is text)
   iron .agent.create name::test budget::abc

   # Correct (budget is number)
   iron .agent.create name::test budget::1000
   ```

2. Check value range:
   ```bash
   # Wrong (negative budget)
   iron .agent.create name::test budget::-500

   # Correct
   iron .agent.create name::test budget::1000
   ```

3. Common validation rules:
   - Budget: Must be positive integer
   - Name: 3-50 characters, alphanumeric and hyphens
   - Email: Valid email format
   - Role: Must be "admin" or "developer"

### Error: Invalid Parameter Format

**Symptom:**
```
Error: Invalid parameter format. Use param::value
```

**Cause:** Wrong parameter syntax.

**Solutions:**

1. Use double colon (::) separator:
   ```bash
   # Wrong
   iron .agent.create name=test budget=1000
   iron .agent.create --name test --budget 1000

   # Correct
   iron .agent.create name::test budget::1000
   ```

2. No spaces around ::
   ```bash
   # Wrong
   iron .agent.create name :: test

   # Correct
   iron .agent.create name::test
   ```

---

## Server Errors

### Error: 404 Not Found

**Symptom:**
```
Error: HTTP request failed: API error (404)
```

**Cause:** Endpoint path is wrong or server routes misconfigured.

**Solutions:**

1. Check server is using `/api/v1/` prefix:
   ```bash
   curl http://localhost:3001/api/v1/agents
   ```

2. Verify server version matches CLI version

3. Check server logs for routing errors

### Error: 500 Internal Server Error

**Symptom:**
```
Error: HTTP request failed: API error (500):
{"error":{"code":"INTERNAL_ERROR","message":"Internal server error"}}
```

**Cause:** Server encountered unexpected error.

**Solutions:**

1. Check server logs for stack trace
2. Verify database is accessible
3. Check server disk space
4. Restart server:
   ```bash
   pkill iron_control_api_server
   cargo run --bin iron_control_api_server
   ```

### Error: 409 Conflict

**Symptom:**
```
Error: HTTP request failed: API error (409):
{"error":{"code":"DUPLICATE_NAME","message":"Agent name already exists"}}
```

**Cause:** Resource with same identifier already exists.

**Solutions:**

1. Use different name:
   ```bash
   iron .agent.create name::my-agent-2 budget::1000
   ```

2. List existing resources:
   ```bash
   iron .agent.list format::json | jq '.[].name'
   ```

3. Delete old resource first (if intended):
   ```bash
   iron .agent.delete id::old-agent-id
   ```

---

## Performance Issues

### Slow Command Execution

**Symptom:** Commands take >5 seconds to complete.

**Cause:** Network latency, slow server, or large dataset.

**Solutions:**

1. Test network latency:
   ```bash
   time curl http://localhost:3001/api/health
   ```

2. Test server directly:
   ```bash
   time curl http://localhost:3001/api/v1/agents
   ```

3. Use pagination for large datasets (future feature)

4. Check server resource usage:
   ```bash
   top -p $(pgrep iron_control_api_server)
   ```

### High Memory Usage

**Symptom:** CLI or server consuming excessive memory.

**Cause:** Large responses, memory leak, or resource accumulation.

**Solutions:**

1. Use filters to reduce response size (future feature)
2. Restart server periodically
3. Monitor with:
   ```bash
   ps aux | grep iron
   ```

---

## Debug Tools

### Enable Verbose Output

Currently not available, but you can:

1. Check environment variables:
   ```bash
   env | grep IRON
   ```

2. Test server health:
   ```bash
   curl -v http://localhost:3001/api/health
   ```

3. Inspect HTTP traffic with curl:
   ```bash
   curl -v -H "Authorization: Bearer $IRON_API_TOKEN" \
     http://localhost:3001/api/v1/agents
   ```

### Test Command Parsing

Use dry-run mode to test parameter parsing:

```bash
iron .agent.create name::test budget::1000 dry::1
# Output: [DRY RUN] Agent would be created (no HTTP request made)
```

If dry-run succeeds, parameters are valid.

### Check Server Logs

```bash
# If server is running in terminal, logs appear there
# Or redirect to file:
cargo run --bin iron_control_api_server > server.log 2>&1 &
tail -f server.log
```

### Test Database Connection

```bash
# Check if database file exists
ls -lh ~/.iron/iron_control.db

# Check database integrity
sqlite3 ~/.iron/iron_control.db "PRAGMA integrity_check;"
```

---

## Common Solutions

### Complete Environment Setup

```bash
# Set all required environment variables
export IRON_CONTROL_API_URL="http://localhost:3001"
export IRON_API_TOKEN="your-token-here"

# Verify
echo "URL: $IRON_CONTROL_API_URL"
echo "Token: ${IRON_API_TOKEN:0:10}..."  # Show first 10 chars only
```

### Reset Configuration

```bash
# Clear environment
unset IRON_CONTROL_API_URL
unset IRON_API_TOKEN

# Set fresh values
export IRON_CONTROL_API_URL="http://localhost:3001"
export IRON_API_TOKEN="new-token-here"
```

### Verify Installation

```bash
# Check CLI binary exists
which iron
# Or
ls -lh /path/to/iron_runtime/dev/target/debug/iron

# Check CLI version
iron --version  # (future feature)

# Test basic command
iron .agent.list dry::1  # Should not error on parsing
```

### Full System Check

```bash
#!/bin/bash
echo "=== Iron CLI System Check ==="

echo -n "CLI binary: "
[ -f ./target/debug/iron ] && echo "✓ Found" || echo "✗ Not found"

echo -n "Server running: "
curl -s http://localhost:3001/api/health > /dev/null && echo "✓ Yes" || echo "✗ No"

echo -n "API URL set: "
[ -n "$IRON_CONTROL_API_URL" ] && echo "✓ $IRON_CONTROL_API_URL" || echo "✗ Not set"

echo -n "API token set: "
[ -n "$IRON_API_TOKEN" ] && echo "✓ ${IRON_API_TOKEN:0:10}..." || echo "✗ Not set"

echo -n "Database exists: "
[ -f ~/.iron/iron_control.db ] && echo "✓ Yes" || echo "✗ No"

echo "=== End Check ==="
```

---

## Error Code Reference

| Code | HTTP | Description | Solution |
|------|------|-------------|----------|
| AUTH_MISSING_TOKEN | 401 | No token provided | Set IRON_API_TOKEN |
| AUTH_INVALID_TOKEN | 401 | Invalid or expired token | Create new token |
| PERMISSION_DENIED | 403 | Insufficient permissions | Request admin access |
| NOT_FOUND | 404 | Resource not found | Check ID is correct |
| DUPLICATE_NAME | 409 | Name already exists | Use different name |
| VALIDATION_ERROR | 400 | Invalid parameters | Fix parameter values |
| INTERNAL_ERROR | 500 | Server error | Check server logs |
| BUDGET_EXCEEDED | 400 | Budget limit reached | Request budget increase |

---

## Getting Help

If you're still stuck after trying these solutions:

1. **Check Documentation**
   - [User Guide](user_guide.md)
   - [API Reference](api_reference.md)

2. **Check Server Logs**
   - Look for stack traces
   - Note error timestamps
   - Identify failing component

3. **Create Minimal Reproduction**
   ```bash
   # Simplest possible command that fails
   iron .agent.list
   ```

4. **Gather Debug Information**
   ```bash
   # System info
   uname -a
   rustc --version

   # Environment
   env | grep IRON

   # Server status
   curl -v http://localhost:3001/api/health
   ```

5. **File Issue**
   - Include error message
   - Include debug information
   - Include steps to reproduce

---

## Prevention Tips

### 1. Always Test with Dry-Run

```bash
# Before any mutation
iron .agent.delete id::important dry::1

# Verify output, then execute
iron .agent.delete id::important
```

### 2. Use Version Control for Config

```bash
# Save working configuration
cat > ~/.iron/env.sh << 'EOF'
export IRON_CONTROL_API_URL="http://localhost:3001"
export IRON_API_TOKEN="your-token-here"
EOF

# Load when needed
source ~/.iron/env.sh
```

### 3. Monitor Server Health

```bash
# Periodic health check
while true; do
  curl -s http://localhost:3001/api/health || echo "Server down!"
  sleep 60
done
```

### 4. Backup Database

```bash
# Daily backup
cp ~/.iron/iron_control.db ~/.iron/iron_control.db.$(date +%Y%m%d)

# Retain 7 days
find ~/.iron -name "iron_control.db.*" -mtime +7 -delete
```

### 5. Test After Updates

```bash
# After updating CLI
cargo build --bin iron

# Test basic commands
iron .agent.list dry::1
iron .provider.list dry::1

# Test server connectivity
curl http://localhost:3001/api/health
```

---

## Advanced Debugging

### Trace HTTP Requests

```bash
# Install mitmproxy
pip install mitmproxy

# Run proxy
mitmproxy -p 8080

# Configure CLI to use proxy
export HTTP_PROXY="http://localhost:8080"
export HTTPS_PROXY="http://localhost:8080"

# Run commands and inspect in mitmproxy
```

### Database Inspection

```bash
# Open database
sqlite3 ~/.iron/iron_control.db

# List tables
.tables

# View schema
.schema agents

# Query data
SELECT * FROM agents;

# Exit
.quit
```

### Network Diagnostics

```bash
# Test DNS
nslookup localhost

# Test port connectivity
nc -zv localhost 3001

# Test full HTTP
curl -v http://localhost:3001/api/health

# Check routing
traceroute localhost
```

---

## Quick Fixes

| Symptom | Quick Fix |
|---------|-----------|
| Command not found | Check spelling, use `--help` |
| Missing token | `export IRON_API_TOKEN="token"` |
| Connection refused | Start server, check URL |
| Invalid token | Create new token |
| Permission denied | Check role, request admin |
| 404 Not Found | Verify server routes |
| 500 Server Error | Check server logs, restart |
| Slow performance | Check network, server load |
| Duplicate name | Use different name |
| Invalid parameters | Check types, use dry-run |

