use serde::{Deserialize, Serialize};

/// Billing period for a workspace spending cap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CapPeriod {
  /// Rolling 24-hour window.
  Day,
  /// Rolling 7-day window.
  Week,
  /// Rolling 30-day window.
  Month,
}

impl CapPeriod {
  fn from_str(s: &str) -> Option<Self> {
    match s.to_lowercase().as_str() {
      "day" => Some(Self::Day),
      "week" => Some(Self::Week),
      "month" => Some(Self::Month),
      _ => None,
    }
  }
}

/// A workspace-wide spending cap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpendingCap {
  /// Cap amount in US cents (e.g. `$100` -> `10000`).
  pub amount_cents: u64,
  /// Billing period over which the cap resets.
  pub period: CapPeriod,
}

/// Result of a successful parse of a usage-policy paste block.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ParsedPolicy {
  /// Optional workspace-wide spending cap from `limit all users $N/<period>`.
  pub spending_cap: Option<SpendingCap>,
  /// Default model all members receive from `default: <model>`.
  pub default_model: Option<String>,
  /// Models gated behind an admin-approval request from `requestable: model[, ...]`.
  pub requestable_models: Vec<String>,
}

/// A per-line parse error.
#[derive(Debug, PartialEq)]
pub struct ParseError {
  /// 1-indexed line number in the original input.
  pub line: usize,
  /// Trimmed content of the offending line.
  pub content: String,
  /// Reason the line was rejected.
  pub kind: ParseErrorKind,
}

/// Reason a line was rejected during policy parsing.
#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
  /// The `$N` amount is missing or not a valid number.
  InvalidSpendAmount,
  /// The period token is not `day`, `week`, or `month`.
  InvalidPeriod(String),
  /// A model identifier is required but was empty.
  EmptyModelId,
  /// The line does not match any supported policy directive.
  MalformedLine,
}

impl core::fmt::Display for ParseError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let kind = match &self.kind {
      ParseErrorKind::InvalidSpendAmount => "invalid spend amount — expected '$N'".to_string(),
      ParseErrorKind::InvalidPeriod(p) => {
        format!("invalid period '{p}' — supported: day, week, month")
      }
      ParseErrorKind::EmptyModelId => "model id must not be empty".to_string(),
      ParseErrorKind::MalformedLine => "cannot parse line".to_string(),
    };
    write!(f, "line {}: {} ({})", self.line, kind, self.content)
  }
}

/// Parses a usage-policy paste block.
///
/// Supported directives (keywords are case-insensitive):
/// - `limit all users $N/<period>` — workspace spending cap
/// - `default: <model>` or `default model: <model>`
/// - `requestable: model[, model, ...]`
///
/// Returns the parsed policy or per-line errors (all-or-nothing).
pub fn parse(input: &str) -> Result<ParsedPolicy, Vec<ParseError>> {
  let mut policy = ParsedPolicy::default();
  let mut errors: Vec<ParseError> = Vec::new();

  for (idx, raw) in input.lines().enumerate() {
    let line_no = idx + 1;
    let trimmed = raw.trim();

    if trimmed.is_empty() || trimmed.starts_with('#') {
      continue;
    }

    let lower = trimmed.to_lowercase();

    if lower.starts_with("limit all users") {
      let rest = trimmed["limit all users".len()..].trim();
      match parse_spend_cap(rest) {
        Ok(cap) => policy.spending_cap = Some(cap),
        Err(kind) => errors.push(ParseError {
          line: line_no,
          content: trimmed.to_string(),
          kind,
        }),
      }
    } else if lower.starts_with("default model:") {
      let model = trimmed["default model:".len()..].trim().to_string();
      if model.is_empty() {
        errors.push(ParseError {
          line: line_no,
          content: trimmed.to_string(),
          kind: ParseErrorKind::EmptyModelId,
        });
      } else {
        policy.default_model = Some(model);
      }
    } else if lower.starts_with("default:") {
      let model = trimmed["default:".len()..].trim().to_string();
      if model.is_empty() {
        errors.push(ParseError {
          line: line_no,
          content: trimmed.to_string(),
          kind: ParseErrorKind::EmptyModelId,
        });
      } else {
        policy.default_model = Some(model);
      }
    } else if lower.starts_with("requestable:") {
      let rest = trimmed["requestable:".len()..].trim();
      let models: Vec<String> = rest
        .split(',')
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .collect();

      if models.is_empty() {
        errors.push(ParseError {
          line: line_no,
          content: trimmed.to_string(),
          kind: ParseErrorKind::EmptyModelId,
        });
      } else {
        policy.requestable_models.extend(models);
      }
    } else {
      errors.push(ParseError {
        line: line_no,
        content: trimmed.to_string(),
        kind: ParseErrorKind::MalformedLine,
      });
    }
  }

  if errors.is_empty() {
    Ok(policy)
  } else {
    Err(errors)
  }
}

fn parse_spend_cap(s: &str) -> Result<SpendingCap, ParseErrorKind> {
  let s = s.strip_prefix('$').ok_or(ParseErrorKind::InvalidSpendAmount)?;
  let (amount_str, period_str) = s
    .split_once('/')
    .ok_or(ParseErrorKind::InvalidSpendAmount)?;

  let amount_dollars: f64 = amount_str
    .trim()
    .parse()
    .map_err(|_| ParseErrorKind::InvalidSpendAmount)?;
  let amount_cents = (amount_dollars * 100.0).round() as u64;

  let period = CapPeriod::from_str(period_str.trim())
    .ok_or_else(|| ParseErrorKind::InvalidPeriod(period_str.trim().to_string()))?;

  Ok(SpendingCap { amount_cents, period })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_multiline_policy() {
    let input = "\
limit all users $100/week
default: gpt-5.4-mini
requestable: claude-4-6-sonnet, gemini-3.1-pro-preview";

    let policy = parse(input).unwrap();
    assert_eq!(
      policy.spending_cap,
      Some(SpendingCap { amount_cents: 10000, period: CapPeriod::Week })
    );
    assert_eq!(policy.default_model.as_deref(), Some("gpt-5.4-mini"));
    assert_eq!(
      policy.requestable_models,
      vec!["claude-4-6-sonnet", "gemini-3.1-pro-preview"]
    );
  }

  #[test]
  fn single_line_deck_format_is_rejected() {
    // The deck shows directives on one line for display; the grammar expects one directive per line.
    let input = "limit all users $100/week - default: gpt-5.4-mini - requestable: claude-4-6-sonnet";
    assert!(parse(input).is_err());
  }

  #[test]
  fn parses_default_model_variant() {
    let input = "default model: gpt-4o";
    let policy = parse(input).unwrap();
    assert_eq!(policy.default_model.as_deref(), Some("gpt-4o"));
  }

  #[test]
  fn rejects_unknown_period() {
    let input = "limit all users $50/fortnight";
    let errs = parse(input).unwrap_err();
    assert!(matches!(&errs[0].kind, ParseErrorKind::InvalidPeriod(p) if p == "fortnight"));
  }

  #[test]
  fn rejects_invalid_amount() {
    let input = "limit all users 50/week";
    let errs = parse(input).unwrap_err();
    assert_eq!(errs[0].kind, ParseErrorKind::InvalidSpendAmount);
  }

  #[test]
  fn rejects_empty_model() {
    let input = "default:";
    let errs = parse(input).unwrap_err();
    assert_eq!(errs[0].kind, ParseErrorKind::EmptyModelId);
  }

  #[test]
  fn ignores_blank_lines_and_comments() {
    let input = "\
# workspace policy
limit all users $200/month

default: gpt-4o";

    let policy = parse(input).unwrap();
    assert_eq!(policy.spending_cap.unwrap().period, CapPeriod::Month);
    assert_eq!(policy.default_model.as_deref(), Some("gpt-4o"));
  }

  #[test]
  fn rejects_per_user_limit_as_malformed() {
    let input = "limit user@example.com $50/week";
    let errs = parse(input).unwrap_err();
    assert_eq!(errs[0].kind, ParseErrorKind::MalformedLine);
  }

  #[test]
  fn idempotent_reparse() {
    let input = "limit all users $100/week\ndefault: gpt-4o\nrequestable: claude-4-6-sonnet";
    assert_eq!(parse(input), parse(input));
  }

  #[test]
  fn all_or_nothing_on_mixed_error() {
    let input = "limit all users $100/week\nbadline";
    assert!(parse(input).is_err());
  }
}
