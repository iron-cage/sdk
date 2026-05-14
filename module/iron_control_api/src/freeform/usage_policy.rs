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
///
/// # Errors
///
/// Returns a list of [`ParseError`]s if any directive is malformed (all-or-nothing).
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
  let s = s
    .strip_prefix('$')
    .ok_or(ParseErrorKind::InvalidSpendAmount)?;
  let (amount_str, period_str) = s
    .split_once('/')
    .ok_or(ParseErrorKind::InvalidSpendAmount)?;

  let amount_dollars: f64 = amount_str
    .trim()
    .parse()
    .map_err(|_| ParseErrorKind::InvalidSpendAmount)?;
  let cents = (amount_dollars * 100.0).round();
  if !cents.is_finite() || cents < 0.0 || cents > u64::MAX as f64 {
    return Err(ParseErrorKind::InvalidSpendAmount);
  }
  #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
  let amount_cents = cents as u64;

  let period = CapPeriod::from_str(period_str.trim())
    .ok_or_else(|| ParseErrorKind::InvalidPeriod(period_str.trim().to_string()))?;

  Ok(SpendingCap {
    amount_cents,
    period,
  })
}
