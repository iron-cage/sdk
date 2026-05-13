use serde::{Deserialize, Serialize};

/// A supported inference provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnownProvider {
  /// Google Gemini.
  Gemini,
  /// OpenAI.
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
      Self::OpenAi => key.starts_with("sk-"),
      Self::Anthropic => key.starts_with("sk-ant-"),
      Self::Gemini => key.starts_with("AIzaSy"),
      Self::Mistral => key.starts_with("sk-"),
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

/// Result of a successful parse of a providers-and-invites paste block.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ParsedProvidersBlock {
  /// Provider key entries extracted from `provider: key` lines.
  pub providers: Vec<ProviderEntry>,
  /// Invite email addresses extracted from bare email lines (lowercased, deduplicated).
  pub invites: Vec<String>,
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

/// Reason a line was rejected during parsing.
#[derive(Debug, PartialEq)]
pub enum LineErrorKind {
  /// The provider label is not in the supported set.
  UnknownProvider(String),
  /// The key does not start with the expected prefix for its provider.
  InvalidKeyPrefix,
  /// The value looks like it should be an email but is malformed.
  InvalidEmail,
  /// The line does not match `provider: key` or a bare email address.
  MalformedLine,
}

impl core::fmt::Display for LineError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let kind = match &self.kind {
      LineErrorKind::UnknownProvider(p) => format!("unknown provider '{p}'"),
      LineErrorKind::InvalidKeyPrefix => "invalid key prefix for provider".to_string(),
      LineErrorKind::InvalidEmail => "invalid email address".to_string(),
      LineErrorKind::MalformedLine => {
        "cannot parse line — expected 'provider: key' or email".to_string()
      }
    };
    write!(f, "line {}: {} ({})", self.line, kind, self.content)
  }
}

fn is_valid_email(s: &str) -> bool {
  let Some((local, domain)) = s.split_once('@') else {
    return false;
  };
  !local.is_empty()
    && domain.contains('.')
    && !domain.starts_with('.')
    && !domain.ends_with('.')
    && domain.len() > 2
}

/// Parses a paste block containing `provider: key` lines and bare email addresses.
///
/// Returns the full parsed block or, on any error, a list of per-line diagnostics.
/// Apply is all-or-nothing: a single invalid line aborts the entire block.
/// Duplicate emails within the same paste are silently deduplicated.
pub fn parse(input: &str) -> Result<ParsedProvidersBlock, Vec<LineError>> {
  let mut block = ParsedProvidersBlock::default();
  let mut errors: Vec<LineError> = Vec::new();
  let mut seen_emails: std::collections::HashSet<String> = std::collections::HashSet::new();

  for (idx, raw) in input.lines().enumerate() {
    let line_no = idx + 1;
    let trimmed = raw.trim();

    if trimmed.is_empty() || trimmed.starts_with('#') {
      continue;
    }

    if let Some((lhs, rhs)) = trimmed.split_once(':') {
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
          if !provider.validate_key_prefix(&key) {
            errors.push(LineError {
              line: line_no,
              content: trimmed.to_string(),
              kind: LineErrorKind::InvalidKeyPrefix,
            });
          } else {
            block.providers.push(ProviderEntry { provider, key });
          }
        }
      }
    } else if is_valid_email(trimmed) {
      let lower = trimmed.to_lowercase();
      if seen_emails.insert(lower.clone()) {
        block.invites.push(lower);
      }
    } else {
      errors.push(LineError {
        line: line_no,
        content: trimmed.to_string(),
        kind: LineErrorKind::MalformedLine,
      });
    }
  }

  if errors.is_empty() {
    Ok(block)
  } else {
    Err(errors)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_canonical_deck_example() {
    let input = "\
gemini:    AIzaSy...xxxx
openai:    sk-proj-...xxxx
anthropic: sk-ant-...xxxx

alice@acmecorp.com
bob@acmecorp.com
carol@acmecorp.com";

    let block = parse(input).unwrap();
    assert_eq!(block.providers.len(), 3);
    assert_eq!(block.providers[0].provider, KnownProvider::Gemini);
    assert_eq!(block.providers[1].provider, KnownProvider::OpenAi);
    assert_eq!(block.providers[2].provider, KnownProvider::Anthropic);
    assert_eq!(
      block.invites,
      vec!["alice@acmecorp.com", "bob@acmecorp.com", "carol@acmecorp.com"]
    );
  }

  #[test]
  fn ignores_blank_lines_and_comments() {
    let input = "\
# workspace bootstrap
openai: sk-proj-key

# team
alice@example.com";

    let block = parse(input).unwrap();
    assert_eq!(block.providers.len(), 1);
    assert_eq!(block.invites.len(), 1);
  }

  #[test]
  fn rejects_unknown_provider() {
    let input = "groq: sk-groq-xxxx";
    let errs = parse(input).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert!(matches!(&errs[0].kind, LineErrorKind::UnknownProvider(p) if p == "groq"));
  }

  #[test]
  fn rejects_wrong_key_prefix_for_provider() {
    let input = "anthropic: sk-proj-wrong";
    let errs = parse(input).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].kind, LineErrorKind::InvalidKeyPrefix);
  }

  #[test]
  fn rejects_invalid_email() {
    let input = "notanemail";
    let errs = parse(input).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].kind, LineErrorKind::MalformedLine);
  }

  #[test]
  fn deduplicates_emails_silently() {
    let input = "alice@example.com\nAlice@example.com";
    let block = parse(input).unwrap();
    assert_eq!(block.invites, vec!["alice@example.com"]);
  }

  #[test]
  fn all_or_nothing_on_mixed_valid_invalid() {
    let input = "openai: sk-proj-good\nbadprovider: key";
    let errs = parse(input).unwrap_err();
    assert_eq!(errs.len(), 1);
  }

  #[test]
  fn empty_input_returns_empty_block() {
    let block = parse("").unwrap();
    assert!(block.providers.is_empty());
    assert!(block.invites.is_empty());
  }

  #[test]
  fn idempotent_reparse_produces_same_result() {
    let input = "openai: sk-proj-abc\nalice@example.com";
    let first = parse(input).unwrap();
    let second = parse(input).unwrap();
    assert_eq!(first, second);
  }
}
