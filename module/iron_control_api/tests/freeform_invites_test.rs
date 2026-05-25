//! Tests for `freeform::invites` paste-block parser.
//!
//! `test_kind`: unit

use iron_control_api::freeform::invites;

#[test]
fn parses_email_list() {
  let input = "\
alice@acmecorp.com
bob@acmecorp.com
carol@acmecorp.com";

  let emails = invites::parse(input).unwrap();
  assert_eq!(
    emails,
    vec![
      "alice@acmecorp.com",
      "bob@acmecorp.com",
      "carol@acmecorp.com"
    ]
  );
}

#[test]
fn ignores_blank_lines_and_comments() {
  let input = "\
# team invites
alice@example.com

bob@example.com";

  let emails = invites::parse(input).unwrap();
  assert_eq!(emails.len(), 2);
}

#[test]
fn deduplicates_emails_silently() {
  let input = "alice@example.com\nAlice@example.com";
  let emails = invites::parse(input).unwrap();
  assert_eq!(emails, vec!["alice@example.com"]);
}

#[test]
fn lowercases_output() {
  let input = "Bob@EXAMPLE.COM";
  let emails = invites::parse(input).unwrap();
  assert_eq!(emails, vec!["bob@example.com"]);
}

#[test]
fn rejects_invalid_line() {
  let input = "notanemail";
  let errs = invites::parse(input).unwrap_err();
  assert_eq!(errs.len(), 1);
  assert_eq!(errs[0].content, "notanemail");
}

#[test]
fn all_or_nothing_on_mixed_valid_invalid() {
  let input = "alice@example.com\nnotanemail\nbob@example.com";
  assert!(invites::parse(input).is_err());
}

#[test]
fn empty_input_returns_empty_vec() {
  let emails = invites::parse("").unwrap();
  assert!(emails.is_empty());
}

#[test]
fn idempotent_reparse_produces_same_result() {
  let input = "alice@example.com\nbob@example.com";
  assert_eq!(invites::parse(input), invites::parse(input));
}
