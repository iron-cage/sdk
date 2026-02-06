# Task 005: Add Gemini inference provider

## Dependencies
- Task 002
- Task 003

## Context
Gemini support is partial and not end-to-end across provider key management, budget handshake, runtime routing, and analytics validation.

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
4. Extend runtime proxy for Gemini routing.
   - Provider detection from path/model.
   - Base URL resolution.
   - Gemini auth header policy.
   - Request and response compatibility handling.
5. Extend analytics provider validation and reporting for Gemini.
6. Add end-to-end integration tests from key creation to analytics ingestion.

## Acceptance criteria
- Gemini provider key can be created, listed, read, updated, and deleted via API.
- Handshake with `provider=gemini` succeeds when key and limits are valid.
- Runtime proxy can route Gemini requests and return successful provider responses.
- Analytics ingestion accepts Gemini provider events.
- Usage and spending analytics include Gemini data.
- OpenAI and Anthropic regression tests remain green.
