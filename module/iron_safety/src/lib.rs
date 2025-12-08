//! Safety module: PII detection and validation
//!
//! Provides real-time PII detection and output sanitization for multi-agent systems.

#![cfg_attr(not(feature = "enabled"), allow(unused))]

#[cfg(feature = "enabled")]
mod implementation
{
  use iron_types::{Result, Error};
  use regex::Regex;
  use std::sync::Arc;

  /// PII detector with configurable patterns
  pub struct PiiDetector
  {
    email_pattern: Arc< Regex >,
    phone_pattern: Arc< Regex >,
  }

  impl PiiDetector
  {
    /// Create new detector with default patterns
    pub fn new() -> Result< Self >
    {
      Ok(Self {
        email_pattern: Arc::new(
          Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
            .map_err(|e| Error::Config(e.to_string()))?
        ),
        phone_pattern: Arc::new(
          Regex::new(r"\d{3}-\d{3}-\d{4}")
            .map_err(|e| Error::Config(e.to_string()))?
        ),
      })
    }

    /// Check if text contains PII
    pub fn check(&self, text: &str) -> bool
    {
      self.email_pattern.is_match(text) || self.phone_pattern.is_match(text)
    }

    /// Redact PII from text
    pub fn redact(&self, text: &str) -> String
    {
      let text = self.email_pattern.replace_all(text, "[EMAIL_REDACTED]");
      self.phone_pattern.replace_all(&text, "[PHONE_REDACTED]").to_string()
    }
  }
}

#[cfg(feature = "enabled")]
pub use implementation::*;
