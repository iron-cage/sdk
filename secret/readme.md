# Secret Management

This directory contains all sensitive credentials and API keys for the Iron Runtime workspace.

## Directory Structure

```
secret/
├── readme.md           # This documentation file (committed)
├── iron.example.sh     # Template for server secrets (committed)
├── api_keys.example.sh # Template for API keys (committed)
├── -iron.sh            # Server secrets: JWT, master key, database (gitignored)
├── -api_keys.sh        # AI provider API keys (gitignored)
└── -*.sh               # Additional service-specific secrets (gitignored)
```

## Naming Convention

All secret files **MUST** start with a hyphen (`-`) prefix:
- `-iron.sh` - Iron Runtime server configuration
- `-api_keys.sh` - AI provider API keys (OpenAI, Anthropic, Gemini)
- `-database.conf` - Database credentials (if needed beyond SQLite)

The hyphen prefix ensures files are:
1. Automatically gitignored (via `-*` pattern)
2. Less likely to be accidentally processed by tools
3. Clearly identifiable as sensitive

## File Format

Secret files use shell-sourceable `key=value` format:

```sh
# Source secrets into your environment
source secret/-iron.sh
source secret/-api_keys.sh
```

## Setup Instructions

### Quick Setup

```bash
# 1. Copy example templates
cp secret/iron.example.sh secret/-iron.sh
cp secret/api_keys.example.sh secret/-api_keys.sh

# 2. Generate and add secrets to -iron.sh
echo "JWT_SECRET: $(openssl rand -hex 32)"
echo "IRON_SECRETS_MASTER_KEY: $(openssl rand -base64 32)"

# 3. Edit -iron.sh and -api_keys.sh with your values
```

### 1. Server Secrets (`-iron.sh`)

Copy from template and fill in generated values:

```bash
cp secret/iron.example.sh secret/-iron.sh
```

Generate secrets:
```bash
# Generate JWT secret (64 hex chars)
openssl rand -hex 32

# Generate master key for encryption (base64)
openssl rand -base64 32
```

### 2. AI Provider API Keys (`-api_keys.sh`)

Copy from template and add your API keys:

```bash
cp secret/api_keys.example.sh secret/-api_keys.sh
```

Get your keys from:
- OpenAI: https://platform.openai.com/api-keys
- Anthropic: https://console.anthropic.com/settings/keys
- Google Gemini: https://aistudio.google.com/app/apikey

## Usage

### Loading Secrets in Shell

```bash
# Load all secrets
source secret/-iron.sh
source secret/-api_keys.sh

# Run server
cargo run --bin iron_control_api_server
```

### Loading Secrets in Rust Tests

Integration tests use the workspace_tools crate to load secrets:

```rust
let api_key = workspace_tools::secret::load_secret("anthropic_api_key")
    .expect("ANTHROPIC_API_KEY not found in secret/-api_keys.sh");
```

### Docker Compose

For Docker deployments, secrets are passed via environment variables:

```bash
source secret/-iron.sh
source secret/-api_keys.sh
docker compose up -d
```

## Security Notes

1. **Never commit secrets** - All `-*` files are gitignored
2. **Backup master key** - Loss of `IRON_SECRETS_MASTER_KEY` = loss of encrypted API keys
3. **Production security** - Use proper secret management (AWS KMS, HashiCorp Vault, etc.)
4. **CI/CD** - Use secure environment variables or secret management systems

## Migration from .env

If you have an existing `.env` file, migrate your secrets:

```bash
# Create secret files from .env
grep -E "^(DATABASE_URL|JWT_SECRET|IRON_SECRETS_MASTER_KEY)" .env > secret/-iron.sh
grep -E "^(OPENAI|ANTHROPIC|GEMINI)" .env > secret/-api_keys.sh

# Remove old .env file
rm .env
```
