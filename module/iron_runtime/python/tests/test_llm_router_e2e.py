#!/usr/bin/env python3
"""End-to-end tests for LlmRouter.

Requires:
- IC_TOKEN environment variable set
- IC_SERVER environment variable set (Iron Cage server URL)
- OpenAI provider key configured in dashboard

Run:
    IC_TOKEN=iron_xxx IC_SERVER=http://localhost:3000 python -m pytest tests/test_llm_router_e2e.py -v
"""

import os
import pytest
from iron_runtime import LlmRouter


# Skip if credentials not available
pytestmark = pytest.mark.skipif(
    not os.environ.get("IC_TOKEN") or not os.environ.get("IC_SERVER"),
    reason="IC_TOKEN and IC_SERVER required for E2E tests"
)


class TestLlmRouterE2E:
    """End-to-end tests for LlmRouter."""

    def test_router_starts_and_stops(self):
        """Test that router starts proxy server and stops cleanly."""
        router = LlmRouter(
            api_key=os.environ["IC_TOKEN"],
            server_url=os.environ["IC_SERVER"],
        )

        assert router.is_running
        assert router.port > 0
        assert "127.0.0.1" in router.base_url
        assert "/v1" in router.base_url

        router.stop()
        assert not router.is_running

    def test_router_context_manager(self):
        """Test context manager auto-cleanup."""
        with LlmRouter(
            api_key=os.environ["IC_TOKEN"],
            server_url=os.environ["IC_SERVER"],
        ) as router:
            assert router.is_running

        # After exiting context, should be stopped
        assert not router.is_running

    def test_openai_chat_completion(self):
        """Test actual OpenAI API call through proxy."""
        try:
            from openai import OpenAI
        except ImportError:
            pytest.skip("openai package not installed")

        router = LlmRouter(
            api_key=os.environ["IC_TOKEN"],
            server_url=os.environ["IC_SERVER"],
        )

        # Skip if provider is not OpenAI
        if router.provider != "openai":
            router.stop()
            pytest.skip(f"Skipping: configured provider is '{router.provider}', not 'openai'")

        try:
            client = OpenAI(
                base_url=router.base_url,
                api_key=router.api_key,
            )

            response = client.chat.completions.create(
                model="gpt-5-nano",
                messages=[{"role": "user", "content": "Say 'test' and nothing else"}],
                max_completion_tokens=10,
            )

            assert response.choices is not None
            assert len(response.choices) > 0
            assert response.choices[0].message.content is not None

        finally:
            router.stop()

    def test_invalid_token_rejected(self):
        """Test that invalid IC_TOKEN is rejected by proxy."""
        try:
            from openai import OpenAI, AuthenticationError
        except ImportError:
            pytest.skip("openai package not installed")

        router = LlmRouter(
            api_key=os.environ["IC_TOKEN"],
            server_url=os.environ["IC_SERVER"],
        )

        try:
            # Use wrong API key with the proxy
            client = OpenAI(
                base_url=router.base_url,
                api_key="wrong_token",  # Wrong token
            )

            with pytest.raises(AuthenticationError):
                client.chat.completions.create(
                    model="gpt-5-nano",
                    messages=[{"role": "user", "content": "test"}],
                )
        finally:
            router.stop()

    def test_custom_cache_ttl(self):
        """Test that custom cache TTL is accepted."""
        router = LlmRouter(
            api_key=os.environ["IC_TOKEN"],
            server_url=os.environ["IC_SERVER"],
            cache_ttl_seconds=60,  # 1 minute
        )

        assert router.is_running
        router.stop()
