use iron_runtime_state::*;
use iron_types::AgentId;

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

  let agent_id = AgentId::parse("agent_550e8400-e29b-41d4-a716-446655440000").unwrap();

  let state = AgentState {
    agent_id: agent_id.clone(),
    status: AgentStatus::Running,
    budget_spent: 10.5,
    pii_detections: 3,
  };

  manager.save_agent_state(state.clone());

  let retrieved = manager.get_agent_state(agent_id.as_str());
  assert!(retrieved.is_some());

  let retrieved = retrieved.unwrap();
  assert_eq!(retrieved.agent_id, agent_id);
  assert_eq!(retrieved.budget_spent, 10.5);
  assert_eq!(retrieved.pii_detections, 3);
}

#[test]
fn test_list_agents()
{
  let manager = StateManager::new();

  let agent_id_1 = AgentId::parse("agent_550e8400-e29b-41d4-a716-446655440001").unwrap();
  let agent_id_2 = AgentId::parse("agent_550e8400-e29b-41d4-a716-446655440002").unwrap();

  manager.save_agent_state(AgentState {
    agent_id: agent_id_1.clone(),
    status: AgentStatus::Running,
    budget_spent: 5.0,
    pii_detections: 1,
  });

  manager.save_agent_state(AgentState {
    agent_id: agent_id_2.clone(),
    status: AgentStatus::Stopped,
    budget_spent: 15.0,
    pii_detections: 0,
  });

  let agents = manager.list_agents();
  assert_eq!(agents.len(), 2);
  assert!(agents.contains(&agent_id_1.as_str().to_string()));
  assert!(agents.contains(&agent_id_2.as_str().to_string()));
}

#[test]
fn test_audit_log()
{
  let manager = StateManager::new();

  let agent_id = AgentId::parse("agent_550e8400-e29b-41d4-a716-446655440000").unwrap();

  let event = AuditEvent {
    agent_id,
    event_type: "pii_detected".to_string(),
    timestamp: 1234567890,
    details: "Email found in output".to_string(),
  };

  // Should not panic
  manager.save_audit_log(event);
}
