//! Tests for the pricing module: `PricingManager`, `Model` getters, cost calculations, and estimation.

use iron_cost::pricing::PricingManager;

// =============================================================================
// PricingManager tests
// =============================================================================

#[test]
fn new_creates_manager_with_embedded_data() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");
  assert!(manager.get("gpt-5.1").is_some());
}

#[test]
fn get_returns_existing_model() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let model = manager.get("gpt-5.1").unwrap();
  // gpt-5.1: input_cost_per_token: 1.25e-06, output_cost_per_token: 1e-05
  assert!((model.input_cost_per_token() - 0.000_001_25).abs() < f64::EPSILON);
  assert!((model.output_cost_per_token() - 0.00001).abs() < f64::EPSILON);
  assert_eq!(model.max_output_tokens(), Some(128_000));
}

#[test]
fn get_returns_none_for_nonexistent_model() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");
  assert!(manager.get("nonexistent-model-xyz-123").is_none());
}

#[test]
fn load_from_file_parses_valid_json() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_output_tokens": 4096
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse valid JSON");

  let model = manager.get("test-model").unwrap();
  assert!((model.input_cost_per_token() - 0.001).abs() < f64::EPSILON);
  assert!((model.output_cost_per_token() - 0.002).abs() < f64::EPSILON);
  assert_eq!(model.max_output_tokens(), Some(4096));
}

#[test]
fn load_from_file_returns_error_for_invalid_json() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");
  assert!(manager.load_from_file("not valid json {{{").is_err());
}

#[test]
fn load_from_file_filters_sample_spec() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "sample_spec": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        },
        "real-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");

  assert!(
    manager.get("sample_spec").is_none(),
    "sample_spec should be filtered"
  );
  assert!(
    manager.get("real-model").is_some(),
    "real model should exist"
  );
}

#[test]
fn load_from_file_filters_zero_cost_models() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "zero-cost-model": {
            "input_cost_per_token": 0.0,
            "output_cost_per_token": 0.0,
            "max_output_tokens": 4096
        },
        "valid-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.0
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");

  assert!(
    manager.get("zero-cost-model").is_none(),
    "zero cost model should be filtered"
  );
  assert!(
    manager.get("valid-model").is_some(),
    "model with any non-zero cost should exist"
  );
}

#[test]
fn load_from_file_sets_name_from_key() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "my-model-name": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("my-model-name")
    .expect("LOUD FAILURE: model should exist");
  assert_eq!(model.name(), "my-model-name");
}

// =============================================================================
// Model getters tests
// =============================================================================

#[test]
fn model_name_returns_correct_value() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");
  assert_eq!(model.name(), "test-model");
}

#[test]
fn model_max_output_tokens_prefers_max_output_tokens_field() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_output_tokens": 8192,
            "max_tokens": 4096
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");
  assert_eq!(model.max_output_tokens(), Some(8192));
}

#[test]
fn model_max_output_tokens_falls_back_to_max_tokens() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_tokens": 4096
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");
  assert_eq!(model.max_output_tokens(), Some(4096));
}

#[test]
fn model_max_tokens_returns_raw_value() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_tokens": 2048
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");
  assert_eq!(model.max_tokens(), Some(2048));
}

// =============================================================================
// Model::calculate_cost tests
// =============================================================================

#[test]
fn calculate_cost_with_zero_tokens() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");
  assert!(model.calculate_cost(0, 0).abs() < f64::EPSILON);
}

#[test]
fn calculate_cost_input_only() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");

  // 1000 input tokens * 0.001 = 1.0
  assert!((model.calculate_cost(1000, 0) - 1.0).abs() < f64::EPSILON);
}

#[test]
fn calculate_cost_output_only() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");

  // 500 output tokens * 0.002 = 1.0
  assert!((model.calculate_cost(0, 500) - 1.0).abs() < f64::EPSILON);
}

#[test]
fn calculate_cost_mixed() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");

  // 1000 * 0.001 + 500 * 0.002 = 1.0 + 1.0 = 2.0
  assert!((model.calculate_cost(1000, 500) - 2.0).abs() < f64::EPSILON);
}

// =============================================================================
// Model::calculate_cost_micros tests (integer API)
// =============================================================================

#[test]
fn calculate_cost_micros_returns_integer() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");

  // 1000 * 0.001 + 500 * 0.002 = $2.0 = 2,000,000 micros
  assert_eq!(model.calculate_cost_micros(1000, 500), 2_000_000);
}

#[test]
fn calculate_cost_micros_precision() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");
  let model = manager
    .get("gpt-5.1")
    .expect("LOUD FAILURE: gpt-5.1 should exist");

  // 1000 input * $0.000_001_25 + 500 output * $0.00001 = $0.00125 + $0.005 = $0.00625
  // = 6250 micros
  assert_eq!(model.calculate_cost_micros(1000, 500), 6250);
}

// =============================================================================
// Model::calculate_max_cost tests
// =============================================================================

#[test]
fn calculate_max_cost_uses_request_max_output() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_output_tokens": 8192
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");

  // 1000 * 0.001 + 100 * 0.002 = 1.0 + 0.2 = 1.2
  assert!((model.calculate_max_cost(1000, Some(100)) - 1.2).abs() < f64::EPSILON);
}

#[test]
fn calculate_max_cost_uses_model_max_when_no_request_max() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_output_tokens": 1000
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");

  // 500 * 0.001 + 1000 * 0.002 = 0.5 + 2.0 = 2.5
  assert!((model.calculate_max_cost(500, None) - 2.5).abs() < f64::EPSILON);
}

#[test]
fn calculate_max_cost_caps_at_model_max() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_output_tokens": 100
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");

  // Request 10000 but model max is 100
  // 500 * 0.001 + 100 * 0.002 = 0.5 + 0.2 = 0.7
  assert!((model.calculate_max_cost(500, Some(10000)) - 0.7).abs() < f64::EPSILON);
}

#[test]
fn calculate_max_cost_uses_default_128000_when_no_max() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.0001,
            "output_cost_per_token": 0.0001
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");

  // 0 * 0.0001 + 128_000 * 0.0001 = 12.8
  assert!((model.calculate_max_cost(0, None) - 12.8).abs() < f64::EPSILON);
}

// =============================================================================
// Model::has_valid_pricing tests
// =============================================================================

#[test]
fn has_valid_pricing_true_for_nonzero_input_cost() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.0
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");
  assert!(model.has_valid_pricing());
}

#[test]
fn has_valid_pricing_true_for_nonzero_output_cost() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.0,
            "output_cost_per_token": 0.001
        }
    }"#;

  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");
  let model = manager
    .get("test-model")
    .expect("LOUD FAILURE: should exist");
  assert!(model.has_valid_pricing());
}

// =============================================================================
// Real model tests (from embedded JSON)
// =============================================================================

#[test]
fn real_model_gpt51_exists() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");
  assert!(
    manager.get("gpt-5.1").is_some(),
    "gpt-5.1 should exist in embedded pricing"
  );
}

#[test]
fn real_model_gpt51_pricing() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");
  let model = manager
    .get("gpt-5.1")
    .expect("LOUD FAILURE: gpt-5.1 should exist");

  assert!((model.input_cost_per_token() - 0.000_001_25).abs() < f64::EPSILON); // 1.25e-06
  assert!((model.output_cost_per_token() - 0.00001).abs() < f64::EPSILON); // 1e-05
  assert_eq!(model.max_output_tokens(), Some(128_000));
  assert_eq!(model.max_tokens(), Some(128_000));
}

#[test]
fn real_model_gpt51_calculate_cost() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");
  let model = manager
    .get("gpt-5.1")
    .expect("LOUD FAILURE: gpt-5.1 should exist");

  // 1000 input * 1.25e-06 + 500 output * 1e-05 = 0.00125 + 0.005 = 0.00625
  assert!((model.calculate_cost(1000, 500) - 0.00625).abs() < f64::EPSILON);
}

#[test]
fn real_model_gpt51_calculate_max_cost() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");
  let model = manager
    .get("gpt-5.1")
    .expect("LOUD FAILURE: gpt-5.1 should exist");

  // With request_max_output = 1000:
  // 1000 input * 1.25e-06 + 1000 output * 1e-05 = 0.00125 + 0.01 = 0.01125
  assert!((model.calculate_max_cost(1000, Some(1000)) - 0.01125).abs() < f64::EPSILON);

  // Without request_max_output (uses model max 128_000):
  // 0 input + 128_000 * 1e-05 = 1.28
  assert!((model.calculate_max_cost(0, None) - 1.28).abs() < f64::EPSILON);
}

// =============================================================================
// Microdollar conversion tests
// =============================================================================

#[test]
fn converter_usd_to_micros() {
  use iron_cost::converter::usd_to_micros;

  assert_eq!(usd_to_micros(1.0), 1_000_000);
  assert_eq!(usd_to_micros(0.5), 500_000);
  assert_eq!(usd_to_micros(0.000_001), 1);
}

#[test]
fn converter_micros_to_usd() {
  use iron_cost::converter::micros_to_usd;

  assert!((micros_to_usd(1_000_000) - 1.0).abs() < f64::EPSILON);
  assert!((micros_to_usd(500_000) - 0.5).abs() < f64::EPSILON);
  assert!((micros_to_usd(1) - 0.000_001).abs() < f64::EPSILON);
}

// =============================================================================
// PricingManager::estimate_max_cost tests
// =============================================================================

#[test]
fn estimate_max_cost_returns_none_for_invalid_json() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");
  assert!(manager.estimate_max_cost(b"not json").is_none());
}

#[test]
fn estimate_max_cost_returns_none_for_missing_model() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");
  assert!(manager.estimate_max_cost(br#"{"messages": []}"#).is_none());
}

#[test]
fn estimate_max_cost_returns_none_for_unknown_model() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");
  assert!(manager
    .estimate_max_cost(br#"{"model": "unknown-model-xyz", "max_tokens": 100}"#)
    .is_none());
}

#[test]
fn estimate_max_cost_estimates_input_from_messages() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;
  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");

  let body = br#"{"model": "test-model", "max_tokens": 1000, "messages": [{"role": "user", "content": "This is a test message that is exactly four hundred characters long. We need to make sure we have enough text here to properly test the token estimation logic. The estimation uses approximately four characters per token which is a reasonable approximation for English text. Adding more text here to reach the target length of four hundred characters exactly."}]}"#;
  let cost = manager
    .estimate_max_cost(body)
    .expect("LOUD FAILURE: should estimate");

  assert!((2_000_000..=2_500_000).contains(&cost), "cost was {cost}");
}

#[test]
fn estimate_max_cost_uses_max_completion_tokens() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;
  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");

  let body = br#"{"model": "test-model", "max_completion_tokens": 500, "messages": [{"role": "user", "content": "Hello, how are you today?"}]}"#;
  let cost = manager
    .estimate_max_cost(body)
    .expect("LOUD FAILURE: should estimate");

  assert!(cost >= 1_000_000, "cost was {cost}");
}

#[test]
fn estimate_max_cost_uses_model_max_output_tokens_when_no_max_tokens() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.0001,
            "output_cost_per_token": 0.0001,
            "max_output_tokens": 8192
        }
    }"#;
  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");

  let body = br#"{"model": "test-model"}"#;
  let cost = manager
    .estimate_max_cost(body)
    .expect("LOUD FAILURE: should estimate");

  // 1000 input * 0.0001 + 8192 output * 0.0001 = 0.9192 USD = 919_200 micros
  assert_eq!(cost, 919_200);
}

#[test]
fn estimate_max_cost_uses_128000_fallback_when_no_model_max() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.000001,
            "output_cost_per_token": 0.000001
        }
    }"#;
  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");

  let body = br#"{"model": "test-model"}"#;
  let cost = manager
    .estimate_max_cost(body)
    .expect("LOUD FAILURE: should estimate");

  // 1000 input + 128_000 output * 0.000001 = 129_000 micros
  assert_eq!(cost, 129_000);
}

#[test]
fn estimate_max_cost_handles_anthropic_system_prompt() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.001
        }
    }"#;
  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");

  let body = br#"{"model": "test-model", "max_tokens": 100, "system": "You are a helpful assistant that answers questions concisely and accurately for the user.", "messages": [{"role": "user", "content": "What is the capital of France? Please tell me."}]}"#;
  let cost = manager
    .estimate_max_cost(body)
    .expect("LOUD FAILURE: should estimate");

  assert!(cost > 100_000, "cost was {cost}");
}

#[test]
fn estimate_max_cost_handles_content_blocks() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.001
        }
    }"#;
  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");

  let body = br#"{"model": "test-model", "max_tokens": 100, "messages": [{"role": "user", "content": [{"type": "text", "text": "What is in this image? Please describe it in detail."}]}]}"#;
  let cost = manager
    .estimate_max_cost(body)
    .expect("LOUD FAILURE: should estimate");

  assert!(cost > 100_000, "cost was {cost}");
}

#[test]
fn estimate_max_cost_minimum_1000_tokens_when_no_content() {
  let manager = PricingManager::new().expect("LOUD FAILURE: should create manager");

  let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.001
        }
    }"#;
  manager
    .load_from_file(json)
    .expect("LOUD FAILURE: should parse");

  let body = br#"{"model": "test-model", "max_tokens": 100, "messages": []}"#;
  let cost = manager
    .estimate_max_cost(body)
    .expect("LOUD FAILURE: should estimate");

  // 1000 minimum input + 100 output = 1100 tokens * $0.001 = $1.10 = 1_100_000 micros
  assert_eq!(cost, 1_100_000);
}
