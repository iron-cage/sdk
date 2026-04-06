# Task 006: Add xAI inference provider

## Goal

Add end-to-end xAI provider support across key management, budget handshake, runtime routing, and analytics so that agents can use xAI models through the platform. The result is observable through successful API operations on xAI provider keys and successful xAI inference requests routed through `api_llm` bindings. Scoped to xAI as a new provider variant using either native `api_llm` bindings or the OpenAI-compatible execution path as a transitional adapter. Testable by creating an xAI key, performing a handshake, routing a request, and verifying analytics ingestion.

## Dependencies
- Task 002
- Task 003
- Task 007

## In Scope

- Adding xAI to the canonical provider registry across control API, token manager, and runtime
- DB constraints and provider validation extensions for xAI keys
- xAI support in handshake and key-fetch flows
- xAI execution through `api_llm` bindings (native or OpenAI-compatible adapter)
- Runtime proxy extensions for xAI routing (provider detection, endpoint resolution)
- Analytics provider validation and usage/spending aggregation for xAI

## Out of Scope

- Direct xAI-specific HTTP execution paths outside the binding adapter layer
- xAI-specific features beyond standard chat completion
- Changes to existing OpenAI, Anthropic, or Gemini binding implementations
- xAI model-specific pricing or cost calculation beyond standard token counting

## Description

xAI is currently absent from the platform across all layers - provider key management, handshake validation, runtime routing, and analytics. This task adds xAI as a first-class provider so that agents can use xAI models (such as Grok) through the existing platform infrastructure.

The implementation follows the same pattern established by the Gemini integration (Task 005) and the binding adapter layer (Task 007). If a native xAI binding is available in `api_llm`, it is used directly. If not, the `api_llm` OpenAI-compatible execution path serves as a transitional adapter, since xAI's API is largely OpenAI-compatible. All xAI auth/header and endpoint behavior is contained within the binding adapter, not in inline runtime branches.

Provider detection from model/path, endpoint resolution, and compatibility handling are aligned with the binding contracts established by the `ProviderRegistry`. Analytics validation and spending aggregation are extended to include xAI, ensuring complete observability. Existing provider flows for OpenAI, Anthropic, and Gemini are verified against regressions.

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

## Work Procedure

1. Add xAI to the canonical provider registry - enums, string constants, and display implementations
2. Update DB constraints and migration to accept xAI as a valid provider value
3. Extend provider validations in route handlers to support xAI keys
4. Add xAI to provider key issuance flow - creation, listing, reading, updating, deletion
5. Add xAI support in handshake and key-fetch flows - validate `provider=xai` and resolve key
6. Implement xAI `ProviderBinding` using native `api_llm` binding or OpenAI-compatible adapter
7. Register xAI binding in `ProviderRegistry`
8. Extend runtime proxy for xAI routing - provider detection from model/path, endpoint resolution
9. Extend analytics provider validation and usage/spending aggregation for xAI
10. Write end-to-end integration tests for xAI flow and regression tests for existing providers

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

## Test Matrix

| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Create xAI provider key | Key created with unique ID | API returns 201 with `provider_key_id` |
| List provider keys including xAI | xAI keys appear in list | Response includes xAI key entries |
| Read xAI provider key | Key details returned | Response contains correct provider and metadata |
| Update xAI provider key | Key updated | Subsequent read reflects changes |
| Delete xAI provider key | Key removed | Subsequent read returns 404 |
| Handshake with `provider=xai` and valid key | Handshake succeeds | Budget reservation granted |
| Handshake with `provider=xai` and no key | Handshake rejected | Missing key error returned |
| Route xAI request through runtime proxy | Response from xAI returned | Successful provider response via binding |
| Submit xAI analytics event | Event accepted | Analytics ingestion returns 200 |
| Query analytics including xAI data | xAI spending included | Aggregation includes xAI provider entries |
| OpenAI-compatible request format to xAI | Request handled correctly | Compatibility verified |
| OpenAI request after xAI integration | OpenAI still works | No regression |
| Anthropic request after xAI integration | Anthropic still works | No regression |
| Gemini request after xAI integration | Gemini still works | No regression |

## Validation Checklist

- [ ] xAI added to canonical provider registry
- [ ] Provider enums, DB constraints, and route validations accept xAI
- [ ] xAI provider key CRUD operations functional
- [ ] Handshake with `provider=xai` succeeds when key and limits are valid
- [ ] xAI requests routed through `api_llm` bindings via `ProviderBinding` trait
- [ ] No direct xAI HTTP execution path outside binding boundary
- [ ] Analytics ingestion accepts xAI events
- [ ] Usage and spending analytics include xAI data
- [ ] Existing provider regression tests pass (OpenAI, Anthropic, Gemini)

## Validation Procedure

1. Run existing test suite to establish baseline - all provider tests pass
2. Create an xAI provider key via API - verify successful creation
3. List provider keys - verify xAI key appears alongside existing providers
4. Perform handshake with `provider=xai` - verify budget reservation succeeds
5. Route an xAI chat completion request through the runtime proxy - verify successful response
6. Verify the request flowed through the `ProviderBinding` trait implementation
7. If using OpenAI-compatible adapter, verify request format compatibility is handled correctly
8. Submit an xAI analytics event - verify ingestion succeeds
9. Query analytics - verify xAI spending data appears in aggregation
10. Run OpenAI, Anthropic, and Gemini integration tests - verify no regressions
11. Run full test suite and confirm zero regressions

## Acceptance Criteria
- xAI provider key can be created, listed, read, updated, and deleted via API.
- Handshake with `provider=xai` succeeds when key and limits are valid.
- Runtime proxy routes xAI requests correctly and returns provider responses via `api_llm` bindings where applicable.
- Analytics ingestion accepts xAI events and reporting includes xAI data.
- No new direct xAI-specific HTTP execution path is introduced outside the approved binding boundary.
- Existing provider flows (OpenAI, Anthropic, Gemini if enabled) are not regressed.
