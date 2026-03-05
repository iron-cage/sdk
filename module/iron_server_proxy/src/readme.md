# iron_server_proxy/src

Server-side LLM proxy: accepts agent HTTP requests, authenticates via IC Token, decrypts provider keys, and forwards to LLM providers.

## Responsibility Table

| File | Responsibility |
|------|---------------|
| main.rs | Binary entry point: env loading, logging init, config parse, server start |
| lib.rs | Crate root: module declarations and public re-exports |
| config.rs | CLI/env configuration via clap (database URL, ports, master key) |
| server.rs | Axum router setup, TCP listener binding, graceful shutdown |
| proxy.rs | Core request pipeline: IC Token auth, key lookup, decrypt, forward, cost tracking |
| state.rs | Shared application state: DB pool, crypto service, HTTP client, pricing manager |
| rate_limiter.rs | Per-IP rate limiting for failed authentication attempts |
| error.rs | `ServerError` (startup) and `ProxyError` (per-request) error types with HTTP responses |
