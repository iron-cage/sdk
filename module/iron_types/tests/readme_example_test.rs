//! Test that readme.md examples compile and work correctly

use iron_types::{Config, CostConfig, ReliabilityConfig, SafetyConfig};

#[test]
fn readme_example_compiles() {
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
    reliability: ReliabilityConfig::default(),
  };

  // Verify values
  assert!(config.safety.pii_detection_enabled);
  assert!((config.cost.budget_usd - 100.0_f64).abs() < f64::EPSILON);
  assert!((config.cost.alert_threshold - 0.8_f64).abs() < f64::EPSILON);
  assert!(!config.reliability.circuit_breaker_enabled);
  assert_eq!(config.reliability.failure_threshold, 0);
}

#[test]
fn config_serialization_works() {
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
    reliability: ReliabilityConfig::default(),
  };

  // Serialize to JSON
  let json = serde_json::to_string(&config).unwrap();
  assert!(json.contains("pii_detection_enabled"));
  assert!(json.contains("budget_usd"));

  // Deserialize back
  let deserialized: Config = serde_json::from_str(&json).unwrap();
  assert!((deserialized.cost.budget_usd - 50.0_f64).abs() < f64::EPSILON);
}
