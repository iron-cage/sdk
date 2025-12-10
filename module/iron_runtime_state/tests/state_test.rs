use iron_runtime_state::*;

#[test]
fn test_state_manager_creation()
{
  let manager = StateManager::new();
  assert_eq!(manager.list_agents().len(), 0);
}

#[test]
fn test_save_and_get_agent_state()
{
  let manager = StateManager::new();

  let state = AgentState {
    agent_id: "test-agent-123".to_string(),
    status: AgentStatus::Running,
    budget_spent: 10.5,
    pii_detections: 3,
  };

  manager.save_agent_state(state.clone());

  let retrieved = manager.get_agent_state("test-agent-123");
  assert!(retrieved.is_some());

  let retrieved = retrieved.unwrap();
  assert_eq!(retrieved.agent_id, "test-agent-123");
  assert_eq!(retrieved.budget_spent, 10.5);
  assert_eq!(retrieved.pii_detections, 3);
}

#[test]
fn test_list_agents()
{
  let manager = StateManager::new();

  manager.save_agent_state(AgentState {
    agent_id: "agent-1".to_string(),
    status: AgentStatus::Running,
    budget_spent: 5.0,
    pii_detections: 1,
  });

  manager.save_agent_state(AgentState {
    agent_id: "agent-2".to_string(),
    status: AgentStatus::Stopped,
    budget_spent: 15.0,
    pii_detections: 0,
  });

  let agents = manager.list_agents();
  assert_eq!(agents.len(), 2);
  assert!(agents.contains(&"agent-1".to_string()));
  assert!(agents.contains(&"agent-2".to_string()));
}

#[test]
fn test_audit_log()
{
  let manager = StateManager::new();

  let event = AuditEvent {
    agent_id: "test-agent".to_string(),
    event_type: "pii_detected".to_string(),
    timestamp: 1234567890,
    details: "Email found in output".to_string(),
  };

  // Should not panic
  manager.save_audit_log(event);
}
