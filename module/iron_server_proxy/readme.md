# iron_server_proxy

Centralized LLM proxy server with IC Token authentication and spending controls.

## Features

- `enabled` (default): Full proxy server functionality
- `full`: All functionality (currently same as `enabled`)


## Quick Start

```bash
# Set required environment variables
export DATABASE_URL=sqlite:///path/to/iron_cage.db
export IRON_SECRETS_MASTER_KEY=<base64-encoded-32-byte-key>
export PROXY_PORT=8081

# Run the proxy server
iron_server_proxy
```

Agents send OpenAI-format requests to the proxy, authenticated via IC Token:

```bash
curl https://host/proxy/v1/chat/completions \
  -H "Authorization: Bearer <IC_TOKEN>" \
  -d '{"model":"claude-opus-4-6","messages":[{"role":"user","content":"Hello"}]}'
```


<details>
<summary>Scope & Boundaries</summary>

**Responsibilities:**
Centralized server proxy that authenticates agents via IC Token, decrypts provider API keys, enforces spending caps with atomic budget reservations, and forwards LLM requests to providers. Supports OpenAI, Anthropic, Gemini, and xAI via `iron_llm_core` translation.

**In Scope:**
- IC Token authentication (SHA-256 hash lookup with constant-time comparison)
- Provider key decryption (AES-256-GCM via `iron_secrets`)
- Atomic spending cap enforcement (pre-flight reservation, post-flight adjustment)
- Request forwarding via `iron_llm_core`
- IP-based auth rate limiting
- Health check endpoint

**Out of Scope:**
- User/RBAC management (see `iron_control_api`)
- Provider key CRUD (see `iron_control_api`)
- Agent management (see `iron_control_api`)
- Format translation logic (see `iron_llm_core`)
- Embedded/local proxy mode (see `iron_runtime`)

</details>


<details>
<summary>Directory Structure</summary>

### Source Files

| File | Responsibility |
|------|----------------|
| lib.rs | Crate root, re-exports AppState and AuthRateLimiter for test access. |
| main.rs | Binary entry point, loads config and starts the server. |
| server.rs | Axum router setup and server startup. |
| config.rs | CLI argument parsing and environment variable configuration. |
| proxy.rs | Core proxy handler: IC Token auth, key decryption, forwarding, cost tracking. |
| state.rs | Shared application state (DB pool, crypto service, pricing manager, HTTP client). |
| rate_limiter.rs | IP-based auth failure rate limiter (sliding window, LRU eviction). |
| error.rs | Proxy error types mapped to HTTP status codes (401, 402, 403, 429, 502). |

</details>


## License

MIT
