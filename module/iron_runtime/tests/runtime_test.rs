use iron_runtime::*;
use std::path::Path;

#[tokio::test]
async fn test_runtime_creation()
{
  let config = RuntimeConfig {
    budget: 50.0,
    verbose: false,
  };

  let _runtime = AgentRuntime::new(config);
  // Runtime created successfully (config fields are private)
}

#[tokio::test]
async fn test_start_agent()
{
  let config = RuntimeConfig {
    budget: 100.0,
    verbose: true,
  };

  let runtime = AgentRuntime::new(config);
  let path = Path::new("test_agent.py");

  // Start agent (will use stub implementation for now)
  let handle = runtime.start_agent(path).await;

  assert!(handle.is_ok());
  let handle = handle.unwrap();
  assert!(handle.agent_id.as_str().starts_with("agent_"));
}

#[tokio::test]
async fn test_get_metrics()
{
  let config = RuntimeConfig {
    budget: 50.0,
    verbose: false,
  };

  let runtime = AgentRuntime::new(config);
  let path = Path::new("test_agent.py");

  // Start agent
  let handle = runtime.start_agent(path).await.unwrap();

  // Get metrics
  let metrics = runtime.get_metrics(handle.agent_id.as_str());
  assert!(metrics.is_some());

  let metrics = metrics.unwrap();
  assert_eq!(metrics.agent_id, handle.agent_id, "Metrics should have correct agent ID");
  assert_eq!(metrics.budget_spent, 0.0, "Newly started agent should have no budget spent");
  assert_eq!(metrics.pii_detections, 0, "Newly started agent should have no PII detections");
}

#[tokio::test]
async fn test_stop_agent()
{
  let config = RuntimeConfig {
    budget: 50.0,
    verbose: false,
  };

  let runtime = AgentRuntime::new(config);
  let path = Path::new("test_agent.py");

  // Start agent
  let handle = runtime.start_agent(path).await.unwrap();

  // Stop agent
  let result = runtime.stop_agent(handle.agent_id.as_str()).await;
  assert!(result.is_ok());

  // Verify state updated
  let metrics = runtime.get_metrics(handle.agent_id.as_str());
  assert!(metrics.is_some());

  // Check status is Stopped
  let metrics = metrics.unwrap();
  assert!(matches!(metrics.status, iron_runtime_state::AgentStatus::Stopped), "Stopped agent should have Stopped status");
}
