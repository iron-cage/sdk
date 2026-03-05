# iron_llm_core/tests

Integration and unit tests for the LLM routing core.

## Responsibility Table

| File | Responsibility |
|------|---------------|
| provider.rs | Tests for provider detection from path prefix and model name |
| translator.rs | Tests for OpenAI <-> Anthropic format translation (request and response) |
| cost.rs | Tests for request cost calculation with various models and token counts |
| error.rs | Tests for error type display and conversion |
