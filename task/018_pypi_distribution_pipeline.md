# Task 018: PyPI distribution pipeline

## Dependencies
- Task 017

## Context
Python SDK requires `maturin build --features full` from source. Not available via `pip install`. This blocks the "drop-in" promise.

Critical areas:
- `module/iron_sdk/Cargo.toml`
- `module/iron_sdk/pyproject.toml`

## Implementation plan
1. Configure maturin for wheel builds (Linux x86_64, macOS arm64/x86_64).
2. Set up GitHub Actions workflow for automated wheel building.
3. Publish to PyPI (or TestPyPI initially).
4. Verify `pip install iron-cage` works on clean Python environment.

## Acceptance criteria
- `pip install iron-cage` works on Linux x86_64 and macOS.
- Package imports correctly: `from iron_cage import LlmRouter`.
- Version matches Cargo.toml version.
