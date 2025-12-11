use iron_cost::budget::CostController;
use iron_cost::error::CostError;

// =============================================================================
// Creation and initialization
// =============================================================================

#[test]
fn new_controller_has_zero_spent() {
    let controller = CostController::new(10.0);
    let (spent, limit) = controller.get_status();
    assert_eq!(spent, 0.0);
    assert_eq!(limit, 10.0);
}

#[test]
fn new_controller_with_zero_budget() {
    let controller = CostController::new(0.0);
    let (spent, limit) = controller.get_status();
    assert_eq!(spent, 0.0);
    assert_eq!(limit, 0.0);
}

// =============================================================================
// Budget checking
// =============================================================================

#[test]
fn check_budget_passes_when_under_limit() {
    let controller = CostController::new(10.0);
    assert!(controller.check_budget().is_ok());
}

#[test]
fn check_budget_fails_when_at_limit() {
    let controller = CostController::new(10.0);
    controller.add_spend(10.0);
    assert!(controller.check_budget().is_err());
}

#[test]
fn check_budget_fails_when_over_limit() {
    let controller = CostController::new(10.0);
    controller.add_spend(15.0);
    assert!(controller.check_budget().is_err());
}

#[test]
fn check_budget_returns_correct_error_values() {
    let controller = CostController::new(5.0);
    controller.add_spend(7.5);

    let err = controller.check_budget().unwrap_err();
    match err {
        CostError::BudgetExceeded { spent_usd, limit_usd } => {
            assert_eq!(spent_usd, 7.5);
            assert_eq!(limit_usd, 5.0);
        }
    }
}

#[test]
fn zero_budget_fails_immediately() {
    let controller = CostController::new(0.0);
    assert!(controller.check_budget().is_err());
}

// =============================================================================
// Spending
// =============================================================================

#[test]
fn add_spend_accumulates() {
    let controller = CostController::new(100.0);
    controller.add_spend(1.0);
    controller.add_spend(2.0);
    controller.add_spend(3.0);

    let (spent, _) = controller.get_status();
    assert_eq!(spent, 6.0);
}

#[test]
fn add_spend_with_small_amounts() {
    let controller = CostController::new(1.0);
    controller.add_spend(0.000001); // 1 microdollar
    controller.add_spend(0.000001);
    controller.add_spend(0.000001);

    let (spent, _) = controller.get_status();
    assert_eq!(spent, 0.000003);
}

// =============================================================================
// Budget modification
// =============================================================================

#[test]
fn set_budget_updates_limit() {
    let controller = CostController::new(10.0);
    controller.set_budget(20.0);

    let (_, limit) = controller.get_status();
    assert_eq!(limit, 20.0);
}

#[test]
fn set_budget_can_unblock_exceeded_budget() {
    let controller = CostController::new(5.0);
    controller.add_spend(7.0);

    // Budget exceeded
    assert!(controller.check_budget().is_err());

    // Increase budget
    controller.set_budget(10.0);

    // Now it should pass
    assert!(controller.check_budget().is_ok());
}

#[test]
fn set_budget_can_block_previously_valid_budget() {
    let controller = CostController::new(10.0);
    controller.add_spend(5.0);

    // Budget is fine
    assert!(controller.check_budget().is_ok());

    // Reduce budget below spent
    controller.set_budget(3.0);

    // Now it should fail
    assert!(controller.check_budget().is_err());
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn handles_large_amounts() {
    let controller = CostController::new(1_000_000.0);
    controller.add_spend(500_000.0);

    let (spent, limit) = controller.get_status();
    assert_eq!(spent, 500_000.0);
    assert_eq!(limit, 1_000_000.0);
    assert!(controller.check_budget().is_ok());
}

#[test]
fn precision_at_microdollar_level() {
    let controller = CostController::new(0.000010); // 10 microdollars
    controller.add_spend(0.000005); // 5 microdollars

    let (spent, limit) = controller.get_status();
    assert_eq!(spent, 0.000005);
    assert_eq!(limit, 0.000010);
    assert!(controller.check_budget().is_ok());

    controller.add_spend(0.000005); // Now at exactly 10 microdollars
    assert!(controller.check_budget().is_err()); // At limit = exceeded
}