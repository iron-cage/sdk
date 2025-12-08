//! Test that readme.md examples compile and work correctly

use iron_cost::BudgetTracker;

#[test]
fn readme_example_compiles()
{
  // Example from readme.md - verify it compiles and works
  let tracker = BudgetTracker::new(100.0);

  // Record agent API call cost
  tracker.record_cost("agent_1", 2.50).unwrap();

  // Check remaining budget
  assert_eq!(tracker.remaining(), 97.50);

  // Tracker automatically enforces budget limits
  let result = tracker.record_cost("agent_2", 150.0);
  assert!(result.is_err());
}

#[test]
fn budget_tracking_accuracy()
{
  let tracker = BudgetTracker::new(50.0);

  tracker.record_cost("agent_1", 10.0).unwrap();
  assert_eq!(tracker.total_spent(), 10.0);
  assert_eq!(tracker.remaining(), 40.0);

  tracker.record_cost("agent_2", 20.0).unwrap();
  assert_eq!(tracker.total_spent(), 30.0);
  assert_eq!(tracker.remaining(), 20.0);

  tracker.record_cost("agent_1", 15.0).unwrap();
  assert_eq!(tracker.total_spent(), 45.0);
  assert_eq!(tracker.remaining(), 5.0);
}

#[test]
fn budget_enforcement()
{
  let tracker = BudgetTracker::new(10.0);

  tracker.record_cost("agent_1", 5.0).unwrap();

  // This should exceed budget
  let result = tracker.record_cost("agent_1", 10.0);
  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("exceeded"));
}
