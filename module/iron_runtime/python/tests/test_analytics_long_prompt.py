"""Test analytics with long prompts from multiple threads."""

import os
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed

# Add parent to path for iron_cage import
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from iron_cage import LlmRouter
from openai import OpenAI


def test_long_prompts():
    """Test with 50k token prompts from 5 threads."""

    openai_key = os.environ.get("OPENAI_API_KEY")
    ic_token = os.environ.get("IC_TOKEN")
    ic_server = os.environ.get("IC_SERVER", "http://localhost:3001")

    if not openai_key:
        print("OPENAI_API_KEY not set")
        return
    if not ic_token:
        print("IC_TOKEN not set")
        return

    num_threads = 5
    # ~50k tokens each (Hello repeated ~50k times)
    long_prompt = "Hello " * 50000

    print(f"Creating LlmRouter...")
    router = LlmRouter(
        provider_key=openai_key,
        api_key=ic_token,
        server_url=ic_server,
        budget=1.0  # $1 budget - this test will be expensive!
    )

    print(f"Router started on port {router.port}")
    print(f"Threads: {num_threads}")
    print(f"Prompt size: ~50k tokens each")
    print(f"Expected cost: ~$0.0375 (5 x $0.0075)")
    print(f"Budget: $1.00")

    client = OpenAI(
        base_url=router.base_url,
        api_key=ic_token
    )

    def make_long_request(thread_id):
        try:
            start = time.time()
            response = client.chat.completions.create(
                model="gpt-4o-mini",
                messages=[{"role": "user", "content": f"Thread {thread_id}: {long_prompt} Say 'done'"}],
                max_tokens=10
            )
            elapsed = time.time() - start
            content = response.choices[0].message.content
            return (thread_id, "success", content, elapsed)
        except Exception as e:
            elapsed = time.time() - start
            return (thread_id, type(e).__name__, str(e)[:100], elapsed)

    print(f"\n--- Sending 5 concurrent 50k-token requests ---\n")
    start_time = time.time()

    with ThreadPoolExecutor(max_workers=num_threads) as executor:
        futures = {executor.submit(make_long_request, i+1): i+1 for i in range(num_threads)}
        results = []
        for future in as_completed(futures):
            result = future.result()
            results.append(result)
            thread_id, status, content, elapsed = result
            if status == "success":
                print(f"  Thread {thread_id}: {content} ({elapsed:.2f}s)")
            else:
                print(f"  Thread {thread_id}: {status} - {content[:50]}... ({elapsed:.2f}s)")

    total_time = time.time() - start_time

    # Summary
    print(f"\n--- Results ---")
    print(f"Total time: {total_time:.2f}s")
    successes = sum(1 for r in results if r[1] == "success")
    print(f"Successes: {successes}/{num_threads}")

    errors = [r for r in results if r[1] != "success"]
    if errors:
        print(f"\nErrors:")
        for thread_id, err_type, msg, elapsed in errors:
            print(f"  Thread {thread_id}: {err_type}")
            print(f"    {msg}")

    print(f"\n--- Budget Status ---")
    status = router.budget_status
    if status:
        spent, limit = status
        print(f"Spent: ${spent:.6f}")
        print(f"Limit: ${limit:.2f}")
        print(f"Remaining: ${limit - spent:.6f}")

    print(f"\n--- Stopping Router ---")
    router.stop()

    print(f"\nDone!")


if __name__ == "__main__":
    test_long_prompts()
