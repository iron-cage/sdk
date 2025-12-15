//! Cost calculation service
//!
//! Converts token usage to USD costs using real provider pricing.

use std::collections::HashMap;

/// Model pricing configuration (USD per 1M tokens)
#[ derive( Debug, Clone, Copy ) ]
struct ModelPricing
{
  /// Cost per 1 million input tokens (USD)
  input_per_1m: f64,
  /// Cost per 1 million output tokens (USD)
  output_per_1m: f64,
}

/// Cost calculator
///
/// Calculates costs from token usage using real provider pricing.
#[ derive( Debug ) ]
pub struct CostCalculator
{
  pricing: HashMap< ( String, String ), ModelPricing >,
}

impl CostCalculator
{
  /// Create new cost calculator with current pricing data
  ///
  /// Pricing data as of December 2025.
  #[ must_use ]
  pub fn new() -> Self
  {
    let mut pricing = HashMap::new();

    // OpenAI pricing (Dec 2025)
    pricing.insert(
      ( "openai".to_string(), "gpt-4-turbo".to_string() ),
      ModelPricing { input_per_1m: 10.0, output_per_1m: 30.0 },
    );
    pricing.insert(
      ( "openai".to_string(), "gpt-3.5-turbo".to_string() ),
      ModelPricing { input_per_1m: 0.5, output_per_1m: 1.5 },
    );

    // Anthropic pricing (Dec 2025)
    pricing.insert(
      ( "anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string() ),
      ModelPricing { input_per_1m: 3.0, output_per_1m: 15.0 },
    );
    pricing.insert(
      ( "anthropic".to_string(), "claude-3-opus-20240229".to_string() ),
      ModelPricing { input_per_1m: 15.0, output_per_1m: 75.0 },
    );
    pricing.insert(
      ( "anthropic".to_string(), "claude-3-haiku-20240307".to_string() ),
      ModelPricing { input_per_1m: 0.25, output_per_1m: 1.25 },
    );

    // Google pricing (Dec 2025)
    pricing.insert(
      ( "google".to_string(), "gemini-1.5-pro".to_string() ),
      ModelPricing { input_per_1m: 1.25, output_per_1m: 5.0 },
    );
    pricing.insert(
      ( "google".to_string(), "gemini-1.5-flash".to_string() ),
      ModelPricing { input_per_1m: 0.075, output_per_1m: 0.30 },
    );

    Self { pricing }
  }

  /// Calculate cost in cents for token usage
  ///
  /// # Arguments
  ///
  /// * `provider` - Provider name (e.g., "openai", "anthropic", "google")
  /// * `model` - Model name (e.g., "gpt-4-turbo", "claude-3-5-sonnet-20241022")
  /// * `input_tokens` - Number of input tokens
  /// * `output_tokens` - Number of output tokens
  ///
  /// # Returns
  ///
  /// Cost in cents (USD cents). Returns 0 for unknown provider/model.
  ///
  /// # Examples
  ///
  /// ```
  /// use iron_token_manager::cost_calculator::CostCalculator;
  ///
  /// let calculator = CostCalculator::new();
  /// let cost = calculator.calculate_cost( "openai", "gpt-4-turbo", 100_000, 50_000 );
  /// assert_eq!( cost, 250 ); // $2.50 = 250 cents
  /// ```
  #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
  #[ must_use ]
  pub fn calculate_cost(
    &self,
    provider: &str,
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
  ) -> i64
  {
    let key = ( provider.to_string(), model.to_string() );

    let Some( pricing ) = self.pricing.get( &key ) else {
      // Unknown provider/model - return zero
      return 0;
    };

    // Calculate cost in USD
    let input_cost_usd = ( input_tokens as f64 / 1_000_000.0 ) * pricing.input_per_1m;
    let output_cost_usd = ( output_tokens as f64 / 1_000_000.0 ) * pricing.output_per_1m;
    let total_cost_usd = input_cost_usd + output_cost_usd;

    // Convert to cents and round
    ( total_cost_usd * 100.0 ).round() as i64
  }

  /// Get list of all supported providers
  ///
  /// # Returns
  ///
  /// Vector of provider names
  #[ must_use ]
  pub fn get_providers( &self ) -> Vec< String >
  {
    let mut providers: Vec< String > = self
      .pricing
      .keys()
      .map( |( provider, _ )| provider.clone() )
      .collect();

    providers.sort();
    providers.dedup();

    providers
  }

  /// Get list of models for a provider
  ///
  /// # Arguments
  ///
  /// * `provider` - Provider name
  ///
  /// # Returns
  ///
  /// Vector of model names for the provider
  #[ must_use ]
  pub fn get_provider_models( &self, provider: &str ) -> Vec< String >
  {
    let mut models: Vec< String > = self
      .pricing
      .keys()
      .filter_map( |( p, model )| if p == provider { Some( model.clone() ) } else { None } )
      .collect();

    models.sort();

    models
  }
}

impl Default for CostCalculator
{
  fn default() -> Self
  {
    Self::new()
  }
}
