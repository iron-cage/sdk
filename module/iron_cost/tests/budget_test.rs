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

#[test]
fn new_controller_has_zero_reserved() {
    let controller = CostController::new(10.0);
    assert_eq!(controller.total_reserved(), 0.0);
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
        CostError::BudgetExceeded { spent_usd, limit_usd, reserved_usd } => {
            assert_eq!(spent_usd, 7.5);
            assert_eq!(limit_usd, 5.0);
            assert_eq!(reserved_usd, 0.0);
        }
        _ => panic!("Expected BudgetExceeded error"),
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

// =============================================================================
// Reservation: Basic operations
// =============================================================================

#[test]
fn reserve_succeeds_when_budget_available() {
    let controller = CostController::new(10.0);
    let reservation = controller.reserve_usd(5.0);
    assert!(reservation.is_ok());
}

#[test]
fn reserve_updates_reserved_amount() {
    let controller = CostController::new(10.0);
    let _res = controller.reserve_usd(3.0).unwrap();
    assert_eq!(controller.total_reserved(), 3.0);
}

#[test]
fn reserve_reduces_available_budget() {
    let controller = CostController::new(10.0);
    let _res = controller.reserve_usd(3.0).unwrap();
    assert_eq!(controller.available(), 7.0);
}

#[test]
fn reserve_fails_when_insufficient_budget() {
    let controller = CostController::new(5.0);
    let result = controller.reserve_usd(10.0);
    assert!(result.is_err());

    match result.unwrap_err() {
        CostError::InsufficientBudget { available_usd, requested_usd } => {
            assert_eq!(available_usd, 5.0);
            assert_eq!(requested_usd, 10.0);
        }
        _ => panic!("Expected InsufficientBudget error"),
    }
}

#[test]
fn reserve_fails_when_budget_already_spent() {
    let controller = CostController::new(10.0);
    controller.add_spend(8.0);

    let result = controller.reserve_usd(5.0);
    assert!(result.is_err());

    match result.unwrap_err() {
        CostError::InsufficientBudget { available_usd, requested_usd } => {
            assert_eq!(available_usd, 2.0);
            assert_eq!(requested_usd, 5.0);
        }
        _ => panic!("Expected InsufficientBudget error"),
    }
}

#[test]
fn reserve_fails_when_budget_already_reserved() {
    let controller = CostController::new(10.0);
    let _res1 = controller.reserve_usd(8.0).unwrap();

    let result = controller.reserve_usd(5.0);
    assert!(result.is_err());

    match result.unwrap_err() {
        CostError::InsufficientBudget { available_usd, requested_usd } => {
            assert_eq!(available_usd, 2.0);
            assert_eq!(requested_usd, 5.0);
        }
        _ => panic!("Expected InsufficientBudget error"),
    }
}

// =============================================================================
// Reservation: Commit
// =============================================================================

#[test]
fn commit_releases_reservation_and_adds_spend() {
    let controller = CostController::new(10.0);
    let reservation = controller.reserve_usd(5.0).unwrap();

    // Before commit: reserved=5, spent=0
    assert_eq!(controller.total_reserved(), 5.0);
    assert_eq!(controller.total_spent(), 0.0);

    controller.commit_usd(reservation, 3.0);

    // After commit: reserved=0, spent=3
    assert_eq!(controller.total_reserved(), 0.0);
    assert_eq!(controller.total_spent(), 3.0);
}

#[test]
fn commit_allows_actual_cost_less_than_reserved() {
    let controller = CostController::new(10.0);
    let reservation = controller.reserve_usd(5.0).unwrap();

    // Actual cost is less than reserved
    controller.commit_usd(reservation, 1.0);

    assert_eq!(controller.total_spent(), 1.0);
    assert_eq!(controller.available(), 9.0);
}

#[test]
fn commit_allows_actual_cost_equal_to_reserved() {
    let controller = CostController::new(10.0);
    let reservation = controller.reserve_usd(5.0).unwrap();

    controller.commit_usd(reservation, 5.0);

    assert_eq!(controller.total_spent(), 5.0);
    assert_eq!(controller.available(), 5.0);
}

#[test]
fn commit_add_spend_first_then_release_reservation() {
    // This test verifies the safe ordering: spend first, then release
    // During the commit, there's a brief moment where both are counted
    let controller = CostController::new(10.0);
    let reservation = controller.reserve_usd(5.0).unwrap();

    // Simulate what happens during commit:
    // The actual implementation adds spend first, then releases reservation
    // This means available = 10 - 0 - 5 = 5 before commit
    assert_eq!(controller.available(), 5.0);

    controller.commit_usd(reservation, 3.0);

    // After commit: spent=3, reserved=0, available=7
    assert_eq!(controller.available(), 7.0);
}

// =============================================================================
// Reservation: Cancel
// =============================================================================

#[test]
fn cancel_releases_reservation_without_spend() {
    let controller = CostController::new(10.0);
    let reservation = controller.reserve_usd(5.0).unwrap();

    assert_eq!(controller.total_reserved(), 5.0);
    assert_eq!(controller.available(), 5.0);

    controller.cancel(reservation);

    assert_eq!(controller.total_reserved(), 0.0);
    assert_eq!(controller.total_spent(), 0.0);
    assert_eq!(controller.available(), 10.0);
}

#[test]
fn cancel_allows_new_reservations() {
    let controller = CostController::new(10.0);

    // Reserve all budget
    let res1 = controller.reserve_usd(10.0).unwrap();
    assert!(controller.reserve_usd(1.0).is_err());

    // Cancel
    controller.cancel(res1);

    // Now we can reserve again
    let res2 = controller.reserve_usd(10.0);
    assert!(res2.is_ok());
}

// =============================================================================
// Reservation: Multiple concurrent reservations
// =============================================================================

#[test]
fn multiple_reservations_accumulate() {
    let controller = CostController::new(10.0);

    let _res1 = controller.reserve_usd(2.0).unwrap();
    let _res2 = controller.reserve_usd(3.0).unwrap();
    let _res3 = controller.reserve_usd(4.0).unwrap();

    assert_eq!(controller.total_reserved(), 9.0);
    assert_eq!(controller.available(), 1.0);
}

#[test]
fn multiple_reservations_block_overspend() {
    let controller = CostController::new(10.0);

    let _res1 = controller.reserve_usd(3.0).unwrap();
    let _res2 = controller.reserve_usd(3.0).unwrap();
    let _res3 = controller.reserve_usd(3.0).unwrap();

    // Only 1.0 available now
    let result = controller.reserve_usd(2.0);
    assert!(result.is_err());
}

#[test]
fn commit_one_of_multiple_reservations() {
    let controller = CostController::new(10.0);

    let res1 = controller.reserve_usd(3.0).unwrap();
    let _res2 = controller.reserve_usd(3.0).unwrap();

    // reserved=6, available=4
    assert_eq!(controller.total_reserved(), 6.0);

    // Commit res1 with actual cost of 2.0
    controller.commit_usd(res1, 2.0);

    // Now: spent=2, reserved=3, available=5
    assert_eq!(controller.total_spent(), 2.0);
    assert_eq!(controller.total_reserved(), 3.0);
    assert_eq!(controller.available(), 5.0);
}

// =============================================================================
// Reservation: Check budget considers reserved
// =============================================================================

#[test]
fn check_budget_fails_when_spent_plus_reserved_at_limit() {
    let controller = CostController::new(10.0);
    controller.add_spend(5.0);
    let _res = controller.reserve_usd(5.0).unwrap();

    // spent=5, reserved=5, limit=10 -> at limit
    assert!(controller.check_budget().is_err());
}

#[test]
fn check_budget_includes_reserved_in_error() {
    let controller = CostController::new(10.0);
    let _res = controller.reserve_usd(7.0).unwrap();

    // Now add spend that pushes us over: spent=4, reserved=7, limit=10 -> over limit
    controller.add_spend(4.0);

    let err = controller.check_budget().unwrap_err();
    match err {
        CostError::BudgetExceeded { spent_usd, limit_usd, reserved_usd } => {
            assert_eq!(spent_usd, 4.0);
            assert_eq!(reserved_usd, 7.0);
            assert_eq!(limit_usd, 10.0);
        }
        _ => panic!("Expected BudgetExceeded error"),
    }
}

// =============================================================================
// Reservation: get_full_status
// =============================================================================

#[test]
fn get_full_status_returns_all_values() {
    let controller = CostController::new(10.0);
    controller.add_spend(2.0);
    let _res = controller.reserve_usd(3.0).unwrap();

    let (spent, reserved, limit) = controller.get_full_status();
    assert_eq!(spent, 2.0);
    assert_eq!(reserved, 3.0);
    assert_eq!(limit, 10.0);
}

// =============================================================================
// Reservation: Microdollar precision
// =============================================================================

#[test]
fn reserve_with_microdollar_precision() {
    let controller = CostController::new(0.000100); // 100 microdollars

    let res = controller.reserve(50).unwrap(); // 50 microdollars
    assert_eq!(res.amount_micros(), 50);
    assert_eq!(controller.total_reserved(), 0.000050);

    controller.commit(res, 30); // actual: 30 microdollars

    assert_eq!(controller.total_spent(), 0.000030);
    assert_eq!(controller.available(), 0.000070);
}

// =============================================================================
// Reservation: Edge cases
// =============================================================================

#[test]
fn reserve_zero_succeeds() {
    let controller = CostController::new(10.0);
    let res = controller.reserve(0);
    assert!(res.is_ok());
    assert_eq!(controller.total_reserved(), 0.0);
}

#[test]
fn commit_zero_actual_cost() {
    let controller = CostController::new(10.0);
    let res = controller.reserve_usd(5.0).unwrap();

    controller.commit_usd(res, 0.0);

    assert_eq!(controller.total_spent(), 0.0);
    assert_eq!(controller.total_reserved(), 0.0);
}

#[test]
fn reserve_exact_available_amount() {
    let controller = CostController::new(10.0);
    controller.add_spend(3.0);

    // Exact available = 7.0
    let res = controller.reserve_usd(7.0);
    assert!(res.is_ok());
    assert_eq!(controller.available(), 0.0);
}

#[test]
fn reserve_one_more_than_available_fails() {
    let controller = CostController::new(10.0);
    controller.add_spend(3.0);

    // One microdollar more than available
    let result = controller.reserve(7_000_001);
    assert!(result.is_err());
}
