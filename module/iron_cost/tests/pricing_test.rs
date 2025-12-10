use iron_cost::pricing::PricingManager;

#[test]
fn load_file(){
    let manager =
        match PricingManager::new(){
            Ok( m ) => m,
            Err(e) => panic!("{}", e)
        };
    let gpt4 = manager.get("openrouter/openai/gpt-4o");
    assert!(gpt4.is_some());
    assert_eq!(gpt4.unwrap().max_tokens(), Some(4096u32));
}

#[test]
fn calculate_cost(){
    let manager =
        match PricingManager::new(){
            Ok( m ) => m,
            Err(e) => panic!("{}", e)
        };
    let model = manager.get("gpt-5.1-codex");
    assert!(model.is_some());
    let model = model.unwrap();
    assert_eq!(model.calculate_cost(500, 100), 0.0016250000000000001);

}