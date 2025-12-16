# iron_config

Unified configuration management for Iron Runtime with precedence-based resolution.

[![Documentation](https://img.shields.io/badge/docs-ironcage.ai-blue.svg)](https://ironcage.ai/docs)

## Configuration Precedence

Configuration is resolved using the following precedence (highest to lowest):

1. **Environment Variables** - `{MODULE}_KEY_NAME` format (highest priority)
2. **Project Config** - `{workspace}/config/{module}.{env}.toml`
3. **User Config** - `~/.config/iron/{module}.toml`
4. **Workspace Defaults** - `{workspace}/config/{module}.default.toml`
5. **Crate Defaults** - Hardcoded defaults in crate (lowest priority)


## Features

- **Precedence-based resolution**: Clear, predictable configuration loading
- **Environment variable overrides**: Override any config value via env vars
- **Multiple file layers**: Project, user, and workspace-level configuration
- **Type-safe deserialization**: Automatic conversion to Rust types via serde
- **Source tracking**: Know which layer provided each configuration value
- **Workspace-relative paths**: Context-independent configuration files


## Quick Start

### Basic Example

```rust
use iron_config::ConfigLoader;
use serde::Deserialize;

#[derive(Deserialize)]
struct DatabaseConfig {
  url: String,
  max_connections: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Load configuration for module
  let loader = ConfigLoader::new("iron_token_manager")?;

  // Get individual values
  let url: String = loader.get("database.url")?;
  println!("Database URL: {}", url);

  // Get configuration section as struct
  let db_config: DatabaseConfig = loader.get_section("database")?;
  println!("Max connections: {}", db_config.max_connections);

  Ok(())
}
```

### Environment Variable Overrides

Environment variables follow the pattern `{MODULE}_{KEY_PATH}`:

```bash
# Override database.url for iron_token_manager
export IRON_TOKEN_MANAGER_DATABASE_URL="sqlite:///custom.db"

# Override database.max_connections
export IRON_TOKEN_MANAGER_DATABASE_MAX_CONNECTIONS="10"

# Override nested values: database.pool.timeout
export IRON_TOKEN_MANAGER_DATABASE_POOL_TIMEOUT="60"
```


<details>
<summary>Configuration Files</summary>

Configuration files use TOML format:

**Project Config** (`{workspace}/config/iron_token_manager.development.toml`):
```toml
[database]
url = "sqlite:///{workspace}/iron.db?mode=rwc"
max_connections = 5
auto_migrate = true

[development]
debug = true
auto_seed = false
```

**User Config** (`~/.config/iron/iron_token_manager.toml`):
```toml
[development]
debug = true  # Personal preference
```

</details>


<details>
<summary>Advanced Usage</summary>

**Load with specific environment**:
```rust
let loader = ConfigLoader::with_env("iron_token_manager", "production")?;
```

**Provide default values**:
```rust
let defaults = r#"
[database]
url = "sqlite:///:memory:"
max_connections = 5
"#;

let loader = ConfigLoader::with_defaults("iron_token_manager", defaults)?;
```

**Get value with source information**:
```rust
let (url, source) = loader.get_with_source::<String>("database.url")?;
println!("Database URL: {} (from {})", url, source);
// Output: Database URL: sqlite:///iron.db (from env:IRON_TOKEN_MANAGER_DATABASE_URL)
```

**Debug configuration**:
```rust
println!("{}", loader.debug_summary());
```

</details>


<details>
<summary>Integration & Migration</summary>

### Adding to your crate

**Cargo.toml**:
```toml
[dependencies]
iron_config = { workspace = true }
serde = { workspace = true, features = ["derive"] }
```

### Migration from custom config

Before (custom config loading):
```rust
// Old: 252 lines of HashMap-based config hierarchy
let config = load_config_from_multiple_sources()?;
```

After (iron_config):
```rust
// New: Single line with full precedence system
let loader = ConfigLoader::new("my_module")?;
let config: MyConfig = loader.get_section("my_section")?;
```

</details>


<details>
<summary>Phase 2 Status</summary>

This crate is part of Phase 2 (Configuration Unification) of the workspace_tools adoption plan:

- ✅ Create unified configuration system
- ✅ Migrate iron_token_manager (verified via 5-tier testing)
- ✅ Migrate iron_cli (verified via 5-tier testing)
- ⏳ Migrate iron_control_api (future work)
- ✅ Eliminate module-specific config loaders (iron_token_manager, iron_cli)
- ⏳ Workspace-wide migration (future phases)

</details>


<details>
<summary>File Structure & Dependencies</summary>

```
iron_config/
├── src/
│   ├── lib.rs           # Public API
│   ├── error.rs         # Error types
│   ├── layer.rs         # Configuration layer implementations
│   └── loader.rs        # ConfigLoader implementation
├── tests/
│   ├── precedence_test.rs   # Precedence system tests
│   ├── env_layer_test.rs    # Environment variable layer tests
│   └── file_layer_test.rs   # File layer tests
├── Cargo.toml
└── readme.md
```

## Dependencies

- **workspace_tools** - Workspace detection and path resolution
- **serde** - Serialization/deserialization
- **toml** - TOML parsing
- **thiserror** - Error handling
- **dirs** - User directory detection

</details>


## Testing

Run tests:
```bash
cargo test --all-features
```

Run tests with clippy:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```


## License

MIT
