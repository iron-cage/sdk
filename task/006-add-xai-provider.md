# Task 006: Add xAI inference provider

## Dependencies
- Task 002
- Task 003
- Task 007

## Context
xAI support is absent in provider key management, handshake validation, runtime routing, and analytics provider validation. Provider integration must use `api_llm` bindings where applicable, instead of adding a parallel direct provider execution path.

Critical areas:
- `module/iron_token_manager/src/provider_key_storage.rs`
- `module/iron_token_manager/migrations/004_create_ai_provider_keys.sql`
- `module/iron_control_api/src/routes/providers.rs`
- `module/iron_control_api/src/routes/budget/handshake.rs`
- `module/iron_control_api/src/routes/analytics/ingestion.rs`
- `module/iron_runtime/src/llm_router/key_fetcher.rs`
- `module/iron_runtime/src/llm_router/proxy.rs`

## Implementation plan
1. Add xAI to the canonical provider registry used by control API, token manager, and runtime.
2. Extend DB constraints and provider validations to support xAI keys.
3. Add xAI support in handshake and key-fetch flows.
4. Route xAI execution through `api_llm` bindings where applicable.
   - Use native xAI binding if available in `api_llm`.
   - If native binding is not available, use `api_llm` OpenAI-compatible execution path as transitional adapter.
   - Keep xAI auth/header and endpoint behavior within binding adapters, not inline runtime branches.
5. Extend runtime proxy for xAI routing.
   - Provider detection from model/path.
   - Endpoint resolution and compatibility handling aligned with binding contracts.
6. Extend analytics provider validation and usage/spending aggregation for xAI.
7. Verify compatibility behavior for OpenAI-compatible request format where applicable.
8. Add end-to-end tests for xAI flow and regressions for existing providers.

## Acceptance criteria
- xAI provider key can be created, listed, read, updated, and deleted via API.
- Handshake with `provider=xai` succeeds when key and limits are valid.
- Runtime proxy routes xAI requests correctly and returns provider responses via `api_llm` bindings where applicable.
- Analytics ingestion accepts xAI events and reporting includes xAI data.
- No new direct xAI-specific HTTP execution path is introduced outside the approved binding boundary.
- Existing provider flows (OpenAI, Anthropic, Gemini if enabled) are not regressed.
