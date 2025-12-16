#!/usr/bin/env python3
"""E2E Budget Tests - Real API calls with budget limit.

Supports two modes:
1. Direct provider key: OPENAI_API_KEY or ANTHROPIC_API_KEY
2. Iron Cage server: IC_TOKEN + IC_SERVER

Run:
    # Direct OpenAI key
    OPENAI_API_KEY=sk-xxx pytest tests/test_budget_e2e.py -v

    # Direct Anthropic key
    ANTHROPIC_API_KEY=sk-ant-xxx pytest tests/test_budget_e2e.py -v

    # Iron Cage server
    IC_TOKEN=ic_xxx IC_SERVER=http://localhost:3000 pytest tests/test_budget_e2e.py -v
"""

import os
import pytest
from iron_cage import LlmRouter


def get_router_config():
    """Get router configuration from environment.

    Returns (config_dict, provider) or (None, None) if not configured.
    """
    # Mode 1: Direct OpenAI key
    if os.environ.get("OPENAI_API_KEY"):
        return {"provider_key": os.environ["OPENAI_API_KEY"]}, "openai"

    # Mode 2: Direct Anthropic key
    if os.environ.get("ANTHROPIC_API_KEY"):
        return {"provider_key": os.environ["ANTHROPIC_API_KEY"]}, "anthropic"

    # Mode 3: Iron Cage server
    if os.environ.get("IC_TOKEN") and os.environ.get("IC_SERVER"):
        return {
            "api_key": os.environ["IC_TOKEN"],
            "server_url": os.environ["IC_SERVER"],
        }, None  # Provider detected from server

    return None, None


# Skip all tests if no credentials configured
CONFIG, EXPECTED_PROVIDER = get_router_config()
pytestmark = pytest.mark.skipif(
    CONFIG is None,
    reason="Set OPENAI_API_KEY, ANTHROPIC_API_KEY, or (IC_TOKEN + IC_SERVER)"
)


class TestBudgetE2EOpenAI:
    """E2E tests with OpenAI (gpt-4o-mini) - ~$0.001 per test"""

    @pytest.fixture
    def router_with_budget(self):
        """Create router with $0.05 budget"""
        router = LlmRouter(**CONFIG, budget=0.05)
        yield router
        router.stop()

    def test_openai_request_tracks_spending(self, router_with_budget):
        """Test: OpenAI request updates spent amount"""
        try:
            from openai import OpenAI
        except ImportError:
            pytest.skip("openai package not installed")

        # Skip if provider is not OpenAI
        if router_with_budget.provider != "openai":
            pytest.skip(f"Provider is '{router_with_budget.provider}', not 'openai'")

        client = OpenAI(
            base_url=router_with_budget.base_url,
            api_key=router_with_budget.api_key,
        )

        # Initial state
        spent_before, limit = router_with_budget.budget_status
        assert spent_before == 0.0
        assert limit == 0.05

        # Make request (gpt-4o-mini is cheap)
        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[{"role": "user", "content": "Say 'test' only"}],
            max_tokens=5,
        )

        # Verify spending tracked
        spent_after, _ = router_with_budget.budget_status
        assert spent_after > 0.0
        assert spent_after < 0.05  # Should be ~$0.0001
        print(f"Cost: ${spent_after:.6f}")

    def test_openai_budget_exceeded_returns_402(self):
        """Test: OpenAI request with zero budget returns 402"""
        try:
            from openai import OpenAI
        except ImportError:
            pytest.skip("openai package not installed")

        router = LlmRouter(**CONFIG, budget=0.0)

        # Skip if provider is not OpenAI
        if router.provider != "openai":
            router.stop()
            pytest.skip(f"Provider is '{router.provider}', not 'openai'")

        client = OpenAI(
            base_url=router.base_url,
            api_key=router.api_key,
        )

        try:
            with pytest.raises(Exception) as exc_info:
                client.chat.completions.create(
                    model="gpt-4o-mini",
                    messages=[{"role": "user", "content": "test"}],
                )

            # Verify it's our custom error, not OpenAI's
            error_str = str(exc_info.value)
            assert "Iron Cage budget limit exceeded" in error_str or "402" in error_str
        finally:
            router.stop()

    def test_openai_spend_until_limit_hit(self):
        """Test: Make requests until budget is exhausted, then get 402.

        This test actually spends money and hits the limit naturally.
        Uses gpt-5-mini for faster spending.
        Adjust BUDGET_USD if test takes too long.
        """
        try:
            from openai import OpenAI
        except ImportError:
            pytest.skip("openai package not installed")

        # Adjust this value if test takes too long
        BUDGET_USD = 0.01

        router = LlmRouter(**CONFIG, budget=BUDGET_USD)

        # Skip if provider is not OpenAI
        if router.provider != "openai":
            router.stop()
            pytest.skip(f"Provider is '{router.provider}', not 'openai'")

        client = OpenAI(
            base_url=router.base_url,
            api_key=router.api_key,
        )

        try:
            request_count = 0
            max_requests = 50  # Safety limit

            while request_count < max_requests:
                spent_before = router.total_spent()

                try:
                    response = client.chat.completions.create(
                        model="gpt-5-mini",
                        messages=[{"role": "user", "content": "Write a long essay about AI."}],
                        max_completion_tokens=4000,
                    )
                    request_count += 1
                    spent_after = router.total_spent()
                    cost_this_request = spent_after - spent_before
                    answer_preview = response.choices[0].message.content[:80] + "..."

                    print(f"\n[Request {request_count}]")
                    print(f"  Answer: {answer_preview}")
                    print(f"  This request: ${cost_this_request:.6f}")
                    print(f"  Total spent: ${spent_after:.6f} / ${BUDGET_USD:.2f}")

                except Exception as e:
                    error_str = str(e)
                    if "Iron Cage budget limit exceeded" in error_str or "402" in error_str:
                        print(f"\n*** Budget exceeded after {request_count} requests! ***")
                        print(f"Final spent: ${router.total_spent():.6f}")
                        break
                    else:
                        raise  # Re-raise non-budget errors
            else:
                pytest.fail(f"Budget not exceeded after {max_requests} requests")

            # Verify we made at least 1 successful request before hitting limit
            assert request_count >= 1, "Should complete at least 1 request before limit"

        finally:
            router.stop()


    def test_openai_concurrent_spend_until_limit_hit(self):
        """Test: 5 concurrent threads making requests until $0.05 budget exhausted.

        Tests that budget enforcement works correctly under concurrent load.
        """
        try:
            from openai import OpenAI
        except ImportError:
            pytest.skip("openai package not installed")

        import threading
        import time

        BUDGET_USD = 0.05
        NUM_THREADS = 5

        router = LlmRouter(**CONFIG, budget=BUDGET_USD)

        if router.provider != "openai":
            router.stop()
            pytest.skip(f"Provider is '{router.provider}', not 'openai'")

        results = {"success": 0, "budget_exceeded": 0, "errors": []}
        results_lock = threading.Lock()

        def worker(thread_id):
            client = OpenAI(
                base_url=router.base_url,
                api_key=router.api_key,
            )

            while True:
                spent_before = router.total_spent()
                try:
                    response = client.chat.completions.create(
                        model="gpt-5-mini",
                        messages=[{"role": "user", "content": "Write about AI."}],
                        max_completion_tokens=2000,
                    )
                    spent_after = router.total_spent()
                    cost = spent_after - spent_before
                    answer = response.choices[0].message.content[:50] + "..."

                    with results_lock:
                        results["success"] += 1
                        print(f"[Thread {thread_id}] Success: ${cost:.6f}, Total: ${spent_after:.6f} | {answer}")

                except Exception as e:
                    error_str = str(e)
                    if "Iron Cage budget limit exceeded" in error_str or "402" in error_str:
                        with results_lock:
                            results["budget_exceeded"] += 1
                            print(f"[Thread {thread_id}] Budget exceeded! Total: ${router.total_spent():.6f}")
                        break
                    else:
                        with results_lock:
                            results["errors"].append(str(e))
                        break

        try:
            print(f"\nStarting {NUM_THREADS} concurrent threads with ${BUDGET_USD} budget...\n")

            threads = []
            for i in range(NUM_THREADS):
                t = threading.Thread(target=worker, args=(i,))
                threads.append(t)
                t.start()

            for t in threads:
                t.join(timeout=120)

            print(f"\n=== Results ===")
            print(f"Successful requests: {results['success']}")
            print(f"Budget exceeded responses: {results['budget_exceeded']}")
            print(f"Final spent: ${router.total_spent():.6f} / ${BUDGET_USD}")
            if results["errors"]:
                print(f"Errors: {results['errors']}")

            # Verify budget enforcement worked
            assert results["budget_exceeded"] > 0, "At least one thread should hit budget limit"
            assert results["success"] >= 1, "At least one request should succeed"
            assert len(results["errors"]) == 0, f"Unexpected errors: {results['errors']}"

        finally:
            router.stop()


    def test_openai_concurrent_overspend_problem(self):
        """Test: Demonstrates budget overspend problem with large concurrent requests.

        With 5 threads and 4000 tokens each (~$0.008/request), multiple requests
        can pass the budget check simultaneously, causing significant overspend.
        This shows the limitation of pre-request checking without reservation.
        """
        try:
            from openai import OpenAI
        except ImportError:
            pytest.skip("openai package not installed")

        import threading

        BUDGET_USD = 0.05
        NUM_THREADS = 5
        MAX_TOKENS = 4000  # x2 tokens = larger requests = more overspend

        router = LlmRouter(**CONFIG, budget=BUDGET_USD)

        if router.provider != "openai":
            router.stop()
            pytest.skip(f"Provider is '{router.provider}', not 'openai'")

        results = {"success": 0, "budget_exceeded": 0, "errors": []}
        results_lock = threading.Lock()

        def worker(thread_id):
            client = OpenAI(
                base_url=router.base_url,
                api_key=router.api_key,
            )

            while True:
                spent_before = router.total_spent()
                try:
                    response = client.chat.completions.create(
                        model="gpt-5-mini",
                        messages=[{"role": "user", "content": "Write a very long detailed essay about AI history and future."}],
                        max_completion_tokens=MAX_TOKENS,
                    )
                    spent_after = router.total_spent()
                    cost = spent_after - spent_before
                    answer = response.choices[0].message.content[:40] + "..."

                    with results_lock:
                        results["success"] += 1
                        print(f"[Thread {thread_id}] ${cost:.4f} | Total: ${spent_after:.4f} | {answer}")

                except Exception as e:
                    error_str = str(e)
                    if "Iron Cage budget limit exceeded" in error_str or "402" in error_str:
                        with results_lock:
                            results["budget_exceeded"] += 1
                            print(f"[Thread {thread_id}] BLOCKED - Budget exceeded! Total: ${router.total_spent():.4f}")
                        break
                    else:
                        with results_lock:
                            results["errors"].append(str(e))
                        break

        try:
            print(f"\n{'='*60}")
            print(f"Testing budget overspend with large concurrent requests")
            print(f"Budget: ${BUDGET_USD} | Threads: {NUM_THREADS} | Tokens: {MAX_TOKENS}")
            print(f"{'='*60}\n")

            threads = []
            for i in range(NUM_THREADS):
                t = threading.Thread(target=worker, args=(i,))
                threads.append(t)
                t.start()

            for t in threads:
                t.join(timeout=180)

            final_spent = router.total_spent()
            overspend = final_spent - BUDGET_USD
            overspend_pct = (overspend / BUDGET_USD) * 100

            print(f"\n{'='*60}")
            print(f"RESULTS:")
            print(f"  Successful requests: {results['success']}")
            print(f"  Budget exceeded responses: {results['budget_exceeded']}")
            print(f"  Budget limit: ${BUDGET_USD:.2f}")
            print(f"  Final spent: ${final_spent:.4f}")
            print(f"  OVERSPEND: ${overspend:.4f} ({overspend_pct:.1f}% over budget)")
            print(f"{'='*60}")

            if overspend > 0:
                print(f"\n⚠️  PROBLEM DEMONSTRATED: Spent ${overspend:.4f} over budget!")
                print(f"   This happens because budget is checked BEFORE request,")
                print(f"   but multiple threads pass the check simultaneously.")

            # Test passes - we're demonstrating the problem exists
            assert results["success"] >= 1
            assert results["budget_exceeded"] > 0

        finally:
            router.stop()


class TestBudgetE2EAnthropic:
    """E2E tests with Anthropic (claude-3-haiku) - ~$0.001 per test"""

    @pytest.fixture
    def router_with_budget(self):
        """Create router with $0.05 budget"""
        router = LlmRouter(**CONFIG, budget=0.05)
        yield router
        router.stop()

    def test_anthropic_request_tracks_spending(self, router_with_budget):
        """Test: Anthropic/Claude request updates spent amount"""
        try:
            from openai import OpenAI
        except ImportError:
            pytest.skip("openai package not installed")

        # Skip if provider is not Anthropic
        if router_with_budget.provider != "anthropic":
            pytest.skip(f"Provider is '{router_with_budget.provider}', not 'anthropic'")

        client = OpenAI(
            base_url=router_with_budget.base_url,
            api_key=router_with_budget.api_key,
        )

        # Initial state
        spent_before, limit = router_with_budget.budget_status
        assert spent_before == 0.0

        # Make Claude request (via OpenAI SDK - router translates)
        response = client.chat.completions.create(
            model="claude-3-haiku-20240307",  # Cheapest Claude model
            messages=[{"role": "user", "content": "Say 'test' only"}],
            max_tokens=5,
        )

        # Verify spending tracked
        spent_after, _ = router_with_budget.budget_status
        assert spent_after > 0.0
        assert spent_after < 0.05
        print(f"Cost: ${spent_after:.6f}")

    def test_anthropic_budget_exceeded_returns_402(self):
        """Test: Anthropic request with zero budget returns 402"""
        try:
            from openai import OpenAI
        except ImportError:
            pytest.skip("openai package not installed")

        router = LlmRouter(**CONFIG, budget=0.0)

        # Skip if provider is not Anthropic
        if router.provider != "anthropic":
            router.stop()
            pytest.skip(f"Provider is '{router.provider}', not 'anthropic'")

        client = OpenAI(
            base_url=router.base_url,
            api_key=router.api_key,
        )

        try:
            with pytest.raises(Exception) as exc_info:
                client.chat.completions.create(
                    model="claude-3-haiku-20240307",
                    messages=[{"role": "user", "content": "test"}],
                )

            error_str = str(exc_info.value)
            assert "Iron Cage budget limit exceeded" in error_str or "402" in error_str
        finally:
            router.stop()


class TestBudgetRuntime:
    """Test budget management at runtime"""

    def test_budget_on_creation(self):
        """Test creating router with budget parameter"""
        router = LlmRouter(**CONFIG, budget=10.0)

        try:
            assert router.budget == 10.0
            assert router.budget_status == (0.0, 10.0)
        finally:
            router.stop()

    def test_no_budget_means_unlimited(self):
        """Test that omitting budget allows unlimited requests"""
        router = LlmRouter(**CONFIG)

        try:
            assert router.budget is None
            assert router.budget_status is None
        finally:
            router.stop()

    def test_set_budget_updates_limit(self):
        """Test set_budget() method updates limit"""
        router = LlmRouter(**CONFIG, budget=5.0)

        try:
            assert router.budget == 5.0
            router.set_budget(20.0)
            assert router.budget == 20.0
        finally:
            router.stop()

    def test_set_budget_allows_more_requests(self):
        """Test: Increasing budget after exceeded allows new requests"""
        try:
            from openai import OpenAI
        except ImportError:
            pytest.skip("openai package not installed")

        router = LlmRouter(**CONFIG, budget=0.0)

        # Skip if provider is not OpenAI
        if router.provider != "openai":
            router.stop()
            pytest.skip(f"Provider is '{router.provider}', not 'openai'")

        client = OpenAI(
            base_url=router.base_url,
            api_key=router.api_key,
        )

        try:
            # First request should fail
            with pytest.raises(Exception):
                client.chat.completions.create(
                    model="gpt-4o-mini",
                    messages=[{"role": "user", "content": "test"}],
                )

            # Increase budget
            router.set_budget(0.05)
            assert router.budget == 0.05

            # Now request should succeed
            response = client.chat.completions.create(
                model="gpt-4o-mini",
                messages=[{"role": "user", "content": "Say ok"}],
                max_tokens=3,
            )
            assert response.choices[0].message.content
            print(f"Response: {response.choices[0].message.content}")
            print(f"Cost: ${router.total_spent():.6f}")
        finally:
            router.stop()
