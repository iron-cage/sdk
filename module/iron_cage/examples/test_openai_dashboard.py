#!/usr/bin/env python3
"""Test iron_cage with real OpenAI key from dashboard.

Usage:
    # First, set up config (one-time):
    python -c "import iron_cage; iron_cage.init('iron_xxx', 'http://localhost:3000')"

    # Or use environment variables:
    export IC_KEY=iron_xxx
    export IC_SERVER=http://localhost:3000

    # Then run this test:
    python examples/test_openai_dashboard.py
"""

import sys
sys.path.insert(0, "python")

from iron_cage import SafetyRuntime
from iron_cage.client import IronCageClient
from iron_cage.config import IronCageConfig


def test_config_loading():
    """Test that config loads correctly."""
    print("=" * 50)
    print("Testing config loading...")

    config = IronCageConfig.load()
    print(f"  ic_key: {config.ic_key[:20]}..." if config.ic_key else "  ic_key: None")
    print(f"  server_url: {config.server_url}")
    print(f"  is_configured: {config.is_configured}")

    if not config.is_configured:
        print("\nERROR: Not configured!")
        print("Run: iron_cage.init('your_ic_key', 'http://localhost:3000')")
        print("Or set IC_KEY and IC_SERVER environment variables")
        return False

    print("  OK")
    return True


def test_key_fetch():
    """Test fetching keys from dashboard."""
    print("=" * 50)
    print("Testing key fetch from dashboard...")

    client = IronCageClient()

    try:
        keys = client.fetch_keys()
        print(f"  provider: {keys['provider']}")
        print(f"  api_key: {keys['api_key'][:20]}..." if keys['api_key'] else "  api_key: None")
        print(f"  base_url: {keys['base_url']}")
        print("  OK")
        return keys
    except Exception as e:
        print(f"  ERROR: {e}")
        return None


def test_openai_call(api_key: str, base_url: str = None):
    """Test actual OpenAI API call."""
    print("=" * 50)
    print("Testing OpenAI API call...")

    try:
        from openai import OpenAI
    except ImportError:
        print("  SKIP: openai package not installed")
        print("  Run: pip install openai")
        return False

    try:
        client_kwargs = {"api_key": api_key}
        if base_url:
            client_kwargs["base_url"] = base_url

        client = OpenAI(**client_kwargs)

        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[{"role": "user", "content": "Say 'Hello from iron_cage!' in exactly 5 words."}],
            max_tokens=20,
        )

        content = response.choices[0].message.content
        print(f"  Response: {content}")
        print(f"  Tokens: {response.usage.prompt_tokens} in, {response.usage.completion_tokens} out")
        print("  OK")
        return True
    except Exception as e:
        print(f"  ERROR: {e}")
        return False


def test_safety_runtime():
    """Test SafetyRuntime with real OpenAI call."""
    print("=" * 50)
    print("Testing SafetyRuntime with OpenAI...")

    try:
        from openai import OpenAI
    except ImportError:
        print("  SKIP: openai package not installed")
        return False

    try:
        # Create runtime with budget tracking
        runtime = SafetyRuntime(
            budget_usd=1.0,
            pii_detection=True,
            circuit_breaker=True,
        )

        print(f"  Server mode: {runtime.is_server_mode}")

        # Define a simple agent function
        def simple_agent():
            # Keys should be injected into env by runtime
            import os
            api_key = os.environ.get("OPENAI_API_KEY")
            base_url = os.environ.get("OPENAI_BASE_URL")

            print(f"  Env OPENAI_API_KEY: {api_key[:20] if api_key else 'None'}...")

            client_kwargs = {"api_key": api_key}
            if base_url:
                client_kwargs["base_url"] = base_url

            client = OpenAI(**client_kwargs)

            response = client.chat.completions.create(
                model="gpt-4o-mini",
                messages=[{"role": "user", "content": "What is 2+2? Answer with just the number."}],
                max_tokens=10,
            )
            return response.choices[0].message.content

        # Run with safety runtime
        result = runtime.run(simple_agent)
        print(f"  Result: {result}")
        print("  OK")
        return True

    except Exception as e:
        print(f"  ERROR: {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    print("\nIron Cage + OpenAI Dashboard Integration Test")
    print("=" * 50)

    # Step 1: Check config
    if not test_config_loading():
        return 1

    # Step 2: Fetch keys
    keys = test_key_fetch()
    if not keys:
        return 1

    if keys["provider"] != "openai":
        print(f"\nWARNING: Provider is '{keys['provider']}', not 'openai'")
        print("This test requires an OpenAI key assigned to your project")
        return 1

    # Step 3: Direct OpenAI call (verify key works)
    if not test_openai_call(keys["api_key"], keys.get("base_url")):
        return 1

    # Step 4: SafetyRuntime with OpenAI
    if not test_safety_runtime():
        return 1

    print("=" * 50)
    print("All tests passed!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
