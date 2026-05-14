//! Tests for `freeform::providers` paste-block parser.
use iron_control_api::freeform::providers::{self, KnownProvider, LineErrorKind};

#[test]
fn parses_multiple_providers() {
  let input = "\
gemini:    AIzaSy...xxxx
openai:    sk-proj-...xxxx
anthropic: sk-ant-...xxxx";

  let entries = providers::parse(input).unwrap();
  assert_eq!(entries.len(), 3);
  assert_eq!(entries[0].provider, KnownProvider::Gemini);
  assert_eq!(entries[1].provider, KnownProvider::OpenAi);
  assert_eq!(entries[2].provider, KnownProvider::Anthropic);
}

#[test]
fn ignores_blank_lines_and_comments() {
  let input = "\
# workspace bootstrap
openai: sk-proj-key

# second provider
gemini: AIzaSy-key";

  let entries = providers::parse(input).unwrap();
  assert_eq!(entries.len(), 2);
}

#[test]
fn rejects_unknown_provider() {
  let input = "groq: sk-groq-xxxx";
  let errs = providers::parse(input).unwrap_err();
  assert_eq!(errs.len(), 1);
  assert!(matches!(&errs[0].kind, LineErrorKind::UnknownProvider(p) if p == "groq"));
}

#[test]
fn rejects_wrong_key_prefix_for_provider() {
  let input = "anthropic: sk-proj-wrong";
  let errs = providers::parse(input).unwrap_err();
  assert_eq!(errs.len(), 1);
  assert_eq!(errs[0].kind, LineErrorKind::InvalidKeyPrefix);
}

#[test]
fn rejects_line_without_colon() {
  let input = "notaproviderline";
  let errs = providers::parse(input).unwrap_err();
  assert_eq!(errs.len(), 1);
  assert_eq!(errs[0].kind, LineErrorKind::MalformedLine);
}

#[test]
fn all_or_nothing_on_mixed_valid_invalid() {
  let input = "openai: sk-proj-good\nbadprovider: key";
  let errs = providers::parse(input).unwrap_err();
  assert_eq!(errs.len(), 1);
}

#[test]
fn empty_input_returns_empty_vec() {
  let entries = providers::parse("").unwrap();
  assert!(entries.is_empty());
}

#[test]
fn idempotent_reparse_produces_same_result() {
  let input = "openai: sk-proj-abc\ngemini: AIzaSy-xyz";
  let first = providers::parse(input).unwrap();
  let second = providers::parse(input).unwrap();
  assert_eq!(first, second);
}
