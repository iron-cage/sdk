use iron_cost::budget::CostController;
use iron_cost::error::CostError;

// =============================================================================
// Creation and initialization
// =============================================================================

#[test]
fn new_controller_has_zero_spent() {
    let controller = CostController::new(10_000_000); // 10 USD
    let (spent, limit) = controller.get_status();
    assert_eq!(spent, 0, "spent should start at zero");
    assert_eq!(limit, 10_000_000, "limit should match constructor value");
}

#[test]
fn new_controller_with_zero_budget() {
    let controller = CostController::new(0);
    let (spent, limit) = controller.get_status();
    assert_eq!(spent, 0, "spent should start at zero");
    assert_eq!(limit, 0, "limit should be zero when constructed with zero budget");
}

#[test]
fn new_controller_has_zero_reserved() {
    let controller = CostController::new(10_000_000); // 10 USD
    assert_eq!(controller.total_reserved(), 0, "reserved should start at zero");
}

// =============================================================================
// Budget checking
// =============================================================================

#[test]
fn check_budget_passes_when_under_limit() {
    let controller = CostController::new(10_000_000); // 10 USD
    assert!(controller.check_budget().is_ok(), "budget check should pass when under limit");
}

#[test]
fn check_budget_fails_when_at_limit() {
    let controller = CostController::new(10_000_000); // 10 USD
    controller.add_spend(10_000_000); // 10 USD
    assert!(controller.check_budget().is_err(), "budget check should fail exactly at limit");
}

#[test]
fn check_budget_fails_when_over_limit() {
    let controller = CostController::new(10_000_000); // 10 USD
    controller.add_spend(15_000_000); // 15 USD
    assert!(controller.check_budget().is_err(), "budget check should fail when over limit");
}

#[test]
fn check_budget_returns_correct_error_values() {
    let controller = CostController::new(5_000_000); // 5 USD
    controller.add_spend(7_500_000); // 7.5 USD

    let err = controller.check_budget().unwrap_err();
    match err {
        CostError::BudgetExceeded { spent_microdollars, limit_microdollars, reserved_microdollars } => {
            assert_eq!(spent_microdollars, 7_500_000, "error should report spent value");
            assert_eq!(limit_microdollars, 5_000_000, "error should report limit value");
            assert_eq!(reserved_microdollars, 0, "error should report reserved value");
        }
        _ => panic!("Expected BudgetExceeded error"),
    }
}

#[test]
fn zero_budget_fails_immediately() {
    let controller = CostController::new(0);
    assert!(controller.check_budget().is_err(), "zero budget should immediately fail");
}

// =============================================================================
// Spending
// =============================================================================

#[test]
fn add_spend_accumulates() {
    let controller = CostController::new(100_000_000); // 100 USD
    controller.add_spend(1_000_000); // 1 USD
    controller.add_spend(2_000_000); // 2 USD
    controller.add_spend(3_000_000); // 3 USD

    let (spent, _) = controller.get_status();
    assert_eq!(spent, 6_000_000, "spend should accumulate across calls");
}

#[test]
fn add_spend_with_small_amounts() {
    let controller = CostController::new(1_000_000); // 1 USD
    controller.add_spend(1); // 1 microdollar
    controller.add_spend(1); // 1 microdollar
    controller.add_spend(1); // 1 microdollar

    let (spent, _) = controller.get_status();
    assert_eq!(spent, 3, "microdollar accumulation should be precise");
}

// =============================================================================
// Budget modification
// =============================================================================

#[test]
fn set_budget_updates_limit() {
    let controller = CostController::new(10_000_000); // 10 USD
    controller.set_budget(20_000_000); // 20 USD

    let (_, limit) = controller.get_status();
    assert_eq!(limit, 20_000_000, "limit should update after set_budget");
}

#[test]
fn set_budget_can_unblock_exceeded_budget() {
    let controller = CostController::new(5_000_000); // 5 USD
    controller.add_spend(7_000_000); // 7 USD

    // Budget exceeded
    assert!(controller.check_budget().is_err(), "budget should be exceeded after overspend");

    // Increase budget
    controller.set_budget(10_000_000); // 10 USD

    // Now it should pass
    assert!(controller.check_budget().is_ok(), "budget should pass after raising limit");
}

#[test]
fn set_budget_can_block_previously_valid_budget() {
    let controller = CostController::new(10_000_000); // 10 USD
    controller.add_spend(5_000_000); // 5 USD

    // Budget is fine
    assert!(controller.check_budget().is_ok(), "budget should pass before reduction");

    // Reduce budget below spent
    controller.set_budget(3_000_000); // 3 USD

    // Now it should fail
    assert!(controller.check_budget().is_err(), "budget should fail after reducing limit below spend");
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn handles_large_amounts() {
    let controller = CostController::new(1_000_000_000_000); // 1 million USD
    controller.add_spend(500_000_000_000); // 500k USD

    let (spent, limit) = controller.get_status();
    assert_eq!(spent, 500_000_000_000, "spent should reflect large additions");
    assert_eq!(limit, 1_000_000_000_000, "limit should retain large value");
    assert!(controller.check_budget().is_ok(), "budget should pass when under large limit");
}

#[test]
fn precision_at_microdollar_level() {
    let controller = CostController::new(10); // 10 microdollars
    controller.add_spend(5); // 5 microdollars

    let (spent, limit) = controller.get_status();
    assert_eq!(spent, 5, "spent should track microdollars precisely");
    assert_eq!(limit, 10, "limit should retain microdollar precision");
    assert!(controller.check_budget().is_ok(), "budget should allow spending below microdollar limit");

    controller.add_spend(5); // Now at exactly 10 microdollars
    assert!(controller.check_budget().is_err(), "budget should fail once at microdollar limit"); // At limit = exceeded
}

// =============================================================================
// Reservation: Basic operations
// =============================================================================

#[test]
fn reserve_succeeds_when_budget_available() {
    let controller = CostController::new(10_000_000); // 10 USD
    let reservation = controller.reserve(5_000_000); // 5 USD
    assert!(reservation.is_ok(), "reservation should succeed when budget available");
}

#[test]
fn reserve_updates_reserved_amount() {
    let controller = CostController::new(10_000_000); // 10 USD
    let _res = controller.reserve(3_000_000).unwrap(); // 3 USD
    assert_eq!(controller.total_reserved(), 3_000_000, "reserved total should update after reservation");
}

#[test]
fn reserve_reduces_available_budget() {
    let controller = CostController::new(10_000_000); // 10 USD
    let _res = controller.reserve(3_000_000).unwrap(); // 3 USD
    assert_eq!(controller.available(), 7_000_000, "available should reduce after reservation");
}

#[test]
fn reserve_fails_when_insufficient_budget() {
    let controller = CostController::new(5_000_000); // 5 USD
    let result = controller.reserve(10_000_000); // 10 USD
    assert!(result.is_err(), "reservation should fail when request exceeds available");

    match result.unwrap_err() {
        CostError::InsufficientBudget { available_microdollars, requested_microdollars } => {
            assert_eq!(available_microdollars, 5_000_000, "error should report available amount");
            assert_eq!(requested_microdollars, 10_000_000, "error should report requested amount");
        }
        _ => panic!("Expected InsufficientBudget error"),
    }
}

#[test]
fn reserve_fails_when_budget_already_spent() {
    let controller = CostController::new(10_000_000); // 10 USD
    controller.add_spend(8_000_000); // 8 USD

    let result = controller.reserve(5_000_000); // 5 USD
    assert!(result.is_err(), "reservation should fail when budget already spent");

    match result.unwrap_err() {
        CostError::InsufficientBudget { available_microdollars, requested_microdollars } => {
            assert_eq!(available_microdollars, 2_000_000, "error should report remaining available amount");
            assert_eq!(requested_microdollars, 5_000_000, "error should report requested amount");
        }
        _ => panic!("Expected InsufficientBudget error"),
    }
}

#[test]
fn reserve_fails_when_budget_already_reserved() {
    let controller = CostController::new(10_000_000); // 10 USD
    let _res1 = controller.reserve(8_000_000).unwrap(); // 8 USD

    let result = controller.reserve(5_000_000); // 5 USD
    assert!(result.is_err(), "reservation should fail when budget already reserved");

    match result.unwrap_err() {
        CostError::InsufficientBudget { available_microdollars, requested_microdollars } => {
            assert_eq!(available_microdollars, 2_000_000, "error should report available after reservation");
            assert_eq!(requested_microdollars, 5_000_000, "error should report requested amount");
        }
        _ => panic!("Expected InsufficientBudget error"),
    }
}

// =============================================================================
// Reservation: Commit
// =============================================================================

#[test]
fn commit_releases_reservation_and_adds_spend() {
    let controller = CostController::new(10_000_000); // 10 USD
    let reservation = controller.reserve(5_000_000).unwrap(); // 5 USD

    // Before commit: reserved=5, spent=0
    assert_eq!(controller.total_reserved(), 5_000_000, "reservation should hold reserved amount before commit");
    assert_eq!(controller.total_spent(), 0, "spend should be zero before commit");

    controller.commit(reservation, 3_000_000); // 3 USD

    // After commit: reserved=0, spent=3
    assert_eq!(controller.total_reserved(), 0, "reservation should be cleared after commit");
    assert_eq!(controller.total_spent(), 3_000_000, "spent should reflect committed amount");
}

#[test]
fn commit_allows_actual_cost_less_than_reserved() {
    let controller = CostController::new(10_000_000); // 10 USD
    let reservation = controller.reserve(5_000_000).unwrap(); // 5 USD

    // Actual cost is less than reserved
    controller.commit(reservation, 1_000_000); // 1 USD

    assert_eq!(controller.total_spent(), 1_000_000, "spent should reflect actual cost after commit");
    assert_eq!(controller.available(), 9_000_000, "available should reflect returned reservation");
}

#[test]
fn commit_allows_actual_cost_equal_to_reserved() {
    let controller = CostController::new(10_000_000); // 10 USD
    let reservation = controller.reserve(5_000_000).unwrap(); // 5 USD

    controller.commit(reservation, 5_000_000); // 5 USD

    assert_eq!(controller.total_spent(), 5_000_000, "spent should match reserved when equal");
    assert_eq!(controller.available(), 5_000_000, "available should reduce after equal commit");
}

#[test]
fn commit_add_spend_first_then_release_reservation() {
    // This test verifies the safe ordering: spend first, then release
    // During the commit, there's a brief moment where both are counted
    let controller = CostController::new(10_000_000); // 10 USD
    let reservation = controller.reserve(5_000_000).unwrap(); // 5 USD

    // Simulate what happens during commit:
    // The actual implementation adds spend first, then releases reservation
    // This means available = 10 - 0 - 5 = 5 before commit
    assert_eq!(controller.available(), 5_000_000, "available should reflect reserved before commit");

    controller.commit(reservation, 3_000_000); // 3 USD

    // After commit: spent=3, reserved=0, available=7
    assert_eq!(controller.available(), 7_000_000, "available should increase after releasing reservation");
}

// =============================================================================
// Reservation: Cancel
// =============================================================================

#[test]
fn cancel_releases_reservation_without_spend() {
    let controller = CostController::new(10_000_000); // 10 USD
    let reservation = controller.reserve(5_000_000).unwrap(); // 5 USD

    assert_eq!(controller.total_reserved(), 5_000_000, "reserved should reflect held amount");
    assert_eq!(controller.available(), 5_000_000, "available should be reduced by reservation");

    controller.cancel(reservation);

    assert_eq!(controller.total_reserved(), 0, "cancel should clear reserved amount");
    assert_eq!(controller.total_spent(), 0, "cancel should not add spend");
    assert_eq!(controller.available(), 10_000_000, "available should return to full after cancel");
}

#[test]
fn cancel_allows_new_reservations() {
    let controller = CostController::new(10_000_000); // 10 USD

    // Reserve all budget
    let res1 = controller.reserve(10_000_000).unwrap(); // 10 USD
    assert!(controller.reserve(1_000_000).is_err(), "cannot reserve beyond full allocation");

    // Cancel
    controller.cancel(res1);

    // Now we can reserve again
    let res2 = controller.reserve(10_000_000); // 10 USD
    assert!(res2.is_ok(), "reservation should succeed after cancel");
}

// =============================================================================
// Reservation: Multiple concurrent reservations
// =============================================================================

#[test]
fn multiple_reservations_accumulate() {
    let controller = CostController::new(10_000_000); // 10 USD

    let _res1 = controller.reserve(2_000_000).unwrap(); // 2 USD
    let _res2 = controller.reserve(3_000_000).unwrap(); // 3 USD
    let _res3 = controller.reserve(4_000_000).unwrap(); // 4 USD

    assert_eq!(controller.total_reserved(), 9_000_000, "total reserved should sum all reservations");
    assert_eq!(controller.available(), 1_000_000, "available should reflect remaining budget");
}

#[test]
fn multiple_reservations_block_overspend() {
    let controller = CostController::new(10_000_000); // 10 USD

    let _res1 = controller.reserve(3_000_000).unwrap(); // 3 USD
    let _res2 = controller.reserve(3_000_000).unwrap(); // 3 USD
    let _res3 = controller.reserve(3_000_000).unwrap(); // 3 USD

    // Only 1.0 available now
    let result = controller.reserve(2_000_000); // 2 USD
    assert!(result.is_err(), "should not reserve beyond remaining budget");
}

#[test]
fn commit_one_of_multiple_reservations() {
    let controller = CostController::new(10_000_000); // 10 USD

    let res1 = controller.reserve(3_000_000).unwrap(); // 3 USD
    let _res2 = controller.reserve(3_000_000).unwrap(); // 3 USD

    // reserved=6, available=4
    assert_eq!(controller.total_reserved(), 6_000_000, "reserved should include both reservations");

    // Commit res1 with actual cost of 2.0
    controller.commit(res1, 2_000_000); // 2 USD

    // Now: spent=2, reserved=3, available=5
    assert_eq!(controller.total_spent(), 2_000_000, "spent should reflect committed amount");
    assert_eq!(controller.total_reserved(), 3_000_000, "remaining reservation should persist");
    assert_eq!(controller.available(), 5_000_000, "available should reflect updated totals");
}

// =============================================================================
// Reservation: Check budget considers reserved
// =============================================================================

#[test]
fn check_budget_fails_when_spent_plus_reserved_at_limit() {
    let controller = CostController::new(10_000_000); // 10 USD
    controller.add_spend(5_000_000); // 5 USD
    let _res = controller.reserve(5_000_000).unwrap(); // 5 USD

    // spent=5, reserved=5, limit=10 -> at limit
    assert!(controller.check_budget().is_err(), "budget check should fail when spent + reserved at limit");
}

#[test]
fn check_budget_includes_reserved_in_error() {
    let controller = CostController::new(10_000_000); // 10 USD
    let _res = controller.reserve(7_000_000).unwrap(); // 7 USD

    // Now add spend that pushes us over: spent=4, reserved=7, limit=10 -> over limit
    controller.add_spend(4_000_000); // 4 USD

    let err = controller.check_budget().unwrap_err();
    match err {
        CostError::BudgetExceeded { spent_microdollars, limit_microdollars, reserved_microdollars } => {
            assert_eq!(spent_microdollars, 4_000_000, "spent should be reported in error");
            assert_eq!(reserved_microdollars, 7_000_000, "reserved should be reported in error");
            assert_eq!(limit_microdollars, 10_000_000, "limit should be reported in error");
        }
        _ => panic!("Expected BudgetExceeded error"),
    }
}

// =============================================================================
// Reservation: get_full_status
// =============================================================================

#[test]
fn get_full_status_returns_all_values() {
    let controller = CostController::new(10_000_000); // 10 USD
    controller.add_spend(2_000_000); // 2 USD
    let _res = controller.reserve(3_000_000).unwrap(); // 3 USD

    let (spent, reserved, limit) = controller.get_full_status();
    assert_eq!(spent, 2_000_000, "full status should include spent");
    assert_eq!(reserved, 3_000_000, "full status should include reserved");
    assert_eq!(limit, 10_000_000, "full status should include limit");
}

// =============================================================================
// Reservation: Microdollar precision
// =============================================================================

#[test]
fn reserve_with_microdollar_precision() {
    let controller = CostController::new(100); // 100 microdollars

    let res = controller.reserve(50).unwrap(); // 50 microdollars
    assert_eq!(res.amount(), 50, "reservation should store exact micro amount");
    assert_eq!(controller.total_reserved(), 50, "reserved total should reflect micro amount");

    controller.commit(res, 30); // actual: 30 microdollars

    assert_eq!(controller.total_spent(), 30, "spent should track micro commit");
    assert_eq!(controller.available(), 70, "available should update after micro commit");
}

// =============================================================================
// Reservation: Edge cases
// =============================================================================

#[test]
fn reserve_zero_succeeds() {
    let controller = CostController::new(10_000_000); // 10 USD
    let res = controller.reserve(0);
    assert!(res.is_ok(), "zero reservation should succeed");
    assert_eq!(controller.total_reserved(), 0, "zero reservation should not change totals");
}

#[test]
fn commit_zero_actual_cost() {
    let controller = CostController::new(10_000_000); // 10 USD
    let res = controller.reserve(5_000_000).unwrap(); // 5 USD

    controller.commit(res, 0);

    assert_eq!(controller.total_spent(), 0, "zero actual cost should not increase spend");
    assert_eq!(controller.total_reserved(), 0, "reservation should be cleared after zero-cost commit");
}

#[test]
fn reserve_exact_available_amount() {
    let controller = CostController::new(10_000_000); // 10 USD
    controller.add_spend(3_000_000); // 3 USD

    // Exact available = 7.0
    let res = controller.reserve(7_000_000); // 7 USD
    assert!(res.is_ok(), "reservation should succeed at exact available amount");
    assert_eq!(controller.available(), 0, "available should be zero after exact reservation");
}

#[test]
fn reserve_one_more_than_available_fails() {
    let controller = CostController::new(10_000_000); // 10 USD
    controller.add_spend(3_000_000); // 3 USD

    // One microdollar more than available
    let result = controller.reserve(7_000_001);
    assert!(result.is_err(), "reservation exceeding available by microdollar should fail");
}
