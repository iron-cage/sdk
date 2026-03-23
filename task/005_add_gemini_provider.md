# Task 005: Add Gemini inference provider

## Dependencies
- Task 002
- Task 003
- Task 007

## Context
Gemini support is partial and not end-to-end across provider key management, budget handshake, runtime routing, and analytics validation. Provider integration must use `api_llm` bindings where applicable, instead of introducing new direct provider-specific execution paths.

Critical areas:
- `module/iron_token_manager/src/provider_key_storage.rs`
- `module/iron_token_manager/migrations/004_create_ai_provider_keys.sql`
- `module/iron_control_api/src/routes/providers.rs`
- `module/iron_control_api/src/routes/budget/handshake.rs`
- `module/iron_control_api/src/routes/agent_provider_key.rs`
- `module/iron_control_api/src/routes/analytics/ingestion.rs`
- `module/iron_runtime/src/llm_router/key_fetcher.rs`
- `module/iron_runtime/src/llm_router/proxy.rs`

## Implementation plan
1. Define one canonical provider identifier for Gemini and apply it consistently across modules.
2. Extend provider enums, DB constraints, and route validations to accept Gemini keys.
3. Add Gemini support in provider key issuance and handshake flows.
4. Route Gemini execution through `api_llm` bindings where applicable.
   - Use the Gemini binding path as the primary provider execution mechanism.
   - Keep provider-specific auth/header and endpoint behavior inside binding adapters.
   - Avoid duplicating direct provider HTTP logic in runtime proxy paths.
5. Extend runtime proxy for Gemini routing and compatibility behavior.
   - Provider detection from path/model.
   - Base URL resolution from provider key metadata.
   - Request and response compatibility handling with existing router contracts.
6. Extend analytics provider validation and reporting for Gemini using canonical provider identifiers.
7. Add end-to-end integration tests from key creation to analytics ingestion.

## Acceptance criteria
- Gemini provider key can be created, listed, read, updated, and deleted via API.
- Handshake with `provider=gemini` succeeds when key and limits are valid.
- Runtime proxy can route Gemini requests and return successful provider responses through `api_llm` bindings where applicable.
- Analytics ingestion accepts Gemini provider events.
- Usage and spending analytics include Gemini data.
- No new direct Gemini-specific HTTP execution path is introduced outside the approved binding boundary.
- OpenAI and Anthropic regression tests remain green.
