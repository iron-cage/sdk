//! Tests for `freeform::company_setup` CSV-line parser.
use iron_control_api::freeform::company_setup::{self, AccountType, ParseErrorKind};

#[test]
fn parses_client_company() {
  let result = company_setup::parse("Acme Corp, acme.com, Client").unwrap();
  assert_eq!(result.name, "Acme Corp");
  assert_eq!(result.domain, "acme.com");
  assert_eq!(result.account_type, AccountType::Client);
}

#[test]
fn parses_internal_company() {
  let result = company_setup::parse("Iron Cage, ironcage.io, Internal").unwrap();
  assert_eq!(result.account_type, AccountType::Internal);
}

#[test]
fn trims_whitespace_around_commas() {
  let result = company_setup::parse("  Acme Corp  ,  acme.com  ,  Client  ").unwrap();
  assert_eq!(result.name, "Acme Corp");
  assert_eq!(result.domain, "acme.com");
}

#[test]
fn account_type_is_case_insensitive() {
  assert!(company_setup::parse("Acme, acme.com, client").is_ok());
  assert!(company_setup::parse("Acme, acme.com, INTERNAL").is_ok());
}

#[test]
fn name_may_contain_commas_if_only_two_splits_used() {
  // splitn(3) keeps everything after the second comma in the third field — this should fail
  // because "Client, Extra" is not a valid account type.
  let err = company_setup::parse("Acme, Corp, acme.com, Client").unwrap_err();
  assert!(matches!(err.kind, ParseErrorKind::UnknownAccountType(_)));
}

#[test]
fn rejects_missing_fields() {
  let err = company_setup::parse("Acme, acme.com").unwrap_err();
  assert_eq!(err.kind, ParseErrorKind::WrongFieldCount);
}

#[test]
fn rejects_empty_name() {
  let err = company_setup::parse(", acme.com, Client").unwrap_err();
  assert_eq!(err.kind, ParseErrorKind::EmptyField("name"));
}

#[test]
fn rejects_empty_domain() {
  let err = company_setup::parse("Acme Corp, , Client").unwrap_err();
  assert_eq!(err.kind, ParseErrorKind::EmptyField("domain"));
}

#[test]
fn rejects_unknown_account_type() {
  let err = company_setup::parse("Acme Corp, acme.com, Partner").unwrap_err();
  assert!(matches!(err.kind, ParseErrorKind::UnknownAccountType(v) if v == "Partner"));
}

#[test]
fn idempotent_reparse() {
  let input = "Acme Corp, acme.com, Client";
  assert_eq!(company_setup::parse(input), company_setup::parse(input));
}
