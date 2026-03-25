# Task 025: CI/CD pipeline

## Goal
Build a comprehensive CI/CD pipeline using GitHub Actions that enforces build quality on every PR (build, test, clippy, coverage), automates release artifacts (PyPI wheels, Docker images, Homebrew formula), and blocks merging when quality gates fail. This replaces the existing incomplete workflow setup with a production-grade pipeline.

## Dependencies
- None (can start during Stage 2)

## In Scope
- GitHub Actions workflow for build, test, clippy, and llvm-cov on push and PR
- Multi-crate test matrix covering all workspace members
- Maturin wheel build and PyPI publish on tagged releases
- Docker image build for the Control API server
- Homebrew formula and `cargo install` verification
- Branch protection rules requiring CI pass before merge

## Out of Scope
- Self-hosted runners or custom CI infrastructure
- Nightly/scheduled builds (only push and PR triggers)
- Cross-compilation for non-Linux platforms in initial setup

## Description
The existing CI setup consists of three workflows that cover deployment checks and a single crate validation, but they do not provide the comprehensive quality gates needed for an open-source project. There is no multi-crate test matrix, no clippy enforcement, no coverage reporting, and no automated release pipeline.

This task creates a unified CI/CD pipeline. The PR workflow runs build, test, clippy (with `-D warnings`), and llvm-cov across all workspace crates, blocking merge on any failure. The release workflow triggers on version tags and builds maturin wheels for PyPI, Docker images for the Control API server, and verifies `cargo install` from the published crate. A Homebrew formula is also generated and maintained so that CLI users can install via `brew install`.

## Context
Three CI workflows exist in `.github/workflows/`: `deploy-check.yml`, `deploy.yaml`, and `iron_token_manager_validation.yml`. However, a comprehensive CI/CD pipeline is missing: no multi-crate test matrix, no clippy enforcement, no coverage reporting, and no automated release/publish workflow. These are needed for open source quality.

Critical areas:
- `.github/workflows/`
- `module/iron_sdk/` (maturin wheel builds)
- `Dockerfile.backend`

## Work Procedure
1. Create `.github/workflows/ci.yml` with a job matrix covering all workspace crates.
2. Add build step with `cargo build --workspace --all-features`.
3. Add test step with `cargo test --workspace --all-features`.
4. Add clippy step with `cargo clippy --workspace --all-features -- -D warnings`.
5. Add coverage step with `cargo llvm-cov --workspace --lcov --output-path lcov.info` and upload to a coverage service.
6. Create `.github/workflows/release.yml` triggered on `v*` tags.
7. Add maturin build and PyPI publish steps to the release workflow using `module/iron_sdk/`.
8. Add Docker build step using `Dockerfile.backend` and push to Docker Hub.
9. Create a Homebrew formula template and add verification that `cargo install` works from the published crate.
10. Configure branch protection rules to require the CI workflow to pass before merging.

## Implementation plan
1. GitHub Actions workflow: build, test, clippy, llvm-cov on push/PR.
2. Maturin wheel build and PyPI publish on tag.
3. Docker image build for Control API server.
4. CLI distribution: Homebrew formula, cargo install verification.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| PR with passing code | CI workflow passes all steps | Green status on PR |
| PR with clippy warning | CI workflow fails at clippy step | PR merge blocked |
| PR with failing test | CI workflow fails at test step | PR merge blocked |
| Tagged release (v1.0.0) | Release workflow builds and publishes | PyPI package and Docker image available |
| `cargo install` from published crate | CLI binary installs successfully | Binary runs `--help` without error |
| Homebrew formula install | CLI installs via brew | `brew install iron-cage` produces working binary |

## Validation Checklist
- [ ] CI workflow runs on every push and PR
- [ ] Build, test, clippy, and coverage steps are all present
- [ ] CI covers all workspace crates (not just a subset)
- [ ] Coverage report is generated and uploaded on each PR
- [ ] Release workflow triggers on version tags
- [ ] Maturin wheel builds and publishes to PyPI
- [ ] Docker image builds from `Dockerfile.backend` and pushes to registry
- [ ] Homebrew formula installs a working CLI binary
- [ ] Branch protection requires CI pass before merge

## Validation Procedure
1. Open a test PR with passing code and verify all CI steps complete successfully.
2. Open a test PR with a deliberate clippy warning and verify the CI blocks the merge.
3. Create a test tag and verify the release workflow triggers and completes.
4. Download the published PyPI wheel and verify `pip install` works.
5. Pull the published Docker image and verify the Control API server starts.
6. Install via the Homebrew formula and verify the CLI binary runs.

## Acceptance Criteria
- PRs blocked if build/test/clippy fails.
- Coverage report generated on each PR.
- Tagged releases automatically publish to PyPI and Docker Hub.
- Homebrew formula installs working CLI binary.
