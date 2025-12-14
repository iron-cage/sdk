//! Build script for compile-time metadata
//!
//! Captures:
//! - Git commit SHA (VERGEN_GIT_SHA)
//! - Build timestamp (VERGEN_BUILD_TIMESTAMP)
//!
//! These are embedded at compile time using env! macro,
//! ensuring version metadata is static (not runtime).

use vergen::EmitBuilder;

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  EmitBuilder::builder()
    .git_sha( true )
    .build_timestamp()
    .emit()?;

  Ok( () )
}
