//! Integration tests for budget enforcement with `iron_cost`
//!
//! Tests integration between `CostCalculator` and `iron_cost::BudgetTracker`.
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_calculate_and_track_cost` | Basic cost calculation + tracking | GPT-4 Turbo, 100k input, 50k output, $10 budget | $2.50 cost tracked, $7.50 remaining | ✅ |
//! | `test_budget_enforcement_prevents_overspending` | Budget enforcement prevents overflow | $0.50 budget, 3 x $0.25 calls | First 2 succeed, 3rd rejected | ✅ |
//! | `test_multiple_providers_cost_tracking` | Multi-provider cost aggregation | `OpenAI` + `Anthropic` + `Google` costs | Total = sum of all costs | ✅ |
//! | `test_per_agent_budget_isolation` | Agents share budget | Agent1 $2.50 + Agent2 $1.58, $10 budget | Remaining = $10 - $2.50 - $1.58 | ✅ |
//! | `test_zero_cost_doesnt_affect_budget` | Zero tokens = zero cost | 0 input, 0 output tokens | $0.00 cost, budget unchanged | ✅ |
//! | `test_unknown_provider_zero_cost_budget_safe` | Unknown provider = zero cost | "`unknown_provider`", 1M tokens | $0.00 cost, budget unchanged | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:**
//! - ✅ Calculate cost + track in budget
//! - ✅ Multiple providers (`OpenAI`, `Anthropic`, `Google`)
//! - ✅ Per-agent tracking with shared budget
//!
//! **Boundary Conditions:**
//! - ✅ Exactly at budget ($0.50 budget, $0.25 + $0.25 = allowed, $0.25 more = denied)
//! - ✅ Zero cost (0 tokens → $0.00 → no budget impact)
//! - ✅ Remaining budget calculation accuracy
//!
//! **Error Conditions:**
//! - ✅ Exceeding budget → `record_cost()` returns Err
//! - ✅ Error message contains "Budget" keyword
//!
//! **Edge Cases:**
//! - ✅ Unknown provider produces zero cost (safe fallback)
//! - ✅ Zero cost doesn't affect budget tracking
//! - ✅ Multi-agent budget sharing (all agents counted toward total)
//! - ✅ Multiple providers aggregated correctly
//!
//! **State Transitions:**
//! - ✅ $10 budget → $2.50 spent → $7.50 remaining
//! - ✅ $0.50 budget → $0.25 spent → $0.25 remaining → $0.25 spent → $0.00 remaining → Deny
//!
//! **Concurrent Access:** Not tested (`BudgetTracker` handles locking internally)
//! **Resource Limits:** Not applicable (in-memory budget tracking)
//! **Precondition Violations:** Not applicable (`BudgetTracker` validates inputs, `CostCalculator` handles unknown providers gracefully)

use iron_token_manager::cost_calculator::CostCalculator;
use iron_cost::BudgetTracker;

#[ test ]
fn test_calculate_and_track_cost()
{
  let calculator = CostCalculator::new();
  let tracker = BudgetTracker::new( 10.0 ); // $10 budget

  // Calculate cost for GPT-4 usage
  let cost_cents = calculator.calculate_cost( "openai", "gpt-4-turbo", 100_000, 50_000 );
  assert_eq!( cost_cents, 250 ); // $2.50

  // Convert to USD and record
  let cost_usd = cost_cents as f64 / 100.0;
  tracker.record_cost( "agent_001", cost_usd ).expect( "Should allow within budget" );

  // Check remaining budget
  let remaining = tracker.remaining();
  assert!( ( remaining - 7.5 ).abs() < 0.01, "Should have $7.50 remaining" );
}

#[ test ]
fn test_budget_enforcement_prevents_overspending()
{
  let calculator = CostCalculator::new();
  let tracker = BudgetTracker::new( 0.50 ); // $0.50 budget (tight)

  // First call: $0.25 (within budget)
  let cost1_cents = calculator.calculate_cost( "openai", "gpt-3.5-turbo", 200_000, 100_000 );
  let cost1_usd = cost1_cents as f64 / 100.0;
  tracker.record_cost( "agent_002", cost1_usd ).expect( "First call should succeed" );

  // Second call: $0.25 (still within budget - total $0.50)
  let cost2_cents = calculator.calculate_cost( "openai", "gpt-3.5-turbo", 200_000, 100_000 );
  let cost2_usd = cost2_cents as f64 / 100.0;
  tracker.record_cost( "agent_002", cost2_usd ).expect( "Second call should succeed" );

  // Third call: $0.25 (would exceed budget - total would be $0.75 > $0.50)
  let cost3_cents = calculator.calculate_cost( "openai", "gpt-3.5-turbo", 200_000, 100_000 );
  let cost3_usd = cost3_cents as f64 / 100.0;
  let result = tracker.record_cost( "agent_002", cost3_usd );

  assert!( result.is_err(), "Should reject call exceeding budget" );
  assert!( result.unwrap_err().to_string().contains( "Budget" ), "Error should mention budget" );
}

#[ test ]
fn test_multiple_providers_cost_tracking()
{
  let calculator = CostCalculator::new();
  let tracker = BudgetTracker::new( 5.0 ); // $5 budget

  // Track OpenAI cost
  let openai_cost_cents = calculator.calculate_cost( "openai", "gpt-4-turbo", 50_000, 25_000 );
  let openai_cost_usd = openai_cost_cents as f64 / 100.0;
  tracker.record_cost( "agent_003", openai_cost_usd ).expect( "Should track OpenAI" );

  // Track Anthropic cost
  let anthropic_cost_cents = calculator.calculate_cost( "anthropic", "claude-3-haiku-20240307", 400_000, 200_000 );
  let anthropic_cost_usd = anthropic_cost_cents as f64 / 100.0;
  tracker.record_cost( "agent_003", anthropic_cost_usd ).expect( "Should track Anthropic" );

  // Track Google cost
  let google_cost_cents = calculator.calculate_cost( "google", "gemini-1.5-flash", 500_000, 250_000 );
  let google_cost_usd = google_cost_cents as f64 / 100.0;
  tracker.record_cost( "agent_003", google_cost_usd ).expect( "Should track Google" );

  // Verify total spent
  let total_spent = tracker.total_spent();
  let expected = openai_cost_usd + anthropic_cost_usd + google_cost_usd;
  assert!( ( total_spent - expected ).abs() < 0.01, "Total should match sum of costs" );
}

#[ test ]
fn test_per_agent_budget_isolation()
{
  let calculator = CostCalculator::new();
  let tracker = BudgetTracker::new( 10.0 ); // $10 total budget

  // Agent 1 uses $2.50
  let cost1_cents = calculator.calculate_cost( "openai", "gpt-4-turbo", 100_000, 50_000 );
  let cost1_usd = cost1_cents as f64 / 100.0;
  tracker.record_cost( "agent_004", cost1_usd ).expect( "Agent 1 should succeed" );

  // Agent 2 uses $1.575
  let cost2_cents = calculator.calculate_cost( "anthropic", "claude-3-5-sonnet-20241022", 150_000, 75_000 );
  let cost2_usd = cost2_cents as f64 / 100.0;
  tracker.record_cost( "agent_005", cost2_usd ).expect( "Agent 2 should succeed" );

  // Verify remaining budget accounts for both
  let remaining = tracker.remaining();
  let expected_remaining = 10.0 - cost1_usd - cost2_usd;
  assert!( ( remaining - expected_remaining ).abs() < 0.01, "Remaining should account for both agents" );
}

#[ test ]
fn test_zero_cost_doesnt_affect_budget()
{
  let calculator = CostCalculator::new();
  let tracker = BudgetTracker::new( 5.0 ); // $5 budget

  // Zero tokens should result in zero cost
  let cost_cents = calculator.calculate_cost( "openai", "gpt-4-turbo", 0, 0 );
  assert_eq!( cost_cents, 0 );

  let cost_usd = cost_cents as f64 / 100.0;
  tracker.record_cost( "agent_006", cost_usd ).expect( "Zero cost should succeed" );

  // Budget should be unchanged
  let remaining = tracker.remaining();
  assert!( ( remaining - 5.0 ).abs() < 0.01, "Budget should be unchanged" );
}

#[ test ]
fn test_unknown_provider_zero_cost_budget_safe()
{
  let calculator = CostCalculator::new();
  let tracker = BudgetTracker::new( 1.0 ); // $1 budget

  // Unknown provider returns zero cost
  let cost_cents = calculator.calculate_cost( "unknown_provider", "unknown_model", 1_000_000, 500_000 );
  assert_eq!( cost_cents, 0 );

  let cost_usd = cost_cents as f64 / 100.0;
  tracker.record_cost( "agent_007", cost_usd ).expect( "Zero cost should succeed" );

  // Budget should be unchanged
  let remaining = tracker.remaining();
  assert!( ( remaining - 1.0 ).abs() < 0.01, "Budget should be unchanged" );
}
