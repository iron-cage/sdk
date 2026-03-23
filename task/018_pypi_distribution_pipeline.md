# Task 018: PyPI distribution pipeline

## Goal
Create a CI pipeline that builds maturin wheels for Linux x86_64 and macOS (arm64, x86_64) and publishes them to PyPI so that users can install the Python SDK via `pip install iron-cage`. This is observable when a clean Python environment can successfully run `pip install iron-cage` and import the package. Scoped to wheel building, CI workflow, and PyPI publishing.

## Dependencies
- Task 017

## In Scope
- Configuring maturin for multi-platform wheel builds (Linux x86_64, macOS arm64, macOS x86_64)
- Setting up GitHub Actions workflow for automated wheel building on release
- Publishing to PyPI (or TestPyPI for initial validation)
- Verifying `pip install iron-cage` on a clean environment
- Ensuring package version matches `Cargo.toml` version

## Out of Scope
- Windows wheel builds
- Linux aarch64 or other non-x86_64 Linux architectures
- Conda distribution
- Source distribution (sdist) publishing

## Description
Currently the Python SDK can only be built from source using `maturin build --features full`, which requires a Rust toolchain and is impractical for Python-only developers. This blocks the "drop-in" promise of the SDK - users should be able to `pip install iron-cage` and start using it immediately.

This task sets up a complete distribution pipeline: maturin is configured for cross-platform wheel builds, a GitHub Actions workflow automates the build matrix (Linux x86_64, macOS arm64, macOS x86_64), and the resulting wheels are published to PyPI. The version is derived from `Cargo.toml` to maintain a single source of truth. Initial publishing may target TestPyPI for validation before promoting to production PyPI.

## Context
Python SDK requires `maturin build --features full` from source. Not available via `pip install`. This blocks the "drop-in" promise.

Critical areas:
- `module/iron_sdk/Cargo.toml`
- `module/iron_sdk/pyproject.toml`

## Work Procedure
1. Review the existing `iron_sdk/Cargo.toml` and `iron_sdk/pyproject.toml` for maturin configuration.
2. Update `pyproject.toml` with correct maturin build settings for multi-platform wheels.
3. Create a GitHub Actions workflow file with a build matrix for Linux x86_64, macOS arm64, and macOS x86_64.
4. Configure the workflow to use `maturin build --release --features full` for each platform.
5. Add a workflow step to upload wheel artifacts.
6. Add a publish job that uploads wheels to TestPyPI on tag push.
7. Test the pipeline by creating a test release tag and verifying wheels appear on TestPyPI.
8. Install from TestPyPI in a clean Python venv and verify `from iron_cage import LlmRouter` works.
9. Update the publish job to target production PyPI once TestPyPI validation passes.
10. Verify the package version matches the `Cargo.toml` version.

## Implementation plan
1. Configure maturin for wheel builds (Linux x86_64, macOS arm64/x86_64).
2. Set up GitHub Actions workflow for automated wheel building.
3. Publish to PyPI (or TestPyPI initially).
4. Verify `pip install iron-cage` works on clean Python environment.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| `maturin build` on Linux x86_64 | Wheel produced for linux x86_64 | `.whl` file exists with correct platform tag |
| `maturin build` on macOS arm64 | Wheel produced for macOS arm64 | `.whl` file exists with correct platform tag |
| `maturin build` on macOS x86_64 | Wheel produced for macOS x86_64 | `.whl` file exists with correct platform tag |
| `pip install iron-cage` in clean venv | Package installs without errors | Exit code 0, no build-from-source fallback |
| `from iron_cage import LlmRouter` | Import succeeds | No ImportError |
| Version check after install | Matches Cargo.toml version | `iron_cage.__version__` equals Cargo.toml version |
| GitHub Actions workflow on tag push | All platform wheels built and uploaded | Workflow completes green, artifacts present |

## Validation List
- [ ] `pyproject.toml` is configured for maturin wheel builds
- [ ] GitHub Actions workflow exists with Linux x86_64, macOS arm64, macOS x86_64 matrix
- [ ] Wheels are published to PyPI (or TestPyPI)
- [ ] `pip install iron-cage` works on a clean Python environment
- [ ] `from iron_cage import LlmRouter` succeeds after pip install
- [ ] Package version matches `Cargo.toml` version
- [ ] Workflow triggers on tag push or release

## Validation Procedure
1. Trigger the GitHub Actions workflow manually or via tag push and verify all matrix jobs pass.
2. Download the produced wheel artifacts and inspect platform tags for correctness.
3. Create a clean Python virtual environment and run `pip install iron-cage` (from TestPyPI or PyPI).
4. In the clean venv, run `python -c "from iron_cage import LlmRouter; print('OK')"` and verify it prints "OK".
5. Compare the installed package version (`pip show iron-cage`) against the version in `Cargo.toml`.

## Acceptance criteria
- `pip install iron-cage` works on Linux x86_64 and macOS.
- Package imports correctly: `from iron_cage import LlmRouter`.
- Version matches Cargo.toml version.
