"""Test analytics sync with different models."""

import os
import sys
import time

# Add parent to path for iron_cage import
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from iron_cage import LlmRouter
from openai import OpenAI


def test_analytics_models():
    """Test analytics with different OpenAI models."""

    openai_key = os.environ.get("OPENAI_API_KEY")
    ic_token = os.environ.get("IC_TOKEN")
    ic_server = os.environ.get("IC_SERVER", "http://localhost:3001")

    if not openai_key:
        print("OPENAI_API_KEY not set")
        return
    if not ic_token:
        print("IC_TOKEN not set")
        return

    # Models to test
    models = [
        "gpt-4o-mini",
        "gpt-4o",
        "gpt-3.5-turbo",
        "gpt-4-turbo",
    ]

    print(f"Creating LlmRouter...")
    print(f"Server: {ic_server}")
    print(f"Models to test: {models}")

    router = LlmRouter(
        provider_key=openai_key,
        api_key=ic_token,
        server_url=ic_server,
        budget=1.0
    )

    print(f"Router started on port {router.port}")

    client = OpenAI(
        base_url=router.base_url,
        api_key=ic_token
    )

    print(f"\n--- Testing different models ---\n")

    results = []
    for model in models:
        print(f"Testing {model}...")
        try:
            start = time.time()
            response = client.chat.completions.create(
                model=model,
                messages=[{"role": "user", "content": "Say 'hello' and nothing else"}],
                max_tokens=10
            )
            elapsed = time.time() - start
            content = response.choices[0].message.content
            results.append((model, "success", content, elapsed))
            print(f"  Response: {content} ({elapsed:.2f}s)")
        except Exception as e:
            results.append((model, "error", str(e), 0))
            print(f"  ERROR: {e}")

    # Summary
    print(f"\n--- Results ---")
    for model, status, content, elapsed in results:
        if status == "success":
            print(f"  {model}: {content} ({elapsed:.2f}s)")
        else:
            print(f"  {model}: FAILED - {content[:50]}...")

    print(f"\n--- Budget Status ---")
    status = router.budget_status
    if status:
        spent, limit = status
        print(f"Spent: ${spent:.6f}")

    print(f"\n--- Stopping Router ---")
    router.stop()

    print(f"\nDone! Check analytics by model in database.")


if __name__ == "__main__":
    test_analytics_models()
