//! Test that readme.md examples compile and work correctly

use iron_reliability::CircuitBreaker;

#[test]
fn readme_example_compiles()
{
  // Example from readme.md - verify circuit breaker API works
  let cb = CircuitBreaker::new(5, 60);

  // Simulate successful calls
  for _ in 0..3 {
    cb.record_success("external_api");
  }

  // Circuit should still be closed
  assert!(!cb.is_open("external_api"));
}

#[test]
fn circuit_breaker_opens_on_failures()
{
  let cb = CircuitBreaker::new(3, 60);

  // Generate failures (below threshold)
  cb.record_failure("failing_api");
  cb.record_failure("failing_api");
  assert!(!cb.is_open("failing_api"));

  // One more failure should open the circuit
  cb.record_failure("failing_api");
  assert!(cb.is_open("failing_api"));
}

#[test]
fn circuit_breaker_resets_on_success()
{
  let cb = CircuitBreaker::new(5, 60);

  // Generate some failures
  for _ in 0..2 {
    cb.record_failure("unstable_api");
  }

  // Circuit shouldnt be open yet (threshold is 5)
  assert!(!cb.is_open("unstable_api"));

  // Successful call should reset failures
  cb.record_success("unstable_api");

  // Circuit should still be closed
  assert!(!cb.is_open("unstable_api"));
}
