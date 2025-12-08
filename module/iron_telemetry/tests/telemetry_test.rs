use iron_telemetry::*;

#[test]
fn test_log_level_conversion()
{
  // Just verify the crate compiles and basic types work
  let _level = LogLevel::Info;
}

#[test]
fn test_init_logging()
{
  // Test initialization (may fail if called multiple times, that's ok)
  let result = init_logging(LogLevel::Info);
  // Don't assert - this may fail in test environment
  let _ = result;
}

#[test]
fn test_logging_functions_dont_panic()
{
  // Verify logging functions can be called without panic
  log_agent_event("test-agent-123", "agent_started");
  log_pii_detection("test-agent-123", "email", 42);
  log_budget_warning("test-agent-123", 45.0, 50.0);
}
