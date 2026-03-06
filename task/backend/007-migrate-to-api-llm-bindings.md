# Task 007: Migrate to `api_llm` bindings

## Dependencies
- None (foundation task; tasks 005 and 006 depend on this)

## Context
The runtime executes all LLM provider requests through raw `reqwest` HTTP calls in `proxy.rs`, with manual auth headers, hardcoded provider URLs, hand-rolled request/response translation, and no streaming support. The `api_llm` workspace (`api_openai`, `api_claude`, `api_gemini`) provides tested, typed HTTP API bindings, but these dependencies are declared in the workspace `Cargo.toml` and not actually used — `iron_token_manager` has them commented out due to a compilation blocker, and `iron_runtime` has no `api_*` dependencies at all.

This task establishes a binding adapter layer for OpenAI and Anthropic, migrates existing providers to it, removes all legacy direct-HTTP code, and makes the layer extensible so that tasks 005/006 can plug in Gemini and xAI by adding a single binding implementation per provider.

Critical areas:
- `module/iron_token_manager/src/provider_adapter.rs`
- `module/iron_token_manager/src/provider_key_storage.rs`
- `module/iron_token_manager/Cargo.toml`
- `module/iron_runtime/src/llm_router/proxy.rs`
- `module/iron_runtime/src/llm_router/translator/request.rs`
- `module/iron_runtime/src/llm_router/translator/response.rs`
- `module/iron_runtime/src/llm_router/key_fetcher.rs`
- `module/iron_runtime/src/llm_router/router.rs`
- `module/iron_runtime/src/llm_router/error.rs`
- `module/iron_runtime/src/llm_router/mod.rs`
- `module/iron_runtime/Cargo.toml`
- `module/iron_runtime_analytics/src/provider_utils.rs`
- `Cargo.toml`

## Implementation plan
1. Unblock `api_llm` crate compilation in the workspace.
   - Verify `api_openai` and `api_claude` compile cleanly under workspace `#[deny(warnings)]` policy.
   - If any crate emits warnings, fix upstream in `api_llm` or pin a fixed version.
   - Confirm workspace `Cargo.toml` versions are current (`api_openai ~0.3.0`, `api_claude ~0.4.0`).
2. Enable `api_*` dependencies in `iron_token_manager` and `iron_runtime`.
   - Uncomment `api_openai`, `api_claude` in `iron_token_manager/Cargo.toml`.
   - Add `api_openai`, `api_claude` dependencies to `iron_runtime/Cargo.toml`.
   - Gate behind optional feature flags (e.g., `provider-bindings`).
   - Verify that `api_openai` and `api_claude` client constructors accept API keys and apply auth headers automatically.
3. Extract `ProviderType` enum to a shared crate (e.g., `iron_types`) or define it in `iron_runtime::llm_router` with `From`/`Into` conversions to the existing enums in `provider_key_storage.rs` and `provider_utils.rs`.
   - The enum must be extensible for future variants (Gemini, xAI) added by tasks 005/006.
   - Replace string-based provider matching in `proxy.rs` with the typed enum.
4. Define a `ProviderBinding` trait in `iron_runtime::llm_router` (e.g., new `binding.rs` module).
   - Methods: `chat_completion`, `chat_completion_stream`, `extract_usage`, `provider_name`.
   - Crate-local `ProviderResponse` and `ProviderChunk` types wrapping common denominator (status, body, usage tokens, finish reason).
   - Reuse or migrate `UsageMetadata` from `iron_token_manager::provider_adapter`.
   - This trait is the single boundary through which all LLM calls must flow.
5. Implement `ProviderBinding` for OpenAI and Anthropic.
   - Each implementation constructs provider-native requests, sends via `api_*` client, maps responses back.
   - Auth headers delegated to `api_*` crates — no manual `x-api-key` or `Bearer` in runtime.
   - Streaming via `api_*` SSE support.
   - The Anthropic binding must accept OpenAI-format chat completion requests and handle translation internally (absorb `translator/request.rs` and `translator/response.rs` logic).
6. Create a `ProviderRegistry` dispatcher mapping `ProviderType` enum to `ProviderBinding` implementations.
   - Support runtime lookup: `registry.get(ProviderType::OpenAI) -> &dyn ProviderBinding`.
   - Constructed in `LlmRouter::create_inner`, stored in `ProxyState`, passed to handlers via `axum::State`.
   - Extensible: tasks 005/006 add Gemini/xAI by registering new bindings.
7. Wire `ProviderRegistry` into `LlmRouter` and replace raw reqwest calls.
   - After provider detection and key fetch, call `registry.get(provider).chat_completion(...)`.
   - Preserve budget check/reservation flow (pre-binding) and cost commit flow (post-binding).
   - Preserve analytics recording using `UsageMetadata` from binding response.
   - Update `fetch_budget_from_handshake` to use detected `ProviderType` instead of hardcoded `"openai"`.
8. Implement streaming through the binding layer.
   - Replace `bytes().await` (full body read) with `chat_completion_stream()` async stream.
   - Forward SSE chunks to client as they arrive.
   - Track token counts from final stream chunk usage metadata.
   - Enforce budget pre-request; record actual cost from stream's final usage data.
9. Remove legacy provider-specific code from runtime.
   - Delete hardcoded base URLs (`api.openai.com`, `api.anthropic.com`) from `proxy.rs`.
   - Delete manual auth header construction from `proxy.rs`.
   - Remove translator module from `proxy.rs` level (logic absorbed into Anthropic binding in step 5).
   - Delete `detect_provider_from_key()` from `key_fetcher.rs`; use `provider` field from server response.
   - Replace raw JSON usage extraction (`serde_json::from_slice` + `response.get("usage")`) and remove `CostInfo` / `calculate_request_cost` from `proxy.rs`.
   - Extend `LlmRouterError` with binding-layer error variants.
10. Clean up stale templates and dead code.
    - Remove `TrackedOpenAIClient` / `TrackedClaudeClient` / `TrackedGeminiClient` templates in `provider_adapter.rs`.
    - Remove blocker comments referencing dead-code warnings.
    - Evaluate `reqwest` scope in `iron_runtime` (retain `blocking` feature for shutdown budget-return path; retain for key_fetcher and budget handshake calls).
    - Remove unused imports and `#[allow(dead_code)]` annotations.
11. Add end-to-end regression tests.
    - Verify OpenAI and Anthropic integration tests pass through the binding layer.
    - Verify streaming, budget enforcement, and cost calculation parity with legacy path.
    - Verify analytics events include correct provider, model, and token counts.
    - Workspace compiles with zero warnings under existing lint policy.

## Acceptance criteria
- `api_openai` and `api_claude` compile without warnings in the workspace.
- A `ProviderBinding` trait exists and is implemented for OpenAI and Anthropic.
- A `ProviderRegistry` dispatches to the correct binding by `ProviderType` and is extensible for future providers.
- `LlmRouter` uses the binding layer for all LLM execution — zero direct `reqwest` calls to provider APIs remain in `proxy.rs`.
- No hardcoded provider URLs anywhere in the codebase.
- No manual provider auth headers constructed in runtime code.
- The translator module is removed from the proxy level (logic encapsulated within the Anthropic binding).
- Provider detection uses `ProviderType` enum, not string matching.
- Streaming works end-to-end for OpenAI and Anthropic via the binding layer.
- Usage metadata is extracted via typed binding responses, not raw JSON parsing.
- Analytics events correctly reflect provider identity from the binding layer.
- `reqwest` remains in `iron_runtime` for key_fetcher, budget handshake, and budget return operations.
- No dead code, blocker comments, or template stubs remain in migration-affected files.
- Workspace compiles with zero warnings under existing lint policy.
- All existing integration tests pass without regression.
