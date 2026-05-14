use serde::{Deserialize, Serialize};

/// Workspace account category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
  /// External client workspace.
  Client,
  /// Internal company workspace.
  Internal,
}

/// Result of a successful parse of a company-setup line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedCompany {
  /// Workspace display name.
  pub name: String,
  /// Primary domain (e.g. `acme.com`).
  pub domain: String,
  /// Account category.
  pub account_type: AccountType,
}

/// Reason a company-setup line was rejected.
#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
  /// The input does not contain exactly three comma-separated fields.
  WrongFieldCount,
  /// A required field was present but empty after trimming.
  EmptyField(&'static str),
  /// The account-type value is not `Client` or `Internal`.
  UnknownAccountType(String),
}

/// A parse error for a company-setup line.
#[derive(Debug, PartialEq)]
pub struct ParseError {
  /// Reason the line was rejected.
  pub kind: ParseErrorKind,
}

impl core::fmt::Display for ParseError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let msg = match &self.kind {
      ParseErrorKind::WrongFieldCount => {
        "expected exactly three comma-separated fields: name, domain, account_type".to_string()
      }
      ParseErrorKind::EmptyField(name) => format!("field '{name}' must not be empty"),
      ParseErrorKind::UnknownAccountType(v) => {
        format!("unknown account type '{v}' — supported: Client, Internal")
      }
    };
    write!(f, "{msg}")
  }
}

/// Parses a single company-setup line in the form `Name, domain, AccountType`.
///
/// All fields are comma-separated. Whitespace around commas is trimmed.
/// `AccountType` is case-insensitive (`client` and `Client` both accepted).
///
/// # Errors
///
/// Returns a [`ParseError`] if the line is malformed, a field is empty,
/// or the account type is not recognised.
pub fn parse(input: &str) -> Result<ParsedCompany, ParseError> {
  let parts: Vec<&str> = input.splitn(3, ',').collect();

  if parts.len() != 3 {
    return Err(ParseError {
      kind: ParseErrorKind::WrongFieldCount,
    });
  }

  let name = parts[0].trim().to_string();
  let domain = parts[1].trim().to_string();
  let type_str = parts[2].trim();

  if name.is_empty() {
    return Err(ParseError {
      kind: ParseErrorKind::EmptyField("name"),
    });
  }
  if domain.is_empty() {
    return Err(ParseError {
      kind: ParseErrorKind::EmptyField("domain"),
    });
  }
  if type_str.is_empty() {
    return Err(ParseError {
      kind: ParseErrorKind::EmptyField("account_type"),
    });
  }

  let account_type = match type_str.to_lowercase().as_str() {
    "client" => AccountType::Client,
    "internal" => AccountType::Internal,
    _ => {
      return Err(ParseError {
        kind: ParseErrorKind::UnknownAccountType(type_str.to_string()),
      })
    }
  };

  Ok(ParsedCompany {
    name,
    domain,
    account_type,
  })
}
