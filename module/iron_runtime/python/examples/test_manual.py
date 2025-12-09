#!/usr/bin/env python3
"""Manual test script for LlmRouter.

Usage:
    export IC_TOKEN=iron_xxx
    export IC_SERVER=http://localhost:3000

    # Test OpenAI (requires OpenAI key configured in dashboard)
    python examples/test_manual.py openai

    # Test Anthropic (requires Anthropic key configured in dashboard)
    python examples/test_manual.py anthropic
"""

import os
import sys


def test_openai(router):
    """Test OpenAI API through LlmRouter."""
    from openai import OpenAI

    print("\n[OpenAI Test]")
    client = OpenAI(base_url=router.base_url, api_key=router.api_key)

    response = client.chat.completions.create(
        model="gpt-5-nano",
        messages=[{"role": "user", "content": "Say 'Hello from LlmRouter!' in exactly 4 words"}],
        max_completion_tokens=500,
        reasoning_effort="low",
    )
    print(f"   Response: {response.choices[0].message.content}")
    print(f"   Tokens: {response.usage.prompt_tokens} in, {response.usage.completion_tokens} out")


def test_anthropic(router):
    """Test Anthropic API through LlmRouter."""
    from anthropic import Anthropic

    print("\n[Anthropic Test]")
    # Anthropic API doesn't use /v1 suffix
    anthropic_base = router.base_url.replace("/v1", "")

    client = Anthropic(base_url=anthropic_base, api_key=router.api_key)

    response = client.messages.create(
        model="claude-sonnet-4-20250514",
        max_tokens=100,
        messages=[{"role": "user", "content": "Say 'Hello from LlmRouter!' in exactly 4 words"}],
    )
    print(f"   Response: {response.content[0].text}")
    print(f"   Tokens: {response.usage.input_tokens} in, {response.usage.output_tokens} out")


def main():
    # Check env vars
    ic_token = os.environ.get("IC_TOKEN")
    ic_server = os.environ.get("IC_SERVER")

    if not ic_token or not ic_server:
        print("ERROR: Set IC_TOKEN and IC_SERVER environment variables")
        print("  export IC_TOKEN=iron_xxx")
        print("  export IC_SERVER=http://localhost:3000")
        sys.exit(1)

    # Get provider from command line
    provider = sys.argv[1] if len(sys.argv) > 1 else "auto"

    print(f"IC_TOKEN: {ic_token[:20]}...")
    print(f"IC_SERVER: {ic_server}")
    print(f"Provider: {provider}")

    # Import and create router
    from iron_runtime import LlmRouter

    print("\n1. Creating LlmRouter...")
    router = LlmRouter(api_key=ic_token, server_url=ic_server)
    print(f"   Proxy running on: {router.base_url}")

    try:
        if provider == "openai":
            test_openai(router)
        elif provider == "anthropic":
            test_anthropic(router)
        else:
            print("\nERROR: Please specify provider: 'openai' or 'anthropic'")
            print("  python test_manual.py openai")
            print("  python test_manual.py anthropic")
            sys.exit(1)
    finally:
        print("\n3. Stopping router...")
        router.stop()

    print("\nAll tests passed!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
