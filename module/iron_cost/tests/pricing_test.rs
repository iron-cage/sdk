//! Tests for pricing module.

use iron_cost::pricing::PricingManager;

// =============================================================================
// PricingManager tests
// =============================================================================

#[test]
fn new_creates_manager_with_embedded_data() {
    let manager = PricingManager::new().expect("should create manager");

    // Should have loaded models from embedded JSON
    assert!(manager.get("gpt-5.1").is_some());
}

#[test]
fn get_returns_existing_model() {
    let manager = PricingManager::new().expect("should create manager");

    let model = manager.get("gpt-5.1");
    assert!(model.is_some());

    let model = model.unwrap();
    // gpt-5.1: input_cost_per_token: 1.25e-06, output_cost_per_token: 1e-05
    assert_eq!(model.input_cost_per_token(), 0.00000125);
    assert_eq!(model.output_cost_per_token(), 0.00001);
    assert_eq!(model.max_output_tokens(), Some(128000));
}

#[test]
fn get_returns_none_for_nonexistent_model() {
    let manager = PricingManager::new().expect("should create manager");

    let model = manager.get("nonexistent-model-xyz-123");
    assert!(model.is_none());
}

#[test]
fn load_from_file_parses_valid_json() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_output_tokens": 4096
        }
    }"#;

    manager.load_from_file(json).expect("should parse valid JSON");

    let model = manager.get("test-model");
    assert!(model.is_some());

    let model = model.unwrap();
    assert_eq!(model.input_cost_per_token(), 0.001);
    assert_eq!(model.output_cost_per_token(), 0.002);
    assert_eq!(model.max_output_tokens(), Some(4096));
}

#[test]
fn load_from_file_returns_error_for_invalid_json() {
    let manager = PricingManager::new().expect("should create manager");

    let result = manager.load_from_file("not valid json {{{");
    assert!(result.is_err());
}

#[test]
fn load_from_file_filters_sample_spec() {
    let manager = PricingManager::new().expect("should create manager");

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

    manager.load_from_file(json).expect("should parse");

    assert!(manager.get("sample_spec").is_none(), "sample_spec should be filtered");
    assert!(manager.get("real-model").is_some(), "real model should exist");
}

#[test]
fn load_from_file_filters_zero_cost_models() {
    let manager = PricingManager::new().expect("should create manager");

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

    manager.load_from_file(json).expect("should parse");

    assert!(manager.get("zero-cost-model").is_none(), "zero cost model should be filtered");
    assert!(manager.get("valid-model").is_some(), "model with any non-zero cost should exist");
}

#[test]
fn load_from_file_sets_name_from_key() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "my-model-name": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

    manager.load_from_file(json).expect("should parse");

    let model = manager.get("my-model-name").expect("model should exist");
    assert_eq!(model.name(), "my-model-name");
}

// =============================================================================
// Model getters tests
// =============================================================================

#[test]
fn model_name_returns_correct_value() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    assert_eq!(model.name(), "test-model");
}

#[test]
fn model_max_output_tokens_prefers_max_output_tokens_field() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_output_tokens": 8192,
            "max_tokens": 4096
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    assert_eq!(model.max_output_tokens(), Some(8192));
}

#[test]
fn model_max_output_tokens_falls_back_to_max_tokens() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_tokens": 4096
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    assert_eq!(model.max_output_tokens(), Some(4096));
}

#[test]
fn model_max_tokens_returns_raw_value() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_tokens": 2048
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    assert_eq!(model.max_tokens(), Some(2048));
}

// =============================================================================
// Model::calculate_cost tests
// =============================================================================

#[test]
fn calculate_cost_with_zero_tokens() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    assert_eq!(model.calculate_cost(0, 0), 0.0);
}

#[test]
fn calculate_cost_input_only() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    // 1000 input tokens * 0.001 = 1.0
    assert_eq!(model.calculate_cost(1000, 0), 1.0);
}

#[test]
fn calculate_cost_output_only() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    // 500 output tokens * 0.002 = 1.0
    assert_eq!(model.calculate_cost(0, 500), 1.0);
}

#[test]
fn calculate_cost_mixed() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    // 1000 * 0.001 + 500 * 0.002 = 1.0 + 1.0 = 2.0
    assert_eq!(model.calculate_cost(1000, 500), 2.0);
}

// =============================================================================
// Model::calculate_max_cost tests
// =============================================================================

#[test]
fn calculate_max_cost_uses_request_max_output() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_output_tokens": 8192
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    // 1000 * 0.001 + 100 * 0.002 = 1.0 + 0.2 = 1.2
    assert_eq!(model.calculate_max_cost(1000, Some(100)), 1.2);
}

#[test]
fn calculate_max_cost_uses_model_max_when_no_request_max() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_output_tokens": 1000
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    // 500 * 0.001 + 1000 * 0.002 = 0.5 + 2.0 = 2.5
    assert_eq!(model.calculate_max_cost(500, None), 2.5);
}

#[test]
fn calculate_max_cost_caps_at_model_max() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.002,
            "max_output_tokens": 100
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    // Request 10000 but model max is 100
    // 500 * 0.001 + 100 * 0.002 = 0.5 + 0.2 = 0.7
    assert_eq!(model.calculate_max_cost(500, Some(10000)), 0.7);
}

#[test]
fn calculate_max_cost_uses_default_128000_when_no_max() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.0001,
            "output_cost_per_token": 0.0001
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    // No max_output_tokens or max_tokens, defaults to 128000
    // 0 * 0.0001 + 128000 * 0.0001 = 12.8
    assert_eq!(model.calculate_max_cost(0, None), 12.8);
}

// =============================================================================
// Model::has_valid_pricing tests
// =============================================================================

#[test]
fn has_valid_pricing_true_for_nonzero_input_cost() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.001,
            "output_cost_per_token": 0.0
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    assert!(model.has_valid_pricing());
}

#[test]
fn has_valid_pricing_true_for_nonzero_output_cost() {
    let manager = PricingManager::new().expect("should create manager");

    let json = r#"{
        "test-model": {
            "input_cost_per_token": 0.0,
            "output_cost_per_token": 0.001
        }
    }"#;

    manager.load_from_file(json).expect("should parse");
    let model = manager.get("test-model").expect("should exist");

    assert!(model.has_valid_pricing());
}

// =============================================================================
// Real model tests (from embedded JSON)
// =============================================================================

#[test]
fn real_model_gpt51_exists() {
    let manager = PricingManager::new().expect("should create manager");

    let model = manager.get("gpt-5.1");
    assert!(model.is_some(), "gpt-5.1 should exist in embedded pricing");
}

#[test]
fn real_model_gpt51_pricing() {
    let manager = PricingManager::new().expect("should create manager");

    let model = manager.get("gpt-5.1").expect("gpt-5.1 should exist");

    // gpt-5.1 pricing from LiteLLM JSON
    assert_eq!(model.input_cost_per_token(), 0.00000125);  // 1.25e-06
    assert_eq!(model.output_cost_per_token(), 0.00001);    // 1e-05
    assert_eq!(model.max_output_tokens(), Some(128000));
    assert_eq!(model.max_tokens(), Some(128000));
}

#[test]
fn real_model_gpt51_calculate_cost() {
    let manager = PricingManager::new().expect("should create manager");

    let model = manager.get("gpt-5.1").expect("gpt-5.1 should exist");

    // 1000 input * 1.25e-06 + 500 output * 1e-05 = 0.00125 + 0.005 = 0.00625
    assert_eq!(model.calculate_cost(1000, 500), 0.00625);
}

#[test]
fn real_model_gpt51_calculate_max_cost() {
    let manager = PricingManager::new().expect("should create manager");

    let model = manager.get("gpt-5.1").expect("gpt-5.1 should exist");

    // With request_max_output = 1000:
    // 1000 input * 1.25e-06 + 1000 output * 1e-05 = 0.00125 + 0.01 = 0.01125
    assert_eq!(model.calculate_max_cost(1000, Some(1000)), 0.01125);

    // Without request_max_output (uses model max 128000):
    // 0 input + 128000 * 1e-05 = 1.28
    assert_eq!(model.calculate_max_cost(0, None), 1.28);
}
