"""Test analytics sync to Control API server."""

import os
import sys
import time

# Add parent to path for iron_cage import
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from iron_cage import LlmRouter
from openai import OpenAI


def test_analytics_sync():
    """Test that analytics events sync to server.

    Uses direct mode (provider_key) with analytics sync (api_key + server_url).
    """

    openai_key = os.environ.get("OPENAI_API_KEY")
    ic_token = os.environ.get("IC_TOKEN")
    ic_server = os.environ.get("IC_SERVER", "http://localhost:3001")

    if not openai_key:
        print("OPENAI_API_KEY not set")
        return
    if not ic_token:
        print("IC_TOKEN not set")
        return

    print(f"Creating LlmRouter with direct mode + server sync...")
    print(f"Server: {ic_server}")

    # Direct mode with analytics sync:
    # - provider_key: Direct OpenAI access (no key fetch from server)
    # - api_key + server_url: Analytics sync to Control API
    router = LlmRouter(
        provider_key=openai_key,  # Direct OpenAI access
        api_key=ic_token,          # For analytics sync auth
        server_url=ic_server,      # Analytics sync destination
        budget=1.0
    )

    print(f"Router started on port {router.port}")
    print(f"Base URL: {router.base_url}")
    print(f"Provider: {router.provider}")

    # Create OpenAI client pointing to our router
    client = OpenAI(
        base_url=router.base_url,
        api_key=ic_token  # Must match the IC token for authentication
    )

    # Make a few requests
    print("\n--- Making LLM requests ---")

    for i in range(2):
        print(f"\nRequest {i+1}...")
        try:
            response = client.chat.completions.create(
                model="gpt-4o-mini",
                messages=[{"role": "user", "content": f"Say 'analytics test {i+1}' and nothing else."}],
                max_tokens=10
            )
            print(f"Response: {response.choices[0].message.content}")
            print(f"Total spent so far: ${router.total_spent():.6f}")
        except Exception as e:
            print(f"Error: {e}")

    # Wait a bit for sync
    print("\n--- Waiting for sync ---")
    time.sleep(2)

    # Stop router (triggers final flush)
    print("\n--- Stopping Router (triggers analytics flush) ---")
    router.stop()

    print("\nAnalytics should now be visible in dashboard!")
    print(f"Check: {ic_server}/api/v1/analytics/spending/total")


if __name__ == "__main__":
    test_analytics_sync()
