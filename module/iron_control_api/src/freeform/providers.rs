use serde::{Deserialize, Serialize};

/// A supported inference provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnownProvider {
  /// Google Gemini.
  Gemini,
  /// `OpenAI`.
  OpenAi,
  /// Anthropic.
  Anthropic,
  /// Mistral AI.
  Mistral,
  /// Cohere.
  Cohere,
}

impl KnownProvider {
  fn from_str(s: &str) -> Option<Self> {
    match s.to_lowercase().as_str() {
      "gemini" => Some(Self::Gemini),
      "openai" => Some(Self::OpenAi),
      "anthropic" => Some(Self::Anthropic),
      "mistral" => Some(Self::Mistral),
      "cohere" => Some(Self::Cohere),
      _ => None,
    }
  }

  fn validate_key_prefix(&self, key: &str) -> bool {
    match self {
      Self::OpenAi | Self::Mistral => key.starts_with("sk-"),
      Self::Anthropic => key.starts_with("sk-ant-"),
      Self::Gemini => key.starts_with("AIzaSy"),
      Self::Cohere => !key.is_empty(),
    }
  }
}

/// A single parsed provider key entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderEntry {
  /// The resolved provider.
  pub provider: KnownProvider,
  /// The raw API key value.
  pub key: String,
}

/// A per-line parse error.
#[derive(Debug, PartialEq)]
pub struct LineError {
  /// 1-indexed line number in the original input.
  pub line: usize,
  /// Trimmed content of the offending line.
  pub content: String,
  /// Reason the line was rejected.
  pub kind: LineErrorKind,
}

/// Reason a line was rejected during provider-key parsing.
#[derive(Debug, PartialEq)]
pub enum LineErrorKind {
  /// The provider label is not in the supported set.
  UnknownProvider(String),
  /// The key does not start with the expected prefix for its provider.
  InvalidKeyPrefix,
  /// The line does not match the expected `provider: key` format.
  MalformedLine,
}

impl core::fmt::Display for LineError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let kind = match &self.kind {
      LineErrorKind::UnknownProvider(p) => format!("unknown provider '{p}'"),
      LineErrorKind::InvalidKeyPrefix => "invalid key prefix for provider".to_string(),
      LineErrorKind::MalformedLine => "expected 'provider: key' format".to_string(),
    };
    write!(f, "line {}: {} ({})", self.line, kind, self.content)
  }
}

/// Parses a paste block of `provider: key` lines.
///
/// Each non-blank, non-comment line must be in the form `provider: key` where
/// `provider` is one of `gemini`, `openai`, `anthropic`, `mistral`, `cohere`.
/// Apply is all-or-nothing: a single invalid line aborts the entire block.
///
/// # Errors
///
/// Returns a list of [`LineError`]s if any line fails validation.
pub fn parse(input: &str) -> Result<Vec<ProviderEntry>, Vec<LineError>> {
  let mut entries: Vec<ProviderEntry> = Vec::new();
  let mut errors: Vec<LineError> = Vec::new();

  for (idx, raw) in input.lines().enumerate() {
    let line_no = idx + 1;
    let trimmed = raw.trim();

    if trimmed.is_empty() || trimmed.starts_with('#') {
      continue;
    }

    let Some((lhs, rhs)) = trimmed.split_once(':') else {
      errors.push(LineError {
        line: line_no,
        content: trimmed.to_string(),
        kind: LineErrorKind::MalformedLine,
      });
      continue;
    };

    let provider_str = lhs.trim();
    let key = rhs.trim().to_string();

    if key.is_empty() {
      errors.push(LineError {
        line: line_no,
        content: trimmed.to_string(),
        kind: LineErrorKind::MalformedLine,
      });
      continue;
    }

    match KnownProvider::from_str(provider_str) {
      None => errors.push(LineError {
        line: line_no,
        content: trimmed.to_string(),
        kind: LineErrorKind::UnknownProvider(provider_str.to_string()),
      }),
      Some(provider) => {
        if provider.validate_key_prefix(&key) {
          entries.push(ProviderEntry { provider, key });
        } else {
          errors.push(LineError {
            line: line_no,
            content: trimmed.to_string(),
            kind: LineErrorKind::InvalidKeyPrefix,
          });
        }
      }
    }
  }

  if errors.is_empty() {
    Ok(entries)
  } else {
    Err(errors)
  }
}
