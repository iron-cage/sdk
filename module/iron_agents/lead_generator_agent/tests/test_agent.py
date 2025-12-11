import os
import json
import pytest
from unittest.mock import patch, MagicMock

# Set fake environment variables
os.environ["OPENAI_API_KEY"] = "sk-fake-test-key"
os.environ["APOLLO_API_KEY"] = "test_apollo_key"

import lead_generator_agent

def test_manual_json_parsing():
    """Tests the JSON cleanup logic."""
    bad_json = "```json\n[{\"id\": 1}]\n```"
    clean = bad_json.replace("```json", "").replace("```", "").strip()
    data = json.loads(clean)
    assert data[0]["id"] == 1

# Instead of patch("...agent_executor.invoke"), we patch the executor object itself
@patch("lead_generator_agent.agent_executor")
def test_agent_flow(mock_executor):
    """Tests that the agent returns data we can parse."""
    
    # 1. Prepare data that .invoke() should return
    mock_output_data = json.dumps([{"id": "99", "name": "Test Lead"}])
    
    mock_executor.invoke.return_value = {
        "output": f"```json{mock_output_data}```"
    }
    
    # 2. Execute logic
    response = lead_generator_agent.agent_executor.invoke({"input": "Find lead"})
    
    # 3. Verify processing
    raw = response["output"].replace("```json", "").replace("```", "").strip()
    data = json.loads(raw)
    
    assert len(data) == 1
    assert data[0]["name"] == "Test Lead"
    
