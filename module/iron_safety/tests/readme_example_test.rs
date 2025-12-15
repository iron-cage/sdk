//! Test that readme.md examples compile and work correctly

use iron_safety::PiiDetector;

#[test]
fn readme_example_compiles()
{
  // Example from readme.md - verify it compiles and works
  let detector = PiiDetector::new().unwrap();

  let text = "Contact me at john@example.com";
  assert!(detector.check(text));

  let safe = detector.redact(text);
  assert_eq!(safe, "Contact me at [EMAIL_REDACTED]");
}

#[test]
fn pii_detection_phone_numbers()
{
  let detector = PiiDetector::new().unwrap();

  let text = "Call 555-123-4567";
  assert!(detector.check(text));

  let safe = detector.redact(text);
  assert_eq!(safe, "Call [PHONE_REDACTED]");
}

#[test]
fn no_false_positives()
{
  let detector = PiiDetector::new().unwrap();

  let text = "No PII here";
  assert!(!detector.check(text));

  let safe = detector.redact(text);
  assert_eq!(safe, "No PII here");
}
