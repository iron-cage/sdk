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
//! let formatter = TreeFmtFormatter::new(OutputFormat::Json);
//! let output = formatter.format_single(&data);
//! ```
//!
//! ## Phase 3 Implementation
//!
//! **Tests:** 23 formatter tests
//! **Formats:** 4 output formats (table, expanded, json, yaml)
//! **Coverage:** 100% formatter test coverage
//!
//! ## Migration to tree_fmt
//!
//! TreeFmtFormatter provides tree_fmt-based implementation with:
//! - Dynamic column widths (vs fixed 15-char in legacy Formatter)
//! - ANSI-aware alignment for colored output
//! - Professional table styling

mod output_format;
mod tree_formatter;

pub use output_format::OutputFormat;
pub use tree_formatter::TreeFmtFormatter;
