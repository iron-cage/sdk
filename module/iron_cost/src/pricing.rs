//! LLM model pricing management.
//!
//! Provides pricing data for various LLM models, loaded from LiteLLM pricing JSON.
//! Uses integer arithmetic (microdollars) internally to avoid floating-point errors.
//!
//! # Precision
//!
//! All costs are stored and calculated in microdollars (1 USD = 1,000,000 micros).
//! Per-token costs are stored as "microdollars per million tokens" to maintain
//! integer precision for small values like $0.00000125/token = $1.25/M tokens = 1,250,000 micros/M.

use std::collections::HashMap;
use std::sync::Arc;
use arc_swap::ArcSwap;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

const PRICING_JSON: &str = include_str!("../asset/pricing.json");
const MAX_OUTPUT_TOKENS: u64 = 128000;
const MICROS_PER_USD: u64 = 1_000_000;
const TOKENS_PER_MILLION: u64 = 1_000_000;

/// Manages LLM model pricing data with thread-safe concurrent access.
///
/// Pricing data is loaded from embedded LiteLLM JSON at construction.
/// Uses `ArcSwap` for lock-free reads and atomic updates.
pub struct PricingManager {
    pricing: ArcSwap<HashMap<String, Model>>,
}

/// Pricing information for a single LLM model.
///
/// All costs stored internally as microdollars per million tokens (u64).
/// This avoids floating-point precision errors in cost calculations.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Model {
    name: String,
    /// Cost per million input tokens in microdollars
    input_cost_per_mtok_micros: u64,
    /// Cost per million output tokens in microdollars
    output_cost_per_mtok_micros: u64,
    max_output_tokens: Option<u64>,
    max_tokens: Option<u64>,
}

/// Intermediate struct for deserializing from JSON (which uses f64)
#[derive(Deserialize)]
struct ModelJson {
    #[serde(alias = "id", default)]
    name: String,
    #[serde(default)]
    input_cost_per_token: f64,
    #[serde(default)]
    output_cost_per_token: f64,
    #[serde(default)]
    max_output_tokens: Option<u64>,
    #[serde(default)]
    max_tokens: Option<u64>,
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = ModelJson::deserialize(deserializer)?;
        Ok(Model::from_json(json))
    }
}

impl Model {
    /// Convert from JSON representation (f64 per-token costs) to internal (micros per million tokens)
    fn from_json(json: ModelJson) -> Self {
        Self {
            name: json.name,
            input_cost_per_mtok_micros: usd_per_token_to_micros_per_mtok(json.input_cost_per_token),
            output_cost_per_mtok_micros: usd_per_token_to_micros_per_mtok(json.output_cost_per_token),
            max_output_tokens: json.max_output_tokens,
            max_tokens: json.max_tokens,
        }
    }

    /// Returns the model identifier.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns cost per input token in USD.
    /// Note: This converts from internal integer representation.
    pub fn input_cost_per_token(&self) -> f64 {
        micros_per_mtok_to_usd_per_token(self.input_cost_per_mtok_micros)
    }

    /// Returns cost per output token in USD.
    /// Note: This converts from internal integer representation.
    pub fn output_cost_per_token(&self) -> f64 {
        micros_per_mtok_to_usd_per_token(self.output_cost_per_mtok_micros)
    }

    /// Returns maximum output tokens, falling back to `max_tokens` if not set.
    pub fn max_output_tokens(&self) -> Option<u64> {
        self.max_output_tokens.or(self.max_tokens)
    }

    /// Returns legacy max_tokens value.
    pub fn max_tokens(&self) -> Option<u64> {
        self.max_tokens
    }

    /// Returns true if model has non-zero pricing data.
    pub fn has_valid_pricing(&self) -> bool {
        self.input_cost_per_mtok_micros > 0 || self.output_cost_per_mtok_micros > 0
    }

    /// Calculates actual cost in microdollars based on token usage.
    ///
    /// Uses integer arithmetic to avoid floating-point precision errors.
    /// Formula: (tokens * micros_per_million_tokens) / 1_000_000
    pub fn calculate_cost_micros(&self, input_len: u64, output_len: u64) -> u64 {
        let input_cost = (input_len * self.input_cost_per_mtok_micros) / TOKENS_PER_MILLION;
        let output_cost = (output_len * self.output_cost_per_mtok_micros) / TOKENS_PER_MILLION;
        input_cost + output_cost
    }

    /// Calculates actual cost in USD based on token usage.
    ///
    /// Convenience method that converts microdollars to USD.
    pub fn calculate_cost(&self, input_len: u32, output_len: u32) -> f64 {
        micros_to_usd(self.calculate_cost_micros(input_len as u64, output_len as u64))
    }

    /// Calculates worst-case cost in microdollars for budget pre-reservation.
    ///
    /// Uses `request_max_output` if provided, otherwise model's max output limit.
    pub fn calculate_max_cost_micros(&self, input_len: u64, request_max_output: Option<u64>) -> u64 {
        let max_output = self.max_output_tokens().unwrap_or(MAX_OUTPUT_TOKENS);
        let output_limit = request_max_output
            .unwrap_or(max_output)
            .min(max_output);

        let input_cost = (input_len * self.input_cost_per_mtok_micros) / TOKENS_PER_MILLION;
        let output_cost = (output_limit * self.output_cost_per_mtok_micros) / TOKENS_PER_MILLION;
        input_cost + output_cost
    }

    /// Calculates worst-case cost in USD for budget pre-reservation.
    ///
    /// Convenience method that converts microdollars to USD.
    pub fn calculate_max_cost(&self, input_len: u32, request_max_output: Option<u32>) -> f64 {
        micros_to_usd(self.calculate_max_cost_micros(
            input_len as u64,
            request_max_output.map(|v| v as u64),
        ))
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

// =============================================================================
// Conversion functions
// =============================================================================

/// Convert USD per token (f64) to microdollars per million tokens (u64)
///
/// Example: $0.00000125/token = $1.25/M tokens = 1,250,000 micros/M tokens
fn usd_per_token_to_micros_per_mtok(usd_per_token: f64) -> u64 {
    // USD/token * 1M tokens/M * 1M micros/USD = micros/M
    let micros = usd_per_token * (TOKENS_PER_MILLION as f64) * (MICROS_PER_USD as f64);
    micros.round().max(0.0) as u64
}

/// Convert microdollars per million tokens (u64) to USD per token (f64)
fn micros_per_mtok_to_usd_per_token(micros_per_mtok: u64) -> f64 {
    (micros_per_mtok as f64) / (TOKENS_PER_MILLION as f64) / (MICROS_PER_USD as f64)
}

/// Convert microdollars to USD
pub fn micros_to_usd(micros: u64) -> f64 {
    micros as f64 / MICROS_PER_USD as f64
}

/// Convert USD to microdollars
pub fn usd_to_micros(usd: f64) -> u64 {
    (usd * MICROS_PER_USD as f64).round().max(0.0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usd_per_token_to_micros_per_mtok() {
        // $0.00000125/token = $1.25/M = 1,250,000 micros/M
        assert_eq!(usd_per_token_to_micros_per_mtok(0.00000125), 1_250_000);

        // $0.00001/token = $10/M = 10,000,000 micros/M
        assert_eq!(usd_per_token_to_micros_per_mtok(0.00001), 10_000_000);

        // $0.001/token = $1000/M = 1,000,000,000 micros/M
        assert_eq!(usd_per_token_to_micros_per_mtok(0.001), 1_000_000_000);
    }

    #[test]
    fn test_micros_per_mtok_to_usd_per_token() {
        // 1,250,000 micros/M = $1.25/M = $0.00000125/token
        assert_eq!(micros_per_mtok_to_usd_per_token(1_250_000), 0.00000125);

        // 10,000,000 micros/M = $10/M = $0.00001/token
        assert_eq!(micros_per_mtok_to_usd_per_token(10_000_000), 0.00001);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original = 0.00000125;
        let micros = usd_per_token_to_micros_per_mtok(original);
        let back = micros_per_mtok_to_usd_per_token(micros);
        assert_eq!(original, back);
    }
}