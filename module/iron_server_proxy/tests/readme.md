# iron_server_proxy/tests

End-to-end and unit tests for the server-side LLM proxy.

## Responsibility Table

| File | Responsibility |
|------|---------------|
| e2e_smoke.rs | Full pipeline e2e tests: IC Token auth, forwarding to mock LLM, error responses |
| rate_limiter.rs | Unit tests for per-IP auth rate limiter: threshold, expiry, independence |
