//! Tests for `freeform::usage_policy` paste-block parser.
//!
//! `test_kind`: unit

use iron_control_api::freeform::usage_policy::{self, CapPeriod, ParseErrorKind, SpendingCap};

#[test]
fn parses_multiline_policy() {
  let input = "\
limit all users $100/week
default: gpt-5.4-mini";

  let policy = usage_policy::parse(input).unwrap();
  assert_eq!(
    policy.spending_cap,
    Some(SpendingCap {
      amount_cents: 10000,
      period: CapPeriod::Week
    })
  );
  assert_eq!(policy.default_model.as_deref(), Some("gpt-5.4-mini"));
}

#[test]
fn single_line_deck_format_is_rejected() {
  let input = "limit all users $100/week - default: gpt-5.4-mini";
  assert!(usage_policy::parse(input).is_err());
}

#[test]
fn parses_default_model_variant() {
  let input = "default model: gpt-4o";
  let policy = usage_policy::parse(input).unwrap();
  assert_eq!(policy.default_model.as_deref(), Some("gpt-4o"));
}

#[test]
fn rejects_unknown_period() {
  let input = "limit all users $50/fortnight";
  let errs = usage_policy::parse(input).unwrap_err();
  assert!(matches!(&errs[0].kind, ParseErrorKind::InvalidPeriod(p) if p == "fortnight"));
}

#[test]
fn rejects_invalid_amount() {
  let input = "limit all users 50/week";
  let errs = usage_policy::parse(input).unwrap_err();
  assert_eq!(errs[0].kind, ParseErrorKind::InvalidSpendAmount);
}

#[test]
fn rejects_empty_model() {
  let input = "default:";
  let errs = usage_policy::parse(input).unwrap_err();
  assert_eq!(errs[0].kind, ParseErrorKind::EmptyModelId);
}

#[test]
fn ignores_blank_lines_and_comments() {
  let input = "\
# workspace policy
limit all users $200/month

default: gpt-4o";

  let policy = usage_policy::parse(input).unwrap();
  assert_eq!(policy.spending_cap.unwrap().period, CapPeriod::Month);
  assert_eq!(policy.default_model.as_deref(), Some("gpt-4o"));
}

#[test]
fn rejects_per_user_limit_as_malformed() {
  let input = "limit user@example.com $50/week";
  let errs = usage_policy::parse(input).unwrap_err();
  assert_eq!(errs[0].kind, ParseErrorKind::MalformedLine);
}

#[test]
fn idempotent_reparse() {
  let input = "limit all users $100/week\ndefault: gpt-4o";
  assert_eq!(usage_policy::parse(input), usage_policy::parse(input));
}

#[test]
fn all_or_nothing_on_mixed_error() {
  let input = "limit all users $100/week\nbadline";
  assert!(usage_policy::parse(input).is_err());
}

#[test]
fn parses_requestable_models() {
  let input = "requestable: claude-4-6-sonnet, gemini-3.1-pro-preview";
  let policy = usage_policy::parse(input).unwrap();
  assert_eq!(
    policy.requestable_models,
    vec![
      "claude-4-6-sonnet".to_string(),
      "gemini-3.1-pro-preview".to_string()
    ]
  );
}

#[test]
fn requestable_models_trims_and_skips_blanks() {
  let input = "requestable:  gpt-4o , , claude-4-6-sonnet ,";
  let policy = usage_policy::parse(input).unwrap();
  assert_eq!(
    policy.requestable_models,
    vec!["gpt-4o".to_string(), "claude-4-6-sonnet".to_string()]
  );
}

#[test]
fn rejects_empty_requestable_list() {
  let input = "requestable:";
  let errs = usage_policy::parse(input).unwrap_err();
  assert_eq!(errs[0].kind, ParseErrorKind::EmptyModelId);
}

#[test]
fn requestable_combines_with_other_directives() {
  let input = "limit all users $100/week\ndefault: gpt-4o\nrequestable: claude-4-6-sonnet";
  let policy = usage_policy::parse(input).unwrap();
  assert!(policy.spending_cap.is_some());
  assert_eq!(policy.default_model.as_deref(), Some("gpt-4o"));
  assert_eq!(
    policy.requestable_models,
    vec!["claude-4-6-sonnet".to_string()]
  );
}
