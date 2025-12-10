//! LLM model pricing management.
//!
//! Provides pricing data for various LLM models, loaded from LiteLLM pricing JSON.
//! Supports cost calculation for budget enforcement.

use std::collections::HashMap;
use std::sync::Arc;
use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const PRICING_JSON: &str = include_str!("../asset/pricing.json");
const MAX_OUTPUT_TOKENS: u32 = 128000;

/// Manages LLM model pricing data with thread-safe concurrent access.
///
/// Pricing data is loaded from embedded LiteLLM JSON at construction.
/// Uses `ArcSwap` for lock-free reads and atomic updates.
pub struct PricingManager {
    pricing: ArcSwap<HashMap<String, Model>>,
}

/// Pricing information for a single LLM model.
///
/// Contains per-token costs and output limits used for cost calculation
/// and budget pre-reservation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Model {
    #[serde(alias = "id", default)]
    name: String,
    #[serde(default)]
    input_cost_per_token: f64,
    #[serde(default)]
    output_cost_per_token: f64,
    #[serde(default)]
    max_output_tokens: Option<u32>,
    #[serde(default)]
    max_tokens: Option<u32>,
}

impl Model {
    /// Returns the model identifier.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns cost per input token in USD.
    pub fn input_cost_per_token(&self) -> f64 {
        self.input_cost_per_token
    }

    /// Returns cost per output token in USD.
    pub fn output_cost_per_token(&self) -> f64 {
        self.output_cost_per_token
    }

    /// Returns maximum output tokens, falling back to `max_tokens` if not set.
    pub fn max_output_tokens(&self) -> Option<u32> {
        self.max_output_tokens.or(self.max_tokens)
    }

    /// Returns legacy max_tokens value.
    pub fn max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    /// Calculates worst-case cost for budget pre-reservation.
    ///
    /// Uses `request_max_output` if provided, otherwise model's max output limit.
    /// Result is used to reserve budget before sending LLM request.
    pub fn calculate_max_cost(&self, input_len: u32, request_max_output: Option<u32>) -> f64 {
        let max_output = self.max_output_tokens().unwrap_or(MAX_OUTPUT_TOKENS);
        let output_limit = request_max_output
            .unwrap_or(max_output)
            .min(max_output);

        input_len as f64 * self.input_cost_per_token + output_limit as f64 * self.output_cost_per_token
    }

    /// Returns true if model has non-zero pricing data.
    pub fn has_valid_pricing(&self) -> bool {
        self.input_cost_per_token > 0.0 || self.output_cost_per_token > 0.0
    }

    /// Calculates actual cost based on token usage.
    pub fn calculate_cost(&self, input_len: u32, output_len: u32) -> f64 {
        input_len as f64 * self.input_cost_per_token + output_len as f64 * self.output_cost_per_token
    }
}

impl PricingManager {
    /// Creates a new PricingManager with embedded LiteLLM pricing data.
    pub fn new() -> Result<PricingManager, String> {
        let manager = PricingManager {
            pricing: ArcSwap::from_pointee(HashMap::new()),
        };
        manager.load_from_file(PRICING_JSON)?;
        Ok(manager)
    }

    /// Loads pricing data from JSON string.
    ///
    /// Filters out invalid entries (sample_spec, models without pricing).
    /// Can be used to reload pricing from external source.
    pub fn load_from_file(&self, json_str: &str) -> Result<(), String> {
        let raw_map: HashMap<String, Value> = serde_json::from_str(json_str)
            .map_err(|_| "Failed to parse json".to_string())?;

        let mut new_map = HashMap::new();

        for (key, value) in raw_map {
            if key == "sample_spec" {
                continue;
            }

            if let Ok(mut model) = serde_json::from_value::<Model>(value) {
                if !model.has_valid_pricing() {
                    continue;
                }

                if model.name.is_empty() {
                    model.name = key.clone();
                }
                new_map.insert(key, model);
            }
        }

        self.pricing.store(Arc::new(new_map));
        Ok(())
    }

    /// Returns pricing for the specified model, if available.
    pub fn get(&self, model_id: &str) -> Option<Model> {
        self.pricing.load().get(model_id).cloned()
    }
}
