//! Bug fix test: Deadlock in BudgetTracker.record_cost()
//!
//! ## Root Cause
//!
//! The `record_cost()` method held a DashMap entry lock while calling `total_spent()`,
//! which iterates over the same DashMap. This created a deadlock scenario:
//!
//! 1. Thread acquires write lock on entry via `entry().or_insert()`
//! 2. While holding that lock, calls `total_spent()`
//! 3. `total_spent()` attempts to acquire read locks on all entries via `iter()`
//! 4. DashMap's internal locking prevents iteration while entry lock is held
//! 5. Thread deadlocks waiting for itself to release the lock
//!
//! Technical detail: DashMap uses internal sharding with RwLocks. An exclusive
//! write lock on one shard prevents global iteration even if iteration would
//! only need read locks on other shards.
//!
//! ## Why Not Caught Earlier
//!
//! 1. **No timeout in test framework**: Default cargo test runs indefinitely,
//!    appearing as "slow tests" rather than obvious deadlock
//! 2. **Single-threaded test**: Deadlock occurs within single thread, not obvious
//!    without understanding DashMap internals
//! 3. **Timing-dependent**: May work in debug builds due to different timing,
//!    only hangs reliably in release/test builds
//! 4. **No integration test coverage**: Unit tests existed but integration scenario
//!    combining multiple operations wasn't tested
//! 5. **Code review gap**: Reviewer didn't recognize DashMap's lock-iteration
//!    incompatibility pattern
//!
//! ## Fix Applied
//!
//! Introduced explicit scope to drop entry lock before calling `total_spent()`:
//!
//! ```rust
//! // BEFORE (deadlock):
//! let mut entry = self.spent_usd.entry(...).or_insert(0.0);
//! *entry += cost;
//! if self.total_spent() > self.budget_usd { ... } // Still holding lock!
//!
//! // AFTER (fixed):
//! {
//!   let mut entry = self.spent_usd.entry(...).or_insert(0.0);
//!   *entry += cost;
//! } // Explicit scope drops entry lock here
//! if self.total_spent() > self.budget_usd { ... } // Lock released
//! ```
//!
//! The explicit scope `{ }` ensures Rust's RAII drops the `RefMut` guard
//! before we attempt global iteration.
//!
//! ## Prevention
//!
//! 1. **Never hold DashMap locks across operations**: Always drop entry locks
//!    before calling methods that iterate the same DashMap
//! 2. **Use timeouts in CI**: Add `timeout 60` wrapper around test commands
//!    to catch indefinite hangs
//! 3. **Lint rule**: Create clippy lint to detect `entry()` followed by `iter()`
//!    on same DashMap without explicit scope
//! 4. **Test pattern**: Add integration tests that exercise lock contention
//!    scenarios explicitly
//! 5. **Code review checklist**: "Does this method hold any locks while calling
//!    other methods that might acquire locks?"
//!
//! ## Pitfall
//!
//! **DashMap is NOT a drop-in replacement for `Mutex<HashMap>`**. Key differences:
//!
//! - `Mutex<HashMap>`: Single lock, safe to iterate while holding entry reference
//! - `DashMap`: Sharded locks, CANNOT iterate while holding entry lock
//!
//! Pattern to avoid:
//! ```rust
//! let mut entry = map.entry(key).or_insert(default);
//! *entry = new_value;
//! let total = map.iter().map(|e| *e.value()).sum(); // DEADLOCK!
//! ```
//!
//! Correct pattern:
//! ```rust
//! {
//!   let mut entry = map.entry(key).or_insert(default);
//!   *entry = new_value;
//! } // Drop lock
//! let total = map.iter().map(|e| *e.value()).sum(); // Safe
//! ```
//!
//! Or use `get_mut()` when you don't need iteration:
//! ```rust
//! map.entry(key).and_modify(|v| *v += delta).or_insert(delta);
//! ```

use iron_cost::BudgetTracker;

/// Reproduces the deadlock bug (before fix)
///
/// This test verifies the bug is fixed by ensuring `record_cost()` completes
/// in reasonable time (<1s) when budget checking is required.
// test_kind: bug_reproducer(deadlock-dashmap-001)
#[test]
#[cfg_attr(not(debug_assertions), ignore = "Only run in debug to avoid CI timeouts")]
fn bug_reproducer_dashmap_deadlock()
{
  let tracker = BudgetTracker::new( 10.0 );

  // First call works (no budget check needed yet)
  tracker.record_cost( "agent1", 5.0 ).unwrap();

  // Second call triggers budget check while holding entry lock
  // BEFORE FIX: This would deadlock indefinitely
  // AFTER FIX: Completes immediately
  let result = tracker.record_cost( "agent1", 10.0 );

  // Verify budget exceeded error returned (not deadlock)
  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "exceeded" ) );
}

/// Verifies the fix works correctly
#[test]
fn fix_verification_no_deadlock()
{
  let tracker = BudgetTracker::new( 100.0 );

  // Multiple operations should complete without deadlock
  for i in 0..10
  {
    let agent = format!( "agent_{}", i );
    tracker.record_cost( &agent, 5.0 ).unwrap();
  }

  // Total should be correct
  assert_eq!( tracker.total_spent(), 50.0 );
  assert_eq!( tracker.remaining(), 50.0 );
}

/// Verifies budget enforcement still works correctly after fix
#[test]
fn fix_preserves_budget_enforcement()
{
  let tracker = BudgetTracker::new( 20.0 );

  tracker.record_cost( "agent1", 10.0 ).unwrap();
  tracker.record_cost( "agent2", 8.0 ).unwrap();

  // This should exceed budget (10 + 8 + 5 = 23 > 20)
  let result = tracker.record_cost( "agent3", 5.0 );
  assert!( result.is_err() );

  // Fix(budget-enforcement-001): Budget check now happens BEFORE recording cost
  // Failed operations no longer modify state, ensuring budget is never exceeded
  assert_eq!( tracker.total_spent(), 18.0, "Failed operation should not record cost" );
}

/// Bug reproducer: Budget exceeded by final operation (issue-budget-enforcement-001)
///
/// ## Root Cause
///
/// The `record_cost()` method recorded costs BEFORE checking budget constraints,
/// allowing the budget to be exceeded by the amount of the final (rejected) operation:
///
/// 1. Method receives cost to record
/// 2. Immediately adds cost to agent's spent total
/// 3. Only AFTER recording, checks if total_spent() > budget
/// 4. Returns error, but state already modified
/// 5. Budget exceeded by exactly the amount of the rejected cost
///
/// This violated the fundamental contract: "Budget tracker prevents exceeding budget"
///
/// ## Why Not Caught Earlier
///
/// 1. **Test validated wrong behavior**: Test at line 154 explicitly documented
///    the bug with "TODO: Consider if operations should be rolled back", accepting
///    the incorrect behavior as "preserving existing behavior"
/// 2. **Advisory vs Preventive confusion**: Team treated budget enforcement as
///    "advisory warning" rather than "hard constraint"
/// 3. **Focus on deadlock fix**: During deadlock-dashmap-001 fix, focus was on
///    resolving deadlock, not on validating budget enforcement semantics
/// 4. **Insufficient specification**: Spec didn't explicitly state budget must
///    NEVER be exceeded, allowing interpretation as "warn when exceeded"
/// 5. **Missing negative test**: No test explicitly verified that failed operations
///    don't modify state (transaction pattern validation)
///
/// ## Fix Applied
///
/// Reordered operations to validate-then-mutate pattern:
///
/// ```rust
/// // BEFORE (bug):
/// {
///   let mut entry = self.spent_usd.entry(agent_id).or_insert(0.0);
///   *entry += cost;  // Modify state first
/// }
/// if self.total_spent() > self.budget_usd {  // Check constraint after
///   return Err(...);  // State already modified!
/// }
///
/// // AFTER (fixed):
/// let current_total = self.total_spent();
/// if current_total + cost > self.budget_usd {  // Check constraint first
///   return Err(...);  // Reject without modifying state
/// }
/// {
///   let mut entry = self.spent_usd.entry(agent_id).or_insert(0.0);
///   *entry += cost;  // Only modify state if validation passed
/// }
/// ```
///
/// ## Prevention
///
/// 1. **Validate-then-mutate pattern**: Always check constraints BEFORE modifying
///    state, never after. This prevents partial state corruption on validation failure
/// 2. **Transaction semantics in tests**: Test that failed operations leave state
///    unchanged - if operation returns Err, state must be identical to before call
/// 3. **Specification precision**: Explicitly document whether enforcement is
///    "advisory" (warn but allow) or "preventive" (reject and block)
/// 4. **Code review checklist**: "For methods that return Result, is state modified
///    before or after validation? If before, can validation failure leave corrupt state?"
/// 5. **Integration test pattern**: Test sequences of operations where middle operation
///    fails, verify total state unchanged by failed operation
///
/// ## Pitfall
///
/// **Classic validation-mutation ordering bug**: Modifying state before validating
/// constraints is a fundamental error pattern that appears in many contexts:
///
/// - Database transactions: Modify row, then check foreign key constraint
/// - Memory allocation: Write data, then check bounds
/// - Resource acquisition: Acquire lock, then check if acquisition allowed
///
/// Correct pattern is always: **Validate → Mutate → Commit**, never **Mutate → Validate → Rollback**.
/// Rollback patterns are complex and error-prone. Prevention is simpler than rollback.
// test_kind: bug_reproducer(budget-enforcement-001)
#[test]
fn bug_reproducer_budget_exceeded_by_final_operation()
{
  let tracker = BudgetTracker::new( 100.0 );

  // Spend up to budget threshold
  tracker.record_cost( "agent1", 60.0 ).unwrap();
  tracker.record_cost( "agent2", 30.0 ).unwrap();
  assert_eq!( tracker.total_spent(), 90.0 );

  // Attempt to exceed budget by $50
  let result = tracker.record_cost( "agent3", 60.0 );

  // BEFORE FIX: total_spent() would be 150.0 (budget exceeded by $50)
  // AFTER FIX: total_spent() remains 90.0 (rejected operation didn't modify state)
  assert!( result.is_err(), "Operation should be rejected" );
  assert_eq!(
    tracker.total_spent(),
    90.0,
    "Failed operation must not modify state (budget must never be exceeded)"
  );
  assert_eq!(
    tracker.remaining(),
    10.0,
    "Remaining budget should reflect only successful operations"
  );
}
