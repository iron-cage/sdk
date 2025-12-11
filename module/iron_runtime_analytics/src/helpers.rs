use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::fmt;

pub fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// LLM Provider enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Anthropic,

    /// Fallback for unknown/unsupported providers during deserialization
    #[serde(other)]
    Unknown,
}

impl Provider {
    /// Get the canonical string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
            Self::Unknown => "unknown",
        }
    }

    /// Convert to Arc<str> for use in event types
    #[must_use]
    pub fn to_arc_str(self) -> Arc<str> {
        Arc::from(self.as_str())
    }
}

// 1. Implement Default
impl Default for Provider {
    fn default() -> Self {
        Self::Unknown
    }
}

// 2. Implement Display (integrates with format! and println!)
impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// 3. Implement standard FromStr (allows "string".parse::<Provider>())
impl FromStr for Provider {
    type Err = (); // Infallible, falls back to Unknown

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.eq_ignore_ascii_case("openai") => Ok(Self::OpenAI),
            s if s.eq_ignore_ascii_case("anthropic") => Ok(Self::Anthropic),
            _ => Ok(Self::Unknown),
        }
    }
}

// Convenience: &str -> Provider
impl From<&str> for Provider {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or_default()
    }
}

// Convenience: Provider -> Arc<str>
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

    Provider::Unknown
}

/// Helper to check prefix case-insensitively without allocation
#[inline]
fn has_prefix_ignore_case(s: &str, prefix: &str) -> bool {
    if s.len() < prefix.len() {
        return false;
    }
    s[..prefix.len()].eq_ignore_ascii_case(prefix)
}