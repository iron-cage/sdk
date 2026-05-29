/// A per-line parse error for an invite list.
#[derive(Debug, PartialEq)]
pub struct ParseError {
  /// 1-indexed line number in the original input.
  pub line: usize,
  /// Trimmed content of the offending line.
  pub content: String,
}

impl core::fmt::Display for ParseError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "line {}: invalid email address ({})",
      self.line, self.content
    )
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

/// Parses a paste block of email addresses (one per line).
///
/// Each non-blank, non-comment line must be a valid email address.
/// Duplicate addresses within the same paste are silently deduplicated.
/// Apply is all-or-nothing: a single invalid line aborts the entire block.
/// All addresses are lowercased in the output.
///
/// # Errors
///
/// Returns a list of [`ParseError`]s if any line is not a valid email address.
pub fn parse(input: &str) -> Result<Vec<String>, Vec<ParseError>> {
  let mut emails: Vec<String> = Vec::new();
  let mut errors: Vec<ParseError> = Vec::new();
  let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

  for (idx, raw) in input.lines().enumerate() {
    let line_no = idx + 1;
    let trimmed = raw.trim();

    if trimmed.is_empty() || trimmed.starts_with('#') {
      continue;
    }

    if is_valid_email(trimmed) {
      let lower = trimmed.to_lowercase();
      if seen.insert(lower.clone()) {
        emails.push(lower);
      }
    } else {
      errors.push(ParseError {
        line: line_no,
        content: trimmed.to_string(),
      });
    }
  }

  if errors.is_empty() {
    Ok(emails)
  } else {
    Err(errors)
  }
}
