# Database Path Standards

## Overview

This document defines the canonical database paths and validation rules for the iron_token_manager module. These standards prevent database inconsistencies and ensure reliable development workflows.

## Canonical Paths

### Development Database

**Single Source of Truth: `./iron.db`**

All development work uses this path:
- Scripts default to `./iron.db`
- Configuration files use `sqlite:///./iron.db?mode=rwc`
- Documentation references `./iron.db`

### Test Databases

**In-Memory: `sqlite::memory:`**

All automated tests use in-memory databases:
- Automatic cleanup after each test
- Perfect isolation between tests
- No file system pollution

### Backup Naming

**Format: `iron_backup_YYYYMMDD_HHMMSS.db`**

Example: `iron_backup_20251211_125024.db`

## Forbidden Patterns

### ❌ NEVER Use

1. **`dev_tokens.db`** - Old path, causes inconsistencies
2. **Non-canonical paths** in scripts, configs, or documentation
3. **Hardcoded paths** without defaults
4. **Inconsistent backup naming**

## Validation System

### Three-Layer Validation

1. **Path Validator** (`scripts/validate_db_paths.sh`)
   - Scans entire codebase for forbidden paths
   - Validates script defaults
   - Checks config files
   - Verifies backup naming

2. **Schema Validator** (`scripts/validate_db_schema.sh`)
   - Confirms 17 tables (11 application + 6 migration guards)
   - Validates 32 indexes
   - Checks foreign key support
   - Verifies migration guards

3. **Seed Data Validator** (`scripts/validate_seed_data.sh`)
   - Validates 3 test users (admin, developer, viewer)
   - Checks 3 test tokens
   - Confirms ≥7 usage records
   - Verifies 3 usage limits

### Running Validators

```bash
# Run all validators
make validate

# Run individual validators
make validate-paths
make validate-schema
make validate-seed
```

## Development Workflows

### Quick Start: Fresh Environment

Get a clean, validated development environment in one command:

```bash
# Full workflow (recommended)
make dev-fresh

# With full test suite
make dev-fresh-test

# Quick mode (no validation)
make dev-fresh-quick
```

### Manual Workflow

```bash
# 1. Reset and seed database
make reset-seed

# 2. Validate everything
make validate

# 3. Run tests
make test
```

### Database Operations

```bash
# Reset database (with backup)
make reset

# Seed test data
make seed

# Both in one command
make reset-seed
```

## Enforcement Mechanisms

### 1. Pre-Commit Hook (Local)

Validates database paths before every commit:

```bash
# Install hook
make install-hooks

# Uninstall hook
make uninstall-hooks
```

The hook will:
- ✅ Block commits with forbidden paths
- ✅ Provide clear error messages
- ✅ Suggest fixes

### 2. CI/CD Validation (Remote)

GitHub Actions workflow runs on every PR:
- Validates all database paths
- Runs complete test suite
- Validates schema and seed data
- Blocks merge if validation fails

### 3. Makefile Integration

All critical commands include validation:
- `make dev-fresh` - Full validation
- `make validate` - Explicit validation
- `make test` - Tests validate implicitly

## Quick Reference

### Commands

| Command | Purpose |
|---------|---------|
| `make dev-fresh` | Fresh environment (reset + seed + validate) |
| `make validate` | Run all validators |
| `make reset-seed` | Reset and seed database |
| `make install-hooks` | Install pre-commit hook |
| `make test` | Full test suite |

### Paths

| Context | Path |
|---------|------|
| Development DB | `./iron.db` |
| Config URL | `sqlite:///./iron.db?mode=rwc` |
| Test DB | `sqlite::memory:` |
| Backups | `./backups/iron_backup_YYYYMMDD_HHMMSS.db` |

### Test Tokens

For manual API testing:

```bash
Admin:      iron_dev_admin_token_001
Developer:  iron_dev_pm_token_002
Viewer:     iron_dev_viewer_token_003
```

### Test Users

```
admin      - role=admin, active=1
developer  - role=user,  active=1
viewer     - role=user,  active=0 (INACTIVE)
```

## Troubleshooting

### Validation Failures

**Path validation fails:**
1. Check for `dev_tokens.db` references: `grep -r "dev_tokens.db" .`
2. Update to `./iron.db`
3. Re-run: `make validate-paths`

**Schema validation fails:**
1. Reset database: `make reset`
2. Re-run: `make validate-schema ./iron.db`
3. Check migration scripts if problems persist

**Seed validation fails:**
1. Re-seed database: `make seed`
2. Re-run: `make validate-seed ./iron.db`
3. Verify seed data in `src/seed.rs`

### Common Issues

**"No such table" errors:**
- Run migrations: `make reset` (applies all migrations)

**"Foreign key constraint failed":**
- Enable foreign keys in your SQL client
- Check seed data references correct usernames

**Database locked:**
- Close all connections to `./iron.db`
- Kill any running server processes

**Stale data:**
- Fresh start: `make dev-fresh`

## Implementation History

### Phase 1: Path Standardization
- ✅ Updated all scripts to use `./iron.db`
- ✅ Updated configuration files
- ✅ Updated Rust code defaults
- ✅ Eliminated all `dev_tokens.db` references

### Phase 2: Validation Scripts
- ✅ Created path validator
- ✅ Created schema validator
- ✅ Created seed data validator
- ✅ Integrated with Makefile

### Phase 3: Automation & Enforcement
- ✅ Created pre-commit hook
- ✅ Created GitHub Actions CI workflow
- ✅ Created dev-fresh workflow
- ✅ Updated documentation

## Design Principles

1. **Single Source of Truth** - One canonical path, everywhere
2. **Fail Loudly** - Validation failures block progress
3. **Easy to Fix** - Clear error messages with solutions
4. **Impossible to Ignore** - Multi-layer enforcement
5. **Developer Friendly** - One-command workflows

## Related Documentation

- [Database Initialization](./database_initialization.md) - Schema and migrations
- [Configuration Guide](./configuration.md) - Config file details
- [Quick Reference](./quick_reference_database.md) - Command cheat sheet
- [Test Database Troubleshooting](./test_database_troubleshooting.md) - Test-specific issues
