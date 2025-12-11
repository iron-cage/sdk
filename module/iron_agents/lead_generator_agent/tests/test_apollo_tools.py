import sys
import os
import json
import pytest
from unittest.mock import patch, MagicMock

# We set fake environment variables BEFORE importing the module.
os.environ["APOLLO_API_KEY"] = "test_key_mock"

from apollo_tools import search_leads, get_lead_details

@patch('apollo_tools.requests.post')
def test_search_leads_success(mock_post):
    """Verifies successful lead search."""
    mock_response = MagicMock()
    mock_response.status_code = 200
    mock_response.json.return_value = {
        "people": [
            {
                "id": "123",
                "first_name": "Taras",
                "last_name": "Shevchenko",
                "organization": {"name": "Kobzar Inc"},
                "title": "Poet"
            }
        ]
    }
    mock_post.return_value = mock_response

    result = search_leads.invoke({
        "job_title": "Poet", 
        "location": "Ukraine"
    })
    
    data = json.loads(result)
    assert len(data) == 1
    assert data[0]["last_name"] == "Shevchenko"

@patch('apollo_tools.requests.get')
def test_get_lead_details_success(mock_get):
    """Verifies retrieval of lead details."""
    mock_response = MagicMock()
    mock_response.status_code = 200
    mock_response.json.return_value = {
        "person": {"id": "123", "email": "taras@example.com"}
    }
    mock_get.return_value = mock_response

    result = get_lead_details.invoke({"apollo_id": "123"})
    data = json.loads(result)
    
    assert data["email"] == "taras@example.com"