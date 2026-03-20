# Task 025: CI/CD pipeline

## Dependencies
- None (can start during Stage 2)

## Context
No CI/CD pipeline exists. Automated builds, tests, linting, coverage, and distribution are needed for open source quality.

Critical areas:
- `.github/workflows/`
- `module/iron_sdk/` (maturin wheel builds)
- `Dockerfile.backend`

## Implementation plan
1. GitHub Actions workflow: build, test, clippy, llvm-cov on push/PR.
2. Maturin wheel build and PyPI publish on tag.
3. Docker image build for Control API server.
4. CLI distribution: Homebrew formula, cargo install verification.

## Acceptance criteria
- PRs blocked if build/test/clippy fails.
- Coverage report generated on each PR.
- Tagged releases automatically publish to PyPI and Docker Hub.
- Homebrew formula installs working CLI binary.
