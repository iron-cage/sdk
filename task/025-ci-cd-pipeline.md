# Task 025: CI/CD pipeline

## Dependencies
- None (can start during Stage 2)

## Context
Three CI workflows exist in `.github/workflows/`: `deploy-check.yml`, `deploy.yaml`, and `iron_token_manager_validation.yml`. However, a comprehensive CI/CD pipeline is missing: no multi-crate test matrix, no clippy enforcement, no coverage reporting, and no automated release/publish workflow. These are needed for open source quality.

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
