//! Output format enum and parsing

use std::fmt;
use std::str::FromStr;

/// Output format for CLI responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat
{
  /// Compact tabular view (default)
  #[default]
  Table,
  /// Detailed multi-line view
  Expanded,
  /// Machine-readable JSON
  Json,
  /// Human-readable YAML
  Yaml,
}

impl FromStr for OutputFormat
{
  type Err = InvalidFormat;

  fn from_str(s: &str) -> Result<Self, Self::Err>
  {
    match s.to_lowercase().as_str()
    {
      "table" => Ok(Self::Table),
      "expanded" => Ok(Self::Expanded),
      "json" => Ok(Self::Json),
      "yaml" => Ok(Self::Yaml),
      _ => Err(InvalidFormat(s.to_string())),
    }
  }
}

/// Error when parsing invalid format string
#[derive(Debug, Clone)]
pub struct InvalidFormat(String);

impl fmt::Display for InvalidFormat
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
  {
    write!(
      f,
      "Invalid output format '{}'. Valid formats: table, expanded, json, yaml",
      self.0
    )
  }
}

impl std::error::Error for InvalidFormat {}
