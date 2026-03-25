# Task 005: Add Gemini inference provider

## Goal

Add end-to-end Gemini provider support across key management, budget handshake, runtime routing, and analytics so that agents can use Gemini models through the platform. The result is observable through successful API operations on Gemini provider keys and successful Gemini inference requests routed through `api_llm` bindings. Scoped to Gemini as a new provider variant using the binding adapter layer - no new direct HTTP execution paths. Testable by creating a Gemini key, performing a handshake, routing a request, and verifying analytics ingestion.

## Dependencies
- Task 002
- Task 003
- Task 007

## In Scope

- Canonical Gemini provider identifier applied consistently across all modules
- Provider enum, DB constraint, and route validation extensions for Gemini
- Gemini support in provider key issuance and handshake flows
- Gemini execution routed through `api_llm` bindings via the `ProviderBinding` trait
- Runtime proxy extensions for Gemini routing (provider detection, base URL resolution, compatibility)
- Analytics provider validation and reporting for Gemini events

## Out of Scope

- Direct Gemini-specific HTTP execution paths outside the binding adapter layer
- Gemini-specific features beyond standard chat completion (e.g., multimodal, code execution)
- Changes to existing OpenAI or Anthropic binding implementations
- Gemini model-specific pricing or cost calculation changes

## Description

Gemini support is currently partial and not integrated end-to-end across the platform. While some provider infrastructure exists, Gemini cannot be used through the full flow of key creation, budget handshake, runtime request routing, and analytics reporting. This task completes the Gemini integration by defining a canonical provider identifier and extending all relevant modules to recognize and handle Gemini as a first-class provider.

The implementation leverages the `api_llm` binding layer established by Task 007. A Gemini `ProviderBinding` implementation is registered with the `ProviderRegistry`, routing all Gemini execution through the standardized binding interface. This avoids introducing parallel direct-HTTP execution paths and maintains architectural consistency with existing providers.

Runtime proxy extensions handle Gemini-specific concerns such as provider detection from path/model patterns, base URL resolution from provider key metadata, and request/response compatibility with the existing router contracts. Analytics validation is extended to accept and report on Gemini provider events using the canonical identifier.

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

## Work Procedure

1. Define the canonical Gemini provider identifier and add it to provider enums across modules
2. Update DB constraints and migration to accept Gemini as a valid provider value
3. Extend route validations in `providers.rs` and `agent_provider_key.rs` for Gemini
4. Add Gemini to provider key issuance flow - creation, listing, reading, updating, deletion
5. Add Gemini support in handshake flow - validate `provider=gemini` and resolve key
6. Implement Gemini `ProviderBinding` using `api_gemini` bindings from `api_llm`
7. Register Gemini binding in `ProviderRegistry`
8. Extend runtime proxy for Gemini routing - provider detection, base URL resolution, compatibility
9. Extend analytics provider validation to accept and report Gemini events
10. Write end-to-end integration tests from key creation through analytics ingestion

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

## Test Matrix

| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Create Gemini provider key | Key created with unique ID | API returns 201 with `provider_key_id` |
| List provider keys including Gemini | Gemini keys appear in list | Response includes Gemini key entries |
| Read Gemini provider key | Key details returned | Response contains correct provider and metadata |
| Update Gemini provider key | Key updated | Subsequent read reflects changes |
| Delete Gemini provider key | Key removed | Subsequent read returns 404 |
| Handshake with `provider=gemini` and valid key | Handshake succeeds | Budget reservation granted |
| Handshake with `provider=gemini` and no key | Handshake rejected | Missing key error returned |
| Route Gemini request through runtime proxy | Response from Gemini returned | Successful provider response via binding |
| Submit Gemini analytics event | Event accepted | Analytics ingestion returns 200 |
| Query analytics including Gemini data | Gemini spending included | Aggregation includes Gemini provider entries |
| OpenAI request after Gemini integration | OpenAI still works | No regression in OpenAI flow |
| Anthropic request after Gemini integration | Anthropic still works | No regression in Anthropic flow |

## Validation Checklist

- [ ] Canonical Gemini provider identifier defined and consistent across modules
- [ ] Provider enums, DB constraints, and route validations accept Gemini
- [ ] Gemini provider key CRUD operations functional
- [ ] Handshake with `provider=gemini` succeeds when key and limits are valid
- [ ] Gemini requests routed through `api_llm` bindings via `ProviderBinding` trait
- [ ] No direct Gemini HTTP execution path outside binding boundary
- [ ] Analytics ingestion accepts Gemini events
- [ ] Usage and spending analytics include Gemini data
- [ ] OpenAI and Anthropic regression tests pass

## Validation Procedure

1. Run existing test suite to establish baseline - OpenAI and Anthropic tests pass
2. Create a Gemini provider key via API - verify successful creation
3. List provider keys - verify Gemini key appears alongside existing providers
4. Perform handshake with `provider=gemini` - verify budget reservation succeeds
5. Route a Gemini chat completion request through the runtime proxy - verify successful response
6. Verify the request flowed through the `ProviderBinding` trait implementation (no direct HTTP)
7. Submit a Gemini analytics event - verify ingestion succeeds
8. Query analytics - verify Gemini spending data appears in aggregation
9. Run OpenAI and Anthropic integration tests - verify no regressions
10. Run full test suite and confirm zero warnings under workspace lint policy

## Acceptance Criteria
- Gemini provider key can be created, listed, read, updated, and deleted via API.
- Handshake with `provider=gemini` succeeds when key and limits are valid.
- Runtime proxy can route Gemini requests and return successful provider responses through `api_llm` bindings where applicable.
- Analytics ingestion accepts Gemini provider events.
- Usage and spending analytics include Gemini data.
- No new direct Gemini-specific HTTP execution path is introduced outside the approved binding boundary.
- OpenAI and Anthropic regression tests remain green.
