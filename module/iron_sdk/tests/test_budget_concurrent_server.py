#!/usr/bin/env python3
"""Concurrent Budget Test - 10 threads through server until blocked.

Connects through server at localhost:3001, server enforces limits.
Budget and API key come from server handshake (Protocol 005 + Feature 014).

Run:
    python tests/test_budget_concurrent_server.py
"""

import os
import sqlite3
import subprocess
import threading
import time

DB_PATH = "/home/tihilya/obox/iron_runtime/iron.db"
PROJECT_ROOT = "/home/tihilya/obox/iron_runtime"
SERVER_URL = "http://localhost:3001"
NUM_THREADS = 10
AGENT_ID = 9999


def create_ic_token(agent_id: int) -> str:
    """Create IC Token using Rust generator.

    Note: Python-generated JWT tokens don't work because PyJWT produces
    different header field order than Rust jsonwebtoken crate:
    - Rust: {"typ":"JWT","alg":"HS256"}
    - Python: {"alg":"HS256","typ":"JWT"}

    This changes the signature, causing validation to fail.
    """
    # Update the Rust example with the correct agent_id
    example_path = os.path.join(PROJECT_ROOT, "module/iron_control_api/examples/gen_ic_token.rs")
    example_code = f'''use iron_control_api::ic_token::{{IcTokenClaims, IcTokenManager}};

fn main() {{
    let secret = "dev-ic-token-secret-change-in-production";
    let manager = IcTokenManager::new(secret.to_string());
    let claims = IcTokenClaims::new(
        "agent_{agent_id}".to_string(),
        "budget_{agent_id}".to_string(),
        vec!["llm:call".to_string(), "analytics:write".to_string()],
        None,
    );
    println!("{{}}", manager.generate_token(&claims).unwrap());
}}
'''
    with open(example_path, 'w') as f:
        f.write(example_code)

    # Run Rust token generator
    result = subprocess.run(
        ["cargo", "run", "--package", "iron_control_api", "--example", "gen_ic_token"],
        cwd=PROJECT_ROOT,
        capture_output=True,
        text=True
    )
    if result.returncode != 0:
        raise RuntimeError(f"Failed to generate IC token: {result.stderr}")

    return result.stdout.strip()


def get_server_usage(user_id: str = "user_admin") -> dict:
    """Get usage_limits from server database."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    try:
        cursor.execute(
            """SELECT max_cost_microdollars_per_month, current_cost_microdollars_this_month
               FROM usage_limits WHERE user_id = ?""",
            (user_id,)
        )
        row = cursor.fetchone()
        if row:
            return {
                "limit_micros": row[0],
                "spent_micros": row[1],
                "limit_usd": row[0] / 1_000_000 if row[0] else 0,
                "spent_usd": row[1] / 1_000_000 if row[1] else 0,
            }
        return {"limit_micros": 0, "spent_micros": 0, "limit_usd": 0, "spent_usd": 0}
    finally:
        conn.close()


def run_concurrent_test():
    """Run 10 concurrent threads through server until blocked."""
    from openai import OpenAI
    from iron_cage import LlmRouter

    # Create IC token for server authentication
    ic_token = create_ic_token(AGENT_ID)
    print(f"IC Token: {ic_token[:50]}...")

    # Connect through server - key fetched automatically via Feature 014
    # Server provides API key based on agent's provider_key_id assignment
    import sys
    print("Creating router...", flush=True)
    router = LlmRouter(
        api_key=ic_token,
        server_url=SERVER_URL,
        budget=None,  # Server-side budget via handshake
    )

    print(f"Router base_url: {router.base_url}", flush=True)
    print(f"Router provider: {router.provider}", flush=True)
    print(f"Router budget: {router.budget} (from server handshake)", flush=True)
    print(f"Router is_running: {router.is_running}", flush=True)
    sys.stdout.flush()

    results = {"success": 0, "blocked": 0, "errors": []}
    lock = threading.Lock()
    stop_flag = threading.Event()

    def worker(tid: int):
        client = OpenAI(base_url=router.base_url, api_key=router.api_key)
        count = 0
        max_req = 30

        while not stop_flag.is_set() and count < max_req:
            try:
                resp = client.chat.completions.create(
                    model="gpt-4o-mini",
                    messages=[{"role": "user", "content": "Write about AI."}],
                    max_tokens=500,
                )
                count += 1
                spent = router.total_spent()
                answer = resp.choices[0].message.content[:35] + "..."

                with lock:
                    results["success"] += 1
                    print(f"[T{tid:02d}] #{count} | ${spent:.4f} | {answer}")

            except Exception as e:
                err = str(e)
                if "budget" in err.lower() or "402" in err:
                    with lock:
                        results["blocked"] += 1
                        print(f"[T{tid:02d}] BLOCKED! Spent: ${router.total_spent():.4f}")
                    break
                else:
                    with lock:
                        results["errors"].append(f"T{tid}: {err[:80]}")
                        print(f"[T{tid:02d}] ERROR: {err[:80]}")
                    break

    # Server status before
    server_before = get_server_usage()
    budget_usd = server_before['limit_usd']

    # Print header
    print("=" * 70)
    print(f"CONCURRENT BUDGET TEST (through server)")
    print(f"  Server: {SERVER_URL}")
    print(f"  Threads: {NUM_THREADS}")
    print(f"  Server budget limit: ${budget_usd:.2f}")
    print(f"  Server already spent: ${server_before['spent_usd']:.4f}")
    print("=" * 70)
    print()

    # Run threads
    threads = [threading.Thread(target=worker, args=(i,)) for i in range(NUM_THREADS)]
    start = time.time()
    for t in threads:
        t.start()
    for t in threads:
        t.join(timeout=180)
    elapsed = time.time() - start

    # Results
    local_spent = router.total_spent()
    server_after = get_server_usage()

    print()
    print("=" * 70)
    print(f"RESULTS (elapsed: {elapsed:.1f}s)")
    print("=" * 70)
    print(f"  Successful requests: {results['success']}")
    print(f"  Blocked by budget:   {results['blocked']}")
    print(f"  Errors:              {len(results['errors'])}")
    print()
    print(f"  LOCAL spent:  ${local_spent:.4f}")
    print(f"  LOCAL budget: {router.budget}")
    print()
    print(f"  SERVER limit: ${server_after['limit_usd']:.4f}")
    print(f"  SERVER spent: ${server_after['spent_usd']:.4f}")
    print()

    # Overspend analysis
    overspend = server_after['spent_usd'] - budget_usd
    if overspend > 0:
        pct = (overspend / budget_usd) * 100
        print(f"  OVERSPEND: ${overspend:.4f} ({pct:.1f}% over budget)")
    else:
        remaining = budget_usd - server_after['spent_usd']
        print(f"  UNDER BUDGET: ${remaining:.4f} remaining")

    # Local vs Server comparison
    diff = abs(local_spent - server_after['spent_usd'])
    print(f"\n  Local vs Server diff: ${diff:.6f}")

    if results["errors"]:
        print(f"\n  Errors: {results['errors'][:3]}")

    print("=" * 70)

    router.stop()


if __name__ == "__main__":
    run_concurrent_test()
