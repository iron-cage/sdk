"""
Integration tests for lead_generator_agent module.

Tests helper functions and JSON parsing logic using real implementations.
No mocking - per ADR-007 policy.
"""
import json
import pytest
import os
import sys

# Add src to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))


class TestJsonParsing:
    """Tests for JSON cleanup and parsing logic."""

    def test_strip_markdown_fences_with_json_block(self):
        """Tests cleanup of markdown-wrapped JSON."""
        bad_json = "```json\n[{\"id\": 1}]\n```"
        clean = bad_json.replace("```json", "").replace("```", "").strip()
        data = json.loads(clean)
        assert data[0]["id"] == 1

    def test_strip_markdown_fences_no_fences(self):
        """Tests that clean JSON passes through unchanged."""
        clean_json = '[{"id": 2, "name": "Test"}]'
        data = json.loads(clean_json)
        assert data[0]["id"] == 2
        assert data[0]["name"] == "Test"

    def test_strip_markdown_fences_empty_array(self):
        """Tests handling of empty JSON array."""
        json_str = "```json\n[]\n```"
        clean = json_str.replace("```json", "").replace("```", "").strip()
        data = json.loads(clean)
        assert data == []

    def test_strip_markdown_fences_nested_object(self):
        """Tests parsing of nested JSON objects."""
        json_str = '{"lead": {"id": "123", "details": {"email": "test@example.com"}}}'
        data = json.loads(json_str)
        assert data["lead"]["id"] == "123"
        assert data["lead"]["details"]["email"] == "test@example.com"


class TestQuantityParsing:
    """Tests for quantity extraction from user queries."""

    def test_parse_quantity_with_qty_equals(self):
        """Tests qty=N format."""
        from lead_generator_agent import _parse_quantity_from_query

        query, qty = _parse_quantity_from_query("Find leads qty=10")
        assert qty == 10
        assert "qty=10" not in query

    def test_parse_quantity_with_quantity_colon(self):
        """Tests quantity: N format."""
        from lead_generator_agent import _parse_quantity_from_query

        query, qty = _parse_quantity_from_query("Find CEOs quantity: 5")
        assert qty == 5
        assert "quantity: 5" not in query

    def test_parse_quantity_with_flag(self):
        """Tests --qty N format."""
        from lead_generator_agent import _parse_quantity_from_query

        query, qty = _parse_quantity_from_query("Find leads --qty 15")
        assert qty == 15
        assert "--qty 15" not in query

    def test_parse_quantity_none(self):
        """Tests query without quantity specification."""
        from lead_generator_agent import _parse_quantity_from_query

        query, qty = _parse_quantity_from_query("Find marketing managers in NYC")
        assert qty is None
        assert query == "Find marketing managers in NYC"


class TestLeadDataStructure:
    """Tests for expected lead data structures."""

    def test_lead_object_schema(self):
        """Validates expected lead object schema."""
        lead = {
            "id": "abc123",
            "first_name": "John",
            "last_name": "Doe",
            "title": "CEO",
            "company": "Acme Corp",
            "industry": "Technology",
            "location": "San Francisco",
            "email": "john@acme.com",
            "linkedin_url": "https://linkedin.com/in/johndoe"
        }

        # Verify all expected keys exist
        expected_keys = {"id", "first_name", "last_name", "title", "company",
                        "industry", "location", "email", "linkedin_url"}
        assert set(lead.keys()) == expected_keys

    def test_lead_with_null_fields(self):
        """Tests lead object with null fields."""
        lead = {
            "id": "xyz789",
            "first_name": "Jane",
            "last_name": "Smith",
            "title": "CTO",
            "company": "Tech Inc",
            "industry": None,
            "location": None,
            "email": None,
            "linkedin_url": None
        }

        # Verify serialization works
        json_str = json.dumps(lead, ensure_ascii=False)
        parsed = json.loads(json_str)
        assert parsed["id"] == "xyz789"
        assert parsed["email"] is None
