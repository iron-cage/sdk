"""Test analytics sync with multiple threads."""

import os
import sys
import time
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed

# Add parent to path for iron_cage import
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from iron_cage import LlmRouter
from openai import OpenAI


def make_requests(client, thread_id, num_requests):
    """Make LLM requests from a single thread."""
    results = []
    for i in range(num_requests):
        try:
            response = client.chat.completions.create(
                model="gpt-4o-mini",
                messages=[{"role": "user", "content": f"Thread {thread_id}, request {i+1}: say 'ok'"}],
                max_tokens=5
            )
            content = response.choices[0].message.content
            results.append((thread_id, i+1, "success", content))
            print(f"  Thread {thread_id} request {i+1}: {content}")
        except Exception as e:
            results.append((thread_id, i+1, "error", str(e)))
            print(f"  Thread {thread_id} request {i+1}: ERROR - {e}")
    return results


def test_analytics_multithread():
    """Test analytics with 5 threads, 3 requests each."""

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
    requests_per_thread = 3
    total_requests = num_threads * requests_per_thread

    print(f"Creating LlmRouter with direct mode + server sync...")
    print(f"Server: {ic_server}")
    print(f"Threads: {num_threads}, Requests per thread: {requests_per_thread}")
    print(f"Total expected requests: {total_requests}")

    router = LlmRouter(
        provider_key=openai_key,
        api_key=ic_token,
        server_url=ic_server,
        budget=1.0
    )

    print(f"Router started on port {router.port}")

    # Create OpenAI client
    client = OpenAI(
        base_url=router.base_url,
        api_key=ic_token
    )

    print(f"\n--- Making {total_requests} LLM requests across {num_threads} threads ---\n")

    start_time = time.time()
    all_results = []

    with ThreadPoolExecutor(max_workers=num_threads) as executor:
        futures = {
            executor.submit(make_requests, client, i+1, requests_per_thread): i+1
            for i in range(num_threads)
        }

        for future in as_completed(futures):
            thread_id = futures[future]
            try:
                results = future.result()
                all_results.extend(results)
            except Exception as e:
                print(f"Thread {thread_id} failed: {e}")

    elapsed = time.time() - start_time

    # Summary
    print(f"\n--- Results ---")
    print(f"Total time: {elapsed:.2f}s")
    print(f"Requests completed: {len(all_results)}/{total_requests}")

    successes = sum(1 for r in all_results if r[2] == "success")
    errors = sum(1 for r in all_results if r[2] == "error")
    print(f"Successes: {successes}, Errors: {errors}")

    print(f"\n--- Budget Status ---")
    status = router.budget_status
    if status:
        spent, limit = status
        print(f"Spent: ${spent:.6f}")
        print(f"Limit: ${limit:.2f}")

    print(f"\n--- Stopping Router (triggers analytics flush) ---")
    router.stop()

    print(f"\nAnalytics sync complete!")
    print(f"Check database: sqlite3 iron.db 'SELECT COUNT(*) FROM analytics_events;'")


if __name__ == "__main__":
    test_analytics_multithread()
