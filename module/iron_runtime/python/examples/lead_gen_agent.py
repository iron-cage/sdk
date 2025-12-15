#!/usr/bin/env python3
"""
Lead Generation Agent - Conference Demo

Features demonstrated:
- Feature #16: Sample Agent Implementation
- Feature #17: Agent Instrumentation (runtime integration)
- Feature #18: Demo Triggers (PII, budget, circuit breaker)

Demo triggers:
- Lead #34: Circuit breaker activation (LinkedIn API failure)
- Lead #67: PII detection (email in output)
- Lead #85: Budget warning (90% threshold)
"""

import sys
from pathlib import Path

# Add parent directory to path for iron_runtime import
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from iron_cage import Runtime
except ImportError:
    print("ERROR: iron_cage not found. Compile Rust extension first:")
    print("  cd /home/user1/pro/lib/willbe/module/iron_runtime")
    print("  maturin develop")
    sys.exit(1)


def simulate_lead_processing(lead_num: int) -> dict:
    """
    Simulate processing a single lead

    Returns lead data with demo triggers at specific indices
    """
    lead = {
        "lead_num": lead_num,
        "company": f"Company {lead_num}",
        "contact": f"contact{lead_num}@example.com",  # Contains email (PII)
    }

    # Demo Trigger #1: Circuit breaker at lead #34
    if lead_num == 34:
        print(f"\n🔴 DEMO TRIGGER: Simulating LinkedIn API failure at lead #{lead_num}")
        lead["linkedin_failed"] = True

    # Demo Trigger #2: PII detection at lead #67
    if lead_num == 67:
        print(f"\n🔴 DEMO TRIGGER: Injecting PII in output at lead #{lead_num}")
        lead["ceo_email"] = "ceo@acme.com"  # This should be detected and redacted

    # Demo Trigger #3: Budget warning at lead #85
    if lead_num == 85:
        print(f"\n🔴 DEMO TRIGGER: Approaching budget threshold at lead #{lead_num}")

    return lead


def main():
    """
    Main demo agent function

    Processes 100 leads with demo triggers at specific points
    """
    print("=" * 60)
    print("Iron Cage Lead Generation Agent - Conference Demo")
    print("=" * 60)

    # Initialize runtime with $50 budget
    budget = 50.0
    print(f"\n📊 Initializing runtime with ${budget} budget...")

    runtime = Runtime(budget=budget, verbose=True)

    # Start agent
    print(f"\n🚀 Starting lead generation agent...")
    agent_id = runtime.start_agent("lead_gen_agent.py")
    print(f"✅ Agent started: {agent_id}")

    # Process 100 leads
    total_leads = 100
    print(f"\n📋 Processing {total_leads} leads...\n")

    for i in range(1, total_leads + 1):
        lead = simulate_lead_processing(i)

        # Show progress every 10 leads
        if i % 10 == 0:
            print(f"  ✓ Processed {i}/{total_leads} leads")

            # Get metrics
            metrics_json = runtime.get_metrics(agent_id)
            if metrics_json:
                print(f"    Metrics: {metrics_json}")

    # Final results
    print(f"\n✅ Completed processing {total_leads} leads")

    # Get final metrics
    metrics_json = runtime.get_metrics(agent_id)
    if metrics_json:
        print(f"\n📊 Final Metrics:")
        print(f"  {metrics_json}")

    # Stop agent
    print(f"\n🛑 Stopping agent...")
    runtime.stop_agent(agent_id)
    print(f"✅ Agent stopped")

    print("\n" + "=" * 60)
    print("Demo Complete!")
    print("=" * 60)


if __name__ == "__main__":
    main()
