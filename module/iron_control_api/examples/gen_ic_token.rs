use iron_control_api::ic_token::{IcTokenClaims, IcTokenManager};

fn main() {
    let secret = "dev-ic-token-secret-change-in-production";
    let manager = IcTokenManager::new(secret.to_string());
    let claims = IcTokenClaims::new(
        "agent_9999".to_string(),
        "budget_9999".to_string(),
        vec!["llm:call".to_string(), "analytics:write".to_string()],
        None,
    );
    println!("{}", manager.generate_token(&claims).unwrap());
}
