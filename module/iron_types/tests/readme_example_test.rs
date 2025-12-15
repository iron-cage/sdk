//! Test that readme.md examples compile and work correctly

use iron_types::{Config, SafetyConfig, CostConfig};

#[test]
fn readme_example_compiles()
{
  // Example from readme.md - verify it compiles
  let config = Config {
    safety: SafetyConfig {
      pii_detection_enabled: true,
      audit_log_path: Some("/var/log/safety.log".into()),
    },
    cost: CostConfig {
      budget_usd: 100.0,
      alert_threshold: 0.8,
    },
    reliability: Default::default(),
  };

  // Verify values
  assert!(config.safety.pii_detection_enabled);
  assert_eq!(config.cost.budget_usd, 100.0);
  assert_eq!(config.cost.alert_threshold, 0.8);
  assert!(!config.reliability.circuit_breaker_enabled);
  assert_eq!(config.reliability.failure_threshold, 0);
}

#[test]
fn config_serialization_works()
{
  // Verify serde integration mentioned in readme
  let config = Config {
    safety: SafetyConfig {
      pii_detection_enabled: true,
      audit_log_path: None,
    },
    cost: CostConfig {
      budget_usd: 50.0,
      alert_threshold: 0.9,
    },
    reliability: Default::default(),
  };

  // Serialize to JSON
  let json = serde_json::to_string(&config).unwrap();
  assert!(json.contains("pii_detection_enabled"));
  assert!(json.contains("budget_usd"));

  // Deserialize back
  let deserialized: Config = serde_json::from_str(&json).unwrap();
  assert_eq!(deserialized.cost.budget_usd, 50.0);
}
