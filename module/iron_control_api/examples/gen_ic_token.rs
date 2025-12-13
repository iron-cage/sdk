//! Generate IC Token for testing
//!
//! Run with: cargo run --package iron_control_api --example gen_ic_token

use iron_control_api::ic_token::{IcTokenClaims, IcTokenManager};

fn main() {
    // Default secret used by server when IC_TOKEN_SECRET env var is not set
    let secret = "dev-ic-token-secret-change-in-production";
    let manager = IcTokenManager::new(secret.to_string());

    let claims = IcTokenClaims::new(
        "agent_1".to_string(),
        "budget_test".to_string(),
        vec!["llm:call".to_string(), "analytics:write".to_string()],
        None, // No expiration
    );

    let token = manager.generate_token(&claims).expect("LOUD FAILURE: Failed to generate token");
    println!("IC_TOKEN={}", token);
}
