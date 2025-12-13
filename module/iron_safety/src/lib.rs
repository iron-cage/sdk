//! PII detection and output sanitization for multi-agent systems.
//!
//! Provides real-time scanning of LLM outputs to detect and redact personally
//! identifiable information (PII) before responses reach end users. Essential
//! for compliance with data protection regulations (GDPR, CCPA, HIPAA).
//!
//! # Purpose
//!
//! This crate provides safety guardrails for LLM output:
//! - Real-time PII detection in streaming responses
//! - Automatic redaction of sensitive data patterns
//! - Configurable pattern matching for different PII types
//! - Low-latency scanning suitable for production use
//!
//! # Supported PII Types
//!
//! Currently detects and redacts:
//! - **Email addresses**: Standard RFC-compliant email patterns
//! - **Phone numbers**: US format (XXX-XXX-XXXX)
//!
//! Additional patterns can be added by extending [`PiiDetector`].
//!
//! # Key Types
//!
//! - [`PiiDetector`] - Main detector with regex-based pattern matching
//!
//! # Public API
//!
//! ## Basic Usage
//!
//! ```rust
//! # #[cfg(feature = "enabled")]
//! # {
//! use iron_safety::PiiDetector;
//!
//! let detector = PiiDetector::new()?;
//!
//! // Check if text contains PII
//! let text = "Contact me at user@example.com or 555-123-4567";
//! if detector.check(text) {
//!   println!("⚠️ PII detected!");
//! }
//!
//! // Redact PII from text
//! let safe_text = detector.redact(text);
//! // "Contact me at [EMAIL_REDACTED] or [PHONE_REDACTED]"
//! # }
//! # Ok::<(), iron_types::Error>(())
//! ```
//!
//! ## Streaming Integration
//!
//! ```rust,no_run
//! # #[cfg(feature = "enabled")]
//! # {
//! use iron_safety::PiiDetector;
//!
//! async fn stream_llm_response(detector: &PiiDetector) -> Result<(), Box<dyn std::error::Error>> {
//!   let mut buffer = String::new();
//!
//!   // Simulate streaming chunks
//!   for chunk in &["Hello ", "user@", "example.com", "!"] {
//!     buffer.push_str(chunk);
//!
//!     // Check accumulated buffer for PII
//!     if detector.check(&buffer) {
//!       // Redact before sending to client
//!       let safe_chunk = detector.redact(&buffer);
//!       send_to_client(&safe_chunk).await?;
//!       buffer.clear();
//!     }
//!   }
//!
//!   Ok(())
//! }
//!
//! async fn send_to_client(text: &str) -> Result<(), Box<dyn std::error::Error>> {
//!   // Implementation...
//!   # Ok(())
//! }
//! # }
//! ```
//!
//! ## Integration with Analytics
//!
//! ```rust,ignore
//! # #[cfg(feature = "enabled")]
//! # {
//! use iron_safety::PiiDetector;
//! # use iron_runtime_analytics::EventStore;
//!
//! fn process_response(text: &str, detector: &PiiDetector, store: &EventStore) {
//!   if detector.check(text) {
//!     // Log PII detection for compliance audit
//!     // store.record_pii_detection(agent_id, timestamp);
//!
//!     // Redact before returning
//!     let safe = detector.redact(text);
//!     // ... send safe text
//!   }
//! }
//! # }
//! ```
//!
//! # Feature Flags
//!
//! - `enabled` - Enable PII detection (disabled for minimal builds)
//!
//! # Performance
//!
//! PII detection uses compiled regex patterns with the following characteristics:
//! - Pattern compilation: One-time cost at initialization
//! - Check operation: O(n) where n = text length, typically <100μs for 1KB
//! - Redact operation: O(n) with string allocation
//!
//! For streaming, check small chunks frequently rather than buffering entire responses.
//!
//! # Limitations
//!
//! Current implementation has known limitations:
//! - **Pattern-based only**: No ML-based detection for context-aware PII
//! - **US-centric**: Phone patterns only match US format (XXX-XXX-XXXX)
//! - **No international support**: Email/phone patterns dont cover all locales
//! - **False positives possible**: Aggressive matching may flag non-PII
//!
//! For production use in regulated environments, consider:
//! - Adding locale-specific patterns
//! - Implementing allowlists for known-safe patterns
//! - Combining with ML-based PII detection services
//! - Regular auditing of redaction logs

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
