"""
Iron Cage - Python bindings for Iron Cage AI agent runtime

Provides:
- Runtime: Agent lifecycle management with safety and cost controls
- LlmRouter: Local proxy for OpenAI/Anthropic API requests

Example:
    from iron_cage import LlmRouter
    from openai import OpenAI

    router = LlmRouter(
        api_key="ic_xxx",
        server_url="https://api.iron-cage.io",
    )
    client = OpenAI(base_url=router.base_url, api_key=router.api_key)
    response = client.chat.completions.create(...)
    router.stop()
"""

__version__ = "0.1.0"
__all__ = ["Runtime", "LlmRouter"]

try:
    # When compiled as a Python extension, import from the Rust module
    from iron_cage.iron_cage import Runtime, LlmRouter
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

    class LlmRouter:
        """Stub LlmRouter class for development without Rust compilation"""

        def __init__(
            self,
            api_key: str,
            server_url: str,
            cache_ttl_seconds: int = 300,
        ):
            self._api_key = api_key
            self._server_url = server_url
            self._port = 8000
            self._running = True
            print(f"[STUB] LlmRouter initialized for {server_url}")

        @property
        def base_url(self) -> str:
            return f"http://127.0.0.1:{self._port}/v1"

        @property
        def api_key(self) -> str:
            return self._api_key

        @property
        def port(self) -> int:
            return self._port

        @property
        def is_running(self) -> bool:
            return self._running

        def stop(self) -> None:
            self._running = False
            print("[STUB] LlmRouter stopped")

        def __enter__(self):
            return self

        def __exit__(self, exc_type, exc_val, exc_tb):
            self.stop()
