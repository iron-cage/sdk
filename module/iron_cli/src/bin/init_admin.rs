//! Iron Cage Admin Account Initialization
//!
//! Standalone CLI tool for creating the initial Admin account during first deployment.
//! Connects directly to the `SQLite` database (bypasses the HTTP API layer).
//!
//! Usage:
//!   iron-init-admin --db-url `sqlite:///var/lib/iron_cage/iron_cage.db`
//!   iron-init-admin  # uses `DATABASE_URL` env var
//!
//! The tool is idempotent: if an admin already exists, it reports this and exits.
//! Generated credentials are written to `.persistent/internal_deployment.md`.

// Binary entry points are allowed to use println! for final output
#![allow(clippy::disallowed_macros)]

use std::{
  fs::{self, Permissions},
  path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use rand::RngExt;
use sqlx::sqlite::SqlitePoolOptions;

/// Iron Cage initial admin account creator
#[derive(Parser, Debug)]
#[command(name = "iron-init-admin")]
#[command(about = "Create initial Admin account for Iron Cage deployment")]
struct Args {
  /// `SQLite` database URL (e.g. `sqlite:///var/lib/iron_cage/iron_cage.db`)
  /// Falls back to `DATABASE_URL` environment variable if not provided.
  #[arg(short, long, env = "DATABASE_URL")]
  db_url: String,

  /// Output directory for credentials file (default: .persistent/)
  #[arg(short, long, default_value = ".persistent")]
  output_dir: PathBuf,

  /// Admin username (default: admin)
  #[arg(short, long, default_value = "admin")]
  username: String,

  /// Admin email (default: admin@ironcage.internal)
  #[arg(short, long, default_value = "admin@ironcage.internal")]
  email: String,
}

/// `BCrypt` cost factor - must match `UserService` (issue-001: never use `DEFAULT_COST`).
const BCRYPT_COST: u32 = 12;

/// Password length for generated admin password.
const PASSWORD_LENGTH: usize = 24;

/// Characters used for password generation (alphanumeric + special).
const PASSWORD_CHARSET: &[u8] =
  b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%&*-_=+";

#[tokio::main]
async fn main() -> Result<()> {
  let args = Args::parse();

  println!("Iron Cage - Admin Account Initialization");
  println!("=========================================");
  println!();

  // Connect to database
  println!("Connecting to database...");
  let pool = SqlitePoolOptions::new()
    .max_connections(1)
    .connect(&args.db_url)
    .await
    .context("Failed to connect to database")?;

  // Apply migrations (idempotent)
  println!("Applying migrations...");
  iron_token_manager::apply_all_migrations(&pool)
    .await
    .map_err(|e| anyhow::anyhow!("{e:?}"))
    .context("Failed to apply migrations")?;

  // Check if admin already exists
  let existing_admin: Option<(String,)> =
    sqlx::query_as("SELECT username FROM users WHERE role = 'admin' AND is_active = 1 LIMIT 1")
      .fetch_optional(&pool)
      .await
      .context("Failed to query users")?;

  if let Some((username,)) = existing_admin {
    println!("\nAdmin account already exists: {username}");
    println!("No action taken. To create another admin, use the Control API.");
    return Ok(());
  }

  // Generate secure password
  let password = generate_password();

  // Hash password with BCrypt (cost=12)
  let password_hash = bcrypt::hash(&password, BCRYPT_COST).context("Failed to hash password")?;

  // Generate user ID
  let user_id = format!("user_{}", uuid::Uuid::new_v4());
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Insert admin user
  sqlx::query(
    "INSERT INTO users (id, username, password_hash, email, role, is_active, created_at) \
     VALUES ($1, $2, $3, $4, 'admin', 1, $5)",
  )
  .bind(&user_id)
  .bind(&args.username)
  .bind(&password_hash)
  .bind(&args.email)
  .bind(now_ms)
  .execute(&pool)
  .await
  .context("Failed to create admin user")?;

  // Write credentials file
  write_credentials_file(&args.output_dir, &args.username, &password, &args.email)?;

  println!();
  println!("Admin account created successfully!");
  println!("  Username: {}", args.username);
  println!("  Email:    {}", args.email);
  println!();
  println!(
    "Credentials saved to: {}/internal_deployment.md",
    args.output_dir.display()
  );
  println!();
  println!("IMPORTANT: Store this file securely and restrict access.");
  println!("           The password is shown only once at generation time.");

  Ok(())
}

/// Generate a cryptographically secure random password
fn generate_password() -> String {
  let mut rng = rand::rng();
  (0..PASSWORD_LENGTH)
    .map(|_| {
      let idx = rng.random_range(0..PASSWORD_CHARSET.len());
      PASSWORD_CHARSET[idx] as char
    })
    .collect()
}

/// Write credentials to `.persistent/internal_deployment.md`
fn write_credentials_file(
  output_dir: &PathBuf,
  username: &str,
  password: &str,
  email: &str,
) -> Result<()> {
  fs::create_dir_all(output_dir)
    .with_context(|| format!("Failed to create directory {}", output_dir.display()))?;

  let file_path = output_dir.join("internal_deployment.md");
  let date = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

  let content = format!(
    "\
# Iron Cage Internal Deployment

Generated: {date}

## Admin Credentials

- Username: {username}
- Password: {password}
- Email: {email}
- Role: admin

## Service Endpoints

- Control API: https://internal.host/api
- Server Proxy: https://internal.host/proxy

## Notes

- This file was generated by `iron-init-admin`
- The password above is the ONLY record - it is not stored in plaintext anywhere else
- Change these endpoints to match your actual deployment
- Restrict access to this file (chmod 600)
"
  );

  fs::write(&file_path, content).context("Failed to write credentials file")?;

  // Set restrictive permissions (Unix only)
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;

    let permissions = Permissions::from_mode(0o600);
    fs::set_permissions(&file_path, permissions).context("Failed to set file permissions")?;
  }

  Ok(())
}
