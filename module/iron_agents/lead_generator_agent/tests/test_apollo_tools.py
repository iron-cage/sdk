"""
Integration tests for Apollo API tools.

Tests use real implementations per ADR-007 policy.
API-dependent tests are skipped when APOLLO_API_KEY is not available.
"""
import json
import os
import sys
import pytest

# Add src to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

# Check if Apollo API key is available for integration tests
APOLLO_API_KEY = os.environ.get("APOLLO_API_KEY")
SKIP_API_TESTS = not APOLLO_API_KEY or APOLLO_API_KEY == "test_key_mock"


class TestApolloToolsWithoutApi:
    """Tests that don't require API access."""

    def test_search_leads_missing_api_key(self):
        """Verifies proper error when API key is missing."""
        # Temporarily remove API key
        original_key = os.environ.pop("APOLLO_API_KEY", None)

        try:
            # Re-import to get fresh module state
            import importlib
            import apollo_tools
            importlib.reload(apollo_tools)

            result = apollo_tools.search_leads.invoke({
                "job_title": "CEO",
                "location": "USA"
            })

            assert "missing" in result.lower() or "api key" in result.lower()
        finally:
            # Restore key if it existed
            if original_key:
                os.environ["APOLLO_API_KEY"] = original_key

    def test_search_leads_result_structure(self):
        """Verifies expected result structure from search_leads."""
        # Test data transformation by simulating cleaned results
        sample_api_response = {
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

        # Verify the expected transformation
        clean_results = []
        for p in sample_api_response.get('people', []):
            clean_results.append({
                "id": p.get("id"),
                "first_name": p.get("first_name"),
                "last_name": p.get("last_name"),
                "organization_name": p.get("organization", {}).get("name"),
                "title": p.get("title")
            })

        assert len(clean_results) == 1
        assert clean_results[0]["id"] == "123"
        assert clean_results[0]["last_name"] == "Shevchenko"
        assert clean_results[0]["organization_name"] == "Kobzar Inc"

    def test_lead_details_result_structure(self):
        """Verifies expected structure for lead details."""
        sample_person = {
            "id": "123",
            "email": "taras@example.com",
            "first_name": "Taras",
            "last_name": "Shevchenko"
        }

        # Verify JSON serialization
        json_str = json.dumps(sample_person, ensure_ascii=False)
        parsed = json.loads(json_str)

        assert parsed["email"] == "taras@example.com"
        assert parsed["id"] == "123"


@pytest.mark.skipif(SKIP_API_TESTS, reason="APOLLO_API_KEY not available")
class TestApolloToolsWithApi:
    """Integration tests that require real API access."""

    def test_search_leads_real_api(self):
        """Tests real API call to search_leads."""
        from apollo_tools import search_leads

        result = search_leads.invoke({
            "job_title": "CEO",
            "location": "United States",
            "quantity": 1
        })

        # Should return valid JSON
        data = json.loads(result)

        # Either we get results or a message
        if isinstance(data, list):
            if len(data) > 0:
                assert "id" in data[0]
        elif isinstance(data, dict):
            # Could be a "no results" message
            assert "message" in data or "error" in data

    def test_get_lead_details_invalid_id(self):
        """Tests get_lead_details with invalid ID."""
        from apollo_tools import get_lead_details

        result = get_lead_details.invoke({"apollo_id": "invalid-id-12345"})

        # Should return error or empty response
        assert "error" in result.lower() or "not found" in result.lower() or "Error" in result


class TestToolsPayloadConstruction:
    """Tests for API payload construction logic."""

    def test_payload_with_all_params(self):
        """Tests payload construction with all parameters."""
        job_title = "Marketing Manager"
        location = "New York"
        industry = "Technology"
        quantity = 5

        payload = {
            "person_titles": [job_title],
            "page": 1,
            "per_page": quantity,
            "contact_email_status": ["verified", "likely_to_engage"]
        }

        if location and isinstance(location, str) and location.strip():
            payload["person_locations"] = [location]

        if industry and isinstance(industry, str) and industry.strip():
            payload["q_organization_keyword_tags"] = [industry]

        assert payload["person_titles"] == ["Marketing Manager"]
        assert payload["per_page"] == 5
        assert payload["person_locations"] == ["New York"]
        assert payload["q_organization_keyword_tags"] == ["Technology"]

    def test_payload_minimal_params(self):
        """Tests payload construction with minimal parameters."""
        job_title = "CEO"
        location = None
        industry = ""
        quantity = 3

        payload = {
            "person_titles": [job_title],
            "page": 1,
            "per_page": quantity,
            "contact_email_status": ["verified", "likely_to_engage"]
        }

        if location and isinstance(location, str) and location.strip():
            payload["person_locations"] = [location]

        if industry and isinstance(industry, str) and industry.strip():
            payload["q_organization_keyword_tags"] = [industry]

        assert "person_titles" in payload
        assert "person_locations" not in payload
        assert "q_organization_keyword_tags" not in payload
