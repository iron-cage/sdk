#!/usr/bin/env python3
"""Manual test script for LlmRouter.

Usage:
    export IC_TOKEN=iron_xxx
    export IC_SERVER=http://localhost:3000

    # Test OpenAI (requires OpenAI key configured in dashboard)
    python examples/test_manual.py openai

    # Test Anthropic (requires Anthropic key configured in dashboard)
    python examples/test_manual.py anthropic

    # Test Gateway mode: OpenAI client → Claude model (requires Anthropic key)
    python examples/test_manual.py gateway
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


def test_gateway(router):
    """Test gateway mode: OpenAI client calling Claude model!

    This demonstrates the unified API - same OpenAI client works for both
    OpenAI and Anthropic models, just change the model name.
    """
    from openai import OpenAI

    print("\n[Gateway Test: OpenAI client → Claude model]")
    client = OpenAI(base_url=router.base_url, api_key=router.api_key)

    # Use OpenAI SDK to call Claude!
    response = client.chat.completions.create(
        model="claude-sonnet-4-20250514",  # Claude model with OpenAI client!
        messages=[
            {"role": "system", "content": "You respond in exactly 4 words."},
            {"role": "user", "content": "Say hello"}
        ],
        max_tokens=50
    )
    print(f"   Response: {response.choices[0].message.content}")
    print(f"   Tokens: {response.usage.prompt_tokens} in, {response.usage.completion_tokens} out")
    print(f"   (OpenAI format response from Claude!)")


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
    from iron_cage import LlmRouter

    print("\n1. Creating LlmRouter...")
    router = LlmRouter(api_key=ic_token, server_url=ic_server)
    print(f"   Proxy running on: {router.base_url}")

    try:
        if provider == "openai":
            test_openai(router)
        elif provider == "anthropic":
            test_anthropic(router)
        elif provider == "gateway":
            test_gateway(router)
        else:
            print("\nERROR: Please specify provider: 'openai', 'anthropic', or 'gateway'")
            print("  python test_manual.py openai     # Test OpenAI API")
            print("  python test_manual.py anthropic  # Test Anthropic API")
            print("  python test_manual.py gateway    # Test OpenAI client → Claude model")
            sys.exit(1)
    finally:
        print("\n3. Stopping router...")
        router.stop()

    print("\nAll tests passed!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
