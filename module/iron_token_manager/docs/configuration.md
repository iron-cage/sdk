# Configuration Guide

Complete guide for configuring `iron_token_manager` using configuration files.

## Overview

The `iron_token_manager` crate supports configuration via TOML files with environment variable overrides. This provides flexibility for different deployment environments while maintaining type safety and validation.

## Configuration Files

### Available Configurations

| File | Environment | Purpose | Git |
|------|-------------|---------|-----|
| `config.dev.toml` | development | Local development | ✅ Committed |
| `config.test.toml` | test | Automated testing | ✅ Committed |
| `config.prod.toml.example` | production | Template only | ✅ Committed |
| `config.prod.toml` | production | Actual production config | ❌ Gitignored |

### Environment Selection

The runtime environment is determined by the `IRON_ENV` environment variable:

```bash
# Development (default)
export IRON_ENV=development  # Loads config.dev.toml

# Test
export IRON_ENV=test         # Loads config.test.toml

# Production
export IRON_ENV=production   # Loads config.prod.toml
```

If `IRON_ENV` is not set, defaults to `"development"`.

## Configuration Schema

### Complete Example

```toml
# config.prod.toml
[database]
url = "sqlite:///./data/tokens.db?mode=rwc"
max_connections = 10
auto_migrate = true
foreign_keys = true

[production]
debug = false
auto_seed = false
```

### Section: `[database]`

Database connection and behavior settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `url` | String | Required | SQLite database URL |
| `max_connections` | u32 | 5 | Connection pool size |
| `auto_migrate` | bool | true | Apply migrations on startup |
| `foreign_keys` | bool | true | Enable FK constraints |

**Database URL Format:**
```
sqlite:///{path}?mode=rwc
```

- Absolute path: `sqlite:///./data/tokens.db?mode=rwc`
- Relative path: `sqlite:///./dev_tokens.db?mode=rwc`
- In-memory: `sqlite:///:memory:?mode=rwc`

### Section: `[development]` (optional)

Development-specific settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `debug` | bool | false | Enable debug logging |
| `auto_seed` | bool | false | Seed data on first run |
| `wipe_and_seed` | bool | false | Wipe and re-seed database on every startup (DESTRUCTIVE!) |

### Section: `[production]` (optional)

Production-specific settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `debug` | bool | false | Enable debug logging (keep false) |
| `auto_seed` | bool | false | Seed data on startup (keep false) |

### Section: `[test]` (optional)

Test-specific settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `use_memory` | bool | true | Use in-memory databases |
| `debug` | bool | false | Enable debug logging |
| `auto_seed` | bool | false | Seed data on startup |
| `wipe_and_seed` | bool | false | Wipe and re-seed database on every startup |

## Environment Variable Overrides

All configuration values can be overridden via environment variables. This is useful for:
- CI/CD pipelines
- Container deployments
- Secret management systems

### Supported Variables

| Variable | Overrides | Example |
|----------|-----------|---------|
| `DATABASE_URL` | `database.url` | `sqlite:///custom.db?mode=rwc` |
| `DATABASE_MAX_CONNECTIONS` | `database.max_connections` | `20` |
| `DATABASE_AUTO_MIGRATE` | `database.auto_migrate` | `true` / `false` |
| `DATABASE_FOREIGN_KEYS` | `database.foreign_keys` | `true` / `false` |

### Precedence Order

1. Environment variables (highest priority)
2. Config file values
3. Default values (lowest priority)

### Example: Docker Deployment

```bash
# Dockerfile
ENV IRON_ENV=production
ENV DATABASE_URL=sqlite:///./data/prod_tokens.db?mode=rwc
ENV DATABASE_MAX_CONNECTIONS=20

# Or docker run
docker run \
  -e IRON_ENV=production \
  -e DATABASE_URL=sqlite:///./data/prod_tokens.db?mode=rwc \
  -e DATABASE_MAX_CONNECTIONS=20 \
  my-app
```

## Usage in Code

### Method 1: Load from Environment (Recommended)

```rust
use iron_token_manager::storage::TokenStorage;

// Respects IRON_ENV (defaults to "development")
let storage = TokenStorage::from_config().await?;
```

**Workflow:**
1. Reads `IRON_ENV` (default: "development")
2. Loads `config.{env}.toml`
3. Applies environment variable overrides
4. Initializes storage

### Method 2: Load Specific File

```rust
use iron_token_manager::config::Config;
use iron_token_manager::storage::TokenStorage;

// Load specific config file
let config = Config::from_file("config.prod.toml")?;
let storage = TokenStorage::from_config_object(&config).await?;
```

### Method 3: Load from Environment Name

```rust
use iron_token_manager::config::Config;
use iron_token_manager::storage::TokenStorage;

// Load config for specific environment
let config = Config::from_env("production")?;
let storage = TokenStorage::from_config_object(&config).await?;
```

### Method 4: Legacy URL (Not Recommended)

```rust
use iron_token_manager::storage::TokenStorage;

// Direct URL (hardcoded max_connections = 5)
let storage = TokenStorage::new("sqlite:///dev_tokens.db?mode=rwc").await?;
```

**Note:** Method 4 is maintained for backward compatibility but lacks configuration flexibility.

## Configuration Best Practices

### Development

✅ **Do:**
- Use `config.dev.toml` with local database path
- Set `auto_migrate = true` (automatic schema updates)
- Keep `auto_seed = false` (use `make db-seed` manually)
- Set `max_connections = 5` (sufficient for development)

❌ **Don't:**
- Commit `config.prod.toml` with secrets
- Use production database in development config

### Testing

✅ **Do:**
- Use in-memory databases (`:memory:`)
- Set `auto_migrate = true` (tests need schema)
- Keep `auto_seed = false` (tests create own data)
- Set `max_connections = 5` (tests run in parallel)

❌ **Don't:**
- Share databases between tests
- Use persistent files in test config

### Production

✅ **Do:**
- Copy `config.prod.toml.example` to `config.prod.toml`
- Use absolute paths for database files
- Set appropriate `max_connections` (10-20 typical)
- Set `auto_migrate = true` (safe with guard tables)
- Keep `auto_seed = false` (NEVER seed in production)
- Set `debug = false` (reduce logging overhead)
- Use environment variables for secrets

❌ **Don't:**
- Commit `config.prod.toml` to git
- Enable `auto_seed` in production
- Use relative paths (deployment directory may vary)
- Share production config publicly

## Migration Behavior

### With `auto_migrate = true` (Default)

```rust
let storage = TokenStorage::from_config().await?;
// Migrations applied automatically ✅
```

**Behavior:**
1. Connects to database
2. Checks migration guard tables
3. Applies missing migrations
4. Ready to use

**Use case:** Development, testing, production with controlled deployments

### With `auto_migrate = false`

```rust
let storage = TokenStorage::from_config().await?;
// Migrations NOT applied ⚠️
// Must run migrations separately
```

**Manual migration:**
```rust
use iron_token_manager::migrations;

let pool = /* create pool */;
migrations::apply_all_migrations(&pool).await?;
```

**Use case:** Production environments with separate migration pipelines

## Seed Data for Development

### Automatic Wipe and Seed

For manual testing and development, you can configure automatic database wipe and seed:

```toml
# config.dev.toml
[development]
wipe_and_seed = true
```

**What it does:**
1. Wipes ALL data from database (DESTRUCTIVE!)
2. Re-applies migrations
3. Seeds with sample test data
4. Runs on EVERY startup

**Sample data includes:**
- 3 users (admin, developer, viewer)
- 2 AI provider keys (OpenAI, Anthropic)
- 5 API tokens (various states: active, inactive, expired)
- 3 usage limits (unlimited, standard, free tier)
- 2 project-provider assignments

**Use case:**
- Manual testing with clean state
- Integration testing with known data
- Development environment reset

**Security warning:**
⚠️ NEVER enable `wipe_and_seed = true` in production! This will delete all data on every startup.

### Manual Seed

Alternatively, seed manually using the API:

```rust
use iron_token_manager::seed::{wipe_database, seed_all};

let pool = storage.pool();
wipe_database(pool).await?;  // Optional: wipe first
seed_all(pool).await?;
```

### Sample Data Details

**Users:**
- Username: `admin` / Password: `password123` / Role: admin
- Username: `developer` / Password: `password123` / Role: user
- Username: `viewer` / Password: `password123` / Role: user (inactive)

**API Tokens:**
- Admin token (never expires)
- Developer token (expires in 30 days)
- Project token (project_alpha)
- Inactive token
- Expired token

**Provider Keys:**
- OpenAI (enabled, $50 balance)
- Anthropic (enabled, $100 balance)

**Usage Limits:**
- Admin: unlimited
- Developer: 1M tokens/day, 60 req/min, $50/month
- Viewer: 100k tokens/day, 10 req/min, free tier

## Security Considerations

### Config File Permissions

```bash
# Production config should be readable only by application user
chmod 600 config.prod.toml
chown app-user:app-group config.prod.toml
```

### Gitignore

Ensure `.gitignore` includes:
```
config.prod.toml
*.db
*.db-shm
*.db-wal
```

### Secret Management

For production deployments, consider:

1. **Environment Variables** (recommended)
   ```bash
   export DATABASE_URL=$(vault read -field=url secret/database)
   ```

2. **Config Management Tools**
   - HashiCorp Vault
   - AWS Secrets Manager
   - Kubernetes Secrets

3. **File Permissions**
   - Restrict config file access (`chmod 600`)
   - Use dedicated service account

## Troubleshooting

### Error: Config file not found

```
Error: TokenError
```

**Cause:** Config file doesn't exist for specified environment

**Solution:**
```bash
# Check IRON_ENV
echo $IRON_ENV

# Check if file exists
ls -la config.${IRON_ENV}.toml

# Create from template (production only)
cp config.prod.toml.example config.prod.toml
```

### Error: Failed to parse config file

```
Error: TokenError
```

**Cause:** Invalid TOML syntax in config file

**Solution:**
```bash
# Validate TOML syntax
toml-cli check config.dev.toml

# Or use online validator
# https://www.toml-lint.com/
```

### Error: Database connection failed

```
Error: TokenError
```

**Cause:** Invalid database URL or insufficient permissions

**Solution:**
```bash
# Check database URL format
sqlite:///./path/to/db.db?mode=rwc

# Check directory exists
mkdir -p $(dirname /path/to/db.db)

# Check permissions
ls -la /path/to/db.db
```

### Migrations Not Applied

**Symptom:** Tables don't exist

**Cause:** `auto_migrate = false` in config

**Solution:**
```toml
# config.prod.toml
[database]
auto_migrate = true  # Enable automatic migrations
```

## Examples

### Development Setup

```bash
# 1. Use default development config
export IRON_ENV=development

# 2. Override database path
export DATABASE_URL=sqlite:///./my-dev.db?mode=rwc

# 3. Run application
cargo run
```

### CI/CD Pipeline

```yaml
# .github/workflows/test.yml
env:
  IRON_ENV: test
  DATABASE_URL: sqlite:///:memory:?mode=rwc
  DATABASE_AUTO_MIGRATE: true

steps:
  - run: cargo test
```

### Production Deployment

```bash
# 1. Create production config
cp config.prod.toml.example config.prod.toml

# 2. Edit production config
vim config.prod.toml

# 3. Set environment
export IRON_ENV=production

# 4. Override sensitive values via environment
export DATABASE_URL=$(vault read -field=url secret/database)

# 5. Deploy
./deploy.sh
```

## API Reference

### `Config` Struct

```rust
pub struct Config {
  pub database: DatabaseConfig,
  pub development: Option<DevelopmentConfig>,
  pub production: Option<ProductionConfig>,
  pub test: Option<TestConfig>,
}
```

### Loading Methods

```rust
// Load from IRON_ENV (default: "development")
let config = Config::load()?;

// Load specific environment
let config = Config::from_env("production")?;

// Load specific file
let config = Config::from_file("config.custom.toml")?;

// Create default development config
let config = Config::default_dev();

// Create default test config
let config = Config::default_test();
```

### TokenStorage Initialization

```rust
// From config (recommended)
let storage = TokenStorage::from_config().await?;

// From config object
let config = Config::load()?;
let storage = TokenStorage::from_config_object(&config).await?;

// Legacy URL method
let storage = TokenStorage::new("sqlite:///db.db?mode=rwc").await?;
```

## References

- **Config files:** `config.*.toml`
- **Config module:** `src/config.rs`
- **TokenStorage:** `src/storage.rs`
- **Database initialization guide:** `docs/database_initialization.md`
- **Quick reference:** `docs/quick_reference_database.md`

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2024-12-11 | 1.0.0 | Initial configuration guide (Phase 3) |
