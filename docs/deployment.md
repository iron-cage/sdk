# Iron Cage Internal Deployment Runbook

Operational guide for deploying and maintaining Iron Cage as an internal centralized
token control platform.

## Service Architecture

```
                          +--------------------+
                          |    nginx (TLS)     |
                          |   ports 443/8443   |
                          +----+----------+----+
                               |          |
                    /api/*     |          |        /v1/*
                               v          v
                   +-----------+--+  +----+-------------+
                   | control_api  |  | server_proxy     |
                   | (port 3000)  |  | (port 8081)      |
                   |              |  |                  |
                   | - User/RBAC  |  | - IC Token auth  |
                   | - Key CRUD   |  | - Key decryption |
                   | - Agent mgmt |  | - LLM forwarding |
                   | - IC Tokens  |  | - Cost tracking  |
                   +-----------+--+  +----+-------------+
                               |          |
                               v          v
                   +-----------+----------+-------------+
                   |           SQLite database          |
                   |        (shared, single file)       |
                   +------------------------------------+
```

Both services share the same SQLite database and `IRON_SECRETS_MASTER_KEY`.

## Environment Variables

| Variable | Required | Used by | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | yes | both | SQLite connection string, e.g. `sqlite:///var/lib/iron_cage/iron.db?mode=rwc` |
| `IRON_SECRETS_MASTER_KEY` | yes | both | Base64-encoded 32-byte AES-256-GCM key for provider key encryption |
| `JWT_SECRET` | yes | control_api | Secret for JWT token signing (must not be the default) |
| `IC_TOKEN_SECRET` | yes | control_api | Secret for IC Token HMAC generation |
| `IP_TOKEN_KEY` | yes | control_api | Hex-encoded 32-byte AES-256-GCM key for IP Token encryption |
| `SERVER_PORT` | no | control_api | Control API listen port (default: 3000) |
| `PROXY_PORT` | no | server_proxy | Proxy listen port (default: 8081) |
| `BIND_ADDR` | no | server_proxy | Proxy bind address (default: `0.0.0.0`) |
| `IRON_DEPLOYMENT_MODE` | recommended | control_api | `production` / `development` / `pilot` |
| `RUST_LOG` | no | both | Log level filter (default: `info`) |

## Initial Setup (First Deployment)

### 1. Generate cryptographic secrets

```bash
# IRON_SECRETS_MASTER_KEY (32 random bytes, base64)
openssl rand -base64 32

# JWT_SECRET (64 random bytes, hex)
openssl rand -hex 64

# IC_TOKEN_SECRET (64 random bytes, hex)
openssl rand -hex 64

# IP_TOKEN_KEY (32 random bytes, hex - must be exactly 64 hex chars)
openssl rand -hex 32
```

Store all secrets in a `.env` file or secrets manager. Never commit them to git.

### 2. Prepare the database directory

```bash
sudo mkdir -p /var/lib/iron_cage
sudo chown iron_cage:iron_cage /var/lib/iron_cage
```

### 3. Start control_api (applies migrations automatically)

```bash
export IRON_DEPLOYMENT_MODE=production
export DATABASE_URL="sqlite:///var/lib/iron_cage/iron.db?mode=rwc"
export IRON_SECRETS_MASTER_KEY="<base64 key from step 1>"
export JWT_SECRET="<hex from step 1>"
export IC_TOKEN_SECRET="<hex from step 1>"
export IP_TOKEN_KEY="<hex from step 1>"

./iron_control_api_server
```

The server applies all pending SQLite migrations on startup (idempotent).

### 4. Start server_proxy

```bash
export DATABASE_URL="sqlite:///var/lib/iron_cage/iron.db?mode=rwc"
export IRON_SECRETS_MASTER_KEY="<same base64 key as control_api>"

./iron_server_proxy
```

### 5. Create the first Admin account

```bash
iron-init-admin -d "sqlite:///var/lib/iron_cage/iron.db?mode=rwc"
```

Credentials are saved to `.persistent/internal_deployment.md` (chmod 600).
Store this file securely - the password is shown only once.

### 6. Configure TLS (nginx reverse proxy)

```nginx
upstream control_api {
    server 127.0.0.1:3000;
}

upstream server_proxy {
    server 127.0.0.1:8081;
}

server {
    listen 443 ssl;
    server_name iron.internal.company.com;

    ssl_certificate     /etc/ssl/certs/iron_cage.pem;
    ssl_certificate_key /etc/ssl/private/iron_cage.key;

    # Control API
    location /api/ {
        proxy_pass http://control_api;
    }

    # LLM Proxy
    location /v1/ {
        proxy_pass http://server_proxy;
        proxy_read_timeout 300s;  # LLM responses can be slow
    }

    # Health checks
    location /health {
        proxy_pass http://server_proxy;
    }
}
```

## Startup

```bash
# 1. Start control_api first (runs migrations)
./iron_control_api_server &

# 2. Start server_proxy
./iron_server_proxy &
```

Order matters: `control_api` creates/migrates the database schema that `server_proxy` reads.

## Shutdown

Both services support graceful shutdown via `SIGINT` (Ctrl+C) or `SIGTERM`:

```bash
kill -TERM <control_api_pid>
kill -TERM <server_proxy_pid>
```

The server_proxy finishes in-flight LLM requests before exiting (up to 5 min timeout).

## IC Token Rotation

IC Tokens authenticate agents against the proxy. Each agent has one active IC Token.

### Generate a new IC Token for an agent

```bash
# Via CLI (requires admin credentials)
iron ic-token generate --agent-id <agent_id>

# Via API
curl -X POST https://iron.internal/api/agents/<agent_id>/ic-token \
  -H "Authorization: Bearer <admin_jwt>" \
  -H "Content-Type: application/json"
```

The response contains the raw IC Token (shown once). The SHA-256 hash is stored in DB.

### Distribute the new token to the agent

Update the agent's configuration with the new token. The agent uses it as a
`Bearer` token in the `Authorization` header (or via `x-api-key` header).

### Revoke the old token

Generating a new IC Token automatically overwrites the previous hash in the
`agents.ic_token_hash` column. No manual revocation step is needed.

To revoke without issuing a new token:

```bash
curl -X DELETE https://iron.internal/api/agents/<agent_id>/ic-token \
  -H "Authorization: Bearer <admin_jwt>"
```

## Provider Key (IP Key) Rotation

Provider keys are encrypted API keys for LLM providers (OpenAI, Anthropic).

### Register a new provider key

```bash
curl -X POST https://iron.internal/api/keys \
  -H "Authorization: Bearer <admin_jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "openai",
    "api_key": "sk-new-key-here",
    "description": "Production OpenAI key (rotated 2026-03)",
    "base_url": null
  }'
```

The API encrypts the key with AES-256-GCM before storing it. The raw key is
never stored in plaintext.

### Assign the key to agents

```bash
# Update agent to use the new key
curl -X PATCH https://iron.internal/api/agents/<agent_id> \
  -H "Authorization: Bearer <admin_jwt>" \
  -H "Content-Type: application/json" \
  -d '{"provider_key_id": <new_key_id>}'
```

### Disable the old key

```bash
curl -X PATCH https://iron.internal/api/keys/<old_key_id> \
  -H "Authorization: Bearer <admin_jwt>" \
  -H "Content-Type: application/json" \
  -d '{"is_enabled": false}'
```

Disabled keys reject all proxy requests immediately with 403.

### Delete the old key (optional)

```bash
curl -X DELETE https://iron.internal/api/keys/<old_key_id> \
  -H "Authorization: Bearer <admin_jwt>"
```

## Troubleshooting

### Agent cannot connect (401 Unauthorized)

1. Verify the IC Token is correct: `Authorization: Bearer <ic_token>`
2. Check that the agent exists and has `ic_token_hash` set:
   ```bash
   sqlite3 /var/lib/iron_cage/iron.db \
     "SELECT id, name, ic_token_hash FROM agents WHERE ic_token_hash IS NOT NULL"
   ```
3. Verify the token hash matches: the proxy hashes the raw token with SHA-256
   and looks it up in the `agents` table.
4. Check proxy logs: `RUST_LOG=debug ./iron_server_proxy`

### Budget exceeded (402 Payment Required)

The response body contains `{"error": {"code": "spending_cap_exceeded"}}`.

1. Check current spending:
   ```bash
   sqlite3 /var/lib/iron_cage/iron.db \
     "SELECT id, spending_used_microdollars, spending_cap_microdollars \
      FROM ai_provider_keys WHERE id = <key_id>"
   ```
2. Increase or remove the cap:
   ```bash
   curl -X PATCH https://iron.internal/api/keys/<key_id> \
     -H "Authorization: Bearer <admin_jwt>" \
     -H "Content-Type: application/json" \
     -d '{"spending_cap_microdollars": 50000000}'
   ```
   Set to `null` to remove the cap entirely.

### Checking logs

Both services output structured logs to stdout. Use `RUST_LOG` to control verbosity:

```bash
# Default (info)
RUST_LOG=info ./iron_server_proxy

# Debug level for proxy module only
RUST_LOG=iron_server_proxy=debug ./iron_server_proxy

# Trace all modules
RUST_LOG=trace ./iron_server_proxy
```

Key log events:
- `iron_server_proxy listening on ...` - proxy started successfully
- `LLM request forwarded` - request sent to provider (includes status, provider, translation flag)
- `Shutdown signal received` - graceful shutdown initiated
