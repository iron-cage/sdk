# Task 017: Complete Python SDK async bridge

## Goal
Replace the placeholder `start_agent()` with a working async bridge using pyo3-async-runtimes so that the Python SDK's `LlmRouter` context manager functions end-to-end, fulfilling promise #5 ("three lines of code"). This is observable when the exact Rusticon presentation Python snippet runs successfully and returns real LLM responses. Scoped to the `iron_sdk` crate's PyO3 bindings and the Python `iron_cage` package.

## Dependencies
- None

## In Scope
- Implementing the async bridge using pyo3-async-runtimes or equivalent
- Replacing the placeholder `start_agent()` with a working implementation
- Making `LlmRouter` context manager (`__enter__`/`__exit__`) functional
- Validating the Rusticon presentation Python snippet works end-to-end
- Ensuring type stubs match the implementation

## Out of Scope
- Distribution via PyPI (covered by Task 018)
- Adding new Python API surface beyond what the Rusticon demo requires
- Python 2 compatibility
- Windows-specific async runtime considerations

## Description
The `iron_sdk` crate has PyO3 bindings that expose a Python package called `iron_cage`, but the core `start_agent()` function currently returns a placeholder string instead of performing real work. The async bridge needed to connect Python's asyncio event loop to Rust's tokio runtime via pyo3-async-runtimes is not implemented, making the SDK non-functional.

This task completes the async bridge so that `start_agent()` returns actual async results from the Rust runtime. The `LlmRouter` context manager must work with Python's `with` statement, and the exact code snippet from the Rusticon presentation - importing `LlmRouter`, using a context manager, calling through the OpenAI SDK, and receiving a response - must execute successfully.

## Context
`iron_sdk` has PyO3 bindings but `start_agent()` returns placeholder string. Async bridge with pyo3-async-runtimes not implemented. This contradicts promise #5 ("three lines of code").

Critical areas:
- `module/iron_sdk/src/lib.rs`
- `module/iron_sdk/python/iron_cage/__init__.py`

## Work Procedure
1. Review the current `iron_sdk/src/lib.rs` to understand the placeholder `start_agent()` implementation.
2. Review `iron_sdk/python/iron_cage/__init__.py` for the Python-side API surface.
3. Add pyo3-async-runtimes (or equivalent) as a dependency in `iron_sdk/Cargo.toml`.
4. Implement the tokio-to-asyncio bridge that allows Rust async functions to be awaited from Python.
5. Replace the placeholder `start_agent()` with a real implementation that initializes the Rust runtime.
6. Implement the `LlmRouter` context manager with working `__enter__` and `__exit__` methods.
7. Ensure the context manager properly starts and stops the Rust runtime and cleans up resources.
8. Update or create type stubs (`.pyi` files) to match the implementation.
9. Test with the exact Rusticon presentation Python snippet end-to-end.
10. Run `cargo test --workspace` and any Python tests to confirm everything passes.

## Implementation plan
1. Implement async bridge using pyo3-async-runtimes or equivalent.
2. Replace placeholder `start_agent()` with working implementation.
3. Ensure `LlmRouter` context manager works end-to-end.
4. Validate the exact Rusticon presentation snippet works:
   `from iron_cage import LlmRouter` -> context manager -> OpenAI SDK -> response.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| `start_agent()` called from Python | Returns async result from Rust runtime | Return value is not a placeholder string |
| `LlmRouter` used as context manager | `__enter__` initializes, `__exit__` cleans up | No resource leaks, no exceptions |
| Rusticon demo snippet executed | Full pipeline works end-to-end | Response received from LLM via router |
| Context manager with exception in body | `__exit__` still cleans up properly | Resources released, no orphan tokio runtime |
| Type stubs imported by type checker | All public methods have stubs | mypy or pyright passes on import |
| Concurrent async calls from Python | Multiple awaits resolve correctly | All responses received without deadlock |

## Validation List
- [ ] `start_agent()` returns a real async result, not a placeholder
- [ ] `LlmRouter.__enter__` initializes the Rust runtime
- [ ] `LlmRouter.__exit__` cleans up resources properly
- [ ] The Rusticon presentation Python code runs successfully
- [ ] Type stubs (`.pyi`) exist and match the implementation
- [ ] pyo3-async-runtimes (or equivalent) is listed as a dependency
- [ ] `cargo test --workspace` passes
- [ ] Python-side tests pass

## Validation Procedure
1. Run the Rusticon presentation Python snippet and verify it produces a real LLM response.
2. Verify `start_agent()` return value is not a placeholder by inspecting the return type.
3. Test the context manager by using `with LlmRouter(...) as router:` and confirming no exceptions.
4. Test cleanup by forcing an exception inside the context manager and verifying `__exit__` runs.
5. Run `mypy` or `pyright` against the type stubs to verify they match the implementation.
6. Run `cargo test --workspace` and confirm no regressions.

## Acceptance criteria
- `start_agent()` returns actual async result, not placeholder.
- LlmRouter context manager (`__enter__`/`__exit__`) works.
- The Rusticon presentation Python code runs successfully.
- Type stubs match implementation.
