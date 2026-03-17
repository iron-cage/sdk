# iron_llm_core

Shared LLM request forwarding, provider detection, and format translation.

## Features

- `enabled` (default): Full forwarding and translation functionality
- `full`: All functionality (currently same as `enabled`)


## Quick Start

```rust
use iron_llm_core::{forward_request, ForwardRequest, detect_provider_from_model};

// Detect provider from model name
let provider = detect_provider_from_model(b"{\"model\":\"claude-opus-4-6\"}");
assert_eq!(provider, Some("anthropic"));

// Translate OpenAI request to Anthropic format
let translated = iron_llm_core::translate_openai_to_anthropic(openai_body)?;
```


<details>
<summary>Scope & Boundaries</summary>

**Responsibilities:**
Core LLM routing logic shared by `iron_runtime` (embedded proxy) and `iron_server_proxy` (centralized server proxy). Handles provider detection, request/response format translation between OpenAI, Anthropic, and Gemini, HTTP forwarding with provider-specific auth headers, and cost calculation.

**In Scope:**
- Provider detection from path prefix (`/anthropic/`, `/openai/`, `/gemini/`, `/xai/`) or model name
- Request translation: OpenAI -> Anthropic (including tools/tool_choice)
- Request translation: OpenAI -> Gemini (including tools/toolConfig)
- Response translation: Anthropic -> OpenAI
- Response translation: Gemini -> OpenAI
- Provider-specific auth headers (`x-api-key`, `x-goog-api-key`, `Bearer`)
- Cost calculation from request/response token usage
- Streaming pass-through

**Out of Scope:**
- IC Token authentication (see `iron_server_proxy`)
- Budget enforcement (see `iron_server_proxy`)
- Agent lifecycle (see `iron_runtime`)
- Provider key storage/encryption (see `iron_token_manager`, `iron_secrets`)

</details>


<details>
<summary>Directory Structure</summary>

### Source Files

| File | Responsibility |
|------|----------------|
| lib.rs | Crate root, re-exports primary types and functions. |
| forward.rs | Core request forwarding: provider detection, translation dispatch, HTTP send, response handling. |
| provider.rs | Provider detection from URL path prefix and model name in request body. |
| translator/mod.rs | Module root for request/response translators. |
| translator/request.rs | OpenAI -> Anthropic and OpenAI -> Gemini request translation (messages, tools, tool_choice). |
| translator/response.rs | Anthropic -> OpenAI and Gemini -> OpenAI response translation (content, tool_calls, usage). |
| cost.rs | Cost calculation from request/response token usage via pricing manager. |
| error.rs | Error types for translation and forwarding failures. |

</details>


## License

MIT
