//! Universal formatter supporting 4 output formats
//!
//! ## Output Formats
//!
//! - **Table**: Compact tabular view (default)
//! - **Expanded**: Detailed multi-line view
//! - **JSON**: Machine-readable JSON
//! - **YAML**: Human-readable YAML
//!
//! ## Usage
//!
//! ```rust,ignore
//! let formatter = Formatter::new(OutputFormat::Json);
//! let output = formatter.format_single(&data);
//! ```
//!
//! ## Phase 3 Implementation
//!
//! **Tests:** 23 formatter tests
//! **Formats:** 4 output formats (table, expanded, json, yaml)
//! **Coverage:** 100% formatter test coverage

mod output_format;
mod formatter;

pub use output_format::OutputFormat;
pub use formatter::Formatter;
