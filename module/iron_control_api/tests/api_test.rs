//! API server creation and state access tests
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_api_server_creation` | Create API server instance | StateManager + port 3000 | Server created successfully | ✅ |
//! | `test_api_state_access` | Access agent state through StateManager | Create StateManager, add agent state, query it | Agent state retrievable | ✅ |

use iron_control_api::*;
use iron_runtime_state::StateManager;
use iron_types::AgentId;
use std::sync::Arc;

#[test]
fn test_api_server_creation()
{
  let state_manager = Arc::new(StateManager::new());
  let server = ApiServer::new(state_manager, 3000);

  // Just verify it compiles and creates
  drop(server);
}

#[tokio::test]
async fn test_api_state_access()
{
  let state_manager = Arc::new(StateManager::new());
  let test_agent_id = AgentId::parse("agent_00000000-0000-0000-0000-000000000123").unwrap();

  // Add test agent
  state_manager.save_agent_state(iron_runtime_state::AgentState {
    agent_id: test_agent_id.clone(),
    status: iron_runtime_state::AgentStatus::Running,
    budget_spent: 10.0,
    pii_detections: 2,
  });

  // Verify state accessible
  let state = state_manager.get_agent_state(test_agent_id.as_str());
  assert!(state.is_some());

  let state = state.unwrap();
  assert_eq!(state.agent_id, test_agent_id);
  assert_eq!(state.budget_spent, 10.0);
}
