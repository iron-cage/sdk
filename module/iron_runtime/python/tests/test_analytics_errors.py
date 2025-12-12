"""Test analytics sync with error scenarios."""

import os
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed

# Add parent to path for iron_cage import
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from iron_cage import LlmRouter
from openai import OpenAI


def test_analytics_errors():
    """Test analytics with various error scenarios."""

    openai_key = os.environ.get("OPENAI_API_KEY")
    ic_token = os.environ.get("IC_TOKEN")
    ic_server = os.environ.get("IC_SERVER", "http://localhost:3001")

    if not openai_key:
        print("OPENAI_API_KEY not set")
        return
    if not ic_token:
        print("IC_TOKEN not set")
        return

    print(f"Creating LlmRouter...")
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

    results = []

    # Test 1: Invalid model name
    print("\n--- Test 1: Invalid model name ---")
    try:
        response = client.chat.completions.create(
            model="gpt-invalid-model-xyz",
            messages=[{"role": "user", "content": "Hello"}],
            max_tokens=10
        )
        results.append(("invalid_model", "unexpected_success", str(response)))
    except Exception as e:
        error_type = type(e).__name__
        results.append(("invalid_model", error_type, str(e)[:100]))
        print(f"  Error: {error_type} - {str(e)[:80]}...")

    # Test 2: Successful request (baseline)
    print("\n--- Test 2: Successful request (baseline) ---")
    try:
        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[{"role": "user", "content": "Say 'ok'"}],
            max_tokens=5
        )
        content = response.choices[0].message.content
        results.append(("success_baseline", "success", content))
        print(f"  Response: {content}")
    except Exception as e:
        results.append(("success_baseline", "error", str(e)[:100]))
        print(f"  Error: {e}")

    # Test 3: Rate limit attempt - burst of concurrent requests
    print("\n--- Test 3: Rate limit attempt (20 concurrent requests) ---")
    def make_request(i):
        try:
            response = client.chat.completions.create(
                model="gpt-4o-mini",
                messages=[{"role": "user", "content": f"Request {i}: say 'ok'"}],
                max_tokens=5
            )
            return ("success", response.choices[0].message.content)
        except Exception as e:
            return (type(e).__name__, str(e)[:50])

    with ThreadPoolExecutor(max_workers=20) as executor:
        futures = [executor.submit(make_request, i) for i in range(20)]
        burst_results = [f.result() for f in as_completed(futures)]

    successes = sum(1 for r in burst_results if r[0] == "success")
    errors = [r for r in burst_results if r[0] != "success"]
    print(f"  Successes: {successes}/20")
    if errors:
        print(f"  Errors: {len(errors)}")
        for err_type, msg in errors[:3]:
            print(f"    - {err_type}: {msg}")
    results.append(("rate_limit_burst", f"{successes}/20 success", str(errors[:2])))

    # Test 4: Empty message
    print("\n--- Test 4: Empty message content ---")
    try:
        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[{"role": "user", "content": ""}],
            max_tokens=10
        )
        results.append(("empty_message", "success", response.choices[0].message.content))
        print(f"  Response: {response.choices[0].message.content}")
    except Exception as e:
        error_type = type(e).__name__
        results.append(("empty_message", error_type, str(e)[:100]))
        print(f"  Error: {error_type} - {str(e)[:80]}...")

    # Test 5: Very long prompt (token limit)
    print("\n--- Test 5: Very long prompt ---")
    try:
        long_prompt = "Hello " * 50000  # Try to exceed token limits
        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[{"role": "user", "content": long_prompt}],
            max_tokens=10
        )
        results.append(("long_prompt", "unexpected_success", ""))
    except Exception as e:
        error_type = type(e).__name__
        results.append(("long_prompt", error_type, str(e)[:100]))
        print(f"  Error: {error_type} - {str(e)[:80]}...")

    # Summary
    print("\n" + "="*60)
    print("SUMMARY")
    print("="*60)
    for test_name, status, detail in results:
        print(f"  {test_name}: {status}")

    print(f"\n--- Budget Status ---")
    status = router.budget_status
    if status:
        spent, limit = status
        print(f"Spent: ${spent:.6f}")

    print(f"\n--- Stopping Router ---")
    router.stop()

    print(f"\nCheck failed events: sqlite3 iron.db \"SELECT * FROM analytics_events WHERE event_type='llm_request_failed';\"")


if __name__ == "__main__":
    test_analytics_errors()
