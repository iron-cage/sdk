//! Utility functions and types for analytics.

use core::str::FromStr;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Get current Unix timestamp in milliseconds.
#[must_use]
pub fn current_time_ms() -> u64 {
  u64::try_from(
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_millis(),
  )
  .unwrap_or(0)
}

/// LLM Provider enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
  /// `OpenAI` provider (e.g. GPT models).
  OpenAI,
  /// Anthropic provider (e.g. Claude models).
  Anthropic,
  /// Google Gemini provider.
  Gemini,
  /// xAI provider (e.g. Grok models).
  #[serde(rename = "xai")]
  XAI,

  /// Fallback for unknown/unsupported providers during deserialization.
  #[serde(other)]
  #[default]
  Unknown,
}

impl Provider {
  /// Get the canonical string representation.
  #[must_use]
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::OpenAI => "openai",
      Self::Anthropic => "anthropic",
      Self::Gemini => "gemini",
      Self::XAI => "xai",
      Self::Unknown => "unknown",
    }
  }

  /// Convert to Arc< str > for use in event types.
  #[must_use]
  pub fn to_arc_str(self) -> Arc<str> {
    Arc::from(self.as_str())
  }
}

impl fmt::Display for Provider {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

impl FromStr for Provider {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      s if s.eq_ignore_ascii_case("openai") => Ok(Self::OpenAI),
      s if s.eq_ignore_ascii_case("anthropic") => Ok(Self::Anthropic),
      s if s.eq_ignore_ascii_case("gemini") => Ok(Self::Gemini),
      s if s.eq_ignore_ascii_case("xai") => Ok(Self::XAI),
      _ => Ok(Self::Unknown),
    }
  }
}

impl From<&str> for Provider {
  fn from(s: &str) -> Self {
    s.parse().unwrap_or_default()
  }
}

impl From<Provider> for Arc<str> {
  fn from(p: Provider) -> Self {
    p.to_arc_str()
  }
}

/// Infer the LLM provider from a model name (text models only).
///
/// Uses zero-allocation checking where possible.
#[must_use]
pub fn infer_provider(model: &str) -> Provider {
  // Check OpenAI (gpt-*, o1-*, o3-*, chatgpt-*)
  if has_prefix_ignore_case(model, "gpt-")
    || has_prefix_ignore_case(model, "o1-")
    || has_prefix_ignore_case(model, "o3-")
    || has_prefix_ignore_case(model, "chatgpt-")
  {
    return Provider::OpenAI;
  }

  // Check Anthropic (claude-*)
  if has_prefix_ignore_case(model, "claude-") {
    return Provider::Anthropic;
  }

  // Check Gemini (gemini-*)
  if has_prefix_ignore_case(model, "gemini-") {
    return Provider::Gemini;
  }

  // Check xAI (grok-*)
  if has_prefix_ignore_case(model, "grok-") {
    return Provider::XAI;
  }

  Provider::Unknown
}

/// Helper to check prefix case-insensitively without allocation.
#[inline]
fn has_prefix_ignore_case(s: &str, prefix: &str) -> bool {
  if s.len() < prefix.len() {
    return false;
  }
  s[..prefix.len()].eq_ignore_ascii_case(prefix)
}
