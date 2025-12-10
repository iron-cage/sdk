"""
Iron Cage - Python bindings for Iron Cage AI agent runtime

Provides safety and cost controls for AI agents via Rust backend.

Example:
    from iron_cage import Runtime

    runtime = Runtime(budget=50.0, verbose=True)
    agent_id = runtime.start_agent("my_agent.py")
    metrics = runtime.get_metrics(agent_id)
    runtime.stop_agent(agent_id)
"""

# PyO3 bindings are loaded from the compiled Rust library
# This file serves as the Python module entrypoint

__version__ = "0.1.0"
__all__ = ["Runtime"]

try:
    # When compiled as a Python extension, import from the Rust module
    from .iron_cage import Runtime
except ImportError:
    # Fallback for development/testing without compilation
    class Runtime:
        """Stub Runtime class for development without Rust compilation"""

        def __init__(self, budget: float, verbose: bool = False):
            self.budget = budget
            self.verbose = verbose
            print(f"[STUB] Runtime initialized with budget=${budget}")

        def start_agent(self, script_path: str) -> str:
            print(f"[STUB] Starting agent from {script_path}")
            return "stub-agent-id"

        def stop_agent(self, agent_id: str) -> None:
            print(f"[STUB] Stopping agent {agent_id}")

        def get_metrics(self, agent_id: str) -> dict:
            print(f"[STUB] Getting metrics for {agent_id}")
            return {
                "agent_id": agent_id,
                "status": "Running",
                "budget_spent": 0.0,
                "pii_detections": 0,
            }
