//! # `iron_secrets`
//!
//! Secure secrets management for AI agents.
//!
//! Provides encrypted storage, access control, audit logging, and runtime injection
//! of sensitive credentials (API keys, database passwords, cookies).
//!
//! ## Architecture
//!
//! This crate provides enterprise-grade secrets management with AES-256-GCM encryption,
//! Argon2id key derivation, and comprehensive audit logging for SOC 2 compliance.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use iron_secrets::SecretsManager;
//!
//! // Master key from environment
//! std::env::set_var("IRON_CAGE_MASTER_KEY", "your-base64-encoded-32-byte-key");
//!
//! let manager = SecretsManager::new(":memory:").await?;
//!
//! // Create secret
//! manager.create_secret("OPENAI_API_KEY", "sk-proj-...", Environment::Production).await?;
//!
//! // Read secret
//! let value = manager.read_secret("OPENAI_API_KEY", Environment::Production).await?;
//!
//! // Update secret (zero-downtime rotation)
//! manager.update_secret("OPENAI_API_KEY", "sk-proj-new-...", Environment::Production).await?;
//! ```
//!
//! ## Compliance
//!
//! Follows specification: `module/iron_secrets/spec.md`
//!
//! ## Features
//!
//! - **`enabled`** (default) - Core secrets management functionality
//! - **`full`** - All features enabled
//!
//! ## Known Pitfalls
//!
//! ### Master Key Loss
//!
//! **Issue**: Loss of master key = irreversible loss of ALL secrets.
//!
//! **Why**: AES-256-GCM encryption with derived keys. Without master key, decryption is
//! cryptographically impossible.
//!
//! **Prevention**:
//! 1. Store master key in secure location (AWS KMS for production)
//! 2. Never commit master key to git
//! 3. Backup master key in separate secure location
//! 4. For pilot: Accept risk (local master key only)
//!
//! ### Secret Redaction in Logs
//!
//! **Issue**: Secrets may accidentally leak in logs if not properly redacted.
//!
//! **Prevention**:
//! 1. Never log plaintext secrets
//! 2. Use zeroize crate to clear secrets from memory
//! 3. Always display masked values in UI
//! 4. Audit log entries for accidental leaks

#![cfg_attr(not(feature = "enabled"), allow(unused))]
#![warn(missing_docs)]

#[cfg(feature = "enabled")]
pub mod error;

/// Secrets management core functionality
#[cfg(feature = "enabled")]
pub mod secrets_manager;

/// Cryptographic operations for secret encryption/decryption
#[cfg(feature = "enabled")]
pub mod crypto;

/// Encrypted storage backend for secrets
#[cfg(feature = "enabled")]
pub mod storage;

/// Access control and authorization for secrets
#[cfg(feature = "enabled")]
pub mod access_control;

/// Audit logging for secrets operations
#[cfg(feature = "enabled")]
pub mod audit;
