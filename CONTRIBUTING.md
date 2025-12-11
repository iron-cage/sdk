# Contributing to Iron Runtime

Thank you for your interest in contributing to Iron Runtime! This guide will help you understand our development workflow and standards.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Documentation Standards](#documentation-standards)
- [Testing Requirements](#testing-requirements)
- [Submitting Changes](#submitting-changes)

## Getting Started

### Prerequisites

- Rust 1.70+ (`rustup update`)
- Node.js 18+ (`node --version`)
- cargo-nextest (`cargo install cargo-nextest`)
- SQLite 3 (`sqlite3 --version`)

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/iron-cage/iron_runtime.git
cd iron_runtime/dev

# Install dependencies
make setup

# Run tests to verify setup
make test

# Start development servers
make dev
```

## Development Workflow

### Daily Commands

```bash
make dev          # Run full stack (API + Dashboard)
make test         # Run all tests
make lint-docs    # Check documentation compliance
make validate     # Full validation before PR
```

### Code Standards

- Follow Rust conventions (rustfmt, clippy)
- Write tests for all new features
- Update documentation when changing APIs
- Run `make validate` before submitting PRs

## Documentation Standards

### ID Format Standards

**Critical Rule:** All entity IDs in documentation MUST use underscore format (`prefix_identifier`).

#### Format Requirements

| Entity Type | Correct Format | Incorrect Format |
|-------------|----------------|------------------|
| Provider ID | `ip_openai_001` | ~~`ip-openai-001`~~ |
| User ID | `user_xyz789` | ~~`user-xyz789`~~ |
| Project ID | `proj_master` | ~~`proj-master`~~ |
| Agent ID | `agent_abc123` | ~~`agent-abc123`~~ |
| IC Token ID | `ic_def456` | ~~`ic-def456`~~ |
| IP Token ID | `ip_ghi789` | ~~`ip-ghi789`~~ |

#### Why This Matters

1. **Consistency**: Uniform format across all documentation
2. **Searchability**: Easy to grep and find all instances
3. **Code Generation**: Documentation examples directly translate to code
4. **API Standards**: Matches actual API implementation

#### Edge Cases (NOT Entity IDs)

These terms use hyphens because they're descriptive, not entity IDs:
- `user-token` (token type descriptor)
- `user-facing` (adjective)
- `user-level` (scope descriptor)

### Checking Compliance

Before submitting documentation changes:

```bash
# Run the lint check
make lint-docs

# Should output:
# ✓ No ID format violations found
```

If violations are found, the script will show:
- File paths and line numbers
- The specific violations
- Expected format for each entity type

### Canonical Examples

All documentation examples should use canonical values from `docs/standards/canonical_examples.md`:

```markdown
**Primary User:** `user_xyz789`
**Primary Providers:** `["ip_openai_001", "ip_anthropic_001"]`
**Primary Project:** `proj_master`
**Primary Agent:** `agent_abc123`
```

Using canonical examples ensures:
- Consistency across all documentation
- Easy cross-referencing between documents
- Recognizable patterns for users

### Documentation Files

- **Protocol Specs** (`docs/protocol/*.md`) - API endpoint specifications
- **Standards** (`docs/standards/*.md`) - Format and design standards
- **Architecture** (`docs/architecture/*.md`) - System design documents
- **Features** (`docs/features/*.md`) - Feature documentation

## Testing Requirements

### Test Levels

```bash
# Level 1: Fast unit tests
cargo nextest run --all-features

# Level 3: Full validation (unit + doc + clippy)
make test  # Runs w3 .test l::3

# Documentation lint
make lint-docs
```

### Pre-Submission Checklist

Before submitting a PR, ensure:

- [ ] All tests pass (`make test`)
- [ ] Documentation lint passes (`make lint-docs`)
- [ ] New features have tests
- [ ] API changes are documented
- [ ] Commit messages are clear and descriptive

## Submitting Changes

### Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/my-new-feature
   ```

2. **Make your changes**
   - Write code
   - Add tests
   - Update documentation

3. **Validate locally**
   ```bash
   make validate      # Full validation
   make lint-docs     # Documentation check
   ```

4. **Commit with clear messages**
   ```bash
   git add .
   git commit -m "feat: add new authentication endpoint"
   ```

5. **Push and create PR**
   ```bash
   git push origin feature/my-new-feature
   # Create PR via GitHub
   ```

### Commit Message Format

Use conventional commits format:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `test:` - Adding or updating tests
- `refactor:` - Code refactoring
- `chore:` - Maintenance tasks

Examples:
```
feat: add budget request approval workflow
fix: handle null provider response correctly
docs: update authentication API examples
test: add integration tests for token rotation
```

### PR Review Process

1. Automated checks run (tests, lint)
2. Maintainer reviews code and documentation
3. Address review feedback
4. PR merged after approval

## Need Help?

- **Questions?** Open a GitHub issue with the `question` label
- **Bug Reports?** Use the bug report template
- **Feature Requests?** Use the feature request template

## Additional Resources

- [Getting Started Guide](docs/getting_started.md)
- [Architecture Overview](docs/architecture/)
- [API Protocol Specs](docs/protocol/)
- [ID Format Standards](docs/standards/id_format_standards.md)
- [Canonical Examples](docs/standards/canonical_examples.md)

## License

By contributing to Iron Runtime, you agree that your contributions will be licensed under the Apache-2.0 License.

---

Thank you for contributing to Iron Runtime! 🦀
