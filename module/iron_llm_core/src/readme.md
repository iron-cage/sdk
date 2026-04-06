# iron_llm_core/src

Shared LLM routing core: provider detection, request/response translation, and forwarding.

## Responsibility Table

| File | Responsibility |
|------|---------------|
| lib.rs | Module declarations, public re-exports, and OpenAI-compatible error response helper |
| forward.rs | Core HTTP forwarding logic: build provider URL, send request, translate response |
| provider.rs | Provider detection from path prefix (`/anthropic/...`) or model name in request body |
| translator.rs | Bidirectional format translation between OpenAI and Anthropic API formats |
| cost.rs | Request cost calculation using token counts and pricing data |
| error.rs | `LlmCoreError` enum for translation and forwarding failures |
