# Python Tooling Standards

**Purpose:** Define canonical Python development workflow for Iron Runtime

**Responsibility:** Specify tooling requirements, workflows, and enforcement

**Status:** Normative (must follow)

**Version:** 1.0.0

---

## TL;DR

- **Use `uv`** for all Python operations (not `pip`, not `virtualenv`)
- **Use `pyproject.toml` only** (no `.python-version`, no `requirements.txt`)
- **`uv.lock` files are gitignored** (regenerate with `uv lock`)
- **Run `make lint-python`** before committing

---

## Core Principles

### 1. Single Source of Truth: `pyproject.toml`

All Python configuration lives in `pyproject.toml`:
- Python version requirement (`requires-python`)
- Runtime dependencies (`dependencies`)
- Development dependencies (`optional-dependencies.dev`)
- Build system configuration (`build-system`)
- Tool configuration (`tool.*`)

### 2. One Tool: `uv`

`uv` handles everything:
- Python version management (auto-download)
- Dependency resolution
- Environment management
- Package installation
- Code execution

### 3. Reproducible Environments: `uv.lock`

Lock files ensure identical environments across:
- Developer machines
- CI/CD systems
- Production deployments

---

## Canonical Workflow

### Initial Setup

```bash
# Install uv (one-time)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Navigate to Python module
cd module/iron_sdk  # or iron_runtime, iron_cli_py

# Install dependencies (auto-downloads Python + creates .venv)
uv sync

# That's it! Python + all dependencies installed.
```

### Adding Dependencies

```bash
# Runtime dependency
uv add requests

# Development dependency
uv add --dev pytest

# This automatically:
# - Adds to pyproject.toml
# - Updates uv.lock
# - Installs the package
```

### Running Code

```bash
# Run script
uv run python main.py

# Run tests
uv run pytest

# Run any command
uv run <command>

# No need to activate virtualenv!
```

### Updating Dependencies

```bash
# Update specific package
uv add --upgrade requests

# Update all dependencies
uv sync --upgrade
```

---

## Prohibited Patterns

### ❌ DON'T Do This

```bash
# Never create version files
touch .python-version  # ❌ WRONG

# Never create requirements files
pip freeze > requirements.txt  # ❌ WRONG

# Never use pip directly
pip install requests  # ❌ WRONG

# Never create virtualenvs manually
python -m venv .venv  # ❌ WRONG
virtualenv venv  # ❌ WRONG

# Never use old uv syntax
uv pip install <package>  # ❌ WRONG (old syntax)

# Never manually edit dependencies
# (edit pyproject.toml by hand)  # ❌ WRONG
```

### ✅ DO This Instead

```bash
# Python version in pyproject.toml
[project]
requires-python = ">=3.8"

# Use uv add for dependencies
uv add requests

# Use uv run for execution
uv run python main.py

# Use uv sync for setup
uv sync
```

---

## Why `uv`?

### 1. **Performance**

```
Benchmark: Installing 100 packages
-----------------------------------
pip:        45 seconds
poetry:     38 seconds
uv:         2 seconds  (20-40x faster!)
```

### 2. **Reproducibility**

`uv.lock` contains:
- Exact package versions
- Cryptographic hashes
- Platform-specific builds
- Complete dependency tree

**Result:** Identical environments guaranteed.

### 3. **Simplicity**

```bash
# Old workflow (pip + virtualenv)
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
pip install -r requirements-dev.txt
pip install -e .

# New workflow (uv)
uv sync
```

### 4. **Automatic Python Management**

`uv` automatically:
- Detects required Python version from `pyproject.toml`
- Downloads correct Python if missing
- Creates isolated environment
- No `pyenv`, no manual Python installation

---

## Migration from `pip`

| Old (`pip` + `virtualenv`) | New (`uv`) |
|----------------------------|------------|
| `pip install requests` | `uv add requests` |
| `pip install -r requirements.txt` | `uv sync` |
| `pip install -e .` | `uv sync` |
| `python -m venv .venv` | Automatic (`uv sync`) |
| `source .venv/bin/activate` | Not needed (`uv run`) |
| `python main.py` | `uv run python main.py` |
| `pytest` | `uv run pytest` |
| `pip freeze > requirements.txt` | `uv lock` (automatic) |
| `pip install --upgrade <pkg>` | `uv add --upgrade <pkg>` |

---

## File Structure

### ✅ Correct Structure

```
module/iron_sdk/
├── pyproject.toml      # All Python configuration
├── uv.lock             # Locked dependencies (gitignored - regenerate)
├── .venv/              # Virtual environment (gitignored)
├── src/
│   └── iron_sdk/
│       └── __init__.py
└── tests/
    └── test_sdk.py
```

### ❌ Incorrect Structure

```
module/iron_sdk/
├── .python-version     # ❌ REMOVE (use pyproject.toml)
├── requirements.txt    # ❌ REMOVE (use pyproject.toml)
├── requirements-dev.txt # ❌ REMOVE (use pyproject.toml)
├── setup.py            # ❌ REMOVE (use pyproject.toml)
└── setup.cfg           # ❌ REMOVE (use pyproject.toml)
```

---

## `pyproject.toml` Structure

### Example

```toml
[project]
name = "iron-sdk"
version = "0.1.0"
description = "Iron Cage SDK for AI agent protection"
requires-python = ">=3.9"  # Python version requirement
dependencies = [
  "iron-runtime>=0.1.0",   # Runtime dependencies
  "requests>=2.31.0",
]

[project.optional-dependencies]
dev = [                    # Development dependencies
  "pytest>=8.0.0",
  "pytest-cov>=4.0.0",
]
test = [                   # Test dependencies
  "pytest-asyncio>=0.24.0",
]
all = [                    # All optional deps
  "iron-sdk[dev,test]",
]

[build-system]
requires = ["setuptools>=65.0", "wheel"]
build-backend = "setuptools.build_meta"

[tool.pytest.ini_options]
testpaths = ["tests"]
python_files = ["test_*.py"]
python_functions = ["test_*"]
```

### Key Fields

| Field | Purpose | Required |
|-------|---------|----------|
| `[project]` | Package metadata | ✅ Yes |
| `requires-python` | Python version constraint | ✅ Yes |
| `dependencies` | Runtime dependencies | ✅ Yes |
| `optional-dependencies` | Dev/test/extra deps | Recommended |
| `[build-system]` | Build tool configuration | ✅ Yes |
| `[tool.*]` | Tool-specific config | Optional |

---

## Enforcement

### Local Checks

Run before every commit:

```bash
# Check Python tooling compliance
make lint-python

# Should output:
# ✓ Python Tooling Compliance: PASS
```

### What Gets Checked

The lint script validates:
- ✅ No `.python-version` files
- ✅ No `requirements.txt` files (except archived modules)
- ✅ No `setup.py` files
- ✅ All Python modules have `pyproject.toml`
- ⚠️ No `pip install` in development docs
- ⚠️ No `uv pip install` (old syntax)
- ⚠️ No `virtualenv` commands

### Pre-Commit Checklist

- [ ] Run `make lint-python` (should pass)
- [ ] No `.python-version` or `requirements.txt` files
- [ ] Tests pass (`uv run pytest`)
- [ ] `uv.lock` files are gitignored (regenerate with `uv lock` if needed)

---

## Troubleshooting

### Q: My IDE can't find packages

**A:** Point IDE to `.venv/bin/python` after running `uv sync`

**VSCode:** Cmd/Ctrl+Shift+P → "Python: Select Interpreter" → `.venv/bin/python`
**PyCharm:** Settings → Project → Python Interpreter → Add → Existing environment → `.venv/bin/python`

### Q: Which Python version am I using?

**A:** Check `pyproject.toml` `requires-python` field

```bash
# See active Python version
uv run python --version

# Check required version
grep "requires-python" module/iron_sdk/pyproject.toml
```

### Q: Can I still use `pip`?

**A:** Not for development. Use `uv add` instead.

For **end users** installing published packages: `uv pip install iron-cage` is fine.
For **developers** working on Iron Runtime: use `uv sync` / `uv add`.

### Q: How do I add a dependency?

**A:** Use `uv add`, not manual edits

```bash
# Add runtime dependency
uv add requests

# Add dev dependency
uv add --dev pytest

# Don't manually edit pyproject.toml!
```

### Q: The lint check is failing

**A:** Read the output, it tells you exactly what to fix

```bash
make lint-python

# Fix issues listed, then verify
make lint-python  # Should pass
```

### Q: Where is my virtualenv?

**A:** `.venv/` directory (auto-created by `uv sync`)

You don't need to activate it - `uv run` handles that automatically.

---

## Common Workflows

### Starting Work on a Python Module

```bash
# 1. Navigate to module
cd module/iron_sdk

# 2. Sync dependencies
uv sync

# 3. Run tests to verify
uv run pytest

# Ready to develop!
```

### Adding a New Dependency

```bash
# 1. Add dependency
uv add requests

# 2. Verify it works
uv run python -c "import requests; print(requests.__version__)"

# 3. Run tests
uv run pytest

# 4. Commit changes
git add pyproject.toml
git commit -m "add requests dependency"
```

### Running Tests

```bash
# All tests
uv run pytest

# Specific test file
uv run pytest tests/test_auth.py

# With coverage
uv run pytest --cov=iron_sdk

# Verbose output
uv run pytest -v
```

### Building Wheels (for PyO3/maturin modules)

```bash
# Development build
cd module/iron_runtime
uv run maturin develop

# Release build
uv run maturin build --release

# The wheel is in target/wheels/
```

---

## Module-Specific Notes

### `iron_runtime` (PyO3 Module)

Uses `maturin` for Rust+Python bindings:

```bash
cd module/iron_runtime

# Install in development mode
uv run maturin develop

# Build release wheel
uv run maturin build --release

# Run tests
uv run pytest python/tests/
```

### `iron_sdk` (Pure Python)

Pure Python package:

```bash
cd module/iron_sdk

# Install dependencies
uv sync

# Run tests
uv run pytest

# Type checking
uv run mypy src/
```

### `iron_cli_py` (Python CLI)

Python CLI wrapper:

```bash
cd module/iron_cli_py

# Install dependencies
uv sync

# Run CLI
uv run iron-py --help

# Run tests
uv run pytest
```

---

## Integration with Makefile

The project Makefile provides shortcuts:

```bash
# Sync all Python modules
make py-sync

# Run Python tests
make py-test

# Build Python wheels
make py-build

# Check Python tooling compliance
make lint-python
```

---

## References

### Related Documentation

- [contributing.md](../../contributing.md) - Complete contributor guide
- [Getting Started](../getting_started.md) - Setup instructions
- [Canonical Examples](canonical_examples.md) - Standard example values

### External Resources

- [uv documentation](https://github.com/astral-sh/uv) - Official uv guide
- [PEP 621](https://peps.python.org/pep-0621/) - pyproject.toml standard
- [Python Packaging Guide](https://packaging.python.org/) - Official Python packaging docs

### Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-11 | Initial release - uv standardization |

---

**Document Version:** 1.0.0
**Last Updated:** 2025-12-11
**Status:** Normative (must follow)
