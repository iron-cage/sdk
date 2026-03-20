# Task 017: Complete Python SDK async bridge

## Dependencies
- None

## Context
`iron_sdk` has PyO3 bindings but `start_agent()` returns placeholder string. Async bridge with pyo3-asyncio not implemented. This contradicts promise #5 ("three lines of code").

Critical areas:
- `module/iron_sdk/src/lib.rs`
- `module/iron_sdk/python/iron_cage/__init__.py`

## Implementation plan
1. Implement async bridge using pyo3-asyncio or equivalent.
2. Replace placeholder `start_agent()` with working implementation.
3. Ensure `LlmRouter` context manager works end-to-end.
4. Validate the exact Rusticon presentation snippet works:
   `from iron_cage import LlmRouter` -> context manager -> OpenAI SDK -> response.

## Acceptance criteria
- `start_agent()` returns actual async result, not placeholder.
- LlmRouter context manager (`__enter__`/`__exit__`) works.
- The Rusticon presentation Python code runs successfully.
- Type stubs match implementation.
