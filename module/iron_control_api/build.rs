//! Build script for compile-time metadata
//!
//! Captures:
//! - Git commit SHA (`VERGEN_GIT_SHA`)
//! - Build timestamp (`VERGEN_BUILD_TIMESTAMP`)
//!
//! These are embedded at compile time using env! macro,
//! ensuring version metadata is static (not runtime).

use vergen_gitcl::{BuildBuilder, Emitter, GitclBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let build = BuildBuilder::default().build_timestamp(true).build()?;
  let gitcl = GitclBuilder::default().sha(false).build()?;

  Emitter::default()
    .add_instructions(&build)?
    .add_instructions(&gitcl)?
    .emit()?;

  Ok(())
}
