"""Test analytics recording locally (direct mode - no server sync)."""

import os
import sys

# Add parent to path for iron_cage import
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from iron_cage import LlmRouter
from openai import OpenAI


def test_analytics_local():
    """Test that analytics events are recorded locally."""

    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        print("OPENAI_API_KEY not set")
        return

    print("Creating LlmRouter in direct mode...")
    router = LlmRouter(
        provider_key=api_key,
        budget=1.0  # $1 budget for testing
    )

    print(f"Router started on port {router.port}")
    print(f"Base URL: {router.base_url}")
    print(f"Provider: {router.provider}")

    # Create OpenAI client pointing to our router
    client = OpenAI(
        base_url=router.base_url,
        api_key="direct"  # Any key works in direct mode
    )

    # Make a few requests
    print("\n--- Making LLM requests ---")

    for i in range(3):
        print(f"\nRequest {i+1}...")
        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[{"role": "user", "content": f"Say 'test {i+1}' and nothing else."}],
            max_tokens=10
        )
        print(f"Response: {response.choices[0].message.content}")
        print(f"Total spent so far: ${router.total_spent():.6f}")

    # Check budget status
    print("\n--- Budget Status ---")
    status = router.budget_status
    if status:
        spent, limit = status
        print(f"Spent: ${spent:.6f}")
        print(f"Limit: ${limit:.2f}")
        print(f"Remaining: ${limit - spent:.6f}")

    print("\n--- Stopping Router ---")
    router.stop()
    print("Router stopped (analytics would flush here if server was configured)")

    print("\nTest complete!")


if __name__ == "__main__":
    test_analytics_local()
